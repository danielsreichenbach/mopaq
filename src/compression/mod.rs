//! Compression module entry point, unified API
//! Handles various compression methods used in MPQ files

mod adpcm;
mod bzip2;
mod huffman;
mod lzma;
mod multi;
mod pkware;
mod sparse;
mod wave;
mod zlib;

// Re-export public interfaces
pub use huffman::{compress_huffman, decompress_huffman};
pub use multi::{compress_block, decompress_block};
pub use pkware::{explode, implode};

use std::io::Error as IoError;
use thiserror::Error;

/// Compression types used in MPQ archives
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// No compression
    None = 0x00,
    /// PKWARE Implode compression
    Implode = 0x08,
    /// Huffman compression
    Huffman = 0x01,
    /// zlib compression
    Zlib = 0x02,
    /// bzip2 compression
    Bzip2 = 0x10,
    /// LZMA compression
    Lzma = 0x12,
    /// Sparse compression
    Sparse = 0x20,
    /// IMA ADPCM compression (mono)
    ImaAdpcmMono = 0x40,
    /// IMA ADPCM compression (stereo)
    ImaAdpcmStereo = 0x80,
    /// WAVE compression
    Wave = 0x81,
}

impl CompressionType {
    /// Gets a CompressionType from its flag value
    pub fn from_flag(flag: u8) -> Option<Self> {
        match flag {
            0x00 => Some(CompressionType::None),
            0x08 => Some(CompressionType::Implode),
            0x01 => Some(CompressionType::Huffman),
            0x02 => Some(CompressionType::Zlib),
            0x10 => Some(CompressionType::Bzip2),
            0x12 => Some(CompressionType::Lzma),
            0x20 => Some(CompressionType::Sparse),
            0x40 => Some(CompressionType::ImaAdpcmMono),
            0x80 => Some(CompressionType::ImaAdpcmStereo),
            0x81 => Some(CompressionType::Wave),
            _ => None,
        }
    }

    /// Returns the flag value for this compression type
    pub fn to_flag(&self) -> u8 {
        *self as u8
    }
}

/// Error types specific to MPQ compression operations
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("I/O error: {0}")]
    IoError(#[from] IoError),

    #[error("Unsupported compression type: {0:?}")]
    UnsupportedType(CompressionType),

    #[error("Decompression error: {0}")]
    DecompressionFailed(String),

    #[error("Compression error: {0}")]
    CompressionFailed(String),

    #[error("Buffer too small: expected at least {expected} bytes, got {actual}")]
    BufferTooSmall { expected: usize, actual: usize },

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Result type for compression operations
pub type CompressionResult<T> = Result<T, CompressionError>;

/// Unified trait for compression algorithms
pub trait Compressor {
    /// Compresses a block of data using this compression method
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>>;

    /// Returns the compression type
    fn compression_type(&self) -> CompressionType;
}

/// Unified trait for decompression algorithms
pub trait Decompressor {
    /// Decompresses a block of data using this compression method
    ///
    /// # Arguments
    /// * `data` - The compressed data
    /// * `expected_size` - The expected size of the decompressed data
    fn decompress(&self, data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>>;

    /// Returns the compression type
    fn compression_type(&self) -> CompressionType;
}

/// Detects the compression types from a sector flag byte
///
/// # Arguments
/// * `flags` - The sector flag byte
///
/// # Returns
/// A vector of detected compression types, in the order they should be applied
pub fn detect_compression_types(flags: u8) -> Vec<CompressionType> {
    let mut types = Vec::new();

    // Check for each compression type bit
    if flags & CompressionType::Implode as u8 != 0 {
        types.push(CompressionType::Implode);
    }

    if flags & CompressionType::Huffman as u8 != 0 {
        types.push(CompressionType::Huffman);
    }

    if flags & CompressionType::Zlib as u8 != 0 {
        types.push(CompressionType::Zlib);
    }

    if flags & CompressionType::Bzip2 as u8 != 0 {
        types.push(CompressionType::Bzip2);
    }

    if flags & CompressionType::Lzma as u8 != 0 {
        types.push(CompressionType::Lzma);
    }

    if flags & CompressionType::Sparse as u8 != 0 {
        types.push(CompressionType::Sparse);
    }

    if flags & CompressionType::ImaAdpcmMono as u8 != 0 {
        types.push(CompressionType::ImaAdpcmMono);
    }

    if flags & CompressionType::ImaAdpcmStereo as u8 != 0 {
        types.push(CompressionType::ImaAdpcmStereo);
    }

    if flags & CompressionType::Wave as u8 != 0 {
        types.push(CompressionType::Wave);
    }

    types
}

/// Builds a compression flag byte from a list of compression types
///
/// # Arguments
/// * `types` - List of compression types to include in the flag
///
/// # Returns
/// The combined flag byte
pub fn build_compression_flag(types: &[CompressionType]) -> u8 {
    let mut flag = 0;
    for compression_type in types {
        flag |= compression_type.to_flag();
    }
    flag
}
