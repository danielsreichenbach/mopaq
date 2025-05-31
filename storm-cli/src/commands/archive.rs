//! Archive-level operations

use anyhow::{Context, Result};
use colored::Colorize;
use mopaq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption, OpenOptions, SignatureStatus};
use serde_json;
use std::path::Path;
use walkdir::WalkDir;

use crate::output::{print_archive_info, print_json};
use crate::{OutputFormat, GLOBAL_OPTS};

#[derive(Debug, Clone)]
pub struct CreateOptions {
    pub version: FormatVersion,
    pub compression: u16,
    pub block_size: u16,
    pub listfile: ListfileOption,
    pub recursive: bool,
    pub follow_symlinks: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            version: FormatVersion::V1,
            compression: mopaq::compression::flags::ZLIB as u16,
            block_size: 3, // 4KB sectors
            listfile: ListfileOption::Generate,
            recursive: true,
            follow_symlinks: false,
            ignore_patterns: vec![],
        }
    }
}

/// Create a new MPQ archive
pub fn create(archive_path: &str, source: &str, options: CreateOptions) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    if !global_opts.quiet {
        println!("Creating archive: {}", archive_path.cyan());
    }

    // Configure builder
    let mut builder = ArchiveBuilder::new()
        .version(options.version)
        .default_compression(options.compression as u8)
        .block_size(options.block_size)
        .listfile_option(options.listfile.clone());

    let source_path = Path::new(source);

    if source_path.is_file() {
        // Add single file
        let file_name = source_path
            .file_name()
            .context("Invalid file name")?
            .to_string_lossy();
        builder = builder.add_file(source, &file_name);
        if !global_opts.quiet {
            println!("Added: {}", file_name);
        }
    } else if source_path.is_dir() {
        // Add directory contents
        builder = add_directory_to_archive(builder, source_path, "", &options, global_opts.quiet)?;
    } else {
        anyhow::bail!("Source path does not exist: {}", source);
    }

    // Build the archive
    builder.build(archive_path)?;

    if !global_opts.quiet {
        println!("{} Archive created successfully", "✓".green());
    }

    Ok(())
}

fn add_directory_to_archive(
    mut builder: ArchiveBuilder,
    dir_path: &Path,
    archive_prefix: &str,
    options: &CreateOptions,
    quiet: bool,
) -> Result<ArchiveBuilder> {
    let walker = if options.recursive {
        WalkDir::new(dir_path)
            .follow_links(options.follow_symlinks)
            .min_depth(1)
    } else {
        WalkDir::new(dir_path)
            .follow_links(options.follow_symlinks)
            .min_depth(1)
            .max_depth(1)
    };

    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check ignore patterns
        let should_ignore = options.ignore_patterns.iter().any(|pattern| {
            path.to_string_lossy().contains(pattern)
                || path
                    .file_name()
                    .map(|name| name.to_string_lossy().contains(pattern))
                    .unwrap_or(false)
        });

        if should_ignore {
            continue;
        }

        // Calculate relative path for archive
        let relative_path = path
            .strip_prefix(dir_path)
            .context("Failed to calculate relative path")?;

        let archive_path = if archive_prefix.is_empty() {
            relative_path.to_string_lossy().to_string()
        } else {
            format!("{}/{}", archive_prefix, relative_path.to_string_lossy())
        };

        // Normalize path separators
        let archive_path = archive_path.replace('\\', "/");

        builder = builder.add_file(path.to_str().unwrap(), &archive_path);

        if !quiet {
            println!("Added: {}", archive_path);
        }
    }

    Ok(builder)
}

/// Show detailed archive information
pub fn info(archive_path: &str) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    let mut archive = Archive::open(archive_path)?;
    print_archive_info(&mut archive, global_opts.output)?;

    Ok(())
}

/// Verify archive integrity
pub fn verify(archive_path: &str, check_crc: bool, check_contents: bool) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    if !global_opts.quiet && global_opts.output == OutputFormat::Text {
        println!("Verifying archive: {}", archive_path.cyan());
    }

    let options = OpenOptions::new();
    let mut archive = Archive::open_with_options(archive_path, options)?;

    // Get archive info for detailed verification information
    let archive_info = archive.get_info()?;

    // Start verification results
    let mut verification_results = VerificationResults {
        archive_path: archive_path.to_string(),
        format_version: archive_info.format_version,
        total_files: 0,
        verified_files: 0,
        errors: Vec::new(),
        warnings: Vec::new(),
        header_checks: HeaderChecks::default(),
        table_checks: TableChecks::default(),
        file_checks: FileChecks::default(),
    };

    // Check archive header and tables integrity
    verification_results.header_checks.signature_valid = true; // MPQ signature already verified during open
    verification_results.header_checks.version_supported = matches!(
        archive_info.format_version,
        FormatVersion::V1 | FormatVersion::V2 | FormatVersion::V3 | FormatVersion::V4
    );

    // Check table integrity
    verification_results.table_checks.hash_table_loaded =
        !archive_info.hash_table_info.failed_to_load;
    verification_results.table_checks.block_table_loaded =
        !archive_info.block_table_info.failed_to_load;

    if let Some(het_info) = &archive_info.het_table_info {
        verification_results.table_checks.het_table_loaded = Some(!het_info.failed_to_load);
    }
    if let Some(bet_info) = &archive_info.bet_table_info {
        verification_results.table_checks.bet_table_loaded = Some(!bet_info.failed_to_load);
    }

    // Check MD5 checksums if available (v4 archives)
    if let Some(md5_status) = &archive_info.md5_status {
        verification_results.table_checks.md5_checksums = Some(Md5Checks {
            header_valid: md5_status.header_valid,
            hash_table_valid: md5_status.hash_table_valid,
            block_table_valid: md5_status.block_table_valid,
            hi_block_table_valid: md5_status.hi_block_table_valid,
            het_table_valid: md5_status.het_table_valid,
            bet_table_valid: md5_status.bet_table_valid,
        });
    }

    // Check digital signature
    verification_results.header_checks.signature_status =
        Some(archive_info.signature_status.clone());

    // Verify individual files
    let file_entries = archive.list()?;
    let files: Vec<String> = file_entries.into_iter().map(|e| e.name).collect();
    verification_results.total_files = files.len();

    for filename in &files {
        // First check if file exists in archive
        if archive.find_file(filename)?.is_some() {
            verification_results.file_checks.files_found += 1;

            if check_contents {
                // Try to read the file to verify it can be decompressed
                match archive.read_file(filename) {
                    Ok(data) => {
                        verification_results.verified_files += 1;
                        verification_results.file_checks.files_readable += 1;

                        // TODO: When CRC checking is implemented, verify sector CRCs here
                        if check_crc {
                            // For now, just note that CRC checking was requested but not available
                            verification_results.warnings.push(format!(
                                "CRC verification requested but not yet implemented for file: {}",
                                filename
                            ));
                        }

                        if global_opts.verbose > 0
                            && !global_opts.quiet
                            && global_opts.output == OutputFormat::Text
                        {
                            println!("{} {} ({} bytes)", "✓".green(), filename, data.len());
                        }
                    }
                    Err(e) => {
                        verification_results
                            .errors
                            .push((filename.clone(), e.to_string()));
                        verification_results.file_checks.files_corrupted += 1;
                        if global_opts.verbose > 0
                            && !global_opts.quiet
                            && global_opts.output == OutputFormat::Text
                        {
                            println!("{} {} - {}", "✗".red(), filename, e);
                        }
                    }
                }
            } else {
                // File found, basic verification passed
                verification_results.verified_files += 1;
                if global_opts.verbose > 0
                    && !global_opts.quiet
                    && global_opts.output == OutputFormat::Text
                {
                    println!("{} {}", "✓".green(), filename);
                }
            }
        } else {
            verification_results
                .errors
                .push((filename.clone(), "File not found in tables".to_string()));
            verification_results.file_checks.files_missing += 1;
            if global_opts.verbose > 0
                && !global_opts.quiet
                && global_opts.output == OutputFormat::Text
            {
                println!("{} {} - File not found", "✗".red(), filename);
            }
        }
    }

    // Print detailed verification results
    print_detailed_verify_result(&verification_results, global_opts.output, global_opts.quiet)?;

    if !verification_results.errors.is_empty() {
        anyhow::bail!(
            "Verification failed with {} errors",
            verification_results.errors.len()
        );
    }

    Ok(())
}

#[derive(Debug)]
struct VerificationResults {
    archive_path: String,
    format_version: FormatVersion,
    total_files: usize,
    verified_files: usize,
    errors: Vec<(String, String)>,
    warnings: Vec<String>,
    header_checks: HeaderChecks,
    table_checks: TableChecks,
    file_checks: FileChecks,
}

#[derive(Debug, Default)]
struct HeaderChecks {
    signature_valid: bool,
    version_supported: bool,
    signature_status: Option<SignatureStatus>,
}

#[derive(Debug, Default)]
struct TableChecks {
    hash_table_loaded: bool,
    block_table_loaded: bool,
    het_table_loaded: Option<bool>,
    bet_table_loaded: Option<bool>,
    md5_checksums: Option<Md5Checks>,
}

#[derive(Debug)]
struct Md5Checks {
    header_valid: bool,
    hash_table_valid: bool,
    block_table_valid: bool,
    hi_block_table_valid: bool,
    het_table_valid: bool,
    bet_table_valid: bool,
}

#[derive(Debug, Default)]
struct FileChecks {
    files_found: usize,     // Files that exist in the archive tables
    files_readable: usize, // Files that can be successfully read/decompressed (subset of files_found)
    files_missing: usize,  // Files listed but not found in tables
    files_corrupted: usize, // Files found but failed to read/decompress (subset of files_found)
}

fn print_detailed_verify_result(
    results: &VerificationResults,
    format: OutputFormat,
    quiet: bool,
) -> Result<()> {
    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Text => {
            println!("\n{}", "Archive Verification Report".bold());
            println!("{}", "=".repeat(60));
            println!("Archive: {}", results.archive_path.cyan());
            println!("Format: MPQ v{}", results.format_version as u16 + 1);

            // Header verification
            println!("\n{}", "Header Verification".bold());
            println!("{}", "-".repeat(60));
            println!(
                "MPQ Signature:      {}",
                if results.header_checks.signature_valid {
                    "Valid".green()
                } else {
                    "Invalid".red()
                }
            );
            println!(
                "Format Version:     {}",
                if results.header_checks.version_supported {
                    format!("Supported (v{})", results.format_version as u16 + 1).green()
                } else {
                    "Unsupported".red()
                }
            );

            if let Some(sig_status) = &results.header_checks.signature_status {
                println!(
                    "Digital Signature:  {}",
                    match sig_status {
                        SignatureStatus::None => "No signature".dimmed(),
                        SignatureStatus::WeakValid => "Weak signature (Valid)".green(),
                        SignatureStatus::WeakInvalid => "Weak signature (Invalid)".red(),
                        SignatureStatus::StrongValid => "Strong signature (Valid)".green(),
                        SignatureStatus::StrongInvalid => "Strong signature (Invalid)".red(),
                        SignatureStatus::StrongNoKey => "Strong signature (No public key)".yellow(),
                    }
                );
            }

            // Table verification
            println!("\n{}", "Table Verification".bold());
            println!("{}", "-".repeat(60));
            println!(
                "Hash Table:         {}",
                if results.table_checks.hash_table_loaded {
                    "Loaded successfully".green()
                } else {
                    "Failed to load".red()
                }
            );
            println!(
                "Block Table:        {}",
                if results.table_checks.block_table_loaded {
                    "Loaded successfully".green()
                } else {
                    "Failed to load".red()
                }
            );

            if let Some(het_loaded) = results.table_checks.het_table_loaded {
                println!(
                    "HET Table:          {}",
                    if het_loaded {
                        "Loaded successfully".green()
                    } else {
                        "Failed to load".red()
                    }
                );
            }

            if let Some(bet_loaded) = results.table_checks.bet_table_loaded {
                println!(
                    "BET Table:          {}",
                    if bet_loaded {
                        "Loaded successfully".green()
                    } else {
                        "Failed to load".red()
                    }
                );
            }

            // MD5 checksums (v4 only)
            if let Some(md5) = &results.table_checks.md5_checksums {
                println!("\n{}", "MD5 Checksum Verification (v4)".bold());
                println!("{}", "-".repeat(60));
                println!(
                    "Header MD5:         {}",
                    if md5.header_valid {
                        "Valid".green()
                    } else {
                        "Invalid".red()
                    }
                );
                println!(
                    "Hash Table MD5:     {}",
                    if md5.hash_table_valid {
                        "Valid".green()
                    } else {
                        "Invalid".red()
                    }
                );
                println!(
                    "Block Table MD5:    {}",
                    if md5.block_table_valid {
                        "Valid".green()
                    } else {
                        "Invalid".red()
                    }
                );
                println!(
                    "Hi-Block Table MD5: {}",
                    if md5.hi_block_table_valid {
                        "Valid".green()
                    } else {
                        "Invalid".red()
                    }
                );
                println!(
                    "HET Table MD5:      {}",
                    if md5.het_table_valid {
                        "Valid".green()
                    } else {
                        "Invalid".red()
                    }
                );
                println!(
                    "BET Table MD5:      {}",
                    if md5.bet_table_valid {
                        "Valid".green()
                    } else {
                        "Invalid".red()
                    }
                );
            }

            // File verification summary
            println!("\n{}", "File Verification Summary".bold());
            println!("{}", "-".repeat(60));
            println!("Total Files:        {}", results.total_files);
            println!(
                "Files Found:        {}",
                results.file_checks.files_found.to_string().green()
            );
            println!(
                "Files Readable:     {}",
                results.file_checks.files_readable.to_string().green()
            );
            println!(
                "Files Missing:      {}",
                if results.file_checks.files_missing > 0 {
                    results.file_checks.files_missing.to_string().red()
                } else {
                    results.file_checks.files_missing.to_string().dimmed()
                }
            );
            println!(
                "Files Corrupted:    {}",
                if results.file_checks.files_corrupted > 0 {
                    results.file_checks.files_corrupted.to_string().red()
                } else {
                    results.file_checks.files_corrupted.to_string().dimmed()
                }
            );

            println!(
                "\nVerified:           {} / {} ({}%)",
                results.verified_files.to_string().green(),
                results.total_files,
                (results.verified_files * 100) / results.total_files.max(1)
            );

            // Warnings
            if !results.warnings.is_empty() {
                println!("\n{}", "Warnings:".yellow());
                for warning in &results.warnings {
                    println!("  ⚠ {}", warning);
                }
            }

            // Errors
            if !results.errors.is_empty() {
                println!("\n{}", "Errors:".red());
                for (file, error) in &results.errors {
                    println!("  ✗ {} - {}", file, error);
                }
            }

            // Overall status
            println!("\n{}", "Overall Status".bold());
            println!("{}", "=".repeat(60));
            if results.errors.is_empty() {
                println!("{} Archive verification PASSED", "✓".green());
            } else {
                println!("{} Archive verification FAILED", "✗".red());
            }
        }
        OutputFormat::Json => {
            let json_result = serde_json::json!({
                "archive": results.archive_path,
                "format_version": results.format_version as u16 + 1,
                "total_files": results.total_files,
                "verified_files": results.verified_files,
                "header_checks": {
                    "signature_valid": results.header_checks.signature_valid,
                    "version_supported": results.header_checks.version_supported,
                    "signature_status": format!("{:?}", results.header_checks.signature_status),
                },
                "table_checks": {
                    "hash_table_loaded": results.table_checks.hash_table_loaded,
                    "block_table_loaded": results.table_checks.block_table_loaded,
                    "het_table_loaded": results.table_checks.het_table_loaded,
                    "bet_table_loaded": results.table_checks.bet_table_loaded,
                    "md5_checksums": results.table_checks.md5_checksums.as_ref().map(|md5| {
                        serde_json::json!({
                            "header_valid": md5.header_valid,
                            "hash_table_valid": md5.hash_table_valid,
                            "block_table_valid": md5.block_table_valid,
                            "hi_block_table_valid": md5.hi_block_table_valid,
                            "het_table_valid": md5.het_table_valid,
                            "bet_table_valid": md5.bet_table_valid,
                        })
                    }),
                },
                "file_checks": {
                    "files_found": results.file_checks.files_found,
                    "files_readable": results.file_checks.files_readable,
                    "files_missing": results.file_checks.files_missing,
                    "files_corrupted": results.file_checks.files_corrupted,
                },
                "warnings": results.warnings,
                "errors": results.errors.iter().map(|(f, e)| {
                    serde_json::json!({"file": f, "error": e})
                }).collect::<Vec<_>>(),
                "passed": results.errors.is_empty(),
            });
            print_json(&json_result)?;
        }
        OutputFormat::Csv => {
            println!("metric,value");
            println!("archive,{}", results.archive_path);
            println!("format_version,{}", results.format_version as u16 + 1);
            println!("total_files,{}", results.total_files);
            println!("verified_files,{}", results.verified_files);
            println!("errors,{}", results.errors.len());
            println!("warnings,{}", results.warnings.len());
            println!("passed,{}", results.errors.is_empty());
        }
    }

    Ok(())
}
