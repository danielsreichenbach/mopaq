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
    // First byte indicates compression type(s) if multiple compression
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    let compression = CompressionMethod::from_flags(method);

    match compression {
        CompressionMethod::None => {
            // No compression, just return the data
            Ok(data.to_vec())
        }
        CompressionMethod::Zlib => decompress_zlib(data, decompressed_size),
        CompressionMethod::BZip2 => decompress_bzip2(data, decompressed_size),
        CompressionMethod::Lzma => decompress_lzma(data, decompressed_size),
        CompressionMethod::Sparse => decompress_sparse(data, decompressed_size),
        CompressionMethod::PKWare => Err(Error::compression(
            "PKWare decompression not yet implemented",
        )),
        CompressionMethod::Huffman => Err(Error::compression(
            "Huffman decompression not yet implemented",
        )),
        CompressionMethod::AdpcmMono | CompressionMethod::AdpcmStereo => Err(Error::compression(
            "ADPCM decompression not yet implemented",
        )),
        CompressionMethod::Multiple(flags) => decompress_multiple(data, flags, decompressed_size),
    }
}

/// Decompress using zlib/deflate
fn decompress_zlib(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::with_capacity(expected_size);

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| Error::compression(format!("Zlib decompression failed: {}", e)))?;

    if decompressed.len() != expected_size {
        return Err(Error::compression(format!(
            "Decompressed size mismatch: expected {}, got {}",
            expected_size,
            decompressed.len()
        )));
    }

    Ok(decompressed)
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
    use lzma::decompress;

    let decompressed = decompress(data)
        .map_err(|e| Error::compression(format!("LZMA decompression failed: {:?}", e)))?;

    if decompressed.len() != expected_size {
        return Err(Error::compression(format!(
            "Decompressed size mismatch: expected {}, got {}",
            expected_size,
            decompressed.len()
        )));
    }

    Ok(decompressed)
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

    // The first byte indicates which compression was used last
    let compression_order = data[0];
    let compressed_data = &data[1..];

    // Decompress using the indicated method
    let mut decompressed = match compression_order {
        flags::ZLIB => decompress_zlib(compressed_data, expected_size)?,
        flags::BZIP2 => decompress_bzip2(compressed_data, expected_size)?,
        flags::SPARSE => decompress_sparse(compressed_data, expected_size)?,
        // PKWare is never used as the final compression in multiple compression
        _ => {
            return Err(Error::compression(format!(
                "Unknown compression order byte: 0x{:02X}",
                compression_order
            )))
        }
    };

    // Note: In practice, PKWare (if present in flags) is applied first,
    // but we handle the decompression in reverse order.
    // Since we don't support PKWare yet, we'll just return what we have.

    if flags & flags::PKWARE != 0 {
        // TODO: Apply PKWare decompression
        log::warn!("Multiple compression with PKWare not fully supported");
    }

    Ok(decompressed)
}

/// Compress data using the specified compression method
pub fn compress(data: &[u8], method: u8) -> Result<Vec<u8>> {
    let compression = CompressionMethod::from_flags(method);

    match compression {
        CompressionMethod::None => Ok(data.to_vec()),
        CompressionMethod::Zlib => compress_zlib(data),
        CompressionMethod::BZip2 => compress_bzip2(data),
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
        assert!(compressed.len() < original.len());

        let decompressed =
            decompress_zlib(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_bzip2_round_trip() {
        let original = b"Hello, World! This is a test of bzip2 compression in MPQ archives.";

        let compressed = compress_bzip2(original).expect("Compression failed");

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
