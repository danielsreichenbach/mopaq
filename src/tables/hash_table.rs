//! Hash table implementation for MPQ archives

use super::{Table, TableError};
use crate::crypto::{CryptoResult, HASH_TABLE_KEY, decrypt_block, encrypt_block};
use std::io::{Read, Seek, SeekFrom, Write};

/// Hash table entry in an MPQ archive
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HashEntry {
    /// First hash of filename (HashA)
    pub name_hash_a: u32,
    /// Second hash of filename (HashB)
    pub name_hash_b: u32,
    /// Language ID of this entry
    pub locale: u16,
    /// Platform ID of this entry
    pub platform: u16,
    /// Index into the block table
    pub block_index: u32,
}

impl HashEntry {
    /// Creates a new empty hash entry
    pub fn new() -> Self {
        Self {
            name_hash_a: 0,
            name_hash_b: 0,
            locale: 0,
            platform: 0,
            block_index: 0xFFFFFFFF, // Marks as empty (standard value in MPQ)
        }
    }

    /// Returns true if this hash entry is empty
    pub fn is_empty(&self) -> bool {
        self.block_index == 0xFFFFFFFF
    }

    /// Returns true if this hash entry has been deleted
    pub fn is_deleted(&self) -> bool {
        self.block_index == 0xFFFFFFFE
    }

    /// Marks this hash entry as deleted
    pub fn mark_deleted(&mut self) {
        self.block_index = 0xFFFFFFFE;
    }
}

/// Hash table in an MPQ archive
pub struct HashTable {
    /// The hash entries
    entries: Vec<HashEntry>,
    /// Size mask for quick lookup (size - 1)
    mask: u32,
}

impl HashTable {
    /// Creates a new hash table with the specified size
    /// The size must be a power of 2
    pub fn new(size: usize) -> Result<Self, TableError> {
        // Verify size is power of 2
        if size == 0 || (size & (size - 1)) != 0 {
            return Err(TableError::InvalidSize(size));
        }

        let mut entries = Vec::with_capacity(size);
        entries.resize(size, HashEntry::new());

        Ok(Self {
            entries,
            mask: (size - 1) as u32,
        })
    }

    /// Gets a reference to an entry at the specified index
    pub fn get(&self, index: usize) -> Option<&HashEntry> {
        self.entries.get(index)
    }

    /// Gets a mutable reference to an entry at the specified index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut HashEntry> {
        self.entries.get_mut(index)
    }

    /// Gets the vector of all hash entries
    pub fn entries(&self) -> &[HashEntry] {
        &self.entries
    }

    /// Gets a mutable vector of all hash entries
    pub fn entries_mut(&mut self) -> &mut [HashEntry] {
        &mut self.entries
    }

    /// Finds a hash entry by its hash values
    pub fn find_entry(&self, hash_a: u32, hash_b: u32, locale: u16) -> Option<(usize, &HashEntry)> {
        let start_index = (hash_a & self.mask) as usize;
        let table_size = self.entries.len();

        // Start from the calculated index and search linearly
        for i in 0..table_size {
            let index = (start_index + i) & (self.mask as usize);
            let entry = &self.entries[index];

            if entry.is_empty() {
                // Empty entry means the file is not in the table
                return None;
            }

            if !entry.is_deleted()
                && entry.name_hash_a == hash_a
                && entry.name_hash_b == hash_b
                && (locale == u16::MAX || entry.locale == locale || entry.locale == 0)
            {
                return Some((index, entry));
            }
        }

        None
    }

    /// Adds a new hash entry to the table
    pub fn add_entry(
        &mut self,
        hash_a: u32,
        hash_b: u32,
        locale: u16,
        platform: u16,
        block_index: u32,
    ) -> Result<usize, TableError> {
        let start_index = (hash_a & self.mask) as usize;
        let table_size = self.entries.len();

        // Find an empty or deleted slot
        for i in 0..table_size {
            let index = (start_index + i) & (self.mask as usize);
            let entry = &mut self.entries[index];

            if entry.is_empty() || entry.is_deleted() {
                // Found an available slot
                entry.name_hash_a = hash_a;
                entry.name_hash_b = hash_b;
                entry.locale = locale;
                entry.platform = platform;
                entry.block_index = block_index;

                return Ok(index);
            }
        }

        // The table is full
        Err(TableError::ReadError("Hash table is full".to_string()))
    }
}

impl Table for HashTable {
    fn size(&self) -> usize {
        self.entries.len()
    }

    fn read_from<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        offset: u64,
        size: usize,
    ) -> Result<(), TableError> {
        // Check if size matches the table size
        if size != self.entries.len() {
            return Err(TableError::InvalidSize(size));
        }

        // Seek to the hash table position
        reader
            .seek(SeekFrom::Start(offset))
            .map_err(|e| TableError::IoError(e))?;

        // Calculate table size in bytes
        let table_bytes = size * std::mem::size_of::<HashEntry>();
        let mut buffer = vec![0u8; table_bytes];

        // Read the encrypted table
        reader
            .read_exact(&mut buffer)
            .map_err(|e| TableError::IoError(e))?;

        // Decrypt the hash table
        decrypt_block(&mut buffer, HASH_TABLE_KEY)
            .map_err(|e| TableError::DecryptionError(e.to_string()))?;

        // Parse the decrypted data into hash entries
        for i in 0..size {
            let offset = i * std::mem::size_of::<HashEntry>();
            let entry = &mut self.entries[i];

            entry.name_hash_a = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);

            entry.name_hash_b = u32::from_le_bytes([
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);

            entry.locale = u16::from_le_bytes([buffer[offset + 8], buffer[offset + 9]]);

            entry.platform = u16::from_le_bytes([buffer[offset + 10], buffer[offset + 11]]);

            entry.block_index = u32::from_le_bytes([
                buffer[offset + 12],
                buffer[offset + 13],
                buffer[offset + 14],
                buffer[offset + 15],
            ]);
        }

        Ok(())
    }

    fn write_to<W: Write + Seek>(&self, writer: &mut W, offset: u64) -> Result<(), TableError> {
        // Seek to the hash table position
        writer
            .seek(SeekFrom::Start(offset))
            .map_err(|e| TableError::IoError(e))?;

        // Calculate table size in bytes
        let size = self.entries.len();
        let table_bytes = size * std::mem::size_of::<HashEntry>();
        let mut buffer = vec![0u8; table_bytes];

        // Pack hash entries into the buffer
        for i in 0..size {
            let offset = i * std::mem::size_of::<HashEntry>();
            let entry = &self.entries[i];

            buffer[offset..offset + 4].copy_from_slice(&entry.name_hash_a.to_le_bytes());
            buffer[offset + 4..offset + 8].copy_from_slice(&entry.name_hash_b.to_le_bytes());
            buffer[offset + 8..offset + 10].copy_from_slice(&entry.locale.to_le_bytes());
            buffer[offset + 10..offset + 12].copy_from_slice(&entry.platform.to_le_bytes());
            buffer[offset + 12..offset + 16].copy_from_slice(&entry.block_index.to_le_bytes());
        }

        // Encrypt the hash table
        encrypt_block(&mut buffer, HASH_TABLE_KEY)
            .map_err(|e| TableError::DecryptionError(e.to_string()))?;

        // Write the encrypted table
        writer
            .write_all(&buffer)
            .map_err(|e| TableError::IoError(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_hash_entry() {
        let mut entry = HashEntry::new();

        // Check initial state
        assert!(entry.is_empty());
        assert!(!entry.is_deleted());

        // Mark as deleted
        entry.mark_deleted();
        assert!(!entry.is_empty());
        assert!(entry.is_deleted());
    }

    #[test]
    fn test_hash_table_creation() {
        // Size is not power of 2
        let result = HashTable::new(10);
        assert!(result.is_err());

        // Zero size
        let result = HashTable::new(0);
        assert!(result.is_err());

        // Valid size
        let table = HashTable::new(16).unwrap();
        assert_eq!(table.size(), 16);
        assert_eq!(table.mask, 15);

        // Check that all entries are empty
        for entry in table.entries() {
            assert!(entry.is_empty());
        }
    }

    #[test]
    fn test_hash_table_find_entry() {
        let mut table = HashTable::new(16).unwrap();

        // Add an entry
        let hash_a = 0x12345678;
        let hash_b = 0x87654321;
        let block_index = 42;

        table.add_entry(hash_a, hash_b, 0, 0, block_index).unwrap();

        // Find the entry
        let (index, entry) = table.find_entry(hash_a, hash_b, 0).unwrap();
        assert_eq!(entry.block_index, block_index);

        // Try to find a non-existent entry
        let result = table.find_entry(0, 0, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_hash_table_read_write() {
        // We'll test by writing a table to a buffer, then reading it back
        let mut original_table = HashTable::new(8).unwrap();

        // Add some entries
        original_table
            .add_entry(0x11111111, 0x22222222, 0, 0, 1)
            .unwrap();
        original_table
            .add_entry(0x33333333, 0x44444444, 0, 0, 2)
            .unwrap();

        // Create a buffer to write to
        let mut buffer = Cursor::new(Vec::new());

        // Write the table to the buffer
        original_table.write_to(&mut buffer, 0).unwrap();

        // Reset cursor position
        buffer.set_position(0);

        // Create a new table to read into
        let mut new_table = HashTable::new(8).unwrap();

        // Read the table from the buffer
        new_table.read_from(&mut buffer, 0, 8).unwrap();

        // Verify the entries were read correctly
        let (_, entry1) = new_table.find_entry(0x11111111, 0x22222222, 0).unwrap();
        assert_eq!(entry1.block_index, 1);

        let (_, entry2) = new_table.find_entry(0x33333333, 0x44444444, 0).unwrap();
        assert_eq!(entry2.block_index, 2);
    }
}
