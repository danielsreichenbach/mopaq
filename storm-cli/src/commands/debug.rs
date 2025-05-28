//! Debug commands for MPQ archives

use anyhow::Result;
use colored::*;
use mopaq::hash::{hash_string, hash_type, jenkins_hash};
use mopaq::{Archive, FormatVersion};
use std::path::Path;

/// Display detailed information about an MPQ archive
pub fn info(archive_path: &str) -> Result<()> {
    println!("{}", "MPQ Archive Information".bold());
    println!();

    // Try to open the archive
    let archive = Archive::open(archive_path)?;
    let header = archive.header();

    // Basic information
    println!("File: {}", archive_path.cyan());
    println!(
        "  {}: {} ({} bytes)",
        "Archive offset".bright_blue(),
        format!("0x{:08X}", archive.archive_offset()).bright_magenta(),
        archive.archive_offset().to_string().dimmed()
    );

    // User data information
    if let Some(user_data) = archive.user_data() {
        println!();
        println!("User Data Header:");
        println!("  User data size: {} bytes", user_data.user_data_size);
        println!("  Header offset: 0x{:08X}", user_data.header_offset);
        println!(
            "  User data header size: {} bytes",
            user_data.user_data_header_size
        );
    }

    // Header information
    println!();
    println!("{}", "MPQ Header".bold());
    println!(
        "  Format version: {} ({})",
        header.format_version as u16,
        format_version_name(header.format_version)
    );
    println!("  Header size: {} bytes", header.header_size);
    println!("  Archive size: {} bytes", header.get_archive_size());
    println!(
        "  Block size: {} (sector size: {} bytes)",
        header.block_size,
        header.sector_size()
    );

    // Table information
    println!();
    println!("{}", "Tables".bold());
    println!("  Hash table:");
    println!("    Position: 0x{:08X}", header.get_hash_table_pos());
    println!(
        "    Entries: {} (must be power of 2)",
        header.hash_table_size
    );

    println!("  Block table:");
    println!("    Position: 0x{:08X}", header.get_block_table_pos());
    println!("    Entries: {}", header.block_table_size);

    // Version-specific information
    if header.format_version as u16 >= 1 {
        if let Some(hi_pos) = header.hi_block_table_pos {
            if hi_pos != 0 {
                println!("  Hi-block table:");
                println!("    Position: 0x{:08X}", hi_pos);
            }
        }
    }

    if header.format_version as u16 >= 2 {
        if let Some(het_pos) = header.het_table_pos {
            if het_pos != 0 {
                println!("  HET table:");
                println!("    Position: 0x{:08X}", het_pos);
            }
        }

        if let Some(bet_pos) = header.bet_table_pos {
            if bet_pos != 0 {
                println!("  BET table:");
                println!("    Position: 0x{:08X}", bet_pos);
            }
        }
    }

    // Version 4 specific information
    if let Some(v4_data) = &header.v4_data {
        println!();
        println!("{}", "Version 4 Extended Data".bold());
        println!("  Compressed table sizes:");
        println!("    Hash table: {} bytes", v4_data.hash_table_size_64);
        println!("    Block table: {} bytes", v4_data.block_table_size_64);
        println!(
            "    Hi-block table: {} bytes",
            v4_data.hi_block_table_size_64
        );
        println!("    HET table: {} bytes", v4_data.het_table_size_64);
        println!("    BET table: {} bytes", v4_data.bet_table_size_64);
        println!("  Raw chunk size: {} bytes", v4_data.raw_chunk_size);

        println!();
        println!("  MD5 Checksums:");
        println!("    Block table: {}", hex_string(&v4_data.md5_block_table));
        println!("    Hash table: {}", hex_string(&v4_data.md5_hash_table));
        println!(
            "    Hi-block table: {}",
            hex_string(&v4_data.md5_hi_block_table)
        );
        println!("    BET table: {}", hex_string(&v4_data.md5_bet_table));
        println!("    HET table: {}", hex_string(&v4_data.md5_het_table));
        println!("    MPQ header: {}", hex_string(&v4_data.md5_mpq_header));
    }

    // File statistics (once implemented)
    // TODO: Add file count, compression ratio, etc.

    Ok(())
}

/// Test crypto functions
pub fn crypto() -> Result<()> {
    use mopaq::crypto::{decrypt_block, encrypt_block, ENCRYPTION_TABLE};

    println!("MPQ Crypto Test");
    println!("===============");
    println!();

    // Show some encryption table values
    println!("Encryption Table Sample Values:");
    println!("  [0x000]: 0x{:08X}", ENCRYPTION_TABLE[0x000]);
    println!("  [0x001]: 0x{:08X}", ENCRYPTION_TABLE[0x001]);
    println!("  [0x100]: 0x{:08X}", ENCRYPTION_TABLE[0x100]);
    println!("  [0x200]: 0x{:08X}", ENCRYPTION_TABLE[0x200]);
    println!("  [0x300]: 0x{:08X}", ENCRYPTION_TABLE[0x300]);
    println!("  [0x400]: 0x{:08X}", ENCRYPTION_TABLE[0x400]);
    println!("  [0x4FF]: 0x{:08X}", ENCRYPTION_TABLE[0x4FF]);

    // Test encryption/decryption
    println!();
    println!("Testing Encryption/Decryption:");

    let original_data = vec![
        0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0, 0xFEDCBA98, 0x76543210, 0xF0DEBC9A,
        0xE1C3A597,
    ];

    let key = 0xC1EB1CEF;

    println!("  Key: 0x{:08X}", key);
    println!();
    println!("  Original data:");
    for (i, &val) in original_data.iter().enumerate() {
        println!("    [{}]: 0x{:08X}", i, val);
    }

    // Encrypt
    let mut data = original_data.clone();
    encrypt_block(&mut data, key);

    println!();
    println!("  Encrypted data:");
    for (i, &val) in data.iter().enumerate() {
        println!("    [{}]: 0x{:08X}", i, val);
    }

    // Decrypt
    decrypt_block(&mut data, key);

    println!();
    println!("  Decrypted data:");
    for (i, &val) in data.iter().enumerate() {
        println!("    [{}]: 0x{:08X}", i, val);
    }

    // Verify round-trip
    if data == original_data {
        println!();
        println!("✓ Encryption/decryption round-trip successful!");
    } else {
        println!();
        println!("✗ Encryption/decryption round-trip failed!");
    }

    Ok(())
}

/// Generate hash values for a filename
pub fn hash(filename: &str, hash_type_name: Option<&str>, all: bool, jenkins: bool) -> Result<()> {
    if jenkins {
        // Generate Jenkins hash
        let hash = jenkins_hash(filename);
        println!("{} \"{}\":", "Jenkins hash for".bold(), filename.cyan());
        println!(
            "  {} (decimal: {})",
            format!("0x{:016X}", hash).bright_magenta(),
            hash.to_string().dimmed()
        );
        return Ok(());
    }

    if all {
        // Generate all hash types
        println!("{} \"{}\":", "Hash values for".bold(), filename.cyan());
        println!();

        let table_offset = hash_string(filename, hash_type::TABLE_OFFSET);
        let name_a = hash_string(filename, hash_type::NAME_A);
        let name_b = hash_string(filename, hash_type::NAME_B);
        let file_key = hash_string(filename, hash_type::FILE_KEY);
        let key2_mix = hash_string(filename, hash_type::KEY2_MIX);

        println!(
            "  {} (0): {} (decimal: {})",
            "TABLE_OFFSET".bright_blue(),
            format!("0x{:08X}", table_offset).bright_magenta(),
            table_offset.to_string().dimmed()
        );
        println!(
            "  {} (1): {} (decimal: {})",
            "NAME_A      ".bright_blue(),
            format!("0x{:08X}", name_a).bright_magenta(),
            name_a.to_string().dimmed()
        );
        println!(
            "  {} (2): {} (decimal: {})",
            "NAME_B      ".bright_blue(),
            format!("0x{:08X}", name_b).bright_magenta(),
            name_b.to_string().dimmed()
        );
        println!(
            "  {} (3): {} (decimal: {})",
            "FILE_KEY    ".bright_blue(),
            format!("0x{:08X}", file_key).bright_magenta(),
            file_key.to_string().dimmed()
        );
        println!(
            "  {} (4): {} (decimal: {})",
            "KEY2_MIX    ".bright_blue(),
            format!("0x{:08X}", key2_mix).bright_magenta(),
            key2_mix.to_string().dimmed()
        );

        println!();
        println!("{}:", "Hash table lookup".bold());
        println!("  {}:", "For a hash table of size 0x1000 (4096)".bold());
        println!(
            "  {}: {} (decimal: {})",
            "Initial index".bright_blue(),
            format!("0x{:04X}", table_offset & 0xFFF).bright_magenta(),
            (table_offset & 0xFFF).to_string().dimmed()
        );

        println!();
        println!("  {}:", "Hash entry would contain".bold());
        println!(
            "    {}: {}",
            "dwName1".bright_blue(),
            format!("0x{:08X}", name_a).bright_magenta()
        );
        println!(
            "    {}: {}",
            "dwName2".bright_blue(),
            format!("0x{:08X}", name_b).bright_magenta()
        );

        // Show path normalization if relevant
        if filename.contains('/') {
            let normalized = filename.replace('/', "\\");
            println!();
            println!(
                "Note: Path normalized from \"{}\" to \"{}\"",
                filename, normalized
            );
        }

        // Show case normalization
        let has_lowercase = filename.chars().any(|c| c.is_ascii_lowercase());
        if has_lowercase {
            println!();
            println!(
                "{}: {}",
                "Note".dimmed(),
                "Filename is case-insensitive (converted to uppercase for hashing)".dimmed()
            );
        }
    } else {
        // Generate specific hash type
        let hash_type_value = match hash_type_name {
            Some("table-offset") | Some("0") => hash_type::TABLE_OFFSET,
            Some("name-a") | Some("1") => hash_type::NAME_A,
            Some("name-b") | Some("2") => hash_type::NAME_B,
            Some("file-key") | Some("3") => hash_type::FILE_KEY,
            Some("key2-mix") | Some("4") => hash_type::KEY2_MIX,
            _ => {
                println!("Invalid hash type. Valid types are:");
                println!("  table-offset (0) - Hash table index calculation");
                println!("  name-a       (1) - First name hash");
                println!("  name-b       (2) - Second name hash");
                println!("  file-key     (3) - File encryption key");
                println!("  key2-mix     (4) - Secondary encryption key");
                return Ok(());
            }
        };

        let hash = hash_string(filename, hash_type_value);
        let type_name = match hash_type_value {
            0 => "TABLE_OFFSET",
            1 => "NAME_A",
            2 => "NAME_B",
            3 => "FILE_KEY",
            4 => "KEY2_MIX",
            _ => "UNKNOWN",
        };

        println!("Hash value for \"{}\" (type: {}):", filename, type_name);
        println!("  0x{:08X} (decimal: {})", hash, hash);
    }

    Ok(())
}

/// Compare hash values for two filenames
pub fn hash_compare(filename1: &str, filename2: &str) -> Result<()> {
    println!("Comparing hash values:");
    println!("  File 1: \"{}\"", filename1);
    println!("  File 2: \"{}\"", filename2);
    println!();

    // Calculate all hash types for both files
    let hashes1 = [
        hash_string(filename1, hash_type::TABLE_OFFSET),
        hash_string(filename1, hash_type::NAME_A),
        hash_string(filename1, hash_type::NAME_B),
        hash_string(filename1, hash_type::FILE_KEY),
        hash_string(filename1, hash_type::KEY2_MIX),
    ];

    let hashes2 = [
        hash_string(filename2, hash_type::TABLE_OFFSET),
        hash_string(filename2, hash_type::NAME_A),
        hash_string(filename2, hash_type::NAME_B),
        hash_string(filename2, hash_type::FILE_KEY),
        hash_string(filename2, hash_type::KEY2_MIX),
    ];

    let hash_names = ["TABLE_OFFSET", "NAME_A", "NAME_B", "FILE_KEY", "KEY2_MIX"];

    println!("MPQ Hash comparison:");
    println!("  Type          File 1        File 2        Match");
    println!("  ----------    ----------    ----------    -----");

    for i in 0..5 {
        let match_str = if hashes1[i] == hashes2[i] {
            "YES"
        } else {
            "NO"
        };
        println!(
            "  {:12}  0x{:08X}    0x{:08X}    {}",
            hash_names[i], hashes1[i], hashes2[i], match_str
        );
    }

    // Jenkins hash comparison
    let jenkins1 = jenkins_hash(filename1);
    let jenkins2 = jenkins_hash(filename2);
    let jenkins_match = if jenkins1 == jenkins2 { "YES" } else { "NO" };

    println!();
    println!("Jenkins hash comparison:");
    println!("  File 1: 0x{:016X}", jenkins1);
    println!("  File 2: 0x{:016X}", jenkins2);
    println!("  Match:  {}", jenkins_match);

    // Check if they would collide in hash table
    let table_sizes = [0x10, 0x100, 0x1000, 0x10000];
    println!();
    println!("Hash table collision check:");

    for &size in &table_sizes {
        let index1 = hashes1[0] & (size - 1);
        let index2 = hashes2[0] & (size - 1);
        let collision = if index1 == index2 {
            "COLLISION"
        } else {
            "No collision"
        };

        println!(
            "  Table size 0x{:04X}: {} (indices: 0x{:04X} vs 0x{:04X})",
            size, collision, index1, index2
        );
    }

    Ok(())
}

/// Display table contents from an MPQ archive
pub fn tables(archive_path: &str, table_type: Option<&str>, limit: Option<usize>) -> Result<()> {
    println!("MPQ Table Contents");
    println!("==================");
    println!();

    // Open the archive
    let mut archive = Archive::open(archive_path)?;

    // Load tables if not already loaded
    if archive.hash_table().is_none() {
        println!("Loading tables...");
        archive.load_tables()?;
    }

    let show_all = table_type.is_none();
    let limit = limit.unwrap_or(20);

    // Display hash table
    if show_all || table_type == Some("hash") {
        if let Some(hash_table) = archive.hash_table() {
            println!("Hash Table (size: {}):", hash_table.size());
            println!("Index | Name1      | Name2      | Locale | Platform | Block Index | Status");
            println!("------|------------|------------|--------|----------|-------------|--------");

            let entries = hash_table.entries();
            let count = if show_all {
                limit.min(entries.len())
            } else {
                entries.len()
            };

            let mut valid_count = 0;
            let mut deleted_count = 0;
            let mut empty_count = 0;

            for (i, entry) in entries.iter().enumerate() {
                if entry.is_valid() {
                    valid_count += 1;
                    if valid_count <= limit || !show_all {
                        println!(
                            "{:5} | 0x{:08X} | 0x{:08X} | {:6} | {:8} | {:11} | Valid",
                            i,
                            entry.name_1,
                            entry.name_2,
                            entry.locale,
                            entry.platform,
                            entry.block_index
                        );
                    }
                } else if entry.is_deleted() {
                    deleted_count += 1;
                } else {
                    empty_count += 1;
                }
            }

            println!();
            println!(
                "Summary: {} valid, {} deleted, {} empty entries",
                valid_count, deleted_count, empty_count
            );
        } else {
            println!("No hash table loaded");
        }
    }

    // Display block table
    if show_all || table_type == Some("block") {
        println!();
        if let Some(block_table) = archive.block_table() {
            println!("Block Table (size: {}):", block_table.size());
            println!("Index | File Pos   | Compressed | File Size  | Flags      | Status");
            println!("------|------------|------------|------------|------------|--------");

            let entries = block_table.entries();
            let count = if show_all {
                limit.min(entries.len())
            } else {
                entries.len()
            };

            let mut valid_count = 0;

            for (i, entry) in entries.iter().enumerate() {
                if entry.exists() {
                    valid_count += 1;
                    if valid_count <= limit || !show_all {
                        let flags_str = format_block_flags(entry.flags);
                        println!(
                            "{:5} | 0x{:08X} | {:10} | {:10} | {:10} | {}",
                            i,
                            entry.file_pos,
                            entry.compressed_size,
                            entry.file_size,
                            flags_str,
                            if entry.exists() { "Exists" } else { "Deleted" }
                        );
                    }
                }
            }

            println!();
            println!("Summary: {} valid entries", valid_count);
        } else {
            println!("No block table loaded");
        }
    }

    // Show specific entry details if requested
    if let Some(index_str) = table_type {
        if let Ok(index) = index_str.parse::<usize>() {
            println!();
            println!("Detailed entry at index {}:", index);

            if let Some(hash_table) = archive.hash_table() {
                if let Some(hash_entry) = hash_table.get(index) {
                    println!("Hash Entry:");
                    println!("  Name 1: 0x{:08X}", hash_entry.name_1);
                    println!("  Name 2: 0x{:08X}", hash_entry.name_2);
                    println!(
                        "  Locale: {} (0x{:04X})",
                        format_locale(hash_entry.locale),
                        hash_entry.locale
                    );
                    println!("  Platform: {}", format_platform(hash_entry.platform));
                    println!("  Block Index: {}", hash_entry.block_index);
                    println!(
                        "  Status: {}",
                        if hash_entry.is_valid() {
                            "Valid"
                        } else if hash_entry.is_deleted() {
                            "Deleted"
                        } else {
                            "Empty"
                        }
                    );

                    if hash_entry.is_valid() {
                        if let Some(block_table) = archive.block_table() {
                            if let Some(block_entry) =
                                block_table.get(hash_entry.block_index as usize)
                            {
                                println!();
                                println!("Corresponding Block Entry:");
                                println!("  File Position: 0x{:08X}", block_entry.file_pos);
                                println!(
                                    "  Compressed Size: {} bytes",
                                    block_entry.compressed_size
                                );
                                println!("  File Size: {} bytes", block_entry.file_size);
                                println!(
                                    "  Compression Ratio: {:.1}%",
                                    if block_entry.file_size > 0 {
                                        100.0 * block_entry.compressed_size as f64
                                            / block_entry.file_size as f64
                                    } else {
                                        0.0
                                    }
                                );
                                println!("  Flags: 0x{:08X}", block_entry.flags);

                                use mopaq::tables::BlockEntry;
                                if block_entry.flags & BlockEntry::FLAG_EXISTS != 0 {
                                    println!("    - EXISTS");
                                }
                                if block_entry.flags & BlockEntry::FLAG_COMPRESS != 0 {
                                    println!("    - COMPRESSED");
                                }
                                if block_entry.flags & BlockEntry::FLAG_ENCRYPTED != 0 {
                                    println!("    - ENCRYPTED");
                                }
                                if block_entry.flags & BlockEntry::FLAG_FIX_KEY != 0 {
                                    println!("    - FIX_KEY");
                                }
                                if block_entry.flags & BlockEntry::FLAG_SINGLE_UNIT != 0 {
                                    println!("    - SINGLE_UNIT");
                                }
                                if block_entry.flags & BlockEntry::FLAG_SECTOR_CRC != 0 {
                                    println!("    - SECTOR_CRC");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get a human-readable name for the format version
fn format_version_name(version: FormatVersion) -> &'static str {
    match version {
        FormatVersion::V1 => "Original/Classic",
        FormatVersion::V2 => "Burning Crusade",
        FormatVersion::V3 => "Cataclysm Beta",
        FormatVersion::V4 => "Cataclysm+",
    }
}

/// Convert a byte array to a hex string
fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<String>()
}

/// Format block flags as a short string
fn format_block_flags(flags: u32) -> String {
    use mopaq::tables::BlockEntry;
    let mut parts = Vec::new();

    if flags & BlockEntry::FLAG_COMPRESS != 0 {
        parts.push("CMP");
    }
    if flags & BlockEntry::FLAG_ENCRYPTED != 0 {
        parts.push("ENC");
    }
    if flags & BlockEntry::FLAG_FIX_KEY != 0 {
        parts.push("FIX");
    }
    if flags & BlockEntry::FLAG_SINGLE_UNIT != 0 {
        parts.push("SGL");
    }

    if parts.is_empty() {
        "NONE".to_string()
    } else {
        parts.join("|")
    }
}

/// Format locale code
fn format_locale(locale: u16) -> &'static str {
    match locale {
        0x0000 => "Neutral",
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
