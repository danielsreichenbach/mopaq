//! Extract command implementation

use anyhow::{Context, Result};
use colored::*;
use mopaq::Archive;
use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

/// Extract files from an MPQ archive
pub fn extract(archive_path: &str, output_dir: &str, specific_file: Option<&str>) -> Result<()> {
    println!("{}: {}", "Opening archive".bold(), archive_path.cyan());
    let mut archive = Archive::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path))?;

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    if let Some(filename) = specific_file {
        // Extract specific file
        extract_single_file(&mut archive, filename, output_dir)
    } else {
        // Extract all files
        extract_all_files(&mut archive, output_dir)
    }
}

/// Extract a single file from the archive
fn extract_single_file(archive: &mut Archive, filename: &str, output_dir: &str) -> Result<()> {
    println!("{}: {}", "Extracting file".bold(), filename.cyan());

    let data = archive
        .read_file(filename)
        .with_context(|| format!("Failed to read file '{}' from archive", filename))?;

    // Convert the archive path to OS-appropriate path
    let output_path = build_output_path(output_dir, filename);

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory for: {:?}", output_path))?;
    }

    fs::write(&output_path, &data)
        .with_context(|| format!("Failed to write file: {:?}", output_path))?;

    println!(
        "{} Extracted {} ({}) to {:?}",
        "✓".green().bold(),
        filename.cyan(),
        format_size(data.len() as u64).yellow(),
        output_path
    );
    Ok(())
}

/// Extract all files from the archive
fn extract_all_files(archive: &mut Archive, output_dir: &str) -> Result<()> {
    println!("Extracting all files to: {}", output_dir);

    // First, try to find and extract (listfile)
    match archive.find_file("(listfile)")? {
        Some(_) => extract_using_listfile(archive, output_dir),
        None => {
            eprintln!("No (listfile) found in archive");
            eprintln!("Cannot extract all files without knowing their names");
            eprintln!("Try extracting specific files with -f option");
            eprintln!();
            eprintln!("Alternatively, you can:");
            eprintln!("  1. Use 'storm-cli list' to see what files can be detected");
            eprintln!("  2. Extract known files individually with -f");
            eprintln!("  3. Use 'storm-cli debug tables' to inspect the archive structure");

            Err(anyhow::anyhow!(
                "Cannot extract all files without a listfile"
            ))
        }
    }
}

/// Extract files using the (listfile) content
fn extract_using_listfile(archive: &mut Archive, output_dir: &str) -> Result<()> {
    // Read and parse the listfile
    let listfile_data = archive
        .read_file("(listfile)")
        .context("Failed to read (listfile)")?;

    let filenames = mopaq::special_files::parse_listfile(&listfile_data)
        .context("Failed to parse (listfile)")?;

    if filenames.is_empty() {
        eprintln!("Warning: (listfile) is empty or contains no valid entries");
        return Ok(());
    }

    println!(
        "{} {} files in (listfile)",
        "Found".green(),
        filenames.len().to_string().bright_blue()
    );
    println!();

    let mut extracted_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;

    for filename in &filenames {
        // Skip the listfile itself
        if filename == "(listfile)" {
            skipped_count += 1;
            continue;
        }

        let output_path = build_output_path(output_dir, filename);

        print!("Extracting {} ", filename.cyan());
        if filename.contains('\\') || filename.contains('/') {
            print!("→ {} ", output_path.display().to_string().dimmed());
        }
        print!("... ");

        match extract_file_safe(archive, filename, output_dir) {
            Ok(size) => {
                println!(
                    "{} ({})",
                    "OK".green().bold(),
                    format_size(size as u64).dimmed()
                );
                extracted_count += 1;
            }
            Err(e) => {
                println!("{}: {}", "FAILED".red().bold(), e.to_string().red());
                failed_count += 1;
            }
        }
    }

    // Summary
    println!();
    println!("{}", "Extraction complete:".bold().underline());
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
