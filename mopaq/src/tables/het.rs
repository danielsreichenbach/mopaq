//! HET (Hash Extended Table) implementation for MPQ v3+ archives

use super::common::{decrypt_table_data, ReadLittleEndian};
use crate::compression::decompress;
use crate::crypto::jenkins_hash;
use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// HET (Hash Entry Table) for v3+ archives
#[derive(Debug)]
pub struct HetTable {
    /// Table header data
    pub header: HetHeader,
    /// Hash table data (variable bit entries)
    pub hash_table: Vec<u8>,
    /// File index data (variable bit entries)
    pub file_indices: Vec<u8>,
}

/// Hash Entry Table (HET) header structure for MPQ v3+
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HetHeader {
    /// Signature 'HET\x1A' (0x1A544548)
    pub signature: u32,
    /// Version (always 1)
    pub version: u32,
    /// Size of the contained table data
    pub data_size: u32,
    /// Size of the entire hash table including header
    pub table_size: u32,
    /// Maximum number of files in the MPQ
    pub max_file_count: u32,
    /// Size of the hash table in bytes
    pub hash_table_size: u32,
    /// Effective size of the hash entry in bits
    pub hash_entry_size: u32,
    /// Total size of file index in bits
    pub total_index_size: u32,
    /// Extra bits in the file index
    pub index_size_extra: u32,
    /// Effective size of the file index in bits
    pub index_size: u32,
    /// Size of the block index subtable in bytes
    pub block_table_size: u32,
}

impl HetTable {
    const SIGNATURE: u32 = 0x1A544548; // "HET\x1A"

    /// Read and decompress/decrypt a HET table
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
                // Compressed - first byte is compression type
                let compression_type = data[0];
                decompress(&data[1..], compression_type, 0)?
            }
        } else {
            return Err(Error::invalid_format("HET table too small"));
        };

        // Parse header
        let header = Self::parse_header(&decompressed_data)?;

        // Validate header
        if header.signature != Self::SIGNATURE {
            return Err(Error::invalid_format("Invalid HET signature"));
        }
        if header.version != 1 {
            return Err(Error::invalid_format("Unsupported HET version"));
        }

        // Extract hash table and file indices
        let header_size = std::mem::size_of::<HetHeader>();
        let hash_table_start = header_size;
        let hash_table_end = hash_table_start + header.hash_table_size as usize;

        let file_indices_start = hash_table_end;
        let file_indices_size = (header.total_index_size as usize).div_ceil(8); // Convert bits to bytes
        let file_indices_end = file_indices_start + file_indices_size;

        if decompressed_data.len() < file_indices_end {
            return Err(Error::invalid_format("HET table data too small"));
        }

        let hash_table = decompressed_data[hash_table_start..hash_table_end].to_vec();
        let file_indices = decompressed_data[file_indices_start..file_indices_end].to_vec();

        Ok(Self {
            header,
            hash_table,
            file_indices,
        })
    }

    /// Parse header from raw bytes
    fn parse_header(data: &[u8]) -> Result<HetHeader> {
        if data.len() < std::mem::size_of::<HetHeader>() {
            return Err(Error::invalid_format("HET header too small"));
        }

        let mut cursor = std::io::Cursor::new(data);
        Ok(HetHeader {
            signature: cursor.read_u32_le()?,
            version: cursor.read_u32_le()?,
            data_size: cursor.read_u32_le()?,
            table_size: cursor.read_u32_le()?,
            max_file_count: cursor.read_u32_le()?,
            hash_table_size: cursor.read_u32_le()?,
            hash_entry_size: cursor.read_u32_le()?,
            total_index_size: cursor.read_u32_le()?,
            index_size_extra: cursor.read_u32_le()?,
            index_size: cursor.read_u32_le()?,
            block_table_size: cursor.read_u32_le()?,
        })
    }

    /// Find a file in the HET table
    pub fn find_file(&self, filename: &str) -> Option<u32> {
        let hash = jenkins_hash(filename);
        let hash_mask = (1u64 << self.header.hash_entry_size) - 1;
        let index_mask = (1u64 << self.header.index_size) - 1;

        // Calculate hash table index
        let hash_table_entries = self.header.hash_table_size * 8 / self.header.hash_entry_size;
        let hash_index = (hash & (hash_table_entries as u64 - 1)) as usize;

        // Read hash entry
        let hash_entry = self.read_hash_entry(hash_index)?;
        let name_hash = hash & hash_mask;

        if (hash_entry & hash_mask) != name_hash {
            return None; // Hash mismatch
        }

        // Extract file index from hash entry
        let file_index = (hash_entry >> self.header.hash_entry_size) & index_mask;

        // Verify file index is valid
        if file_index >= self.header.max_file_count as u64 {
            return None;
        }

        Some(file_index as u32)
    }

    /// Read a hash entry from bit-packed data
    fn read_hash_entry(&self, index: usize) -> Option<u64> {
        let bit_offset = index * self.header.hash_entry_size as usize;
        let byte_offset = bit_offset / 8;
        let bit_shift = bit_offset % 8;

        if byte_offset + 8 > self.hash_table.len() {
            return None;
        }

        // Read 64 bits starting from byte_offset
        let mut value = 0u64;
        for i in 0..8 {
            if byte_offset + i < self.hash_table.len() {
                value |= (self.hash_table[byte_offset + i] as u64) << (i * 8);
            }
        }

        // Shift and mask to get the actual entry
        let entry = (value >> bit_shift) & ((1u64 << self.header.hash_entry_size) - 1);
        Some(entry)
    }
}
