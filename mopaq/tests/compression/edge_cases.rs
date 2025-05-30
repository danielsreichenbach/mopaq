//! Edge case and error handling tests

use mopaq::compression::{compress, decompress, flags};

#[test]
fn test_empty_data_compression() {
    let empty = b"";
    
    // Most compression algorithms should handle empty data
    for method in &[flags::ZLIB, flags::BZIP2, flags::LZMA] {
        match compress(empty, *method) {
            Ok(compressed) => {
                // If compression succeeds, decompression should too
                let result = decompress(&compressed, *method, 0);
                assert!(result.is_ok());
            }
            Err(_) => {
                // Some algorithms might reject empty input
            }
        }
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
    
    let compressed = compress(&mostly_zeros, flags::SPARSE).expect("Compression failed");
    
    // Should compress to just a few bytes
    assert!(compressed.len() < 10);
    
    let decompressed = decompress(&compressed, flags::SPARSE, mostly_zeros.len())
        .expect("Decompression failed");
    
    assert_eq!(decompressed, mostly_zeros);
}

#[test]
fn test_compression_efficiency() {
    // Test that compression actually reduces size for suitable data
    let repetitive = b"AAAAAAAAAA".repeat(100);
    
    let zlib_compressed = compress(&repetitive, flags::ZLIB).expect("Compression failed");
    assert!(zlib_compressed.len() < repetitive.len() / 2);
    
    let bzip2_compressed = compress(&repetitive, flags::BZIP2).expect("Compression failed");
    assert!(bzip2_compressed.len() < repetitive.len() / 2);
    
    let lzma_compressed = compress(&repetitive, flags::LZMA).expect("Compression failed");
    assert!(lzma_compressed.len() < repetitive.len() / 2);
}

#[test]
fn test_unimplemented_methods() {
    let data = b"test data";
    
    // These should return errors
    let result = compress(data, flags::HUFFMAN);
    assert!(result.is_err());
    
    let result = compress(data, flags::PKWARE);
    assert!(result.is_err());
    
    let result = compress(data, flags::ADPCM_MONO);
    assert!(result.is_err());
    
    // Multiple compression should also fail for compress
    let result = compress(data, flags::ZLIB | flags::PKWARE);
    assert!(result.is_err());
}