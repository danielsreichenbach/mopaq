//! Extract command implementation

use anyhow::{Context, Result};
use mopaq::Archive;
use std::fs;
use std::path::Path;

/// Extract files from an MPQ archive
pub fn extract(archive_path: &str, output_dir: &str, specific_file: Option<&str>) -> Result<()> {
    println!("Opening archive: {}", archive_path);
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
    println!("Extracting file: {}", filename);

    let data = archive
        .read_file(filename)
        .with_context(|| format!("Failed to read file '{}' from archive", filename))?;

    let output_path = Path::new(output_dir).join(filename);

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory for: {:?}", output_path))?;
    }

    fs::write(&output_path, &data)
        .with_context(|| format!("Failed to write file: {:?}", output_path))?;

    println!(
        "Extracted {} ({} bytes) to {:?}",
        filename,
        data.len(),
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

    let listfile_content = String::from_utf8_lossy(&listfile_data);
    let filenames = parse_listfile(&listfile_content);

    if filenames.is_empty() {
        eprintln!("Warning: (listfile) is empty or contains no valid entries");
        return Ok(());
    }

    println!("Found {} files in (listfile)", filenames.len());
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

        print!("Extracting {}... ", filename);

        match extract_file_safe(archive, filename, output_dir) {
            Ok(size) => {
                println!("OK ({} bytes)", size);
                extracted_count += 1;
            }
            Err(e) => {
                println!("FAILED: {}", e);
                failed_count += 1;
            }
        }
    }

    // Summary
    println!();
    println!("Extraction complete:");
    println!("  Files extracted: {}", extracted_count);
    println!("  Files failed: {}", failed_count);
    println!("  Files skipped: {}", skipped_count);

    if failed_count > 0 {
        println!();
        println!("Note: Some files failed to extract. Common reasons:");
        println!("  - File is referenced in (listfile) but not actually present");
        println!("  - File uses unsupported compression (PKWare, Huffman, etc.)");
        println!("  - File is corrupted or has invalid data");
    }

    Ok(())
}

/// Safely extract a file, returning the file size on success
fn extract_file_safe(archive: &mut Archive, filename: &str, output_dir: &str) -> Result<usize> {
    let data = archive.read_file(filename)?;
    let output_path = Path::new(output_dir).join(filename);

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, &data)?;
    Ok(data.len())
}

/// Parse a listfile into individual filenames
fn parse_listfile(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| {
            // Skip empty lines and comments
            !line.is_empty() && !line.starts_with(';') && !line.starts_with('#')
        })
        .map(|line| {
            // Handle different listfile formats
            // Some listfiles use semicolon as separator
            if let Some(pos) = line.find(';') {
                line[..pos].trim().to_string()
            } else {
                line.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_listfile() {
        let content = r#"
; This is a comment
file1.txt
file2.dat
# Another comment
dir/file3.bin

file4.txt;1234
; file5.txt - this is commented out
        "#;

        let files = parse_listfile(content);
        assert_eq!(files.len(), 4);
        assert_eq!(files[0], "file1.txt");
        assert_eq!(files[1], "file2.dat");
        assert_eq!(files[2], "dir/file3.bin");
        assert_eq!(files[3], "file4.txt");
    }

    #[test]
    fn test_parse_empty_listfile() {
        let content = r#"
; Only comments
# More comments
        "#;

        let files = parse_listfile(content);
        assert_eq!(files.len(), 0);
    }
}
