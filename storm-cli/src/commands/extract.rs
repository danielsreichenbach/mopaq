//! Extract command implementation

use crate::{output, GlobalOptions, OutputFormat, GLOBAL_OPTS};
use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use mopaq::Archive;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

#[derive(Serialize, Deserialize)]
struct ExtractResult {
    archive: String,
    output_dir: String,
    mode: String, // "single" or "all"
    total_files: usize,
    extracted: usize,
    failed: usize,
    skipped: usize,
    files: Vec<FileExtractResult>,
}

#[derive(Serialize, Deserialize)]
struct FileExtractResult {
    filename: String,
    output_path: String,
    size: u64,
    status: String, // "success", "failed", "skipped"
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Extract files from an MPQ archive
pub fn extract(archive_path: &str, output_dir: &str, specific_file: Option<&str>) -> Result<()> {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    if !opts.quiet {
        if output::use_color() {
            println!("{}: {}", "Opening archive".bold(), archive_path.cyan());
        } else {
            println!("Opening archive: {}", archive_path);
        }
    }

    let mut archive = Archive::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path))?;

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    if let Some(filename) = specific_file {
        // Extract specific file
        extract_single_file(&mut archive, filename, output_dir, archive_path)
    } else {
        // Extract all files
        extract_all_files(&mut archive, output_dir, archive_path)
    }
}

/// Extract a single file from the archive
fn extract_single_file(
    archive: &mut Archive,
    filename: &str,
    output_dir: &str,
    archive_path: &str,
) -> Result<()> {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    if !opts.quiet && opts.output == OutputFormat::Text {
        if output::use_color() {
            println!("{}: {}", "Extracting file".bold(), filename.cyan());
        } else {
            println!("Extracting file: {}", filename);
        }
    }

    let start_time = std::time::Instant::now();
    let result = extract_file_safe(archive, filename, output_dir);

    match opts.output {
        OutputFormat::Json | OutputFormat::Csv => {
            // Structured output
            let (status, error, size) = match &result {
                Ok(size) => ("success".to_string(), None, *size as u64),
                Err(e) => ("failed".to_string(), Some(e.to_string()), 0),
            };

            let output_path = build_output_path(output_dir, filename);
            let extract_result = ExtractResult {
                archive: archive_path.to_string(),
                output_dir: output_dir.to_string(),
                mode: "single".to_string(),
                total_files: 1,
                extracted: if status == "success" { 1 } else { 0 },
                failed: if status == "failed" { 1 } else { 0 },
                skipped: 0,
                files: vec![FileExtractResult {
                    filename: filename.to_string(),
                    output_path: output_path.display().to_string(),
                    size,
                    status,
                    error,
                }],
            };

            output::print_output(&extract_result)?;
        }
        OutputFormat::Text => {
            // Text output
            match result {
                Ok(size) => {
                    let output_path = build_output_path(output_dir, filename);
                    let elapsed = start_time.elapsed();

                    if !opts.quiet {
                        if output::use_color() {
                            println!(
                                "{} Extracted {} ({}) to {:?} in {:.2}s",
                                "✓".green().bold(),
                                filename.cyan(),
                                format_size(size as u64).yellow(),
                                output_path,
                                elapsed.as_secs_f64()
                            );
                        } else {
                            println!(
                                "✓ Extracted {} ({} bytes) to {:?} in {:.2}s",
                                filename,
                                size,
                                output_path,
                                elapsed.as_secs_f64()
                            );
                        }
                    }
                }
                Err(e) => {
                    if output::use_color() {
                        eprintln!(
                            "{} Failed to extract '{}': {}",
                            "✗".red().bold(),
                            filename,
                            e.to_string().red()
                        );
                    } else {
                        eprintln!("✗ Failed to extract '{}': {}", filename, e);
                    }
                    return Err(e);
                }
            }
        }
    }

    result.map(|_| ())
}

/// Extract all files from the archive
fn extract_all_files(archive: &mut Archive, output_dir: &str, archive_path: &str) -> Result<()> {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    if !opts.quiet && opts.output == OutputFormat::Text {
        println!("Extracting all files to: {}", output_dir);
    }

    // First, try to find and extract (listfile)
    match archive.find_file("(listfile)")? {
        Some(_) => extract_using_listfile(archive, output_dir, archive_path),
        None => {
            if opts.output == OutputFormat::Text {
                eprintln!("No (listfile) found in archive");
                eprintln!("Cannot extract all files without knowing their names");
                eprintln!("Try extracting specific files with -f option");
                eprintln!();
                eprintln!("Alternatively, you can:");
                eprintln!("  1. Use 'storm-cli list' to see what files can be detected");
                eprintln!("  2. Extract known files individually with -f");
                eprintln!("  3. Use 'storm-cli debug tables' to inspect the archive structure");
            }

            Err(anyhow::anyhow!(
                "Cannot extract all files without a listfile"
            ))
        }
    }
}

/// Extract files using the (listfile) content
fn extract_using_listfile(
    archive: &mut Archive,
    output_dir: &str,
    archive_path: &str,
) -> Result<()> {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    // Read and parse the listfile
    let listfile_data = archive
        .read_file("(listfile)")
        .context("Failed to read (listfile)")?;

    let filenames = mopaq::special_files::parse_listfile(&listfile_data)
        .context("Failed to parse (listfile)")?;

    if filenames.is_empty() {
        if !opts.quiet && opts.output == OutputFormat::Text {
            eprintln!("Warning: (listfile) is empty or contains no valid entries");
        }
        return Ok(());
    }

    let total_files = filenames.len();

    if !opts.quiet && opts.output == OutputFormat::Text {
        if output::use_color() {
            println!(
                "{} {} files in (listfile)",
                "Found".green(),
                total_files.to_string().bright_blue()
            );
        } else {
            println!("Found {} files in (listfile)", total_files);
        }
        println!();
    }

    let mut extracted_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut file_results = Vec::new();

    // Create progress bar for text output
    let progress = if !opts.quiet && opts.output == OutputFormat::Text {
        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    for filename in &filenames {
        // Skip the listfile itself
        if filename == "(listfile)" {
            skipped_count += 1;
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
            continue;
        }

        let output_path = build_output_path(output_dir, filename);

        // Update progress message
        if let Some(ref pb) = progress {
            pb.set_message(format!("Extracting {}", filename));
        }

        // Log verbose info
        output::verbose_println(
            1,
            &format!("Extracting {} → {}", filename, output_path.display()),
        );

        match extract_file_safe(archive, filename, output_dir) {
            Ok(size) => {
                if opts.output == OutputFormat::Text && !opts.quiet && progress.is_none() {
                    // Only print individual files if not using progress bar
                    print!("Extracting {} ", filename.cyan());
                    if filename.contains('\\') || filename.contains('/') {
                        print!("→ {} ", output_path.display().to_string().dimmed());
                    }
                    println!(
                        "... {} ({})",
                        "OK".green().bold(),
                        format_size(size as u64).dimmed()
                    );
                }

                file_results.push(FileExtractResult {
                    filename: filename.clone(),
                    output_path: output_path.display().to_string(),
                    size: size as u64,
                    status: "success".to_string(),
                    error: None,
                });

                extracted_count += 1;
            }
            Err(e) => {
                if opts.output == OutputFormat::Text && !opts.quiet && progress.is_none() {
                    print!("Extracting {} ", filename.cyan());
                    if filename.contains('\\') || filename.contains('/') {
                        print!("→ {} ", output_path.display().to_string().dimmed());
                    }
                    println!("{}: {}", "FAILED".red().bold(), e.to_string().red());
                }

                file_results.push(FileExtractResult {
                    filename: filename.clone(),
                    output_path: output_path.display().to_string(),
                    size: 0,
                    status: "failed".to_string(),
                    error: Some(e.to_string()),
                });

                failed_count += 1;
            }
        }

        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message("Extraction complete");
    }

    // Handle output based on format
    match opts.output {
        OutputFormat::Json | OutputFormat::Csv => {
            let extract_result = ExtractResult {
                archive: archive_path.to_string(),
                output_dir: output_dir.to_string(),
                mode: "all".to_string(),
                total_files,
                extracted: extracted_count,
                failed: failed_count,
                skipped: skipped_count,
                files: file_results,
            };

            output::print_output(&extract_result)?;
        }
        OutputFormat::Text => {
            if !opts.quiet {
                // Summary
                println!();
                println!("{}", "Extraction complete:".bold().underline());

                if output::use_color() {
                    println!(
                        "  {}: {}",
                        "Files extracted".green(),
                        extracted_count.to_string().green()
                    );
                    println!(
                        "  {}: {}",
                        "Files failed".red(),
                        failed_count.to_string().red()
                    );
                    println!(
                        "  {}: {}",
                        "Files skipped".yellow(),
                        skipped_count.to_string().yellow()
                    );
                } else {
                    println!("  Files extracted: {}", extracted_count);
                    println!("  Files failed: {}", failed_count);
                    println!("  Files skipped: {}", skipped_count);
                }

                if failed_count > 0 {
                    println!();
                    println!(
                        "{}",
                        "Note: Some files failed to extract. Common reasons:".yellow()
                    );
                    println!(
                        "  {} File is referenced in (listfile) but not actually present",
                        "-".dimmed()
                    );
                    println!(
                        "  {} File uses unsupported compression (PKWare, Huffman, etc.)",
                        "-".dimmed()
                    );
                    println!("  {} File is corrupted or has invalid data", "-".dimmed());
                }
            }
        }
    }

    Ok(())
}

/// Safely extract a file, returning the file size on success
fn extract_file_safe(archive: &mut Archive, filename: &str, output_dir: &str) -> Result<usize> {
    // Read the file data
    let data = archive
        .read_file(filename)
        .with_context(|| format!("Failed to read file '{}' from archive", filename))?;

    // Convert the archive path to OS-appropriate path
    let output_path = build_output_path(output_dir, filename);

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create directory structure for: {:?}",
                output_path
            )
        })?;
    }

    // Write the file
    fs::write(&output_path, &data)
        .with_context(|| format!("Failed to write file to: {:?}", output_path))?;

    Ok(data.len())
}

/// Build an OS-appropriate output path from an archive filename
fn build_output_path(output_dir: &str, archive_filename: &str) -> PathBuf {
    // Replace both forward slashes and backslashes with the OS separator
    let normalized_path = if MAIN_SEPARATOR == '\\' {
        // On Windows, just replace forward slashes
        archive_filename.replace('/', "\\")
    } else {
        // On Unix, replace backslashes with forward slashes
        archive_filename.replace('\\', "/")
    };

    Path::new(output_dir).join(normalized_path)
}

/// Format file size in human-readable format
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
