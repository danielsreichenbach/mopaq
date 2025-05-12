//! Sparse compression implementation for MPQ archives

use super::{CompressionError, CompressionResult, CompressionType, Compressor, Decompressor};

/// Compresses data using Sparse compression
pub fn compress_sparse(data: &[u8]) -> CompressionResult<Vec<u8>> {
    // Stub implementation
    Err(CompressionError::UnsupportedType(CompressionType::Sparse))
}

/// Decompresses data using Sparse compression
pub fn decompress_sparse(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    // Stub implementation
    Err(CompressionError::UnsupportedType(CompressionType::Sparse))
}

// Struct implementations would go here
