use mopaq::{compression, Archive, ArchiveBuilder, FormatVersion, ListfileOption};
use std::fs;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_v4_comprehensive() {
    // Create temporary directory for test files
    let temp_dir = TempDir::new().unwrap();

    // Create test files with different content
    let file1_path = temp_dir.path().join("test1.txt");
    let file2_path = temp_dir.path().join("test2.bin");
    let file3_path = temp_dir.path().join("subdir/test3.dat");

    fs::write(&file1_path, b"Hello from test file 1!\n").unwrap();
    fs::write(&file2_path, vec![0xFF; 1024]).unwrap(); // Binary data
    fs::create_dir_all(file3_path.parent().unwrap()).unwrap();
    fs::write(&file3_path, b"Nested file content").unwrap();

    // Create v4 archive with various options
    let archive_file = NamedTempFile::new().unwrap();
    let archive_path = archive_file.path();

    ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .block_size(4) // 8KB sectors
        .default_compression(compression::flags::ZLIB)
        .generate_crcs(true) // Enable CRC generation
        .compress_tables(true) // Enable table compression
        .add_file(&file1_path, "files/test1.txt")
        .add_file_with_options(
            &file2_path,
            "files/test2.bin",
            compression::flags::BZIP2,
            false,
            0,
        )
        .add_file_data(b"Direct data content".to_vec(), "direct.txt")
        .add_file_data_with_encryption(
            b"Encrypted content".to_vec(),
            "encrypted.txt",
            compression::flags::ZLIB,
            true, // use_fix_key
            0,
        )
        .build(archive_path)
        .unwrap();

    // Verify archive creation
    assert!(archive_path.exists());
    let file_size = fs::metadata(archive_path).unwrap().len();
    assert!(file_size > 0);

    // Open and verify the archive
    let mut archive = Archive::open(archive_path).unwrap();

    // Check format version
    assert_eq!(archive.header().format_version, FormatVersion::V4);

    // Get archive info
    let info = archive.get_info().unwrap();
    println!("Archive info: {:?}", info);

    // Verify MD5 checksums
    assert!(info.md5_status.is_some());
    let md5_status = info.md5_status.unwrap();
    assert!(md5_status.header_valid);
    assert!(md5_status.hash_table_valid);
    assert!(md5_status.block_table_valid);
    assert!(md5_status.hi_block_table_valid);
    assert!(md5_status.het_table_valid);
    assert!(md5_status.bet_table_valid);

    // List files
    let files = archive.list().unwrap();
    assert_eq!(files.len(), 5); // 4 files + listfile

    // Verify file contents
    let content1 = archive.read_file("files/test1.txt").unwrap();
    assert_eq!(content1, b"Hello from test file 1!\n");

    let content2 = archive.read_file("files/test2.bin").unwrap();
    assert_eq!(content2.len(), 1024);
    assert!(content2.iter().all(|&b| b == 0xFF));

    let content3 = archive.read_file("direct.txt").unwrap();
    assert_eq!(content3, b"Direct data content");

    let content4 = archive.read_file("encrypted.txt").unwrap();
    assert_eq!(content4, b"Encrypted content");

    // Test file exists by checking list
    let file_entries = archive.list_with_hashes().unwrap();
    let file_entry = file_entries
        .iter()
        .find(|e| e.name == "files/test1.txt")
        .unwrap();
    assert_eq!(file_entry.name, "files/test1.txt");

    // Test listfile
    let listfile = archive.read_file("(listfile)").unwrap();
    let listfile_str = String::from_utf8_lossy(&listfile);
    assert!(listfile_str.contains("files/test1.txt"));
    assert!(listfile_str.contains("encrypted.txt"));
}

#[test]
fn test_v4_empty_archive() {
    // Test creating an empty v4 archive
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .listfile_option(ListfileOption::None)
        .build(path)
        .unwrap();

    // Open and verify
    let mut archive = Archive::open(path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V4);

    // Should have no files
    let files = archive.list().unwrap();
    assert_eq!(files.len(), 0);

    // MD5 checksums should still be valid
    let info = archive.get_info().unwrap();
    let md5_status = info.md5_status.unwrap();
    assert!(md5_status.header_valid);
    assert!(md5_status.hash_table_valid);
    assert!(md5_status.block_table_valid);
}

#[test]
fn test_v4_large_file_support() {
    // Test v4's ability to handle large file offsets
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create several files to push offsets beyond 32-bit range
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .listfile_option(ListfileOption::None);

    // Add multiple 1MB files
    for i in 0..10 {
        let data = vec![i as u8; 1024 * 1024]; // 1MB each
        builder = builder.add_file_data(data, &format!("large_{}.dat", i));
    }

    builder.build(path).unwrap();

    // Verify all files can be read
    let mut archive = Archive::open(path).unwrap();
    for i in 0..10 {
        let data = archive.read_file(&format!("large_{}.dat", i)).unwrap();
        assert_eq!(data.len(), 1024 * 1024);
        assert!(data.iter().all(|&b| b == i as u8));
    }
}
