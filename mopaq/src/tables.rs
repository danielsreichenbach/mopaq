//! MPQ table structures (hash, block, HET, BET)

use crate::crypto::decrypt_block;
use crate::hash::{hash_string, hash_type};
use crate::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

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
            name_1: cursor.read_u32::<LittleEndian>()?,
            name_2: cursor.read_u32::<LittleEndian>()?,
            locale: cursor.read_u16::<LittleEndian>()?,
            platform: cursor.read_u16::<LittleEndian>()?,
            block_index: cursor.read_u32::<LittleEndian>()?,
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
    pub const FLAG_IMPLODE: u32 = 0x00000100;
    pub const FLAG_COMPRESS: u32 = 0x00000200;
    pub const FLAG_ENCRYPTED: u32 = 0x00010000;
    pub const FLAG_FIX_KEY: u32 = 0x00020000;
    pub const FLAG_PATCH_FILE: u32 = 0x00100000;
    pub const FLAG_SINGLE_UNIT: u32 = 0x01000000;
    pub const FLAG_DELETE_MARKER: u32 = 0x02000000;
    pub const FLAG_SECTOR_CRC: u32 = 0x04000000;
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
            file_pos: cursor.read_u32::<LittleEndian>()?,
            compressed_size: cursor.read_u32::<LittleEndian>()?,
            file_size: cursor.read_u32::<LittleEndian>()?,
            flags: cursor.read_u32::<LittleEndian>()?,
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

        // Decrypt the table
        let key = hash_string("(hash table)", hash_type::FILE_KEY);

        // Cast to u32 array for decryption
        let ptr = raw_data.as_mut_ptr() as *mut u32;
        let u32_len = byte_size / 4;
        let u32_data = unsafe { std::slice::from_raw_parts_mut(ptr, u32_len) };

        decrypt_block(u32_data, key);

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
                if locale == 0 || entry.locale == 0 || entry.locale == locale {
                    if entry.is_valid() {
                        return Some((index, entry));
                    }
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

        // Decrypt the table
        let key = hash_string("(block table)", hash_type::FILE_KEY);

        // Cast to u32 array for decryption
        let ptr = raw_data.as_mut_ptr() as *mut u32;
        let u32_len = byte_size / 4;
        let u32_data = unsafe { std::slice::from_raw_parts_mut(ptr, u32_len) };

        decrypt_block(u32_data, key);

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
            entries.push(reader.read_u16::<LittleEndian>()?);
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
