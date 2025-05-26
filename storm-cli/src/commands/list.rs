//! List command implementation

use anyhow::{Context, Result};
use mopaq::{tables::BlockEntry, Archive};

/// List files in an MPQ archive
pub fn list(archive_path: &str, verbose: bool, show_all: bool) -> Result<()> {
    let mut archive = Archive::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path))?;

    println!("Archive: {}", archive_path);

    // Try to list using (listfile) first
    if let Ok(Some(_)) = archive.find_file("(listfile)") {
        list_using_listfile(&mut archive, verbose)?;
    } else if show_all {
        // Show all entries from tables even without filenames
        list_all_entries(&archive)?;
    } else {
        println!("No (listfile) found in archive");
        println!("Use --all to show all entries by index");
    }

    Ok(())
}

/// List files using the (listfile)
fn list_using_listfile(archive: &mut Archive, verbose: bool) -> Result<()> {
    let listfile_data = archive
        .read_file("(listfile)")
        .context("Failed to read (listfile)")?;

    let listfile_content = String::from_utf8_lossy(&listfile_data);
    let filenames = parse_listfile(&listfile_content);

    if filenames.is_empty() {
        println!("(listfile) is empty");
        return Ok(());
    }

    println!("Files in archive:");
    println!();

    if verbose {
        // Detailed listing with file information
        println!(
            "{:<50} {:>12} {:>12} {:>8} {:<20}",
            "Filename", "Size", "Compressed", "Ratio", "Flags"
        );
        println!("{}", "-".repeat(104));

        for filename in &filenames {
            if let Ok(Some(file_info)) = archive.find_file(filename) {
                let ratio = if file_info.file_size > 0 {
                    format!(
                        "{:.1}%",
                        100.0 * file_info.compressed_size as f64 / file_info.file_size as f64
                    )
                } else {
                    "N/A".to_string()
                };

                let flags = format_file_flags(file_info.flags);

                println!(
                    "{:<50} {:>12} {:>12} {:>8} {:<20}",
                    filename,
                    format_size(file_info.file_size),
                    format_size(file_info.compressed_size),
                    ratio,
                    flags
                );
            } else {
                println!("{:<50} (not found in hash table)", filename);
            }
        }
    } else {
        // Simple listing
        for filename in &filenames {
            println!("{}", filename);
        }
    }

    println!();
    println!("Total files: {}", filenames.len());

    Ok(())
}

/// List all entries from the tables (without filenames)
fn list_all_entries(archive: &Archive) -> Result<()> {
    let hash_table = archive
        .hash_table()
        .ok_or_else(|| anyhow::anyhow!("Hash table not loaded"))?;
    let block_table = archive
        .block_table()
        .ok_or_else(|| anyhow::anyhow!("Block table not loaded"))?;

    println!("All entries in archive (by index):");
    println!();
    println!(
        "{:<10} {:>12} {:>12} {:>8} {:<20}",
        "Index", "Size", "Compressed", "Ratio", "Flags"
    );
    println!("{}", "-".repeat(66));

    let mut count = 0;
    for (i, hash_entry) in hash_table.entries().iter().enumerate() {
        if hash_entry.is_valid() {
            if let Some(block_entry) = block_table.get(hash_entry.block_index as usize) {
                if block_entry.exists() {
                    let ratio = if block_entry.file_size > 0 {
                        format!(
                            "{:.1}%",
                            100.0 * block_entry.compressed_size as f64
                                / block_entry.file_size as f64
                        )
                    } else {
                        "N/A".to_string()
                    };

                    let flags = format_file_flags(block_entry.flags);

                    println!(
                        "{:<10} {:>12} {:>12} {:>8} {:<20}",
                        format!("#{}", i),
                        format_size(block_entry.file_size as u64),
                        format_size(block_entry.compressed_size as u64),
                        ratio,
                        flags
                    );
                    count += 1;
                }
            }
        }
    }

    println!();
    println!("Total entries: {}", count);

    Ok(())
}

/// Parse a listfile into individual filenames
fn parse_listfile(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with(';') && !line.starts_with('#'))
        .map(|line| {
            if let Some(pos) = line.find(';') {
                line[..pos].trim().to_string()
            } else {
                line.to_string()
            }
        })
        .collect()
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

/// Format file flags as a readable string
fn format_file_flags(flags: u32) -> String {
    let mut parts = Vec::new();

    if flags & BlockEntry::FLAG_COMPRESS != 0 {
        parts.push("COMPRESSED");
    }
    if flags & BlockEntry::FLAG_ENCRYPTED != 0 {
        parts.push("ENCRYPTED");
    }
    if flags & BlockEntry::FLAG_FIX_KEY != 0 {
        parts.push("FIX_KEY");
    }
    if flags & BlockEntry::FLAG_SINGLE_UNIT != 0 {
        parts.push("SINGLE_UNIT");
    }
    if flags & BlockEntry::FLAG_PATCH_FILE != 0 {
        parts.push("PATCH");
    }
    if flags & BlockEntry::FLAG_SECTOR_CRC != 0 {
        parts.push("CRC");
    }

    if parts.is_empty() {
        "NONE".to_string()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_format_file_flags() {
        assert_eq!(format_file_flags(0x80000000), "NONE");
        assert_eq!(format_file_flags(0x80000200), "COMPRESSED");
        assert_eq!(format_file_flags(0x80010200), "COMPRESSED, ENCRYPTED");
    }
}
