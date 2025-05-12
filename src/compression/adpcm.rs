//! IMA ADPCM compression implementation for MPQ archives

use super::{CompressionError, CompressionResult, CompressionType, Compressor, Decompressor};

/// Compresses data using IMA ADPCM (mono)
pub fn compress_adpcm_mono(data: &[u8]) -> CompressionResult<Vec<u8>> {
    // Stub implementation
    Err(CompressionError::UnsupportedType(
        CompressionType::ImaAdpcmMono,
    ))
}

/// Decompresses data using IMA ADPCM (mono)
pub fn decompress_adpcm_mono(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    // Stub implementation
    Err(CompressionError::UnsupportedType(
        CompressionType::ImaAdpcmMono,
    ))
}

/// Compresses data using IMA ADPCM (stereo)
pub fn compress_adpcm_stereo(data: &[u8]) -> CompressionResult<Vec<u8>> {
    // Stub implementation
    Err(CompressionError::UnsupportedType(
        CompressionType::ImaAdpcmStereo,
    ))
}

/// Decompresses data using IMA ADPCM (stereo)
pub fn decompress_adpcm_stereo(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    // Stub implementation
    Err(CompressionError::UnsupportedType(
        CompressionType::ImaAdpcmStereo,
    ))
}

// Struct implementations would go here
