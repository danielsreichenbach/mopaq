//! Edge case and error handling tests

use crate::compression::test_helpers::{compress_with_method, test_round_trip};
use mopaq::compression::{decompress, flags};

#[test]
fn test_empty_data_compression() {
    let empty = b"";

    // Most compression algorithms should handle empty data
    for method in &[flags::ZLIB, flags::BZIP2, flags::LZMA] {
        // Use round trip test which handles all the edge cases
        test_round_trip(empty, *method).expect("Empty data round trip should succeed");
    }
}

#[test]
fn test_empty_compressed_data_decompression() {
    let empty = b"";

    // Decompressing empty data should fail
    let result = decompress(empty, flags::ZLIB, 100);
    assert!(result.is_err());
}

#[test]
fn test_invalid_compressed_data() {
    let garbage = b"This is not compressed data!";

    // All compression methods should fail to decompress garbage
    for method in &[flags::ZLIB, flags::BZIP2, flags::LZMA] {
        let result = decompress(garbage, *method, 100);
        assert!(result.is_err());
    }
}

#[test]
fn test_sparse_compression_efficiency() {
    // Sparse compression should be very efficient for data with lots of zeros
    let mostly_zeros = vec![0u8; 1000];

    let compressed =
        compress_with_method(&mostly_zeros, flags::SPARSE).expect("Compression failed");

    // Check if compression was beneficial
    if !compressed.is_empty() && compressed[0] == flags::SPARSE {
        // Should compress to just a few bytes (method byte + control bytes)
        assert!(compressed.len() < 15);
    }

    // Test round trip
    test_round_trip(&mostly_zeros, flags::SPARSE).expect("Sparse efficiency round trip failed");
}

#[test]
fn test_compression_efficiency() {
    // Test that compression actually reduces size for suitable data
    let repetitive = b"AAAAAAAAAA".repeat(100);

    // Test each compression method
    for &method in &[flags::ZLIB, flags::BZIP2, flags::LZMA] {
        let compressed = compress_with_method(&repetitive, method).expect("Compression failed");

        // Check if compression was beneficial
        if !compressed.is_empty() && compressed[0] == method {
            // Should compress well for repetitive data
            assert!(
                compressed.len() < repetitive.len() / 2,
                "Method 0x{:02X} should compress repetitive data to less than 50%",
                method
            );
        }

        // Test round trip
        test_round_trip(&repetitive, method).expect("Compression efficiency round trip failed");
    }
}

#[test]
fn test_unimplemented_methods() {
    let data = b"test data";

    // These should return errors
    let result = compress_with_method(data, flags::HUFFMAN);
    assert!(result.is_err());

    let result = compress_with_method(data, flags::PKWARE);
    assert!(result.is_err());

    // Note: ADPCM is now implemented, so removing that test

    // Multiple compression should also fail for compress
    let result = compress_with_method(data, flags::ZLIB | flags::PKWARE);
    assert!(result.is_err());
}
