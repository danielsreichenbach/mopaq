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

/// Extended header that precedes HET/BET tables
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) struct ExtendedHeader {
    /// Signature 'HET\x1A' (0x1A544548) or 'BET\x1A' (0x1A544542)
    pub signature: u32,
    /// Version (always 1)
    pub version: u32,
    /// Size of the contained table data (excluding this header)
    pub data_size: u32,
}

/// Hash Entry Table (HET) header structure for MPQ v3+
/// This follows the extended header in the file
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HetHeader {
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

        // Check if we have at least the extended header (12 bytes)
        if data.len() < 12 {
            return Err(Error::invalid_format(
                "HET table too small for extended header",
            ));
        }

        // Parse the extended header (first 12 bytes - never encrypted)
        let ext_signature = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let ext_version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let ext_data_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

        log::debug!(
            "HET extended header: sig=0x{:08X}, ver={}, data_size={}",
            ext_signature,
            ext_version,
            ext_data_size
        );

        // Verify extended header
        if ext_signature != Self::SIGNATURE {
            return Err(Error::invalid_format("Invalid HET extended signature"));
        }

        // The data after the extended header may be encrypted
        if key != 0 && data.len() > 12 {
            log::debug!(
                "Decrypting HET data after extended header with key 0x{:08X}",
                key
            );
            let data_portion = &mut data[12..];
            decrypt_table_data(data_portion, key);
        }

        // Check for compression by comparing sizes
        let total_size = data.len();
        let expected_uncompressed_size = ext_data_size as usize + 12; // data_size + header

        log::debug!(
            "HET table total_size={}, expected_uncompressed_size={}",
            total_size,
            expected_uncompressed_size
        );

        let table_data = if expected_uncompressed_size > total_size {
            // Table is compressed - the data after extended header contains compressed data
            log::debug!("HET table is compressed");

            if data.len() <= 12 {
                return Err(Error::invalid_format(
                    "No compressed data after HET extended header",
                ));
            }

            // First byte after extended header is compression type
            let compression_type = data[12];
            log::debug!("HET compression type: 0x{:02X}", compression_type);

            // Decompress the data (skip extended header and compression byte)
            let compressed_data = &data[13..];
            let mut decompressed =
                decompress(compressed_data, compression_type, ext_data_size as usize)?;

            // Reconstruct the full table with extended header
            let mut full_table = Vec::with_capacity(12 + decompressed.len());
            full_table.extend_from_slice(&data[..12]); // Extended header
            full_table.append(&mut decompressed); // Decompressed data
            full_table
        } else {
            // Table is not compressed
            log::debug!("HET table is NOT compressed");
            data
        };

        // Parse header - skip the extended header (first 12 bytes)
        let header = Self::parse_header(&table_data[12..])?;

        // Copy values from packed struct to avoid alignment issues
        let table_size = header.table_size;
        let max_file_count = header.max_file_count;
        let hash_table_size = header.hash_table_size;
        let hash_entry_size = header.hash_entry_size;
        let total_index_size = header.total_index_size;
        let index_size = header.index_size;

        log::debug!(
            "HET header parsed: table_size={}, max_file_count={}, hash_table_size={}, hash_entry_size={}, total_index_size={}, index_size={}",
            table_size,
            max_file_count,
            hash_table_size,
            hash_entry_size,
            total_index_size,
            index_size
        );

        // No need to validate signature/version - they're in the extended header
        // which we already validated above

        // Extract hash table and file indices - data starts after extended header
        let data_start = 12; // Extended header size
        let header_size = std::mem::size_of::<HetHeader>();
        let hash_table_start = data_start + header_size;
        let hash_table_end = hash_table_start + header.hash_table_size as usize;

        let file_indices_start = hash_table_end;
        let file_indices_size = (header.total_index_size as usize).div_ceil(8); // Convert bits to bytes
        let file_indices_end = file_indices_start + file_indices_size;

        log::debug!(
            "HET table layout: data_start={}, header_size={}, hash_table: {}..{}, indices: {}..{}, total_needed={}",
            data_start,
            header_size,
            hash_table_start,
            hash_table_end,
            file_indices_start,
            file_indices_end,
            file_indices_end
        );

        if table_data.len() < file_indices_end {
            return Err(Error::invalid_format(format!(
                "HET table data too small: have {} bytes, need {} bytes",
                table_data.len(),
                file_indices_end
            )));
        }

        let hash_table = table_data[hash_table_start..hash_table_end].to_vec();
        let file_indices = table_data[file_indices_start..file_indices_end].to_vec();

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
        let hash_mask = if self.header.hash_entry_size >= 64 {
            u64::MAX
        } else {
            (1u64 << self.header.hash_entry_size) - 1
        };
        let index_mask = if self.header.index_size >= 64 {
            u64::MAX
        } else {
            (1u64 << self.header.index_size) - 1
        };

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
        let mask = if self.header.hash_entry_size >= 64 {
            u64::MAX
        } else {
            (1u64 << self.header.hash_entry_size) - 1
        };
        let entry = (value >> bit_shift) & mask;
        Some(entry)
    }
}
