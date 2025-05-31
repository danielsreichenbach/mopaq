//! Archive-level operations

use anyhow::{Context, Result};
use colored::Colorize;
use mopaq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption, OpenOptions};
use std::path::Path;
use walkdir::WalkDir;

use crate::output::{print_archive_info, print_verify_result};
use crate::GLOBAL_OPTS;

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
pub fn verify(archive_path: &str, _check_crc: bool, check_contents: bool) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    if !global_opts.quiet {
        println!("Verifying archive: {}", archive_path.cyan());
    }

    let options = OpenOptions::new();
    // TODO: Add checksum verification when available
    // if check_crc {
    //     options.verify_checksums(true);
    // }

    let mut archive = Archive::open_with_options(archive_path, options)?;
    let file_entries = archive.list()?;
    let files: Vec<String> = file_entries.into_iter().map(|e| e.name).collect();

    let mut total_files = 0;
    let mut verified_files = 0;
    let mut errors = Vec::new();

    for filename in &files {
        total_files += 1;

        if check_contents {
            // Try to read the file to verify it can be decompressed
            match archive.read_file(filename) {
                Ok(_) => {
                    verified_files += 1;
                    if global_opts.verbose > 0 && !global_opts.quiet {
                        println!("{} {}", "✓".green(), filename);
                    }
                }
                Err(e) => {
                    errors.push((filename.clone(), e.to_string()));
                    if global_opts.verbose > 0 && !global_opts.quiet {
                        println!("{} {} - {}", "✗".red(), filename, e);
                    }
                }
            }
        } else {
            // Just check if file exists in archive by trying to find it
            if archive.find_file(filename)?.is_some() {
                verified_files += 1;
                if global_opts.verbose > 0 && !global_opts.quiet {
                    println!("{} {}", "✓".green(), filename);
                }
            } else {
                errors.push((filename.clone(), "File not found".to_string()));
                if global_opts.verbose > 0 && !global_opts.quiet {
                    println!("{} {} - File not found", "✗".red(), filename);
                }
            }
        }
    }

    print_verify_result(
        total_files,
        verified_files,
        &errors,
        global_opts.output,
        global_opts.quiet,
    )?;

    if !errors.is_empty() {
        anyhow::bail!("Verification failed with {} errors", errors.len());
    }

    Ok(())
}
