//! Zlib compression tests

use mopaq::compression::{compress, decompress, flags};

#[test]
fn test_zlib_compression() {
    let test_data = include_bytes!("../../README.md");

    // Compress
    let compressed = compress(test_data, flags::ZLIB).expect("Compression failed");

    // Should be smaller than original
    assert!(compressed.len() < test_data.len());
    println!(
        "Zlib compression ratio: {:.1}%",
        100.0 * compressed.len() as f64 / test_data.len() as f64
    );

    // Decompress
    let decompressed =
        decompress(&compressed, flags::ZLIB, test_data.len()).expect("Decompression failed");

    // Should match original
    assert_eq!(decompressed, test_data);
}

#[test]
fn test_zlib_empty_data() {
    // Test empty data handling
    let empty: &[u8] = &[];

    // Compress empty data
    let compressed = compress(empty, flags::ZLIB).expect("Compression failed");

    // Decompress
    let decompressed = decompress(&compressed, flags::ZLIB, 0).expect("Decompression failed");

    assert_eq!(decompressed.len(), 0);
}

#[test]
fn test_zlib_invalid_data() {
    // Try to decompress random data that isn't valid compressed data
    let invalid_data = vec![0xFF, 0xDE, 0xAD, 0xBE, 0xEF];

    let result = decompress(&invalid_data, flags::ZLIB, 100);
    assert!(result.is_err(), "Decompressing invalid data should fail");
}

#[test]
fn test_zlib_size_mismatch() {
    let test_data = b"Test data";
    let compressed = compress(test_data, flags::ZLIB).expect("Compression failed");

    // Try to decompress with wrong expected size (larger than actual)
    // This should succeed - expected_size is just a hint for buffer allocation
    let result = decompress(&compressed, flags::ZLIB, 1000);

    assert!(
        result.is_ok(),
        "Decompression should succeed even with wrong expected size"
    );
    let decompressed = result.unwrap();
    assert_eq!(
        decompressed, test_data,
        "Decompressed data should match original"
    );
    assert_eq!(
        decompressed.len(),
        test_data.len(),
        "Actual size is {} not the expected 1000",
        test_data.len()
    );
}

#[test]
fn test_zlib_size_too_small() {
    let test_data = b"This is a longer test string that will compress";
    let compressed = compress(test_data, flags::ZLIB).expect("Compression failed");

    // Try to decompress with a smaller expected size
    // The implementation will still decompress all the data
    let result = decompress(&compressed, flags::ZLIB, 5);

    match result {
        Ok(decompressed) => {
            // The decompression succeeds and returns all the data
            // even though we only "expected" 5 bytes
            assert_eq!(
                decompressed, test_data,
                "Should return all decompressed data regardless of expected size"
            );
            println!(
                "Decompressed {} bytes even though we expected only 5",
                decompressed.len()
            );
        }
        Err(_) => {
            // Some implementations might fail if the buffer is too small
            // This is also acceptable behavior
            println!("Decompression failed with size mismatch");
        }
    }
}

#[test]
fn test_zlib_large_data() {
    // Test with larger data (1MB of repeated pattern)
    let pattern = b"The quick brown fox jumps over the lazy dog. ";
    let mut large_data = Vec::new();
    for _ in 0..(1024 * 1024 / pattern.len()) {
        large_data.extend_from_slice(pattern);
    }

    let compressed = compress(&large_data, flags::ZLIB).expect("Compression failed");
    println!(
        "Large data zlib ratio: {:.1}%",
        100.0 * compressed.len() as f64 / large_data.len() as f64
    );

    let decompressed =
        decompress(&compressed, flags::ZLIB, large_data.len()).expect("Decompression failed");
    assert_eq!(decompressed, large_data);
}

#[test]
fn test_zlib_binary_data() {
    // Test with binary data
    let mut binary_data = Vec::new();
    for i in 0..1000 {
        binary_data.push((i % 256) as u8);
        binary_data.push(((i * 7) % 256) as u8);
        binary_data.push(((i * 13) % 256) as u8);
        binary_data.push(((i * 31) % 256) as u8);
    }

    let compressed = compress(&binary_data, flags::ZLIB).expect("Compression failed");
    let decompressed =
        decompress(&compressed, flags::ZLIB, binary_data.len()).expect("Decompression failed");
    assert_eq!(decompressed, binary_data);
}