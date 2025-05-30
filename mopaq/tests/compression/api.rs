//! Generic compression API tests

use mopaq::compression::{compress, decompress, flags, CompressionMethod};

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
fn test_compression_method_detection() {
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
    assert_eq!(
        CompressionMethod::from_flags(flags::SPARSE),
        CompressionMethod::Sparse
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::HUFFMAN),
        CompressionMethod::Huffman
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::PKWARE),
        CompressionMethod::PKWare
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::ADPCM_MONO),
        CompressionMethod::AdpcmMono
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::ADPCM_STEREO),
        CompressionMethod::AdpcmStereo
    );

    // Test multiple compression detection
    let multi = flags::ZLIB | flags::PKWARE;
    assert!(CompressionMethod::from_flags(multi).is_multiple());
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
fn test_unimplemented_compression_methods() {
    let test_data = b"Test data";

    // These compression methods are not yet implemented
    let unimplemented_methods = [
        flags::HUFFMAN,
        flags::PKWARE,
        flags::ADPCM_MONO,
        flags::ADPCM_STEREO,
    ];

    for method in &unimplemented_methods {
        let result = compress(test_data, *method);
        assert!(
            result.is_err(),
            "Compression with unimplemented method 0x{:02X} should fail",
            method
        );
    }
}

#[test]
fn test_empty_data_decompression_error() {
    // Decompressing empty data should fail for all methods
    let empty = b"";

    let methods = [flags::ZLIB, flags::BZIP2, flags::LZMA, flags::SPARSE];

    for method in &methods {
        let result = decompress(empty, *method, 100);
        assert!(
            result.is_err(),
            "Decompressing empty data with method 0x{:02X} should fail",
            method
        );
    }
}

#[test]
fn test_compression_flags() {
    // Test that all flag constants are correct
    assert_eq!(flags::HUFFMAN, 0x01);
    assert_eq!(flags::ZLIB, 0x02);
    assert_eq!(flags::PKWARE, 0x08);
    assert_eq!(flags::BZIP2, 0x10);
    assert_eq!(flags::SPARSE, 0x20);
    assert_eq!(flags::ADPCM_MONO, 0x40);
    assert_eq!(flags::ADPCM_STEREO, 0x80);
    assert_eq!(flags::LZMA, 0x12);
}
