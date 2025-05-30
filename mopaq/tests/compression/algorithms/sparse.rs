//! Sparse/RLE compression tests

use mopaq::compression::{compress, decompress, flags};

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
fn test_sparse_compression_decompression() {
    // Test compression and decompression round trip
    let test_data = b"Data\0\0\0\0\0\0\0\0with\0\0\0\0lots\0\0\0\0\0\0\0\0of\0\0\0zeros";

    let compressed = compress(test_data, flags::SPARSE).expect("Compression failed");

    // Sparse should be very efficient for data with lots of zeros
    assert!(compressed.len() < test_data.len());
    println!(
        "Sparse compression ratio for data with zeros: {:.1}%",
        100.0 * compressed.len() as f64 / test_data.len() as f64
    );

    let decompressed =
        decompress(&compressed, flags::SPARSE, test_data.len()).expect("Decompression failed");

    assert_eq!(decompressed, test_data);
}

#[test]
fn test_sparse_all_zeros() {
    // Test compression of all zeros - should be extremely efficient
    let all_zeros = vec![0u8; 1000];

    let compressed = compress(&all_zeros, flags::SPARSE).expect("Compression failed");

    // Should compress to just a few bytes (control bytes + end marker)
    assert!(compressed.len() < 20);
    println!(
        "Sparse compression of 1000 zeros: {} bytes",
        compressed.len()
    );

    let decompressed =
        decompress(&compressed, flags::SPARSE, all_zeros.len()).expect("Decompression failed");

    assert_eq!(decompressed, all_zeros);
}

#[test]
fn test_sparse_no_zeros() {
    // Test compression of data with no zeros - should not compress well
    let no_zeros: Vec<u8> = (1..=255).collect();

    let compressed = compress(&no_zeros, flags::SPARSE).expect("Compression failed");

    // Should be larger than original due to control bytes
    assert!(compressed.len() > no_zeros.len());

    let decompressed =
        decompress(&compressed, flags::SPARSE, no_zeros.len()).expect("Decompression failed");

    assert_eq!(decompressed, no_zeros);
}
