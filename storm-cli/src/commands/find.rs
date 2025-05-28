//! Find command implementation

use anyhow::{Context, Result};
use colored::*;
use mopaq::{tables::BlockEntry, Archive};

/// Find a specific file in an MPQ archive
pub fn find(archive_path: &str, filename: &str, verbose: bool) -> Result<()> {
    let archive = Archive::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path))?;

    println!(
        "{} '{}' in archive: {}",
        "Searching for".bold(),
        filename.cyan(),
        archive_path.bright_blue()
    );
    println!();

    match archive.find_file(filename)? {
        Some(file_info) => {
            println!("{} File found!", "✓".green().bold());
            println!();

            // Basic information
            println!("{}", "File Information:".bold().underline());
            println!("  {}: {}", "Filename".bold(), file_info.filename.cyan());
            println!(
                "  {}: {}",
                "Hash table index".bold(),
                file_info.hash_index.to_string().bright_blue()
            );
            println!(
                "  {}: {}",
                "Block table index".bold(),
                file_info.block_index.to_string().bright_blue()
            );
            println!(
                "  {}: {} ({})",
                "File size".bold(),
                file_info.file_size.to_string().green(),
                format_size(file_info.file_size).dimmed()
            );
            println!(
                "  {}: {} ({})",
                "Compressed size".bold(),
                file_info.compressed_size.to_string().yellow(),
                format_size(file_info.compressed_size).dimmed()
            );

            // Compression ratio
            if file_info.file_size > 0 {
                let ratio = 100.0 * file_info.compressed_size as f64 / file_info.file_size as f64;
                let ratio_colored = if ratio < 50.0 {
                    format!("{:.1}%", ratio).green()
                } else if ratio < 80.0 {
                    format!("{:.1}%", ratio).yellow()
                } else {
                    format!("{:.1}%", ratio).red()
                };
                println!("  {}: {}", "Compression ratio".bold(), ratio_colored);
            }

            // File position
            println!(
                "  {}: {} (offset {} in archive)",
                "File position".bold(),
                format!("0x{:08X}", file_info.file_pos).bright_magenta(),
                (file_info.file_pos - archive.archive_offset())
                    .to_string()
                    .dimmed()
            );

            // Locale information
            println!(
                "  {}: {} ({})",
                "Locale".bold(),
                format_locale(file_info.locale),
                format!("0x{:04X}", file_info.locale).dimmed()
            );

            // Flags
            println!(
                "  {}: {}",
                "Flags".bold(),
                format!("0x{:08X}", file_info.flags).bright_magenta()
            );
            if file_info.is_compressed() {
                println!("    {} COMPRESSED", "-".dimmed());
            }
            if file_info.is_encrypted() {
                println!("    {} ENCRYPTED", "-".dimmed());
                if file_info.has_fix_key() {
                    println!("    {} FIX_KEY", "-".dimmed());
                }
            }
            if file_info.is_single_unit() {
                println!("    {} SINGLE_UNIT", "-".dimmed());
            }
            if file_info.has_sector_crc() {
                println!("    {} SECTOR_CRC", "-".dimmed());
            }
            if file_info.flags & BlockEntry::FLAG_PATCH_FILE != 0 {
                println!("    {} PATCH_FILE", "-".dimmed());
            }

            if verbose {
                println!();
                println!("Additional Details:");

                // Get block table entry for more details
                if let Some(block_table) = archive.block_table() {
                    if let Some(block_entry) = block_table.get(file_info.block_index) {
                        println!("  Block table entry:");
                        println!("    Raw file position: 0x{:08X}", block_entry.file_pos);
                        println!(
                            "    Raw compressed size: 0x{:08X}",
                            block_entry.compressed_size
                        );
                        println!("    Raw file size: 0x{:08X}", block_entry.file_size);
                        println!("    Raw flags: 0x{:08X}", block_entry.flags);
                    }
                }

                // Hash values
                if let Some(hash_table) = archive.hash_table() {
                    if let Some(hash_entry) = hash_table.get(file_info.hash_index) {
                        println!();
                        println!("  Hash table entry:");
                        println!("    Name hash A: 0x{:08X}", hash_entry.name_1);
                        println!("    Name hash B: 0x{:08X}", hash_entry.name_2);
                        println!(
                            "    Platform: {} (0x{:04X})",
                            format_platform(hash_entry.platform),
                            hash_entry.platform
                        );
                    }
                }

                // Calculate expected hash values
                use mopaq::{hash_string, hash_type};
                println!();
                println!("  Expected hash values:");
                println!(
                    "    TABLE_OFFSET: 0x{:08X}",
                    hash_string(filename, hash_type::TABLE_OFFSET)
                );
                println!(
                    "    NAME_A: 0x{:08X}",
                    hash_string(filename, hash_type::NAME_A)
                );
                println!(
                    "    NAME_B: 0x{:08X}",
                    hash_string(filename, hash_type::NAME_B)
                );
                println!(
                    "    FILE_KEY: 0x{:08X}",
                    hash_string(filename, hash_type::FILE_KEY)
                );

                // Sector information
                if !file_info.is_single_unit() && file_info.file_size > 0 {
                    let sector_size = archive.header().sector_size();
                    let sector_count =
                        (file_info.file_size as usize + sector_size - 1) / sector_size;
                    println!();
                    println!("  Sector information:");
                    println!("    Sector size: {} bytes", sector_size);
                    println!("    Number of sectors: {}", sector_count);
                    println!(
                        "    Sector offset table size: {} bytes",
                        (sector_count + 1) * 4
                    );
                }
            }

            Ok(())
        }
        None => {
            println!(
                "{} File '{}' not found in archive",
                "✗".red().bold(),
                filename
            );

            if verbose {
                // Show hash calculation details
                use mopaq::{hash_string, hash_type};
                let hash_offset = hash_string(filename, hash_type::TABLE_OFFSET);
                let hash_a = hash_string(filename, hash_type::NAME_A);
                let hash_b = hash_string(filename, hash_type::NAME_B);

                println!();
                println!("Hash lookup details:");
                println!("  Filename: {}", filename);
                println!("  TABLE_OFFSET hash: 0x{:08X}", hash_offset);
                println!("  NAME_A hash: 0x{:08X}", hash_a);
                println!("  NAME_B hash: 0x{:08X}", hash_b);

                if let Some(hash_table) = archive.hash_table() {
                    let initial_index = hash_offset as usize & (hash_table.size() - 1);
                    println!(
                        "  Initial hash table index: {} (table size: {})",
                        initial_index,
                        hash_table.size()
                    );
                }

                println!();
                println!("Possible reasons:");
                println!("  - File doesn't exist in the archive");
                println!("  - Filename case mismatch (try different case)");
                println!("  - Wrong path separator (use \\ instead of /)");
                println!("  - File was deleted from archive");
            }

            Err(anyhow::anyhow!("File '{}' not found in archive", filename))
        }
    }
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

/// Format locale code
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
        0x0416 => "Portuguese (BR)",
        0x0816 => "Portuguese (PT)",
        0x040a => "Spanish (ES)",
        0x080a => "Spanish (MX)",
        _ => "Unknown",
    }
}

/// Format platform code
fn format_platform(platform: u16) -> &'static str {
    match platform {
        0 => "Default",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_locale() {
        assert_eq!(format_locale(0x0000), "Neutral/Default");
        assert_eq!(format_locale(0x0409), "English (US)");
        assert_eq!(format_locale(0x0407), "German");
        assert_eq!(format_locale(0xFFFF), "Unknown");
    }
}
