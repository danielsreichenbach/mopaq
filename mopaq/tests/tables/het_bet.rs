use mopaq::{Archive, ArchiveBuilder, FormatVersion};
use tempfile::TempDir;

#[test]
fn test_create_v3_archive_with_het_bet() {
    let _ = env_logger::builder().is_test(true).try_init();

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

    // Ensure tables are loaded
    archive.load_tables().expect("Failed to load tables");

    // Check that HET/BET tables exist
    log::debug!("Test: Loading tables explicitly");

    println!("Header version: {:?}", archive.header().format_version);
    println!("HET table pos: {:?}", archive.header().het_table_pos);
    println!("BET table pos: {:?}", archive.header().bet_table_pos);
    println!("HET table: {:?}", archive.het_table().is_some());
    println!("BET table: {:?}", archive.bet_table().is_some());

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
fn test_v3_archive_with_classic_table_compatibility() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("v3_compat.mpq");

    // Create v3 archive (now creates both HET/BET and classic tables)
    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .add_file_data(b"Test content 1".to_vec(), "file1.txt")
        .add_file_data(b"Test content 2".to_vec(), "file2.txt")
        .build(&archive_path)
        .expect("Failed to create archive");

    // Open and verify
    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    // V3 archives now have HET/BET tables
    assert!(archive.het_table().is_some(), "HET table should exist");
    assert!(archive.bet_table().is_some(), "BET table should exist");

    // Files should be accessible through HET/BET tables
    assert!(archive.find_file("file1.txt").unwrap().is_some());
    assert!(archive.find_file("file2.txt").unwrap().is_some());

    // Verify we can read files
    let data = archive.read_file("file1.txt").unwrap();
    assert_eq!(data, b"Test content 1");

    // Also verify classic tables exist for compatibility
    assert!(
        archive.hash_table().is_some(),
        "Hash table should exist for compatibility"
    );
    assert!(
        archive.block_table().is_some(),
        "Block table should exist for compatibility"
    );
}

#[test]
fn test_jenkins_hash_lookup() {
    use mopaq::jenkins_hash;

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
