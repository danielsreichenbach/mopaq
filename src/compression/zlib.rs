//! zlib compression implementation for MPQ archives

use super::{CompressionError, CompressionResult, CompressionType, Compressor, Decompressor};

#[cfg(feature = "zlib")]
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
#[cfg(feature = "zlib")]
use std::io::{Read, Write};

/// Compresses data using zlib
pub fn compress_zlib(data: &[u8]) -> CompressionResult<Vec<u8>> {
    #[cfg(feature = "zlib")]
    {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(data)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))
    }

    #[cfg(not(feature = "zlib"))]
    {
        Err(CompressionError::UnsupportedType(CompressionType::Zlib))
    }
}

/// Decompresses data using zlib
pub fn decompress_zlib(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    #[cfg(feature = "zlib")]
    {
        let mut decoder = ZlibDecoder::new(data);
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

    #[cfg(not(feature = "zlib"))]
    {
        Err(CompressionError::UnsupportedType(CompressionType::Zlib))
    }
}

/// zlib compressor implementation
pub struct ZlibCompressor;

impl Compressor for ZlibCompressor {
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        compress_zlib(data)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Zlib
    }
}

/// zlib decompressor implementation
pub struct ZlibDecompressor;

impl Decompressor for ZlibDecompressor {
    fn decompress(&self, data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
        decompress_zlib(data, expected_size)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Zlib
    }
}

#[cfg(all(test, feature = "zlib"))]
mod tests {
    use super::*;

    #[test]
    fn test_zlib_roundtrip() {
        // Test data
        let original = b"This is test data for zlib compression. It should compress well and decompress back to the original.";

        // Compress
        let compressed = compress_zlib(original).expect("Compression failed");

        // Should be smaller than original
        assert!(compressed.len() < original.len());

        // Decompress
        let decompressed =
            decompress_zlib(&compressed, original.len()).expect("Decompression failed");

        // Check that we got the original data back
        assert_eq!(decompressed, original);
    }
}
