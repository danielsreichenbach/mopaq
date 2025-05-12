//! Extended block table implementation for MPQ archives

use super::{Table, TableError};
use crate::crypto::{MPQ_EXTENDED_BLOCK_TABLE_KEY, decrypt_block, encrypt_block};
use std::io::{Read, Seek, SeekFrom, Write};

/// Extended block table entry in an MPQ archive (v2+)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExtBlockEntry {
    /// High 16 bits of file offset
    pub offset_high: u16,
    /// High 16 bits of compressed size
    pub compressed_size_high: u16,
    /// High 16 bits of file size
    pub file_size_high: u16,
}

impl ExtBlockEntry {
    /// Creates a new empty extended block entry
    pub fn new() -> Self {
        Self {
            offset_high: 0,
            compressed_size_high: 0,
            file_size_high: 0,
        }
    }
}

/// Extended block table in an MPQ archive
pub struct ExtendedBlockTable {
    /// The extended block entries
    entries: Vec<ExtBlockEntry>,
}

impl ExtendedBlockTable {
    /// Creates a new extended block table with the specified size
    pub fn new(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        entries.resize(size, ExtBlockEntry::new());

        Self { entries }
    }

    /// Gets a reference to an entry at the specified index
    pub fn get(&self, index: usize) -> Option<&ExtBlockEntry> {
        self.entries.get(index)
    }

    /// Gets a mutable reference to an entry at the specified index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut ExtBlockEntry> {
        self.entries.get_mut(index)
    }

    /// Gets the vector of all extended block entries
    pub fn entries(&self) -> &[ExtBlockEntry] {
        &self.entries
    }

    /// Gets a mutable vector of all extended block entries
    pub fn entries_mut(&mut self) -> &mut [ExtBlockEntry] {
        &mut self.entries
    }

    /// Adds a new entry to the extended block table
    pub fn add_entry(&mut self, entry: ExtBlockEntry) -> usize {
        let index = self.entries.len();
        self.entries.push(entry);
        index
    }
}

impl Table for ExtendedBlockTable {
    fn size(&self) -> usize {
        self.entries.len()
    }

    fn read_from<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        offset: u64,
        size: usize,
    ) -> Result<(), TableError> {
        // Resize the entries vector if needed
        if self.entries.len() != size {
            self.entries = Vec::with_capacity(size);
            self.entries.resize(size, ExtBlockEntry::new());
        }

        // Seek to the extended block table position
        reader
            .seek(SeekFrom::Start(offset))
            .map_err(|e| TableError::IoError(e))?;

        // Calculate table size in bytes
        let entry_size = std::mem::size_of::<ExtBlockEntry>();
        let table_bytes = size * entry_size;
        let mut buffer = vec![0u8; table_bytes];

        // Read the encrypted table
        reader
            .read_exact(&mut buffer)
            .map_err(|e| TableError::IoError(e))?;

        // Decrypt the extended block table
        decrypt_block(&mut buffer, MPQ_EXTENDED_BLOCK_TABLE_KEY)
            .map_err(|e| TableError::DecryptionError(e.to_string()))?;

        // Parse the decrypted data into extended block entries
        for i in 0..size {
            let offset = i * entry_size;
            let entry = &mut self.entries[i];

            entry.offset_high = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);

            entry.compressed_size_high =
                u16::from_le_bytes([buffer[offset + 2], buffer[offset + 3]]);

            entry.file_size_high = u16::from_le_bytes([buffer[offset + 4], buffer[offset + 5]]);
        }

        Ok(())
    }

    fn write_to<W: Write + Seek>(&self, writer: &mut W, offset: u64) -> Result<(), TableError> {
        // Seek to the extended block table position
        writer
            .seek(SeekFrom::Start(offset))
            .map_err(|e| TableError::IoError(e))?;

        // Calculate table size in bytes
        let size = self.entries.len();
        let entry_size = std::mem::size_of::<ExtBlockEntry>();
        let table_bytes = size * entry_size;
        let mut buffer = vec![0u8; table_bytes];

        // Pack extended block entries into the buffer
        for i in 0..size {
            let offset = i * entry_size;
            let entry = &self.entries[i];

            buffer[offset..offset + 2].copy_from_slice(&entry.offset_high.to_le_bytes());
            buffer[offset + 2..offset + 4]
                .copy_from_slice(&entry.compressed_size_high.to_le_bytes());
            buffer[offset + 4..offset + 6].copy_from_slice(&entry.file_size_high.to_le_bytes());
        }

        // Encrypt the extended block table
        encrypt_block(&mut buffer, MPQ_EXTENDED_BLOCK_TABLE_KEY)
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
    fn test_ext_block_entry() {
        let entry = ExtBlockEntry::new();

        // Test empty entry
        assert_eq!(entry.offset_high, 0);
        assert_eq!(entry.compressed_size_high, 0);
        assert_eq!(entry.file_size_high, 0);
    }

    #[test]
    fn test_ext_block_table_creation() {
        let table = ExtendedBlockTable::new(10);
        assert_eq!(table.size(), 10);

        // Check that all entries are empty
        for entry in table.entries() {
            assert_eq!(entry.offset_high, 0);
            assert_eq!(entry.compressed_size_high, 0);
            assert_eq!(entry.file_size_high, 0);
        }
    }

    #[test]
    fn test_ext_block_table_get() {
        let mut table = ExtendedBlockTable::new(5);

        // Modify an entry
        let index = 2;
        if let Some(entry) = table.get_mut(index) {
            entry.offset_high = 1;
            entry.compressed_size_high = 2;
            entry.file_size_high = 3;
        }

        // Get the entry back
        let entry = table.get(index).unwrap();
        assert_eq!(entry.offset_high, 1);
        assert_eq!(entry.compressed_size_high, 2);
        assert_eq!(entry.file_size_high, 3);
    }

    #[test]
    fn test_ext_block_table_add_entry() {
        let mut table = ExtendedBlockTable::new(0);

        // Add some entries
        let entry1 = ExtBlockEntry {
            offset_high: 1,
            compressed_size_high: 2,
            file_size_high: 3,
        };

        let entry2 = ExtBlockEntry {
            offset_high: 4,
            compressed_size_high: 5,
            file_size_high: 6,
        };

        let index1 = table.add_entry(entry1);
        let index2 = table.add_entry(entry2);

        assert_eq!(index1, 0);
        assert_eq!(index2, 1);
        assert_eq!(table.size(), 2);

        // Check the entries
        let stored_entry1 = table.get(index1).unwrap();
        assert_eq!(stored_entry1.offset_high, entry1.offset_high);
        assert_eq!(
            stored_entry1.compressed_size_high,
            entry1.compressed_size_high
        );
        assert_eq!(stored_entry1.file_size_high, entry1.file_size_high);

        let stored_entry2 = table.get(index2).unwrap();
        assert_eq!(stored_entry2.offset_high, entry2.offset_high);
        assert_eq!(
            stored_entry2.compressed_size_high,
            entry2.compressed_size_high
        );
        assert_eq!(stored_entry2.file_size_high, entry2.file_size_high);
    }

    #[test]
    fn test_ext_block_table_read_write() {
        // Create an extended block table with some entries
        let mut original_table = ExtendedBlockTable::new(0);

        original_table.add_entry(ExtBlockEntry {
            offset_high: 1,
            compressed_size_high: 2,
            file_size_high: 3,
        });

        original_table.add_entry(ExtBlockEntry {
            offset_high: 4,
            compressed_size_high: 5,
            file_size_high: 6,
        });

        // Create a buffer to write to
        let mut buffer = Cursor::new(Vec::new());

        // Write the table to the buffer
        original_table.write_to(&mut buffer, 0).unwrap();

        // Reset cursor position
        buffer.set_position(0);

        // Create a new table to read into
        let mut new_table = ExtendedBlockTable::new(0);

        // Read the table from the buffer
        new_table.read_from(&mut buffer, 0, 2).unwrap();

        // Verify the entries were read correctly
        assert_eq!(new_table.size(), 2);

        let entry1 = new_table.get(0).unwrap();
        assert_eq!(entry1.offset_high, 1);
        assert_eq!(entry1.compressed_size_high, 2);
        assert_eq!(entry1.file_size_high, 3);

        let entry2 = new_table.get(1).unwrap();
        assert_eq!(entry2.offset_high, 4);
        assert_eq!(entry2.compressed_size_high, 5);
        assert_eq!(entry2.file_size_high, 6);
    }
}
