//! MPQ table structures (hash, block, HET, BET)

use crate::compression::decompress;
use crate::crypto::decrypt_block;
use crate::hash::{hash_string, hash_type, jenkins_hash};
use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// Helper trait for reading little-endian integers
trait ReadLittleEndian: Read {
    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
    
    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
    
    fn read_u64_le(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl<R: Read> ReadLittleEndian for R {}

/// Hash table entry (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HashEntry {
    /// The hash of the full file name (part A)
    pub name_1: u32,
    /// The hash of the full file name (part B)
    pub name_2: u32,
    /// The language of the file (Windows LANGID)
    pub locale: u16,
    /// The platform the file is used for
    pub platform: u16,
    /// Block table index or special value
    pub block_index: u32,
}

impl HashEntry {
    /// Value indicating the hash entry has never been used
    pub const EMPTY_NEVER_USED: u32 = 0xFFFFFFFF;
    /// Value indicating the hash entry was deleted
    pub const EMPTY_DELETED: u32 = 0xFFFFFFFE;

    /// Create an empty hash entry
    pub fn empty() -> Self {
        Self {
            name_1: 0,
            name_2: 0,
            locale: 0,
            platform: 0,
            block_index: Self::EMPTY_NEVER_USED,
        }
    }

    /// Check if this entry has never been used
    pub fn is_empty(&self) -> bool {
        self.block_index == Self::EMPTY_NEVER_USED
    }

    /// Check if this entry was deleted
    pub fn is_deleted(&self) -> bool {
        self.block_index == Self::EMPTY_DELETED
    }

    /// Check if this entry contains valid file information
    pub fn is_valid(&self) -> bool {
        self.block_index < Self::EMPTY_DELETED
    }

    /// Read a hash entry from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(Error::invalid_format("Hash entry too small"));
        }

        let mut cursor = std::io::Cursor::new(data);
        Ok(Self {
            name_1: cursor.read_u32_le()?,
            name_2: cursor.read_u32_le()?,
            locale: cursor.read_u16_le()?,
            platform: cursor.read_u16_le()?,
            block_index: cursor.read_u32_le()?,
        })
    }
}

/// Block table entry (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockEntry {
    /// Offset of the beginning of the file data, relative to the beginning of the archive
    pub file_pos: u32,
    /// Compressed file size
    pub compressed_size: u32,
    /// Size of uncompressed file
    pub file_size: u32,
    /// Flags for the file
    pub flags: u32,
}

impl BlockEntry {
    // Flag constants
    /// File is compressed using PKWARE Data compression library
    pub const FLAG_IMPLODE: u32 = 0x00000100;
    /// File is compressed using one or more compression methods
    pub const FLAG_COMPRESS: u32 = 0x00000200;
    /// File is encrypted
    pub const FLAG_ENCRYPTED: u32 = 0x00010000;
    /// The decryption key for the file is adjusted by the block position
    pub const FLAG_FIX_KEY: u32 = 0x00020000;
    /// The file is a patch file
    pub const FLAG_PATCH_FILE: u32 = 0x00100000;
    /// File is stored as a single unit, not split into sectors
    pub const FLAG_SINGLE_UNIT: u32 = 0x01000000;
    /// File is a deletion marker
    pub const FLAG_DELETE_MARKER: u32 = 0x02000000;
    /// File has checksums for each sector
    pub const FLAG_SECTOR_CRC: u32 = 0x04000000;
    /// File exists in the archive
    pub const FLAG_EXISTS: u32 = 0x80000000;

    /// Check if the file is compressed
    pub fn is_compressed(&self) -> bool {
        (self.flags & (Self::FLAG_IMPLODE | Self::FLAG_COMPRESS)) != 0
    }

    /// Check if the file is encrypted
    pub fn is_encrypted(&self) -> bool {
        (self.flags & Self::FLAG_ENCRYPTED) != 0
    }

    /// Check if the file is stored as a single unit
    pub fn is_single_unit(&self) -> bool {
        (self.flags & Self::FLAG_SINGLE_UNIT) != 0
    }

    /// Check if the file has sector CRCs
    pub fn has_sector_crc(&self) -> bool {
        (self.flags & Self::FLAG_SECTOR_CRC) != 0
    }

    /// Check if the file exists
    pub fn exists(&self) -> bool {
        (self.flags & Self::FLAG_EXISTS) != 0
    }

    /// Check if the file uses fixed key encryption
    pub fn has_fix_key(&self) -> bool {
        (self.flags & Self::FLAG_FIX_KEY) != 0
    }

    /// Read a block entry from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(Error::invalid_format("Block entry too small"));
        }

        let mut cursor = std::io::Cursor::new(data);
        Ok(Self {
            file_pos: cursor.read_u32_le()?,
            compressed_size: cursor.read_u32_le()?,
            file_size: cursor.read_u32_le()?,
            flags: cursor.read_u32_le()?,
        })
    }
}

/// Hash table
#[derive(Debug)]
pub struct HashTable {
    entries: Vec<HashEntry>,
}

impl HashTable {
    /// Create a new empty hash table
    pub fn new(size: usize) -> Result<Self> {
        // Validate size is power of 2
        if !crate::is_power_of_two(size as u32) {
            return Err(Error::hash_table("Hash table size must be power of 2"));
        }

        let entries = vec![HashEntry::empty(); size];
        Ok(Self { entries })
    }

    /// Read and decrypt a hash table from the archive
    pub fn read<R: Read + Seek>(reader: &mut R, offset: u64, size: u32) -> Result<Self> {
        // Validate size
        if !crate::is_power_of_two(size) {
            return Err(Error::hash_table("Hash table size must be power of 2"));
        }

        // Seek to hash table position
        reader.seek(SeekFrom::Start(offset))?;

        // Read raw data
        let byte_size = size as usize * 16; // 16 bytes per entry
        let mut raw_data = vec![0u8; byte_size];
        reader.read_exact(&mut raw_data)?;

        // Decrypt the table - SAFE VERSION
        let key = hash_string("(hash table)", hash_type::FILE_KEY);

        // Convert to u32s, decrypt, then convert back
        let mut u32_buffer: Vec<u32> = raw_data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        decrypt_block(&mut u32_buffer, key);

        // Write decrypted u32s back to bytes
        for (chunk, &decrypted) in raw_data.chunks_exact_mut(4).zip(&u32_buffer) {
            chunk.copy_from_slice(&decrypted.to_le_bytes());
        }

        // Parse entries
        let mut entries = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            let offset = i * 16;
            let entry = HashEntry::from_bytes(&raw_data[offset..offset + 16])?;
            entries.push(entry);
        }

        Ok(Self { entries })
    }

    /// Get all entries
    pub fn entries(&self) -> &[HashEntry] {
        &self.entries
    }

    /// Get a specific entry
    pub fn get(&self, index: usize) -> Option<&HashEntry> {
        self.entries.get(index)
    }

    /// Get the size of the hash table
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Find a file in the hash table
    pub fn find_file(&self, filename: &str, locale: u16) -> Option<(usize, &HashEntry)> {
        // Calculate hash values
        let name_a = hash_string(filename, hash_type::NAME_A);
        let name_b = hash_string(filename, hash_type::NAME_B);
        let start_index = hash_string(filename, hash_type::TABLE_OFFSET) as usize;

        let table_size = self.entries.len();
        let mut index = start_index & (table_size - 1);

        // Linear probing to find the file
        loop {
            let entry = &self.entries[index];

            // Check if this is our file
            if entry.name_1 == name_a && entry.name_2 == name_b {
                // Check locale (0 = default/any locale)
                if (locale == 0 || entry.locale == 0 || entry.locale == locale) && entry.is_valid()
                {
                    return Some((index, entry));
                }
            }

            // If we hit an empty entry that was never used, file doesn't exist
            if entry.is_empty() {
                return None;
            }

            // Continue to next entry
            index = (index + 1) & (table_size - 1);

            // If we've wrapped around to where we started, file doesn't exist
            if index == (start_index & (table_size - 1)) {
                return None;
            }
        }
    }

    /// Create a new hash table with mutable entries
    pub fn new_mut(size: usize) -> Result<Self> {
        // Validate size is power of 2
        if !crate::is_power_of_two(size as u32) {
            return Err(Error::hash_table("Hash table size must be power of 2"));
        }

        let entries = vec![HashEntry::empty(); size];
        Ok(Self { entries })
    }

    /// Get a mutable reference to a specific entry
    pub fn get_mut(&mut self, index: usize) -> Option<&mut HashEntry> {
        self.entries.get_mut(index)
    }

    /// Get mutable access to all entries
    pub fn entries_mut(&mut self) -> &mut [HashEntry] {
        &mut self.entries
    }

    /// Clear all entries to empty state
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = HashEntry::empty();
        }
    }
}

/// Block table
#[derive(Debug)]
pub struct BlockTable {
    entries: Vec<BlockEntry>,
}

impl BlockTable {
    /// Create a new empty block table
    pub fn new(size: usize) -> Result<Self> {
        let entries = vec![
            BlockEntry {
                file_pos: 0,
                compressed_size: 0,
                file_size: 0,
                flags: 0,
            };
            size
        ];
        Ok(Self { entries })
    }

    /// Read and decrypt a block table from the archive
    pub fn read<R: Read + Seek>(reader: &mut R, offset: u64, size: u32) -> Result<Self> {
        // Seek to block table position
        reader.seek(SeekFrom::Start(offset))?;

        // Read raw data
        let byte_size = size as usize * 16; // 16 bytes per entry
        let mut raw_data = vec![0u8; byte_size];
        reader.read_exact(&mut raw_data)?;

        // Decrypt the table - SAFE VERSION
        let key = hash_string("(block table)", hash_type::FILE_KEY);

        // Convert to u32s, decrypt, then convert back
        let mut u32_buffer: Vec<u32> = raw_data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        decrypt_block(&mut u32_buffer, key);

        // Write decrypted u32s back to bytes
        for (chunk, &decrypted) in raw_data.chunks_exact_mut(4).zip(&u32_buffer) {
            chunk.copy_from_slice(&decrypted.to_le_bytes());
        }

        // Parse entries
        let mut entries = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            let offset = i * 16;
            let entry = BlockEntry::from_bytes(&raw_data[offset..offset + 16])?;
            entries.push(entry);
        }

        Ok(Self { entries })
    }

    /// Get all entries
    pub fn entries(&self) -> &[BlockEntry] {
        &self.entries
    }

    /// Get a specific entry
    pub fn get(&self, index: usize) -> Option<&BlockEntry> {
        self.entries.get(index)
    }

    /// Get the size of the block table
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Create a new block table with mutable entries
    pub fn new_mut(size: usize) -> Result<Self> {
        let entries = vec![
            BlockEntry {
                file_pos: 0,
                compressed_size: 0,
                file_size: 0,
                flags: 0,
            };
            size
        ];
        Ok(Self { entries })
    }

    /// Get a mutable reference to a specific entry
    pub fn get_mut(&mut self, index: usize) -> Option<&mut BlockEntry> {
        self.entries.get_mut(index)
    }

    /// Get mutable access to all entries
    pub fn entries_mut(&mut self) -> &mut [BlockEntry] {
        &mut self.entries
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = BlockEntry {
                file_pos: 0,
                compressed_size: 0,
                file_size: 0,
                flags: 0,
            };
        }
    }
}

/// Hi-block table for archives > 4GB (v2+)
#[derive(Debug)]
pub struct HiBlockTable {
    entries: Vec<u16>,
}

impl HiBlockTable {
    /// Read the hi-block table
    pub fn read<R: Read + Seek>(reader: &mut R, offset: u64, size: u32) -> Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;

        let mut entries = Vec::with_capacity(size as usize);
        for _ in 0..size {
            entries.push(reader.read_u16_le()?);
        }

        Ok(Self { entries })
    }

    /// Get a hi-block entry
    pub fn get(&self, index: usize) -> Option<u16> {
        self.entries.get(index).copied()
    }

    /// Calculate full 64-bit file position
    pub fn get_file_pos_high(&self, index: usize) -> u64 {
        self.get(index).unwrap_or(0) as u64
    }
}

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

/// Helper function to decrypt table data
fn decrypt_table_data(data: &mut [u8], key: u32) {
    // Convert to u32 array for decryption
    let mut u32_buffer: Vec<u32> = data
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    decrypt_block(&mut u32_buffer, key);

    // Convert back to bytes
    for (i, &val) in u32_buffer.iter().enumerate() {
        let bytes = val.to_le_bytes();
        data[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_entry_states() {
        let empty = HashEntry::empty();
        assert!(empty.is_empty());
        assert!(!empty.is_deleted());
        assert!(!empty.is_valid());

        let deleted = HashEntry {
            name_1: 0,
            name_2: 0,
            locale: 0,
            platform: 0,
            block_index: HashEntry::EMPTY_DELETED,
        };
        assert!(!deleted.is_empty());
        assert!(deleted.is_deleted());
        assert!(!deleted.is_valid());

        let valid = HashEntry {
            name_1: 0x12345678,
            name_2: 0x9ABCDEF0,
            locale: 0,
            platform: 0,
            block_index: 0,
        };
        assert!(!valid.is_empty());
        assert!(!valid.is_deleted());
        assert!(valid.is_valid());
    }

    #[test]
    fn test_block_entry_flags() {
        let compressed = BlockEntry {
            file_pos: 0,
            compressed_size: 100,
            file_size: 200,
            flags: BlockEntry::FLAG_COMPRESS | BlockEntry::FLAG_EXISTS,
        };
        assert!(compressed.is_compressed());
        assert!(!compressed.is_encrypted());
        assert!(compressed.exists());

        let encrypted = BlockEntry {
            file_pos: 0,
            compressed_size: 100,
            file_size: 100,
            flags: BlockEntry::FLAG_ENCRYPTED | BlockEntry::FLAG_FIX_KEY | BlockEntry::FLAG_EXISTS,
        };
        assert!(encrypted.is_encrypted());
        assert!(encrypted.has_fix_key());
        assert!(!encrypted.is_compressed());
    }

    #[test]
    fn test_hash_table_size_validation() {
        // Valid sizes (powers of 2)
        assert!(HashTable::new(16).is_ok());
        assert!(HashTable::new(256).is_ok());
        assert!(HashTable::new(4096).is_ok());

        // Invalid sizes
        assert!(HashTable::new(15).is_err());
        assert!(HashTable::new(100).is_err());
        assert!(HashTable::new(0).is_err());
    }
}
