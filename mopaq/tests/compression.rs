//! Integration tests for compression functionality

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
fn test_bzip2_compression() {
    let test_data = b"This is a test string that should compress well because it has repeated patterns. \
                      This is a test string that should compress well because it has repeated patterns. \
                      This is a test string that should compress well because it has repeated patterns.";

    // Compress
    let compressed = compress(test_data, flags::BZIP2).expect("Compression failed");

    // Should be smaller than original
    assert!(compressed.len() < test_data.len());
    println!(
        "BZip2 compression ratio: {:.1}%",
        100.0 * compressed.len() as f64 / test_data.len() as f64
    );

    // Decompress
    let decompressed =
        decompress(&compressed, flags::BZIP2, test_data.len()).expect("Decompression failed");

    // Should match original
    assert_eq!(decompressed, test_data);
}

#[test]
fn test_sparse_decompression() {
    // Create sparse-compressed data manually
    // Format: [count|data] or [0x80|zero_count]
    let compressed = vec![
        5, b'H', b'e', b'l', b'l', b'o', // "Hello"
        0x8A, // 10 zeros
        5, b'W', b'o', b'r', b'l', b'd', // "World"
        0x85, // 5 zeros
        3, b'E', b'n', b'd', // "End"
        0xFF, // End marker
    ];

    let expected = b"Hello\0\0\0\0\0\0\0\0\0\0World\0\0\0\0\0End";

    let decompressed =
        decompress(&compressed, flags::SPARSE, expected.len()).expect("Decompression failed");

    assert_eq!(decompressed, expected);
}

#[test]
fn test_no_compression() {
    let test_data = b"This data is not compressed";

    // "Compress" with no compression
    let compressed = compress(test_data, 0).expect("Compression failed");
    assert_eq!(compressed, test_data);

    // "Decompress" with no compression
    let decompressed = decompress(&compressed, 0, test_data.len()).expect("Decompression failed");
    assert_eq!(decompressed, test_data);
}

#[test]
fn test_multiple_compression() {
    // Test multiple compression (zlib as final compression)
    // Format: [compression_order_byte][compressed_data]
    let original = b"This is test data for multiple compression. It should compress well.";

    // First compress with zlib
    let zlib_compressed = compress(original, flags::ZLIB).expect("Zlib compression failed");

    // Create multiple compression data (zlib was last)
    let mut multi_compressed = vec![flags::ZLIB];
    multi_compressed.extend_from_slice(&zlib_compressed);

    // Decompress with multiple flag
    let multi_flag = flags::ZLIB | flags::PKWARE;
    let decompressed = decompress(&multi_compressed, multi_flag, original.len())
        .expect("Multiple decompression failed");

    assert_eq!(decompressed, original);
}

#[test]
fn test_empty_data() {
    // Test empty data handling
    let empty: &[u8] = &[];

    // Compress empty data
    let compressed = compress(empty, flags::ZLIB).expect("Compression failed");

    // Decompress
    let decompressed = decompress(&compressed, flags::ZLIB, 0).expect("Decompression failed");

    assert_eq!(decompressed.len(), 0);
}

#[test]
fn test_compression_invalid_data() {
    // Try to decompress random data that isn't valid compressed data
    let invalid_data = vec![0xFF, 0xDE, 0xAD, 0xBE, 0xEF];

    let result = decompress(&invalid_data, flags::ZLIB, 100);
    assert!(result.is_err(), "Decompressing invalid data should fail");
}

#[test]
fn test_compression_size_mismatch() {
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
fn test_compression_size_too_small() {
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
fn test_large_data_compression() {
    // Test with larger data (1MB of repeated pattern)
    let pattern = b"The quick brown fox jumps over the lazy dog. ";
    let mut large_data = Vec::new();
    for _ in 0..(1024 * 1024 / pattern.len()) {
        large_data.extend_from_slice(pattern);
    }

    // Test zlib
    let zlib_compressed = compress(&large_data, flags::ZLIB).expect("Compression failed");
    println!(
        "Large data zlib ratio: {:.1}%",
        100.0 * zlib_compressed.len() as f64 / large_data.len() as f64
    );

    let decompressed =
        decompress(&zlib_compressed, flags::ZLIB, large_data.len()).expect("Decompression failed");
    assert_eq!(decompressed, large_data);

    // Test bzip2 (should compress better for repeated patterns)
    let bzip2_compressed = compress(&large_data, flags::BZIP2).expect("Compression failed");
    println!(
        "Large data bzip2 ratio: {:.1}%",
        100.0 * bzip2_compressed.len() as f64 / large_data.len() as f64
    );

    let decompressed = decompress(&bzip2_compressed, flags::BZIP2, large_data.len())
        .expect("Decompression failed");
    assert_eq!(decompressed, large_data);
}

#[test]
fn test_binary_data_compression() {
    // Test with binary data
    let mut binary_data = Vec::new();
    for i in 0..1000 {
        binary_data.push((i % 256) as u8);
        binary_data.push(((i * 7) % 256) as u8);
        binary_data.push(((i * 13) % 256) as u8);
        binary_data.push(((i * 31) % 256) as u8);
    }

    // Test zlib
    let compressed = compress(&binary_data, flags::ZLIB).expect("Compression failed");
    let decompressed =
        decompress(&compressed, flags::ZLIB, binary_data.len()).expect("Decompression failed");
    assert_eq!(decompressed, binary_data);
}

#[test]
fn test_compression_method_detection() {
    use mopaq::compression::CompressionMethod;

    assert_eq!(CompressionMethod::from_flags(0), CompressionMethod::None);
    assert_eq!(
        CompressionMethod::from_flags(flags::ZLIB),
        CompressionMethod::Zlib
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::BZIP2),
        CompressionMethod::BZip2
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::LZMA),
        CompressionMethod::Lzma
    );

    // Test multiple compression detection
    let multi = flags::ZLIB | flags::PKWARE;
    assert!(CompressionMethod::from_flags(multi).is_multiple());
}
