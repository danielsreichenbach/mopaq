//! Round-trip compression tests for all algorithms

use mopaq::compression::{compress, decompress, flags};

#[test]
fn test_zlib_round_trip() {
    let test_cases = vec![
        b"Hello, World!".to_vec(),
        b"A".repeat(1000), // Highly compressible
        vec![0u8; 100],    // All zeros
        (0u8..255).collect::<Vec<u8>>(), // All byte values
    ];

    for original in test_cases {
        let compressed = compress(&original, flags::ZLIB).expect("Compression failed");
        let decompressed = decompress(&compressed, flags::ZLIB, original.len())
            .expect("Decompression failed");
        
        assert_eq!(decompressed, original);
    }
}

#[test]
fn test_bzip2_round_trip() {
    let test_cases = vec![
        b"Hello, World!".to_vec(),
        b"B".repeat(1000), // Highly compressible
        vec![0u8; 100],    // All zeros
    ];

    for original in test_cases {
        let compressed = compress(&original, flags::BZIP2).expect("Compression failed");
        let decompressed = decompress(&compressed, flags::BZIP2, original.len())
            .expect("Decompression failed");
        
        assert_eq!(decompressed, original);
    }
}

#[test]
fn test_lzma_round_trip() {
    let test_cases = vec![
        b"Hello, World!".to_vec(),
        b"C".repeat(1000), // Highly compressible
        vec![0u8; 100],    // All zeros
        b"The quick brown fox jumps over the lazy dog".to_vec(),
    ];

    for original in test_cases {
        let compressed = compress(&original, flags::LZMA).expect("Compression failed");
        let decompressed = decompress(&compressed, flags::LZMA, original.len())
            .expect("Decompression failed");
        
        assert_eq!(decompressed, original);
    }
}

#[test]
fn test_sparse_round_trip() {
    let test_cases = vec![
        b"Hello\0\0\0World".to_vec(),
        vec![0u8; 1000], // All zeros - should compress very well
        b"No zeros here!".to_vec(),
        vec![1, 2, 3, 0, 0, 0, 0, 0, 4, 5, 6], // Mixed data
    ];

    for original in test_cases {
        let compressed = compress(&original, flags::SPARSE).expect("Compression failed");
        let decompressed = decompress(&compressed, flags::SPARSE, original.len())
            .expect("Decompression failed");
        
        assert_eq!(decompressed, original);
    }
}

#[test]
fn test_no_compression() {
    let original = b"This is uncompressed data";
    
    let compressed = compress(original, 0).expect("Compression failed");
    assert_eq!(compressed, original);
    
    let decompressed = decompress(&compressed, 0, original.len())
        .expect("Decompression failed");
    assert_eq!(decompressed, original);
}