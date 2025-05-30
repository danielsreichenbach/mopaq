//! Main compression logic

use crate::{Error, Result};
use super::algorithms;
use super::methods::CompressionMethod;

/// Compress data using the specified compression method
pub fn compress(data: &[u8], method: u8) -> Result<Vec<u8>> {
    let compression = CompressionMethod::from_flags(method);

    match compression {
        CompressionMethod::None => Ok(data.to_vec()),
        CompressionMethod::Zlib => algorithms::zlib::compress(data),
        CompressionMethod::BZip2 => algorithms::bzip2::compress(data),
        CompressionMethod::Lzma => algorithms::lzma::compress(data),
        CompressionMethod::Sparse => algorithms::sparse::compress(data),
        _ => Err(Error::compression("Compression method not yet implemented")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::flags;

    #[test]
    fn test_compress_api_small_data() {
        // Test that the public compress API works with small data
        let original = b"Small data";

        // Test uncompressed
        let result = compress(original, 0).expect("Compression failed");
        assert_eq!(result, original);

        // Test zlib - might not reduce size for small data
        let compressed = compress(original, flags::ZLIB).expect("Compression failed");
        let decompressed = super::super::decompress::decompress(&compressed, flags::ZLIB, original.len())
            .expect("Decompression failed");
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_lzma_api() {
        let original = b"Test data for LZMA compression through the public API";

        // Test through our wrapper API
        let compressed = compress(original, flags::LZMA).expect("Compression failed");
        let decompressed = super::super::decompress::decompress(&compressed, flags::LZMA, original.len())
            .expect("Decompression failed");

        assert_eq!(decompressed, original);
    }
}