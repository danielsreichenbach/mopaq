//! bzip2 compression implementation for MPQ archives

use super::{CompressionError, CompressionResult, CompressionType, Compressor, Decompressor};

#[cfg(feature = "bzip2")]
use bzip2::{Compression, read::BzDecoder, write::BzEncoder};
#[cfg(feature = "bzip2")]
use std::io::{Read, Write};

/// Compresses data using bzip2
pub fn compress_bzip2(data: &[u8]) -> CompressionResult<Vec<u8>> {
    #[cfg(feature = "bzip2")]
    {
        let mut encoder = BzEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(data)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
    }

    #[cfg(not(feature = "bzip2"))]
    {
        Err(CompressionError::UnsupportedType(CompressionType::Bzip2))
    }
}

/// Decompresses data using bzip2
pub fn decompress_bzip2(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    #[cfg(feature = "bzip2")]
    {
        let mut decoder = BzDecoder::new(data);
        let mut decompressed = Vec::with_capacity(expected_size);

        let bytes_read = decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

        if bytes_read != expected_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Expected {} bytes, got {}",
                expected_size, bytes_read
            )));
        }

        Ok(decompressed)
    }

    #[cfg(not(feature = "bzip2"))]
    {
        Err(CompressionError::UnsupportedType(CompressionType::Bzip2))
    }
}

/// bzip2 compressor implementation
pub struct Bzip2Compressor;

impl Compressor for Bzip2Compressor {
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        compress_bzip2(data)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Bzip2
    }
}

/// bzip2 decompressor implementation
pub struct Bzip2Decompressor;

impl Decompressor for Bzip2Decompressor {
    fn decompress(&self, data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
        decompress_bzip2(data, expected_size)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Bzip2
    }
}

#[cfg(all(test, feature = "bzip2"))]
mod tests {
    use super::*;

    #[test]
    fn test_bzip2_roundtrip() {
        // Test data
        let original = b"This is test data for bzip2 compression. It should compress well and decompress back to the original.";

        // Compress
        let compressed = compress_bzip2(original).expect("Compression failed");

        // Should be smaller than original
        assert!(compressed.len() < original.len());

        // Decompress
        let decompressed =
            decompress_bzip2(&compressed, original.len()).expect("Decompression failed");

        // Check that we got the original data back
        assert_eq!(decompressed, original);
    }
}
