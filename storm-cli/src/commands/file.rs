//! File-level operations within archives

use anyhow::{Context, Result};
use colored::Colorize;
use glob::Pattern;
use mopaq::Archive;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

use crate::output::{print_file_info, print_file_list};
use crate::GLOBAL_OPTS;

/// List files in an archive
pub fn list(archive_path: &str, all: bool, pattern: Option<&str>, regex: bool) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    let mut archive = Archive::open(archive_path)?;
    let file_entries = archive.list()?;
    let mut files: Vec<String> = file_entries.into_iter().map(|e| e.name).collect();

    // Apply pattern filter if provided
    if let Some(pat) = pattern {
        if regex {
            let re = Regex::new(pat).context("Invalid regex pattern")?;
            files.retain(|f| re.is_match(f));
        } else {
            let glob = Pattern::new(pat).context("Invalid glob pattern")?;
            files.retain(|f| glob.matches(f));
        }
    }

    // Sort files
    files.sort();

    print_file_list(&files, all, global_opts.output)?;

    Ok(())
}

/// Extract files from an archive
pub fn extract(
    archive_path: &str,
    file: Option<&str>,
    output: Option<&str>,
    preserve_path: bool,
) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    let mut archive = Archive::open(archive_path)?;

    if let Some(filename) = file {
        // Extract single file
        let data = archive
            .read_file(filename)
            .context(format!("Failed to read file: {}", filename))?;

        let output_path = if let Some(out) = output {
            PathBuf::from(out)
        } else if preserve_path {
            PathBuf::from(filename)
        } else {
            PathBuf::from(Path::new(filename).file_name().unwrap())
        };

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&output_path, data)?;

        if !global_opts.quiet {
            println!("Extracted: {} -> {}", filename, output_path.display());
        }
    } else {
        // Extract all files
        let output_dir = output.unwrap_or(".");
        let file_entries = archive.list()?;
        let files: Vec<String> = file_entries.into_iter().map(|e| e.name).collect();

        for filename in &files {
            let data = match archive.read_file(filename) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to extract {}: {}", filename, e);
                    continue;
                }
            };

            let output_path = if preserve_path {
                PathBuf::from(output_dir).join(filename)
            } else {
                PathBuf::from(output_dir).join(Path::new(filename).file_name().unwrap())
            };

            // Create parent directories if needed
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&output_path, data)?;

            if !global_opts.quiet {
                println!("Extracted: {}", filename);
            }
        }

        if !global_opts.quiet {
            println!("{} Extracted {} files", "✓".green(), files.len());
        }
    }

    Ok(())
}

/// Add files to an existing archive
pub fn add(
    _archive_path: &str,
    _files: &[String],
    _compression: Option<u16>,
    _path: Option<&str>,
) -> Result<()> {
    // TODO: Implement file addition to existing archives
    anyhow::bail!("Adding files to existing archives is not yet implemented");
}

/// Remove files from an archive
pub fn remove(_archive_path: &str, _files: &[String]) -> Result<()> {
    // TODO: Implement file removal from archives
    anyhow::bail!("Removing files from archives is not yet implemented");
}

/// Find files in an archive
pub fn find(archive_path: &str, pattern: &str, regex: bool, ignore_case: bool) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    let mut archive = Archive::open(archive_path)?;
    let file_entries = archive.list()?;
    let files: Vec<String> = file_entries.into_iter().map(|e| e.name).collect();

    let matches: Vec<String> = if regex {
        let re = if ignore_case {
            Regex::new(&format!("(?i){}", pattern))
        } else {
            Regex::new(pattern)
        }
        .context("Invalid regex pattern")?;

        files.into_iter().filter(|f| re.is_match(f)).collect()
    } else {
        let glob = if ignore_case {
            Pattern::new(&pattern.to_lowercase()).context("Invalid glob pattern")?
        } else {
            Pattern::new(pattern).context("Invalid glob pattern")?
        };

        files
            .into_iter()
            .filter(|f| {
                if ignore_case {
                    glob.matches(&f.to_lowercase())
                } else {
                    glob.matches(f)
                }
            })
            .collect()
    };

    if matches.is_empty() {
        if !global_opts.quiet {
            println!("No files found matching pattern: {}", pattern);
        }
    } else {
        for file in &matches {
            println!("{}", file);
        }
        if !global_opts.quiet && global_opts.verbose > 0 {
            println!("\nFound {} matching files", matches.len());
        }
    }

    Ok(())
}

/// Show detailed file information
pub fn info(archive_path: &str, filename: &str) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    let mut archive = Archive::open(archive_path)?;

    // Get file info by reading the file
    let file_data = archive
        .read_file(filename)
        .context(format!("File not found: {}", filename))?;
    let file_size = file_data.len() as u64;

    print_file_info(filename, file_size, global_opts.output)?;

    Ok(())
}
