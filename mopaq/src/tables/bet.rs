//! BET (Block Extended Table) implementation for MPQ v3+ archives

use super::common::{decrypt_table_data, ReadLittleEndian};
use crate::compression::decompress;
use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// BET (Block Entry Table) for v3+ archives
#[derive(Debug)]
pub struct BetTable {
    /// Table header data
    pub header: BetHeader,
    /// File flags array
    pub file_flags: Vec<u32>,
    /// File table (bit-packed)
    pub file_table: Vec<u8>,
    /// BET hash array
    pub bet_hashes: Vec<u64>,
}

/// Block Entry Table (BET) header structure for MPQ v3+
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BetHeader {
    /// Signature 'BET\x1A' (0x1A544542)
    pub signature: u32,
    /// Version (always 1)
    pub version: u32,
    /// Size of the contained table data
    pub data_size: u32,
    /// Size of the entire table including header
    pub table_size: u32,
    /// Number of files in BET table
    pub file_count: u32,
    /// Unknown, typically 0x10
    pub unknown_08: u32,
    /// Size of one table entry in bits
    pub table_entry_size: u32,
    /// Bit positions for various fields
    pub bit_index_file_pos: u32,
    /// Bit index for file size field
    pub bit_index_file_size: u32,
    /// Bit index for compressed size field
    pub bit_index_cmp_size: u32,
    /// Bit index for flag index field
    pub bit_index_flag_index: u32,
    /// Bit index for unknown field
    pub bit_index_unknown: u32,
    /// Bit counts for various fields
    pub bit_count_file_pos: u32,
    /// Bit count for file size field
    pub bit_count_file_size: u32,
    /// Bit count for compressed size field
    pub bit_count_cmp_size: u32,
    /// Bit count for flag index field
    pub bit_count_flag_index: u32,
    /// Bit count for unknown field
    pub bit_count_unknown: u32,
    /// BET hash information
    pub total_bet_hash_size: u32,
    /// Extra bits in BET hash size
    pub bet_hash_size_extra: u32,
    /// Size of BET hash
    pub bet_hash_size: u32,
    /// Size of BET hash array
    pub bet_hash_array_size: u32,
    /// Number of flags
    pub flag_count: u32,
}

impl BetTable {
    const SIGNATURE: u32 = 0x1A544542; // "BET\x1A"

    /// Read and decompress/decrypt a BET table
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        offset: u64,
        compressed_size: u64,
        key: u32,
    ) -> Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;

        // Read the compressed/encrypted data
        let mut data = vec![0u8; compressed_size as usize];
        reader.read_exact(&mut data)?;

        // Decrypt if needed
        if key != 0 {
            decrypt_table_data(&mut data, key);
        }

        // Check for compression
        let decompressed_data = if data.len() >= 4 {
            let first_dword = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            if first_dword == Self::SIGNATURE {
                // Not compressed
                data
            } else {
                // Compressed
                let compression_type = data[0];
                decompress(&data[1..], compression_type, 0)?
            }
        } else {
            return Err(Error::invalid_format("BET table too small"));
        };

        // Parse header
        let header = Self::parse_header(&decompressed_data)?;

        // Validate header
        if header.signature != Self::SIGNATURE {
            return Err(Error::invalid_format("Invalid BET signature"));
        }
        if header.version != 1 {
            return Err(Error::invalid_format("Unsupported BET version"));
        }

        // Parse the rest of the table
        let mut cursor =
            std::io::Cursor::new(&decompressed_data[std::mem::size_of::<BetHeader>()..]);

        // Read file flags
        let mut file_flags = Vec::with_capacity(header.flag_count as usize);
        for _ in 0..header.flag_count {
            file_flags.push(cursor.read_u32_le()?);
        }

        // Calculate sizes
        let file_table_size =
            (header.file_count as usize * header.table_entry_size as usize).div_ceil(8);
        let mut file_table = vec![0u8; file_table_size];
        cursor.read_exact(&mut file_table)?;

        // Read BET hashes
        let hash_count = header.bet_hash_array_size / 8; // Each hash is 8 bytes
        let mut bet_hashes = Vec::with_capacity(hash_count as usize);
        for _ in 0..hash_count {
            bet_hashes.push(cursor.read_u64_le()?);
        }

        Ok(Self {
            header,
            file_flags,
            file_table,
            bet_hashes,
        })
    }

    /// Parse header from raw bytes
    fn parse_header(data: &[u8]) -> Result<BetHeader> {
        if data.len() < std::mem::size_of::<BetHeader>() {
            return Err(Error::invalid_format("BET header too small"));
        }

        let mut cursor = std::io::Cursor::new(data);
        Ok(BetHeader {
            signature: cursor.read_u32_le()?,
            version: cursor.read_u32_le()?,
            data_size: cursor.read_u32_le()?,
            table_size: cursor.read_u32_le()?,
            file_count: cursor.read_u32_le()?,
            unknown_08: cursor.read_u32_le()?,
            table_entry_size: cursor.read_u32_le()?,
            bit_index_file_pos: cursor.read_u32_le()?,
            bit_index_file_size: cursor.read_u32_le()?,
            bit_index_cmp_size: cursor.read_u32_le()?,
            bit_index_flag_index: cursor.read_u32_le()?,
            bit_index_unknown: cursor.read_u32_le()?,
            bit_count_file_pos: cursor.read_u32_le()?,
            bit_count_file_size: cursor.read_u32_le()?,
            bit_count_cmp_size: cursor.read_u32_le()?,
            bit_count_flag_index: cursor.read_u32_le()?,
            bit_count_unknown: cursor.read_u32_le()?,
            total_bet_hash_size: cursor.read_u32_le()?,
            bet_hash_size_extra: cursor.read_u32_le()?,
            bet_hash_size: cursor.read_u32_le()?,
            bet_hash_array_size: cursor.read_u32_le()?,
            flag_count: cursor.read_u32_le()?,
        })
    }

    /// Get file information by index
    pub fn get_file_info(&self, index: u32) -> Option<BetFileInfo> {
        if index >= self.header.file_count {
            return None;
        }

        // Read bit-packed entry
        let entry_bits = self.read_table_entry(index as usize)?;

        // Extract fields
        let file_pos = self.extract_bits(
            entry_bits,
            self.header.bit_index_file_pos,
            self.header.bit_count_file_pos,
        );
        let file_size = self.extract_bits(
            entry_bits,
            self.header.bit_index_file_size,
            self.header.bit_count_file_size,
        );
        let cmp_size = self.extract_bits(
            entry_bits,
            self.header.bit_index_cmp_size,
            self.header.bit_count_cmp_size,
        );
        let flag_index = self.extract_bits(
            entry_bits,
            self.header.bit_index_flag_index,
            self.header.bit_count_flag_index,
        ) as u32;

        // Get flags
        let flags = if flag_index < self.header.flag_count {
            self.file_flags[flag_index as usize]
        } else {
            0
        };

        Some(BetFileInfo {
            file_pos,
            file_size,
            compressed_size: cmp_size,
            flags,
        })
    }

    /// Read a table entry from bit-packed data
    fn read_table_entry(&self, index: usize) -> Option<u64> {
        let bit_offset = index * self.header.table_entry_size as usize;
        let byte_offset = bit_offset / 8;
        let bit_shift = bit_offset % 8;

        if byte_offset + 8 > self.file_table.len() {
            return None;
        }

        // Read enough bytes to get the full entry
        let mut value = 0u64;
        let bytes_needed = (bit_shift + self.header.table_entry_size as usize)
            .div_ceil(8)
            .min(8);

        for i in 0..bytes_needed {
            if byte_offset + i < self.file_table.len() {
                value |= (self.file_table[byte_offset + i] as u64) << (i * 8);
            }
        }

        // Shift and mask to get the actual entry
        let entry = (value >> bit_shift) & ((1u64 << self.header.table_entry_size) - 1);
        Some(entry)
    }

    /// Extract bits from a value
    fn extract_bits(&self, value: u64, bit_offset: u32, bit_count: u32) -> u64 {
        let mask = (1u64 << bit_count) - 1;
        (value >> bit_offset) & mask
    }
}

/// File information from BET table
#[derive(Debug)]
pub struct BetFileInfo {
    /// File position in archive
    pub file_pos: u64,
    /// Uncompressed file size
    pub file_size: u64,
    /// Compressed file size
    pub compressed_size: u64,
    /// File flags
    pub flags: u32,
}
