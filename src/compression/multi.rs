//! Multi-compression handling for MPQ archives
//! Allows applying multiple compression methods in sequence

use super::{
    CompressionError, CompressionResult, CompressionType, adpcm, bzip2, huffman, lzma, pkware,
    sparse, wave, zlib,
};

/// Compresses a block of data using the specified compression methods
///
/// # Arguments
/// * `data` - The data to compress
/// * `types` - The compression methods to apply, in order
///
/// # Returns
/// The compressed data with a flag byte indicating the used compression types
pub fn compress_block(data: &[u8], types: &[CompressionType]) -> CompressionResult<Vec<u8>> {
    if types.is_empty() {
        // No compression, just return the original data
        return Ok(data.to_vec());
    }

    // Start with the original data
    let mut compressed = data.to_vec();
    let mut used_types = Vec::new();

    // Apply each compression method in sequence
    for &compression_type in types {
        let prev_size = compressed.len();

        // Skip if the data is already small enough
        if prev_size <= 32 {
            continue;
        }

        // Try to compress with this method
        let newly_compressed = match compression_type {
            CompressionType::None => continue,

            CompressionType::Implode => pkware::implode(&compressed)?,

            CompressionType::Huffman => huffman::compress_huffman(&compressed)?,

            CompressionType::Zlib => {
                #[cfg(feature = "zlib")]
                {
                    zlib::compress_zlib(&compressed)?
                }

                #[cfg(not(feature = "zlib"))]
                {
                    return Err(CompressionError::UnsupportedType(compression_type));
                }
            }

            CompressionType::Bzip2 => {
                #[cfg(feature = "bzip2")]
                {
                    bzip2::compress_bzip2(&compressed)?
                }

                #[cfg(not(feature = "bzip2"))]
                {
                    return Err(CompressionError::UnsupportedType(compression_type));
                }
            }

            CompressionType::Lzma => {
                #[cfg(feature = "lzma")]
                {
                    lzma::compress_lzma(&compressed)?
                }

                #[cfg(not(feature = "lzma"))]
                {
                    return Err(CompressionError::UnsupportedType(compression_type));
                }
            }

            CompressionType::Sparse => sparse::compress_sparse(&compressed)?,

            CompressionType::ImaAdpcmMono => adpcm::compress_adpcm_mono(&compressed)?,

            CompressionType::ImaAdpcmStereo => adpcm::compress_adpcm_stereo(&compressed)?,

            CompressionType::Wave => wave::compress_wave(&compressed)?,
        };

        // Only use this compression if it actually made the data smaller
        if newly_compressed.len() < prev_size {
            compressed = newly_compressed;
            used_types.push(compression_type);
        }
    }

    // If no compression was used or compressed data is larger than original,
    // return the original data
    if used_types.is_empty() || compressed.len() >= data.len() {
        return Ok(data.to_vec());
    }

    // Create the final compressed block with the compression flag byte
    let mut result = Vec::with_capacity(compressed.len() + 1);

    // Add the compression flag byte
    let compression_flags = super::build_compression_flag(&used_types);
    result.push(compression_flags);

    // Add the compressed data
    result.extend_from_slice(&compressed);

    Ok(result)
}

/// Decompresses a block of data using the specified compression methods
///
/// # Arguments
/// * `data` - The compressed data with a leading flag byte
/// * `expected_size` - The expected size of the decompressed data
///
/// # Returns
/// The decompressed data
pub fn decompress_block(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    if data.is_empty() {
        return Err(CompressionError::InvalidData("Empty data".to_string()));
    }

    // Get the compression flag byte
    let flags = data[0];

    // If flags is 0, no compression was used
    if flags == 0 {
        // Return the data without the flag byte
        return Ok(data[1..].to_vec());
    }

    // Detect compression types from the flag byte
    let types = super::detect_compression_types(flags);

    // If no compression types were detected, return the data as is
    if types.is_empty() {
        return Ok(data[1..].to_vec());
    }

    // Start with the data after the flag byte
    let mut decompressed = data[1..].to_vec();

    // Apply decompression methods in reverse order
    for &compression_type in types.iter().rev() {
        // Decompress with this method
        decompressed = match compression_type {
            CompressionType::None => decompressed,

            CompressionType::Implode => pkware::explode(&decompressed, expected_size)?,

            CompressionType::Huffman => huffman::decompress_huffman(&decompressed, expected_size)?,

            CompressionType::Zlib => {
                #[cfg(feature = "zlib")]
                {
                    zlib::decompress_zlib(&decompressed, expected_size)?
                }

                #[cfg(not(feature = "zlib"))]
                {
                    return Err(CompressionError::UnsupportedType(compression_type));
                }
            }

            CompressionType::Bzip2 => {
                #[cfg(feature = "bzip2")]
                {
                    bzip2::decompress_bzip2(&decompressed, expected_size)?
                }

                #[cfg(not(feature = "bzip2"))]
                {
                    return Err(CompressionError::UnsupportedType(compression_type));
                }
            }

            CompressionType::Lzma => {
                #[cfg(feature = "lzma")]
                {
                    lzma::decompress_lzma(&decompressed, expected_size)?
                }

                #[cfg(not(feature = "lzma"))]
                {
                    return Err(CompressionError::UnsupportedType(compression_type));
                }
            }

            CompressionType::Sparse => sparse::decompress_sparse(&decompressed, expected_size)?,

            CompressionType::ImaAdpcmMono => {
                adpcm::decompress_adpcm_mono(&decompressed, expected_size)?
            }

            CompressionType::ImaAdpcmStereo => {
                adpcm::decompress_adpcm_stereo(&decompressed, expected_size)?
            }

            CompressionType::Wave => wave::decompress_wave(&decompressed, expected_size)?,
        };
    }

    // Verify the decompressed size
    if decompressed.len() != expected_size {
        return Err(CompressionError::DecompressionFailed(format!(
            "Decompressed size mismatch: got {}, expected {}",
            decompressed.len(),
            expected_size
        )));
    }

    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_flags() {
        // Test single compression types
        let types = vec![CompressionType::Huffman];
        let flags = super::super::build_compression_flag(&types);
        assert_eq!(flags, 0x01);

        let types = vec![CompressionType::Zlib];
        let flags = super::super::build_compression_flag(&types);
        assert_eq!(flags, 0x02);

        let types = vec![CompressionType::Implode];
        let flags = super::super::build_compression_flag(&types);
        assert_eq!(flags, 0x08);

        // Test multiple compression types
        let types = vec![CompressionType::Huffman, CompressionType::Zlib];
        let flags = super::super::build_compression_flag(&types);
        assert_eq!(flags, 0x03);

        let types = vec![
            CompressionType::Huffman,
            CompressionType::Zlib,
            CompressionType::Implode,
        ];
        let flags = super::super::build_compression_flag(&types);
        assert_eq!(flags, 0x0B);
    }

    #[test]
    fn test_detect_compression_types() {
        // Test single compression types
        let flags = 0x01;
        let types = super::super::detect_compression_types(flags);
        assert_eq!(types, vec![CompressionType::Huffman]);

        let flags = 0x02;
        let types = super::super::detect_compression_types(flags);
        assert_eq!(types, vec![CompressionType::Zlib]);

        let flags = 0x08;
        let types = super::super::detect_compression_types(flags);
        assert_eq!(types, vec![CompressionType::Implode]);

        // Test multiple compression types
        let flags = 0x03;
        let types = super::super::detect_compression_types(flags);
        assert_eq!(types, vec![CompressionType::Huffman, CompressionType::Zlib]);

        let flags = 0x0B;
        let types = super::super::detect_compression_types(flags);
        assert_eq!(
            types,
            vec![
                CompressionType::Huffman,
                CompressionType::Zlib,
                CompressionType::Implode
            ]
        );
    }

    // More comprehensive tests will be added after implementing the compression methods
}
