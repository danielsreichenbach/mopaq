//! Debug commands for MPQ archives

use anyhow::Result;
use mopaq::{Archive, FormatVersion};
use std::path::Path;

/// Display detailed information about an MPQ archive
pub fn info(archive_path: &str) -> Result<()> {
    println!("MPQ Archive Information");
    println!("======================");
    println!();

    // Try to open the archive
    let archive = Archive::open(archive_path)?;
    let header = archive.header();

    // Basic information
    println!("File: {}", archive_path);
    println!(
        "Archive offset: 0x{:08X} ({} bytes)",
        archive.archive_offset(),
        archive.archive_offset()
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
    println!("MPQ Header:");
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
    println!("Tables:");
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
            println!("  Hi-block table:");
            println!("    Position: 0x{:08X}", hi_pos);
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
        println!("Version 4 Extended Data:");
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
    println!("  [0x002]: 0x{:08X}", ENCRYPTION_TABLE[0x002]);
    println!("  [0x010]: 0x{:08X}", ENCRYPTION_TABLE[0x010]);

    println!("  [0x080]: 0x{:08X}", ENCRYPTION_TABLE[0x080]);
    println!("  [0x081]: 0x{:08X}", ENCRYPTION_TABLE[0x081]);
    println!("  [0x082]: 0x{:08X}", ENCRYPTION_TABLE[0x082]);
    println!("  [0x08F]: 0x{:08X}", ENCRYPTION_TABLE[0x08F]);

    println!("  [0x0F0]: 0x{:08X}", ENCRYPTION_TABLE[0x0F0]);
    println!("  [0x0F1]: 0x{:08X}", ENCRYPTION_TABLE[0x0F1]);
    println!("  [0x0F2]: 0x{:08X}", ENCRYPTION_TABLE[0x0F2]);
    println!("  [0x0FF]: 0x{:08X}", ENCRYPTION_TABLE[0x0FF]);

    println!("  [0x100]: 0x{:08X}", ENCRYPTION_TABLE[0x100]);
    println!("  [0x101]: 0x{:08X}", ENCRYPTION_TABLE[0x101]);
    println!("  [0x102]: 0x{:08X}", ENCRYPTION_TABLE[0x102]);
    println!("  [0x10F]: 0x{:08X}", ENCRYPTION_TABLE[0x10F]);

    println!("  [0x200]: 0x{:08X}", ENCRYPTION_TABLE[0x200]);
    println!("  [0x201]: 0x{:08X}", ENCRYPTION_TABLE[0x201]);
    println!("  [0x202]: 0x{:08X}", ENCRYPTION_TABLE[0x202]);
    println!("  [0x20F]: 0x{:08X}", ENCRYPTION_TABLE[0x20F]);

    println!("  [0x200]: 0x{:08X}", ENCRYPTION_TABLE[0x200]);
    println!("  [0x201]: 0x{:08X}", ENCRYPTION_TABLE[0x201]);
    println!("  [0x202]: 0x{:08X}", ENCRYPTION_TABLE[0x202]);
    println!("  [0x20F]: 0x{:08X}", ENCRYPTION_TABLE[0x20F]);

    println!("  [0x300]: 0x{:08X}", ENCRYPTION_TABLE[0x300]);
    println!("  [0x301]: 0x{:08X}", ENCRYPTION_TABLE[0x301]);
    println!("  [0x302]: 0x{:08X}", ENCRYPTION_TABLE[0x302]);
    println!("  [0x30F]: 0x{:08X}", ENCRYPTION_TABLE[0x30F]);

    println!("  [0x400]: 0x{:08X}", ENCRYPTION_TABLE[0x400]);
    println!("  [0x401]: 0x{:08X}", ENCRYPTION_TABLE[0x401]);
    println!("  [0x402]: 0x{:08X}", ENCRYPTION_TABLE[0x402]);
    println!("  [0x40F]: 0x{:08X}", ENCRYPTION_TABLE[0x40F]);

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
