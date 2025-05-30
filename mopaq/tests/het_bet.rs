use mopaq::{Archive, ArchiveBuilder, FormatVersion};
use tempfile::TempDir;

#[test]
#[ignore = "HET/BET table creation not yet implemented"]
fn test_create_v3_archive_with_het_bet() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("v3_het_bet.mpq");

    // Create v3 archive (HET/BET tables should be created automatically)
    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .add_file_data(b"Test content 1".to_vec(), "file1.txt")
        .add_file_data(b"Test content 2".to_vec(), "file2.txt")
        .add_file_data(b"Test content 3".to_vec(), "folder/file3.txt")
        .build(&archive_path)
        .expect("Failed to create archive");

    // Open and verify
    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    // Check that HET/BET tables exist
    assert!(archive.het_table().is_some(), "HET table should exist");
    assert!(archive.bet_table().is_some(), "BET table should exist");

    // Verify we can find files using HET/BET
    assert!(archive.find_file("file1.txt").unwrap().is_some());
    assert!(archive.find_file("file2.txt").unwrap().is_some());
    assert!(archive.find_file("folder/file3.txt").unwrap().is_some());
    assert!(archive.find_file("nonexistent.txt").unwrap().is_none());

    // Verify we can read files
    let data = archive.read_file("file1.txt").unwrap();
    assert_eq!(data, b"Test content 1");
}

#[test]
fn test_v3_archive_fallback_to_classic_tables() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("v3_classic.mpq");

    // Create v3 archive (currently falls back to classic tables)
    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .add_file_data(b"Test content 1".to_vec(), "file1.txt")
        .add_file_data(b"Test content 2".to_vec(), "file2.txt")
        .build(&archive_path)
        .expect("Failed to create archive");

    // Open and verify
    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    // Currently V3 archives fall back to classic tables
    assert!(
        archive.het_table().is_none(),
        "HET table should not exist yet"
    );
    assert!(
        archive.bet_table().is_none(),
        "BET table should not exist yet"
    );

    // But files should still be accessible through classic tables
    assert!(archive.find_file("file1.txt").unwrap().is_some());
    assert!(archive.find_file("file2.txt").unwrap().is_some());

    // Verify we can read files
    let data = archive.read_file("file1.txt").unwrap();
    assert_eq!(data, b"Test content 1");
}

#[test]
fn test_jenkins_hash_lookup() {
    use mopaq::hash::jenkins_hash;

    // Test Jenkins hash for known values
    let filenames = vec![
        "war3map.j",
        "(listfile)",
        "units\\human\\footman.mdx",
        "interface\\glue\\mainmenu.blp",
    ];

    for filename in filenames {
        let hash = jenkins_hash(filename);
        println!("Jenkins hash for '{}': 0x{:016X}", filename, hash);

        // Verify hash is non-zero
        assert_ne!(hash, 0);
    }
}

#[test]
fn test_bet_bit_packing() {
    // Test bit extraction logic
    let test_value: u64 = 0x123456789ABCDEF0;

    // Extract 8 bits starting at bit 4
    let extracted = (test_value >> 4) & ((1u64 << 8) - 1);
    assert_eq!(extracted, 0xEF);

    // Extract 16 bits starting at bit 8
    let extracted = (test_value >> 8) & ((1u64 << 16) - 1);
    assert_eq!(extracted, 0xBCDE);
}
