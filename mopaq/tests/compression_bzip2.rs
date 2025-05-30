//! BZip2 compression tests

use mopaq::compression::{compress, decompress, flags};

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
fn test_bzip2_large_data() {
    // Test with larger data (1MB of repeated pattern)
    // BZip2 should compress repeated patterns very well
    let pattern = b"The quick brown fox jumps over the lazy dog. ";
    let mut large_data = Vec::new();
    for _ in 0..(1024 * 1024 / pattern.len()) {
        large_data.extend_from_slice(pattern);
    }

    let compressed = compress(&large_data, flags::BZIP2).expect("Compression failed");
    println!(
        "Large data bzip2 ratio: {:.1}%",
        100.0 * compressed.len() as f64 / large_data.len() as f64
    );

    let decompressed = decompress(&compressed, flags::BZIP2, large_data.len())
        .expect("Decompression failed");
    assert_eq!(decompressed, large_data);
}