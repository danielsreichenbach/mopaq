//! Test script for compression + encryption scenarios

use mopaq::{compression::flags as CompressionFlags, Archive, ArchiveBuilder, Result};
use std::fs;

fn main() -> Result<()> {
    println!("Testing Compression + Encryption Scenarios");
    println!("==========================================\n");

    // Test 1: Single-unit file with encryption and compression
    println!("Test 1: Single-unit file (encrypted + compressed)");
    {
        let test_data = b"This is test data that should be compressed and encrypted!";
        ArchiveBuilder::new()
            .add_file_data_with_options(
                test_data.to_vec(),
                "test1.txt",
                CompressionFlags::ZLIB,
                true, // Enable encryption
                0,    // Default locale
            )
            .build("test1.mpq")?;

        let mut archive = Archive::open("test1.mpq")?;
        let data = archive.read_file("test1.txt")?;
        assert_eq!(data, test_data);
        println!("✓ Successfully created and read single-unit encrypted+compressed file");

        let files = archive.list()?;
        if let Some(file) = files.iter().find(|f| f.name == "test1.txt") {
            println!("  File size: {} bytes", file.size);
            println!("  Compressed size: {} bytes", file.compressed_size);
            println!("  Encrypted: {}", file.is_encrypted());
            println!("  Compressed: {}", file.is_compressed());
        }
    }

    // Test 2: Multi-sector file with encryption and compression
    println!("\nTest 2: Multi-sector file (encrypted + compressed)");
    {
        let large_data: Vec<u8> = (0..50000).map(|i| (i % 256) as u8).collect();
        ArchiveBuilder::new()
            .block_size(2) // 2048 byte sectors to force multi-sector
            .add_file_data_with_options(
                large_data.clone(),
                "test2.bin",
                CompressionFlags::ZLIB,
                true, // Enable encryption
                0,    // Default locale
            )
            .build("test2.mpq")?;

        let mut archive = Archive::open("test2.mpq")?;
        let data = archive.read_file("test2.bin")?;
        assert_eq!(data, large_data);
        println!("✓ Successfully created and read multi-sector encrypted+compressed file");

        let files = archive.list()?;
        if let Some(file) = files.iter().find(|f| f.name == "test2.bin") {
            println!("  File size: {} bytes", file.size);
            println!("  Compressed size: {} bytes", file.compressed_size);
            println!("  Encrypted: {}", file.is_encrypted());
            println!("  Compressed: {}", file.is_compressed());
            println!("  Single unit: {}", file.is_single_unit());
        }
    }

    // Test 3: Different compression methods with encryption
    println!("\nTest 3: Different compression methods with encryption");
    {
        let test_data = b"Test data for different compression methods! ".repeat(20);

        // ZLIB
        ArchiveBuilder::new()
            .add_file_data_with_options(
                test_data.clone(),
                "zlib.dat",
                CompressionFlags::ZLIB,
                true,
                0,
            )
            .build("test3_zlib.mpq")?;

        // BZIP2
        ArchiveBuilder::new()
            .add_file_data_with_options(
                test_data.clone(),
                "bzip2.dat",
                CompressionFlags::BZIP2,
                true,
                0,
            )
            .build("test3_bzip2.mpq")?;

        // Verify ZLIB
        let mut archive = Archive::open("test3_zlib.mpq")?;
        let data = archive.read_file("zlib.dat")?;
        assert_eq!(data, test_data);
        println!("✓ ZLIB + encryption works");

        // Verify BZIP2
        let mut archive = Archive::open("test3_bzip2.mpq")?;
        let data = archive.read_file("bzip2.dat")?;
        assert_eq!(data, test_data);
        println!("✓ BZIP2 + encryption works");
    }

    // Test 4: FIX_KEY encryption with compression
    println!("\nTest 4: FIX_KEY encryption with compression");
    {
        let test_data = b"Test data with FIX_KEY encryption!";
        ArchiveBuilder::new()
            .add_file_data_with_encryption(
                test_data.to_vec(),
                "fixkey.txt",
                CompressionFlags::ZLIB,
                true, // Use FIX_KEY
                0,    // Default locale
            )
            .build("test4.mpq")?;

        let mut archive = Archive::open("test4.mpq")?;
        let data = archive.read_file("fixkey.txt")?;
        assert_eq!(data, test_data);
        println!("✓ FIX_KEY encryption with compression works");

        let files = archive.list()?;
        if let Some(file) = files.iter().find(|f| f.name == "fixkey.txt") {
            println!("  Has FIX_KEY: {}", file.has_fix_key());
        }
    }

    // Test 5: Create archive from raw test data and verify with CLI
    println!("\nTest 5: Create archive from test data");
    {
        // Create a compressed and encrypted archive
        ArchiveBuilder::new()
            .add_file("test-data/raw-data/simple/readme.txt", "readme.txt")
            .add_file_with_options(
                "test-data/raw-data/simple/data.txt",
                "data.txt",
                CompressionFlags::ZLIB,
                true, // Encrypt
                0,
            )
            .add_file_with_options(
                "test-data/raw-data/simple/config.ini",
                "config.ini",
                CompressionFlags::BZIP2,
                true, // Encrypt
                0,
            )
            .build("test5_mixed.mpq")?;

        println!("✓ Created test5_mixed.mpq with encrypted files from test data");

        // Verify
        let mut archive = Archive::open("test5_mixed.mpq")?;
        let files = archive.list()?;
        println!("  Archive contains {} files:", files.len());
        for file in &files {
            println!(
                "    - {} ({} bytes, {}{})",
                file.name,
                file.size,
                if file.is_encrypted() {
                    "encrypted"
                } else {
                    "plain"
                },
                if file.is_compressed() {
                    ", compressed"
                } else {
                    ""
                }
            );
        }

        // Read encrypted files
        let data = archive.read_file("data.txt")?;
        println!(
            "  Successfully read encrypted data.txt: {} bytes",
            data.len()
        );

        let config = archive.read_file("config.ini")?;
        println!(
            "  Successfully read encrypted config.ini: {} bytes",
            config.len()
        );
    }

    // Clean up
    println!("\nCleaning up test files...");
    fs::remove_file("test1.mpq").ok();
    fs::remove_file("test2.mpq").ok();
    fs::remove_file("test3_zlib.mpq").ok();
    fs::remove_file("test3_bzip2.mpq").ok();
    fs::remove_file("test4.mpq").ok();
    fs::remove_file("test5_mixed.mpq").ok();
    fs::remove_file("test_encryption_compression.rs").ok();

    println!("\n✅ All tests passed! Encryption + compression is working correctly.");
    Ok(())
}
