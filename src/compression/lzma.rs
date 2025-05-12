//! LZMA compression implementation for MPQ archives

use super::{CompressionError, CompressionResult, CompressionType, Compressor, Decompressor};

#[cfg(feature = "lzma")]
use std::io::{Read, Write};
#[cfg(feature = "lzma")]
use xz2::stream::{LzmaReader, LzmaWriter};

/// Compresses data using LZMA
pub fn compress_lzma(data: &[u8]) -> CompressionResult<Vec<u8>> {
    #[cfg(feature = "lzma")]
    {
        let mut compressed = Vec::new();
        {
            let mut encoder = LzmaWriter::new_compressor(&mut compressed, 9)
                .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

            encoder
                .write_all(data)
                .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;

            encoder
                .finish()
                .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
        }
        Ok(compressed)
    }

    #[cfg(not(feature = "lzma"))]
    {
        Err(CompressionError::UnsupportedType(CompressionType::Lzma))
    }
}

/// Decompresses data using LZMA
pub fn decompress_lzma(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    #[cfg(feature = "lzma")]
    {
        let mut decompressed = Vec::with_capacity(expected_size);
        {
            let mut decoder = LzmaReader::new_decompressor(data)
                .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

            let bytes_read = decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;

            if bytes_read != expected_size {
                return Err(CompressionError::DecompressionFailed(format!(
                    "Expected {} bytes, got {}",
                    expected_size, bytes_read
                )));
            }
        }
        Ok(decompressed)
    }

    #[cfg(not(feature = "lzma"))]
    {
        Err(CompressionError::UnsupportedType(CompressionType::Lzma))
    }
}

/// LZMA compressor implementation
pub struct LzmaCompressor;

impl Compressor for LzmaCompressor {
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        compress_lzma(data)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Lzma
    }
}

/// LZMA decompressor implementation
pub struct LzmaDecompressor;

impl Decompressor for LzmaDecompressor {
    fn decompress(&self, data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
        decompress_lzma(data, expected_size)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Lzma
    }
}

#[cfg(all(test, feature = "lzma"))]
mod tests {
    use super::*;

    #[test]
    fn test_lzma_roundtrip() {
        // Test data
        let original = b"This is test data for LZMA compression. It should compress well and decompress back to the original.";

        // Compress
        let compressed = compress_lzma(original).expect("Compression failed");

        // Should be smaller than original (for larger data)
        // Note: LZMA has high overhead for small data

        // Decompress
        let decompressed =
            decompress_lzma(&compressed, original.len()).expect("Decompression failed");

        // Check that we got the original data back
        assert_eq!(decompressed, original);
    }
}
