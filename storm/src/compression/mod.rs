//! Compression and decompression algorithms for MPQ files

use crate::Result;

/// Compression method flags
pub mod flags {
    pub const HUFFMAN: u8 = 0x01;
    pub const ZLIB: u8 = 0x02;
    pub const PKWARE: u8 = 0x08;
    pub const BZIP2: u8 = 0x10;
    pub const SPARSE: u8 = 0x20;
    pub const ADPCM_MONO: u8 = 0x40;
    pub const ADPCM_STEREO: u8 = 0x80;
    pub const LZMA: u8 = 0x12;
}

/// Decompress data using the specified compression method
pub fn decompress(_data: &[u8], _method: u8, _decompressed_size: usize) -> Result<Vec<u8>> {
    todo!("Implement decompression")
}

/// Compress data using the specified compression method
pub fn compress(_data: &[u8], _method: u8) -> Result<Vec<u8>> {
    todo!("Implement compression")
}
