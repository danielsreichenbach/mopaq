//! Compression and decompression algorithms for MPQ files

use crate::{Error, Result};
use std::io::{Read, Write};

/// Compression method flags
pub mod flags {
    pub const HUFFMAN: u8 = 0x01; // Huffman encoding (WAVE files only)
    pub const ZLIB: u8 = 0x02; // Deflate/zlib compression
    pub const PKWARE: u8 = 0x08; // PKWare DCL compression
    pub const BZIP2: u8 = 0x10; // BZip2 compression
    pub const SPARSE: u8 = 0x20; // Sparse/RLE compression
    pub const ADPCM_MONO: u8 = 0x40; // IMA ADPCM mono
    pub const ADPCM_STEREO: u8 = 0x80; // IMA ADPCM stereo
    pub const LZMA: u8 = 0x12; // LZMA compression (not a flag combination)
}

/// Compression methods enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    None,
    Huffman,
    Zlib,
    PKWare,
    BZip2,
    Sparse,
    AdpcmMono,
    AdpcmStereo,
    Lzma,
    Multiple(u8), // For combined compression
}

impl CompressionMethod {
    /// Determine compression method(s) from flags
    pub fn from_flags(flags: u8) -> Self {
        // Check for LZMA first (special case, not a bit flag)
        if flags == flags::LZMA {
            return CompressionMethod::Lzma;
        }

        // Check for single compression methods
        match flags {
            0 => CompressionMethod::None,
            flags::HUFFMAN => CompressionMethod::Huffman,
            flags::ZLIB => CompressionMethod::Zlib,
            flags::PKWARE => CompressionMethod::PKWare,
            flags::BZIP2 => CompressionMethod::BZip2,
            flags::SPARSE => CompressionMethod::Sparse,
            flags::ADPCM_MONO => CompressionMethod::AdpcmMono,
            flags::ADPCM_STEREO => CompressionMethod::AdpcmStereo,
            _ => CompressionMethod::Multiple(flags),
        }
    }

    /// Check if this is a multi-compression method
    pub fn is_multiple(&self) -> bool {
        matches!(self, CompressionMethod::Multiple(_))
    }
}

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
        CompressionMethod::Zlib => decompress_zlib(data, decompressed_size),
        CompressionMethod::BZip2 => decompress_bzip2(data, decompressed_size),
        CompressionMethod::Lzma => decompress_lzma(data, decompressed_size),
        CompressionMethod::Sparse => decompress_sparse(data, decompressed_size),
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

/// Decompress using zlib/deflate
fn decompress_zlib(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;

    // Validate zlib header (should start with 0x78)
    if !data.is_empty() && data[0] != 0x78 {
        log::warn!(
            "Data doesn't start with zlib header (got 0x{:02X}), attempting decompression anyway",
            data[0]
        );
    }

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::with_capacity(expected_size);

    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => {
            if decompressed.len() != expected_size {
                log::warn!(
                    "Decompressed size mismatch: expected {}, got {}",
                    expected_size,
                    decompressed.len()
                );
                // Some MPQ files have incorrect size info, so we'll allow this
            }
            Ok(decompressed)
        }
        Err(e) => {
            log::error!("Zlib decompression failed: {}", e);
            log::debug!(
                "First 16 bytes of data: {:02X?}",
                &data[..16.min(data.len())]
            );
            Err(Error::compression(format!(
                "Zlib decompression failed: {}",
                e
            )))
        }
    }
}

/// Decompress using BZip2
fn decompress_bzip2(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use bzip2::read::BzDecoder;

    let mut decoder = BzDecoder::new(data);
    let mut decompressed = Vec::with_capacity(expected_size);

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| Error::compression(format!("BZip2 decompression failed: {}", e)))?;

    if decompressed.len() != expected_size {
        return Err(Error::compression(format!(
            "Decompressed size mismatch: expected {}, got {}",
            expected_size,
            decompressed.len()
        )));
    }

    Ok(decompressed)
}

/// Decompress using LZMA
fn decompress_lzma(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use std::io::{BufReader, Cursor};

    let cursor = Cursor::new(data);
    let mut input = BufReader::new(cursor);
    let mut output = Vec::with_capacity(expected_size);

    // Try LZMA format first
    match lzma_rs::lzma_decompress(&mut input, &mut output) {
        Ok(()) => {
            if expected_size > 0 && output.len() != expected_size {
                log::warn!(
                    "LZMA decompressed size mismatch: expected {}, got {}",
                    expected_size,
                    output.len()
                );
            }
            Ok(output)
        }
        Err(e) => {
            // If LZMA fails, try XZ format
            let cursor = Cursor::new(data);
            let mut input = BufReader::new(cursor);
            let mut output = Vec::with_capacity(expected_size);

            match lzma_rs::xz_decompress(&mut input, &mut output) {
                Ok(()) => Ok(output),
                Err(xz_err) => {
                    log::error!("LZMA decompression failed: {:?}", e);
                    log::error!("XZ decompression also failed: {:?}", xz_err);
                    log::debug!(
                        "First 16 bytes of data: {:02X?}",
                        &data[..16.min(data.len())]
                    );
                    Err(Error::compression(format!(
                        "LZMA/XZ decompression failed: LZMA: {:?}, XZ: {:?}",
                        e, xz_err
                    )))
                }
            }
        }
    }
}

/// Decompress sparse/RLE compressed data
fn decompress_sparse(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Sparse compression is a simple RLE format
    let mut output = Vec::with_capacity(expected_size);
    let mut pos = 0;

    while pos < data.len() && output.len() < expected_size {
        // Read control byte
        let control = data[pos];
        pos += 1;

        if control == 0xFF {
            // End of stream marker
            break;
        }

        if control & 0x80 != 0 {
            // Run of zeros
            let count = (control & 0x7F) as usize;
            output.resize(output.len() + count, 0);
        } else {
            // Copy bytes
            let count = control as usize;
            if pos + count > data.len() {
                return Err(Error::compression(
                    "Sparse decompression: unexpected end of data",
                ));
            }
            output.extend_from_slice(&data[pos..pos + count]);
            pos += count;
        }
    }

    // Pad with zeros if needed
    if output.len() < expected_size {
        output.resize(expected_size, 0);
    }

    Ok(output)
}

/// Handle multiple compression methods
fn decompress_multiple(data: &[u8], flags: u8, expected_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    // For multiple compression, we need to check which methods are actually used
    // The data format depends on which compressions are applied

    // Check if PKWARE is in the flags - it's always applied first if present
    let has_pkware = (flags & flags::PKWARE) != 0;

    // Determine the other compression method (applied last)
    let final_compression = if flags & flags::ZLIB != 0 {
        flags::ZLIB
    } else if flags & flags::BZIP2 != 0 {
        flags::BZIP2
    } else if flags & flags::SPARSE != 0 {
        flags::SPARSE
    } else {
        return Err(Error::compression(format!(
            "Multiple compression flag set but no known compression methods: 0x{:02X}",
            flags
        )));
    };

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
        flags::ZLIB => decompress_zlib(compressed_data, expected_size),
        flags::BZIP2 => decompress_bzip2(compressed_data, expected_size),
        flags::SPARSE => decompress_sparse(compressed_data, expected_size),
        _ => {
            // Try each method if we're not sure
            log::warn!(
                "Unknown compression byte 0x{:02X}, trying available methods",
                compression_used
            );

            // Try zlib first (most common)
            if let Ok(result) = decompress_zlib(data, expected_size) {
                return Ok(result);
            }

            // Try bzip2
            if let Ok(result) = decompress_bzip2(data, expected_size) {
                return Ok(result);
            }

            // Try sparse
            if let Ok(result) = decompress_sparse(data, expected_size) {
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

/// Compress data using the specified compression method
pub fn compress(data: &[u8], method: u8) -> Result<Vec<u8>> {
    let compression = CompressionMethod::from_flags(method);

    match compression {
        CompressionMethod::None => Ok(data.to_vec()),
        CompressionMethod::Zlib => compress_zlib(data),
        CompressionMethod::BZip2 => compress_bzip2(data),
        CompressionMethod::Lzma => compress_lzma(data),
        _ => Err(Error::compression("Compression method not yet implemented")),
    }
}

/// Compress using zlib/deflate
fn compress_zlib(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| Error::compression(format!("Zlib compression failed: {}", e)))?;

    encoder
        .finish()
        .map_err(|e| Error::compression(format!("Zlib compression failed: {}", e)))
}

/// Compress using BZip2
fn compress_bzip2(data: &[u8]) -> Result<Vec<u8>> {
    use bzip2::write::BzEncoder;
    use bzip2::Compression;

    let mut encoder = BzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| Error::compression(format!("BZip2 compression failed: {}", e)))?;

    encoder
        .finish()
        .map_err(|e| Error::compression(format!("BZip2 compression failed: {}", e)))
}

/// Compress using LZMA
fn compress_lzma(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::{BufReader, Cursor};

    let cursor = Cursor::new(data);
    let mut input = BufReader::new(cursor);
    let mut output = Vec::new();

    // Use LZMA format (not XZ) for MPQ compatibility
    match lzma_rs::lzma_compress(&mut input, &mut output) {
        Ok(()) => Ok(output),
        Err(e) => Err(Error::compression(format!(
            "LZMA compression failed: {:?}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_method_from_flags() {
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

        // Multiple compression
        let multi = flags::ZLIB | flags::PKWARE;
        assert!(CompressionMethod::from_flags(multi).is_multiple());
    }

    #[test]
    fn test_zlib_round_trip() {
        let original = b"Hello, World! This is a test of zlib compression in MPQ archives.";

        let compressed = compress_zlib(original).expect("Compression failed");

        // Note: Small data might not compress well due to compression headers
        // In MPQ, the implementation would check if compression actually helps
        println!(
            "Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

        let decompressed =
            decompress_zlib(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_zlib_compression_efficiency() {
        // Create highly compressible data
        let original: Vec<u8> = "A".repeat(1000).into_bytes();

        let compressed = compress_zlib(&original).expect("Compression failed");

        // This highly repetitive data should compress well
        assert!(
            compressed.len() < original.len() / 2,
            "Highly repetitive data should compress to less than 50% of original size"
        );

        let decompressed =
            decompress_zlib(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_bzip2_round_trip() {
        let original = b"Hello, World! This is a test of bzip2 compression in MPQ archives.";

        let compressed = compress_bzip2(original).expect("Compression failed");

        // Note: Small data might not compress well
        println!(
            "Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

        let decompressed =
            decompress_bzip2(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_sparse_decompression() {
        // Test sparse format: [count | data] or [0x80 | zero_count]
        let compressed = vec![
            5, b'H', b'e', b'l', b'l', b'o', // 5 bytes of data
            0x85, // 5 zeros (0x80 | 5)
            5, b'W', b'o', b'r', b'l', b'd', // 5 bytes of data
            0xFF, // End marker
        ];

        let decompressed = decompress_sparse(&compressed, 15).expect("Decompression failed");
        let expected = b"Hello\0\0\0\0\0World";

        assert_eq!(decompressed, expected);
    }

    #[test]
    fn test_lzma_round_trip() {
        use std::io::{BufReader, Cursor};

        let original = b"Hello, World! This is a test of LZMA compression in MPQ archives. \
                     LZMA should provide good compression ratios.";

        // Test compression
        let cursor = Cursor::new(original);
        let mut input = BufReader::new(cursor);
        let mut compressed = Vec::new();

        lzma_rs::lzma_compress(&mut input, &mut compressed).expect("Compression failed");

        println!(
            "LZMA - Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

        // Test decompression
        let cursor = Cursor::new(&compressed);
        let mut input = BufReader::new(cursor);
        let mut decompressed = Vec::new();

        lzma_rs::lzma_decompress(&mut input, &mut decompressed).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_lzma_api() {
        let original = b"Test data for LZMA compression through the public API";

        // Test through our wrapper API
        let compressed = compress(original, flags::LZMA).expect("Compression failed");
        let decompressed =
            decompress(&compressed, flags::LZMA, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_xz_format() {
        use std::io::{BufReader, Cursor};

        let original = b"Test data for XZ format";

        // Test XZ compression
        let cursor = Cursor::new(original);
        let mut input = BufReader::new(cursor);
        let mut compressed = Vec::new();

        lzma_rs::xz_compress(&mut input, &mut compressed).expect("XZ compression failed");

        // Test XZ decompression
        let cursor = Cursor::new(&compressed);
        let mut input = BufReader::new(cursor);
        let mut decompressed = Vec::new();

        lzma_rs::xz_decompress(&mut input, &mut decompressed).expect("XZ decompression failed");

        assert_eq!(decompressed, original);
    }
    #[test]
    fn test_compress_api_small_data() {
        // Test that the public compress API works with small data
        let original = b"Small data";

        // Test uncompressed
        let result = compress(original, 0).expect("Compression failed");
        assert_eq!(result, original);

        // Test zlib - might not reduce size for small data
        let compressed = compress(original, flags::ZLIB).expect("Compression failed");
        let decompressed =
            decompress(&compressed, flags::ZLIB, original.len()).expect("Decompression failed");
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_api() {
        let original = b"Test data for compression";

        // Test uncompressed
        let result = decompress(original, 0, original.len()).expect("Decompression failed");
        assert_eq!(result, original);

        // Test zlib
        let compressed = compress_zlib(original).expect("Compression failed");
        let result =
            decompress(&compressed, flags::ZLIB, original.len()).expect("Decompression failed");
        assert_eq!(result, original);
    }
}
