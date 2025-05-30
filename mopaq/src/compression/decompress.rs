//! Main decompression logic and multi-compression handling

use super::algorithms;
use super::methods::{flags, CompressionMethod};
use crate::{Error, Result};

/// Decompress data using the specified compression method
pub fn decompress(data: &[u8], method: u8, decompressed_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    // Check if this is IMPLODE flag rather than COMPRESS
    if method == 0 {
        // No compression
        return Ok(data.to_vec());
    }

    // Log what we're trying to decompress for debugging
    log::debug!(
        "Decompressing {} bytes to {} bytes with method 0x{:02X}",
        data.len(),
        decompressed_size,
        method
    );

    let compression = CompressionMethod::from_flags(method);

    match compression {
        CompressionMethod::None => Ok(data.to_vec()),
        CompressionMethod::Zlib => algorithms::zlib::decompress(data, decompressed_size),
        CompressionMethod::BZip2 => algorithms::bzip2::decompress(data, decompressed_size),
        CompressionMethod::Lzma => algorithms::lzma::decompress(data, decompressed_size),
        CompressionMethod::Sparse => algorithms::sparse::decompress(data, decompressed_size),
        CompressionMethod::PKWare => {
            log::error!("PKWare decompression requested but not implemented");
            Err(Error::compression(
                "PKWare decompression not yet implemented",
            ))
        }
        CompressionMethod::Huffman => {
            log::error!("Huffman decompression requested but not implemented");
            Err(Error::compression(
                "Huffman decompression not yet implemented",
            ))
        }
        CompressionMethod::AdpcmMono | CompressionMethod::AdpcmStereo => {
            log::error!("ADPCM decompression requested but not implemented");
            Err(Error::compression(
                "ADPCM decompression not yet implemented",
            ))
        }
        CompressionMethod::Multiple(flags) => {
            log::debug!("Multiple compression with flags 0x{:02X}", flags);
            decompress_multiple(data, flags, decompressed_size)
        }
    }
}

/// Handle multiple compression methods
fn decompress_multiple(data: &[u8], flags: u8, expected_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    // For multiple compression, we need to check which methods are actually used
    // The data format depends on which compressions are applied

    // Check for ADPCM compression - it's always applied first if present
    let has_adpcm_mono = (flags & flags::ADPCM_MONO) != 0;
    let has_adpcm_stereo = (flags & flags::ADPCM_STEREO) != 0;
    let has_adpcm = has_adpcm_mono || has_adpcm_stereo;

    // If only ADPCM is set, it's not actually multiple compression
    if flags == flags::ADPCM_MONO || flags == flags::ADPCM_STEREO {
        return Err(Error::compression(format!(
            "ADPCM compression (0x{:02X}) not yet implemented",
            flags
        )));
    }

    // Check if PKWARE is in the flags - it's applied before the final compression
    let has_pkware = (flags & flags::PKWARE) != 0;

    // Determine the final compression method (applied last)
    let final_compression = if flags & flags::HUFFMAN != 0 {
        flags::HUFFMAN
    } else if flags & flags::ZLIB != 0 {
        flags::ZLIB
    } else if flags & flags::BZIP2 != 0 {
        flags::BZIP2
    } else if flags & flags::SPARSE != 0 {
        flags::SPARSE
    } else if has_adpcm {
        // ADPCM with no other compression - should have been caught above
        return Err(Error::compression(format!(
            "Invalid compression flags: 0x{:02X}",
            flags
        )));
    } else {
        return Err(Error::compression(format!(
            "Multiple compression flag set but no known compression methods: 0x{:02X}",
            flags
        )));
    };

    // For ADPCM + other compression combinations
    if has_adpcm {
        // Common combinations:
        // 0x41: Mono ADPCM + Huffman
        // 0x48: Mono ADPCM + PKWare (Implode)
        // 0x81: Stereo ADPCM + Huffman
        // 0x88: Stereo ADPCM + PKWare (Implode)
        log::warn!(
            "ADPCM + {} compression (flags: 0x{:02X}) not yet implemented",
            match final_compression {
                flags::HUFFMAN => "Huffman",
                flags::PKWARE => "PKWare",
                _ => "other",
            },
            flags
        );
        return Err(Error::compression(format!(
            "ADPCM compression combinations (0x{:02X}) not yet implemented",
            flags
        )));
    }

    // If we have PKWare, the first byte tells us the actual compression used
    let (compression_used, compressed_data) = if has_pkware {
        // First byte indicates which compression was actually used
        // If it matches our final compression, only that was used
        // Otherwise, both PKWare and the final compression were used
        let first_byte = data[0];

        // Check if only one compression was actually applied
        if first_byte == final_compression {
            // Only the final compression was used (PKWare didn't help)
            (final_compression, &data[1..])
        } else {
            // Both compressions were used - this is the complex case
            // For now, we'll skip PKWare decompression and try to handle the data
            log::warn!("Multiple compression with PKWare detected - attempting to decompress without PKWare");

            // The data might start with a compression byte or might be raw compressed data
            // Let's try to detect based on the byte value
            if first_byte <= 0x10 || first_byte == 0x20 {
                // Looks like a compression type byte
                (first_byte, &data[1..])
            } else {
                // Probably compressed data - assume it's the final compression
                (final_compression, data)
            }
        }
    } else {
        // No PKWare, just the single compression method
        (final_compression, data)
    };

    // Decompress using the detected method
    match compression_used {
        flags::HUFFMAN => {
            log::error!("Huffman decompression requested but not implemented");
            Err(Error::compression(
                "Huffman decompression not yet implemented",
            ))
        }
        flags::ZLIB => algorithms::zlib::decompress(compressed_data, expected_size),
        flags::BZIP2 => algorithms::bzip2::decompress(compressed_data, expected_size),
        flags::SPARSE => algorithms::sparse::decompress(compressed_data, expected_size),
        _ => {
            // Try each method if we're not sure
            log::warn!(
                "Unknown compression byte 0x{:02X}, trying available methods",
                compression_used
            );

            // Try zlib first (most common)
            if let Ok(result) = algorithms::zlib::decompress(data, expected_size) {
                return Ok(result);
            }

            // Try bzip2
            if let Ok(result) = algorithms::bzip2::decompress(data, expected_size) {
                return Ok(result);
            }

            // Try sparse
            if let Ok(result) = algorithms::sparse::decompress(data, expected_size) {
                return Ok(result);
            }

            Err(Error::compression(format!(
                "Failed to decompress with any method. First byte: 0x{:02X}, flags: 0x{:02X}",
                data.first().unwrap_or(&0),
                flags
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_api() {
        let original = b"Test data for compression";

        // Test uncompressed
        let result = decompress(original, 0, original.len()).expect("Decompression failed");
        assert_eq!(result, original);

        // Test zlib
        let compressed = algorithms::zlib::compress(original).expect("Compression failed");
        let result =
            decompress(&compressed, flags::ZLIB, original.len()).expect("Decompression failed");
        assert_eq!(result, original);
    }
}
