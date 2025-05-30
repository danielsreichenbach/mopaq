//! List command implementation

use crate::{output, OutputFormat, GLOBAL_OPTS};
use anyhow::{Context, Result};
use colored::*;
use mopaq::{tables::BlockEntry, Archive};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct FileListEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hash_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name_hash_a: Option<String>, // Hex format
    #[serde(skip_serializing_if = "Option::is_none")]
    name_hash_b: Option<String>, // Hex format
    #[serde(skip_serializing_if = "Option::is_none")]
    locale: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<u16>,
    size: u64,
    compressed_size: u64,
    compression_ratio: f64,
    flags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ArchiveInfo {
    path: String,
    mode: String, // "listfile" or "table_entries"
    total_entries: usize,
    entries: Vec<FileListEntry>,
}

/// List files in an MPQ archive
pub fn list(archive_path: &str, verbose: bool, show_all: bool) -> Result<()> {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");
    let mut archive = Archive::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path))?;

    // For JSON/CSV output, collect all data first
    if matches!(opts.output, OutputFormat::Json | OutputFormat::Csv) {
        return list_structured(&mut archive, archive_path, verbose, show_all);
    }

    // Text output
    if output::use_color() {
        println!("{}: {}", "Archive".bold(), archive_path.cyan());
    } else {
        println!("Archive: {}", archive_path);
    }

    // Determine listing mode based on show_all flag
    if show_all {
        // Show all entries from tables (regardless of listfile existence)
        if verbose {
            list_all_entries_verbose(&archive)?;
        } else {
            list_all_entries(&archive)?;
        }
    } else {
        // Only use listfile - error if not found
        match archive.find_file("(listfile)")? {
            Some(_) => list_using_listfile(&mut archive, verbose)?,
            None => {
                println!(
                    "{} {}",
                    "⚠".yellow(),
                    "No (listfile) found in archive".yellow()
                );
                println!();
                println!("This archive does not contain a file list.");
                println!(
                    "Use {} to show all entries by their hash table index.",
                    "--all".cyan()
                );

                // Return early - don't list anything
                return Ok(());
            }
        }
    }

    Ok(())
}

/// List files in a structured format (JSON/CSV)
fn list_structured(
    archive: &mut Archive,
    archive_path: &str,
    verbose: bool,
    show_all: bool,
) -> Result<()> {
    let mut entries = Vec::new();
    let mode: String;

    if show_all {
        // Show all entries from tables
        mode = "table_entries".to_string();

        // Check if we have HET/BET tables first (v3+ archives)
        if let (Some(het), Some(bet)) = (archive.het_table(), archive.bet_table()) {
            if het.header.max_file_count > 0 && bet.header.file_count > 0 {
                // Use HET/BET tables
                for i in 0..bet.header.file_count {
                    if let Some(bet_info) = bet.get_file_info(i) {
                        // Only include files that actually exist
                        if bet_info.flags & BlockEntry::FLAG_EXISTS != 0 {
                            let ratio = if bet_info.file_size > 0 {
                                100.0 * bet_info.compressed_size as f64 / bet_info.file_size as f64
                            } else {
                                100.0
                            };

                            entries.push(FileListEntry {
                                filename: None,
                                hash_index: None, // Not applicable for HET/BET
                                block_index: Some(i as usize),
                                name_hash_a: None, // HET uses Jenkins hash
                                name_hash_b: None,
                                locale: None, // Not stored in HET/BET
                                platform: None,
                                size: bet_info.file_size,
                                compressed_size: bet_info.compressed_size,
                                compression_ratio: ratio,
                                flags: format_file_flags_vec(bet_info.flags),
                            });
                        }
                    }
                }
            }
        } else {
            // Fall back to hash/block tables
            let hash_table = archive
                .hash_table()
                .ok_or_else(|| anyhow::anyhow!("Hash table not loaded"))?;
            let block_table = archive
                .block_table()
                .ok_or_else(|| anyhow::anyhow!("Block table not loaded"))?;

            for (i, hash_entry) in hash_table.entries().iter().enumerate() {
                if hash_entry.is_valid() {
                    if let Some(block_entry) = block_table.get(hash_entry.block_index as usize) {
                        if block_entry.exists() {
                            let ratio = if block_entry.file_size > 0 {
                                100.0 * block_entry.compressed_size as f64
                                    / block_entry.file_size as f64
                            } else {
                                100.0
                            };

                            entries.push(FileListEntry {
                                filename: None,
                                hash_index: Some(i),
                                block_index: Some(hash_entry.block_index as usize),
                                name_hash_a: Some(format!("0x{:08X}", hash_entry.name_1)),
                                name_hash_b: Some(format!("0x{:08X}", hash_entry.name_2)),
                                locale: Some(hash_entry.locale),
                                platform: Some(hash_entry.platform),
                                size: block_entry.file_size as u64,
                                compressed_size: block_entry.compressed_size as u64,
                                compression_ratio: ratio,
                                flags: format_file_flags_vec(block_entry.flags),
                            });
                        }
                    }
                }
            }
        }
    } else {
        // Only use listfile
        mode = "listfile".to_string();

        // Check if listfile exists
        match archive.find_file("(listfile)")? {
            Some(_) => {
                let listfile_data = archive
                    .read_file("(listfile)")
                    .context("Failed to read (listfile)")?;

                let filenames = mopaq::special_files::parse_listfile(&listfile_data)
                    .context("Failed to parse (listfile)")?;

                for filename in &filenames {
                    if let Ok(Some(file_info)) = archive.find_file(filename) {
                        // If verbose, also get hash table info
                        let (hash_info, block_idx) = if verbose {
                            if let Some(hash_table) = archive.hash_table() {
                                let mut found_hash = None;
                                let mut found_block_idx = None;

                                for (idx, entry) in hash_table.entries().iter().enumerate() {
                                    if entry.is_valid()
                                        && entry.block_index as usize == file_info.block_index
                                    {
                                        found_hash = Some((idx, entry));
                                        found_block_idx = Some(entry.block_index as usize);
                                        break;
                                    }
                                }
                                (found_hash, found_block_idx)
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        };

                        let ratio = if file_info.file_size > 0 {
                            100.0 * file_info.compressed_size as f64 / file_info.file_size as f64
                        } else {
                            100.0
                        };

                        entries.push(FileListEntry {
                            filename: Some(filename.clone()),
                            hash_index: hash_info.map(|(idx, _)| idx),
                            block_index: block_idx,
                            name_hash_a: hash_info
                                .map(|(_, entry)| format!("0x{:08X}", entry.name_1)),
                            name_hash_b: hash_info
                                .map(|(_, entry)| format!("0x{:08X}", entry.name_2)),
                            locale: hash_info.map(|(_, entry)| entry.locale),
                            platform: hash_info.map(|(_, entry)| entry.platform),
                            size: file_info.file_size,
                            compressed_size: file_info.compressed_size,
                            compression_ratio: ratio,
                            flags: format_file_flags_vec(file_info.flags),
                        });
                    }
                }
            }
            None => {
                // Return empty result with appropriate mode
                // This allows JSON/CSV output to show that no listfile was found
            }
        }
    }

    let archive_info = ArchiveInfo {
        path: archive_path.to_string(),
        mode,
        total_entries: entries.len(),
        entries,
    };

    output::print_output(&archive_info)?;
    Ok(())
}

/// List files using the (listfile)
fn list_using_listfile(archive: &mut Archive, verbose: bool) -> Result<()> {
    let listfile_data = archive
        .read_file("(listfile)")
        .context("Failed to read (listfile)")?;

    let filenames = mopaq::special_files::parse_listfile(&listfile_data)
        .context("Failed to parse (listfile)")?;

    if filenames.is_empty() {
        println!("{} {}", "⚠".yellow(), "(listfile) is empty".yellow());
        return Ok(());
    }

    println!("{}", "Files in archive:".bold());
    println!();

    if verbose {
        // Detailed listing with file information
        println!(
            "{:<50} {:>12} {:>12} {:>8} {:<20}",
            "Filename".bold().underline(),
            "Size".bold().underline(),
            "Compressed".bold().underline(),
            "Ratio".bold().underline(),
            "Flags".bold().underline()
        );

        for filename in &filenames {
            if let Ok(Some(file_info)) = archive.find_file(filename) {
                let ratio = if file_info.file_size > 0 {
                    let ratio_val =
                        100.0 * file_info.compressed_size as f64 / file_info.file_size as f64;
                    if ratio_val < 50.0 {
                        format!("{:.1}%", ratio_val).green()
                    } else if ratio_val < 80.0 {
                        format!("{:.1}%", ratio_val).yellow()
                    } else {
                        format!("{:.1}%", ratio_val).normal()
                    }
                } else {
                    "N/A".dimmed()
                };

                let flags = format_file_flags(file_info.flags);
                let flags_colored = if flags.contains("ENCRYPTED") {
                    flags.red()
                } else if flags.contains("COMPRESSED") {
                    flags.cyan()
                } else {
                    flags.normal()
                };

                println!(
                    "{:<50} {:>12} {:>12} {:>8} {:<20}",
                    filename.normal(),
                    format_size(file_info.file_size).bright_white(),
                    format_size(file_info.compressed_size).dimmed(),
                    ratio,
                    flags_colored
                );
            } else {
                println!("{:<50} {}", filename, "(not found in hash table)".red());
            }
        }
    } else {
        // Simple listing
        for filename in &filenames {
            println!("  {}", filename);
        }
    }

    println!();
    println!(
        "{}: {}",
        "Total files".bold(),
        filenames.len().to_string().green()
    );

    Ok(())
}

/// List all entries from the tables (without filenames)
fn list_all_entries(archive: &Archive) -> Result<()> {
    // Check if we have HET/BET tables first (v3+ archives)
    if let (Some(het), Some(bet)) = (archive.het_table(), archive.bet_table()) {
        if het.header.max_file_count > 0 && bet.header.file_count > 0 {
            println!(
                "{}",
                "All entries in archive (using HET/BET tables):".bold()
            );
            println!();

            println!(
                "{:<10} {:>12} {:>12} {:>8} {:<20}",
                "File Idx".bold().underline(),
                "Size".bold().underline(),
                "Compressed".bold().underline(),
                "Ratio".bold().underline(),
                "Flags".bold().underline()
            );

            let mut count = 0;
            for i in 0..bet.header.file_count {
                if let Some(bet_info) = bet.get_file_info(i) {
                    // Only include files that actually exist
                    if bet_info.flags & BlockEntry::FLAG_EXISTS != 0 {
                        let ratio = if bet_info.file_size > 0 {
                            let ratio_val =
                                100.0 * bet_info.compressed_size as f64 / bet_info.file_size as f64;
                            if ratio_val < 50.0 {
                                format!("{:.1}%", ratio_val).green()
                            } else if ratio_val < 80.0 {
                                format!("{:.1}%", ratio_val).yellow()
                            } else {
                                format!("{:.1}%", ratio_val).normal()
                            }
                        } else {
                            "N/A".dimmed()
                        };

                        let flags = format_file_flags(bet_info.flags);
                        let flags_colored = if flags.contains("ENCRYPTED") {
                            flags.red()
                        } else if flags.contains("COMPRESSED") {
                            flags.cyan()
                        } else {
                            flags.normal()
                        };

                        println!(
                            "{:<10} {:>12} {:>12} {:>8} {:<20}",
                            format!("#{}", i).bright_blue(),
                            format_size(bet_info.file_size).bright_white(),
                            format_size(bet_info.compressed_size).dimmed(),
                            ratio,
                            flags_colored
                        );
                        count += 1;
                    }
                }
            }

            println!();
            println!("{}: {}", "Total entries".bold(), count.to_string().green());

            // Note that HET/BET tables don't store locale/platform
            println!();
            println!(
                "{}",
                "Note: HET/BET tables don't store locale/platform data. Use debug tables command for more details."
                    .dimmed()
            );

            return Ok(());
        }
    }

    // Fall back to hash/block tables
    let hash_table = archive
        .hash_table()
        .ok_or_else(|| anyhow::anyhow!("Hash table not loaded"))?;
    let block_table = archive
        .block_table()
        .ok_or_else(|| anyhow::anyhow!("Block table not loaded"))?;

    println!("{}", "All entries in archive (by index):".bold());
    println!();

    // Updated header to include hash values
    println!(
        "{:<8} {:<10} {:<10} {:>12} {:>12} {:>8} {:<20} {:<10}",
        "Hash Idx".bold().underline(),
        "Name A".bold().underline(),
        "Name B".bold().underline(),
        "Size".bold().underline(),
        "Compressed".bold().underline(),
        "Ratio".bold().underline(),
        "Flags".bold().underline(),
        "Block Idx".bold().underline()
    );

    let mut count = 0;
    for (i, hash_entry) in hash_table.entries().iter().enumerate() {
        if hash_entry.is_valid() {
            if let Some(block_entry) = block_table.get(hash_entry.block_index as usize) {
                if block_entry.exists() {
                    let ratio = if block_entry.file_size > 0 {
                        let ratio_val = 100.0 * block_entry.compressed_size as f64
                            / block_entry.file_size as f64;
                        if ratio_val < 50.0 {
                            format!("{:.1}%", ratio_val).green()
                        } else if ratio_val < 80.0 {
                            format!("{:.1}%", ratio_val).yellow()
                        } else {
                            format!("{:.1}%", ratio_val).normal()
                        }
                    } else {
                        "N/A".dimmed()
                    };

                    let flags = format_file_flags(block_entry.flags);
                    let flags_colored = if flags.contains("ENCRYPTED") {
                        flags.red()
                    } else if flags.contains("COMPRESSED") {
                        flags.cyan()
                    } else {
                        flags.normal()
                    };

                    println!(
                        "{:<8} {:<10} {:<10} {:>12} {:>12} {:>8} {:<20} {:<10}",
                        format!("#{}", i).bright_blue(),
                        format!("0x{:08X}", hash_entry.name_1).bright_magenta(),
                        format!("0x{:08X}", hash_entry.name_2).bright_magenta(),
                        format_size(block_entry.file_size as u64).bright_white(),
                        format_size(block_entry.compressed_size as u64).dimmed(),
                        ratio,
                        flags_colored,
                        format!("#{}", hash_entry.block_index).dimmed()
                    );
                    count += 1;
                }
            }
        }
    }

    println!();
    println!("{}: {}", "Total entries".bold(), count.to_string().green());

    // Add a note about locale/platform if any entries have non-zero values
    let has_locale_platform = hash_table
        .entries()
        .iter()
        .any(|e| e.is_valid() && (e.locale != 0 || e.platform != 0));

    if has_locale_platform {
        println!();
        println!(
            "{}",
            "Note: Some entries have locale/platform values. Use verbose mode to see details."
                .dimmed()
        );
    }

    Ok(())
}

/// List all entries from the tables with verbose information
fn list_all_entries_verbose(archive: &Archive) -> Result<()> {
    let hash_table = archive
        .hash_table()
        .ok_or_else(|| anyhow::anyhow!("Hash table not loaded"))?;
    let block_table = archive
        .block_table()
        .ok_or_else(|| anyhow::anyhow!("Block table not loaded"))?;

    println!("{}", "All entries in archive (detailed view):".bold());
    println!();

    let mut count = 0;
    for (i, hash_entry) in hash_table.entries().iter().enumerate() {
        if hash_entry.is_valid() {
            if let Some(block_entry) = block_table.get(hash_entry.block_index as usize) {
                if block_entry.exists() {
                    // Print hash table entry info
                    println!("{}", format!("Hash Entry #{}", i).bright_blue().bold());
                    println!(
                        "  {}: {}",
                        "Name Hash A".bold(),
                        format!("0x{:08X}", hash_entry.name_1).bright_magenta()
                    );
                    println!(
                        "  {}: {}",
                        "Name Hash B".bold(),
                        format!("0x{:08X}", hash_entry.name_2).bright_magenta()
                    );
                    println!(
                        "  {}: {} ({})",
                        "Locale".bold(),
                        hash_entry.locale,
                        format_locale(hash_entry.locale)
                    );
                    println!("  {}: {}", "Platform".bold(), hash_entry.platform);
                    println!("  {}: {}", "Block Index".bold(), hash_entry.block_index);

                    // Print block table entry info
                    println!("  {}:", "Block Entry".bold());
                    println!(
                        "    {}: {} ({})",
                        "File Size".bold(),
                        block_entry.file_size,
                        format_size(block_entry.file_size as u64)
                    );
                    println!(
                        "    {}: {} ({})",
                        "Compressed Size".bold(),
                        block_entry.compressed_size,
                        format_size(block_entry.compressed_size as u64)
                    );

                    let ratio = if block_entry.file_size > 0 {
                        100.0 * block_entry.compressed_size as f64 / block_entry.file_size as f64
                    } else {
                        100.0
                    };
                    println!("    {}: {:.1}%", "Compression Ratio".bold(), ratio);

                    println!(
                        "    {}: 0x{:08X} ({})",
                        "Flags".bold(),
                        block_entry.flags,
                        format_file_flags(block_entry.flags)
                    );

                    println!();
                    count += 1;
                }
            }
        }
    }

    println!(
        "{}: {}",
        "Total valid entries".bold(),
        count.to_string().green()
    );

    Ok(())
}

// Helper function to format locale
fn format_locale(locale: u16) -> &'static str {
    match locale {
        0x0000 => "Neutral/Default",
        0x0409 => "English (US)",
        0x0809 => "English (UK)",
        0x0407 => "German",
        0x040c => "French",
        0x0410 => "Italian",
        0x0405 => "Czech",
        0x0411 => "Japanese",
        0x0412 => "Korean",
        0x0404 => "Chinese (Traditional)",
        0x0804 => "Chinese (Simplified)",
        0x0419 => "Russian",
        0x0415 => "Polish",
        _ => "Unknown",
    }
}

/// Format file flags as a vector of strings
fn format_file_flags_vec(flags: u32) -> Vec<String> {
    let mut parts = Vec::new();

    if flags & BlockEntry::FLAG_COMPRESS != 0 {
        parts.push("COMPRESSED".to_string());
    }
    if flags & BlockEntry::FLAG_ENCRYPTED != 0 {
        parts.push("ENCRYPTED".to_string());
    }
    if flags & BlockEntry::FLAG_FIX_KEY != 0 {
        parts.push("FIX_KEY".to_string());
    }
    if flags & BlockEntry::FLAG_SINGLE_UNIT != 0 {
        parts.push("SINGLE_UNIT".to_string());
    }
    if flags & BlockEntry::FLAG_PATCH_FILE != 0 {
        parts.push("PATCH".to_string());
    }
    if flags & BlockEntry::FLAG_SECTOR_CRC != 0 {
        parts.push("CRC".to_string());
    }

    if parts.is_empty() {
        parts.push("NONE".to_string());
    }

    parts
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
