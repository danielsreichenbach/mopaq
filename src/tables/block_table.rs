//! Block table

use super::{Table, TableError};
use crate::crypto::{BLOCK_TABLE_KEY, decrypt_block, encrypt_block};
use std::io::{Read, Seek, SeekFrom, Write};

/// Block flags - these define various properties of the file
pub mod block_flags {
    /// File is compressed
    pub const COMPRESSED: u32 = 0x00000200;
    /// File is encrypted
    pub const ENCRYPTED: u32 = 0x00010000;
    /// File is a patch file
    pub const PATCH_FILE: u32 = 0x00100000;
    /// Single unit file (not split into sectors)
    pub const SINGLE_UNIT: u32 = 0x01000000;
    /// File exists (deleted files may still have entries)
    pub const EXISTS: u32 = 0x80000000;

    // Compression method masks
    /// Mask for compression methods
    pub const COMPRESSION_MASK: u32 = 0x0000FF00;
    /// Position of compression bits
    pub const COMPRESSION_BITS: u32 = 8;

    // Specific compression types
    /// PKWARE Implode compression
    pub const COMPRESS_PKWARE: u32 = 0x00000100;
    /// Huffman compression
    pub const COMPRESS_HUFFMAN: u32 = 0x00000200;
    /// zlib compression
    pub const COMPRESS_ZLIB: u32 = 0x00000800;
    /// bzip2 compression
    pub const COMPRESS_BZIP2: u32 = 0x00001000;
    /// LZMA compression
    pub const COMPRESS_LZMA: u32 = 0x00002000;
    /// Sparse compression
    pub const COMPRESS_SPARSE: u32 = 0x00004000;
    /// IMA ADPCM compression (mono)
    pub const COMPRESS_IMA_ADPCM_MONO: u32 = 0x00008000;
    /// IMA ADPCM compression (stereo)
    pub const COMPRESS_IMA_ADPCM_STEREO: u32 = 0x00010000;
}

/// Block table entry in an MPQ archive
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockEntry {
    /// Offset of the file in the archive
    pub offset: u32,
    /// Compressed size of the file
    pub compressed_size: u32,
    /// Uncompressed size of the file
    pub file_size: u32,
    /// Flags (encryption, compression, etc.)
    pub flags: u32,
}

impl BlockEntry {
    /// Creates a new empty block entry
    pub fn new() -> Self {
        Self {
            offset: 0,
            compressed_size: 0,
            file_size: 0,
            flags: 0,
        }
    }

    /// Returns true if this file is compressed
    pub fn is_compressed(&self) -> bool {
        (self.flags & block_flags::COMPRESSED) != 0
    }

    /// Returns true if this file is encrypted
    pub fn is_encrypted(&self) -> bool {
        (self.flags & block_flags::ENCRYPTED) != 0
    }

    /// Returns true if this file is a patch file
    pub fn is_patch_file(&self) -> bool {
        (self.flags & block_flags::PATCH_FILE) != 0
    }

    /// Returns true if this file is a single unit (not split into sectors)
    pub fn is_single_unit(&self) -> bool {
        (self.flags & block_flags::SINGLE_UNIT) != 0
    }

    /// Returns true if this file exists (not deleted)
    pub fn exists(&self) -> bool {
        (self.flags & block_flags::EXISTS) != 0
    }

    /// Gets the compression methods used for this file
    pub fn compression_mask(&self) -> u32 {
        self.flags & block_flags::COMPRESSION_MASK
    }
}

impl Default for BlockEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Block table in an MPQ archive
pub struct BlockTable {
    /// The block entries
    entries: Vec<BlockEntry>,
}

impl BlockTable {
    /// Creates a new block table with the specified size
    pub fn new(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        entries.resize(size, BlockEntry::new());

        Self { entries }
    }

    /// Gets a reference to an entry at the specified index
    pub fn get(&self, index: usize) -> Option<&BlockEntry> {
        self.entries.get(index)
    }

    /// Gets a mutable reference to an entry at the specified index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut BlockEntry> {
        self.entries.get_mut(index)
    }

    /// Gets the vector of all block entries
    pub fn entries(&self) -> &[BlockEntry] {
        &self.entries
    }

    /// Gets a mutable vector of all block entries
    pub fn entries_mut(&mut self) -> &mut [BlockEntry] {
        &mut self.entries
    }

    /// Adds a new entry to the block table
    pub fn add_entry(&mut self, entry: BlockEntry) -> usize {
        let index = self.entries.len();
        self.entries.push(entry);
        index
    }
}

impl Table for BlockTable {
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
            self.entries.resize(size, BlockEntry::new());
        }

        // Seek to the block table position
        reader
            .seek(SeekFrom::Start(offset))
            .map_err(|e| TableError::IoError(e))?;

        // Calculate table size in bytes
        let table_bytes = size * std::mem::size_of::<BlockEntry>();
        let mut buffer = vec![0u8; table_bytes];

        // Read the encrypted table
        reader
            .read_exact(&mut buffer)
            .map_err(|e| TableError::IoError(e))?;

        // Decrypt the block table
        decrypt_block(&mut buffer, BLOCK_TABLE_KEY)
            .map_err(|e| TableError::DecryptionError(e.to_string()))?;

        // Parse the decrypted data into block entries
        for i in 0..size {
            let offset = i * std::mem::size_of::<BlockEntry>();
            let entry = &mut self.entries[i];

            entry.offset = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);

            entry.compressed_size = u32::from_le_bytes([
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);

            entry.file_size = u32::from_le_bytes([
                buffer[offset + 8],
                buffer[offset + 9],
                buffer[offset + 10],
                buffer[offset + 11],
            ]);

            entry.flags = u32::from_le_bytes([
                buffer[offset + 12],
                buffer[offset + 13],
                buffer[offset + 14],
                buffer[offset + 15],
            ]);
        }

        Ok(())
    }

    fn write_to<W: Write + Seek>(&self, writer: &mut W, offset: u64) -> Result<(), TableError> {
        // Seek to the block table position
        writer
            .seek(SeekFrom::Start(offset))
            .map_err(|e| TableError::IoError(e))?;

        // Calculate table size in bytes
        let size = self.entries.len();
        let table_bytes = size * std::mem::size_of::<BlockEntry>();
        let mut buffer = vec![0u8; table_bytes];

        // Pack block entries into the buffer
        for i in 0..size {
            let offset = i * std::mem::size_of::<BlockEntry>();
            let entry = &self.entries[i];

            buffer[offset..offset + 4].copy_from_slice(&entry.offset.to_le_bytes());
            buffer[offset + 4..offset + 8].copy_from_slice(&entry.compressed_size.to_le_bytes());
            buffer[offset + 8..offset + 12].copy_from_slice(&entry.file_size.to_le_bytes());
            buffer[offset + 12..offset + 16].copy_from_slice(&entry.flags.to_le_bytes());
        }

        // Encrypt the block table
        encrypt_block(&mut buffer, BLOCK_TABLE_KEY)
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
    fn test_block_entry() {
        let mut entry = BlockEntry::new();

        // Test empty entry
        assert_eq!(entry.offset, 0);
        assert_eq!(entry.compressed_size, 0);
        assert_eq!(entry.file_size, 0);
        assert_eq!(entry.flags, 0);

        // Test flag checking
        entry.flags = block_flags::COMPRESSED | block_flags::ENCRYPTED;
        assert!(entry.is_compressed());
        assert!(entry.is_encrypted());
        assert!(!entry.is_patch_file());
        assert!(!entry.is_single_unit());

        // Test compression mask
        entry.flags = block_flags::COMPRESS_ZLIB | block_flags::COMPRESS_HUFFMAN;
        assert_eq!(
            entry.compression_mask(),
            block_flags::COMPRESS_ZLIB | block_flags::COMPRESS_HUFFMAN
        );
    }

    #[test]
    fn test_block_table_creation() {
        let table = BlockTable::new(10);
        assert_eq!(table.size(), 10);

        // Check that all entries are empty
        for entry in table.entries() {
            assert_eq!(entry.offset, 0);
            assert_eq!(entry.compressed_size, 0);
            assert_eq!(entry.file_size, 0);
            assert_eq!(entry.flags, 0);
        }
    }

    #[test]
    fn test_block_table_get() {
        let mut table = BlockTable::new(5);

        // Modify an entry
        let index = 2;
        if let Some(entry) = table.get_mut(index) {
            entry.offset = 1000;
            entry.compressed_size = 500;
            entry.file_size = 1000;
            entry.flags = block_flags::COMPRESSED;
        }

        // Get the entry back
        let entry = table.get(index).unwrap();
        assert_eq!(entry.offset, 1000);
        assert_eq!(entry.compressed_size, 500);
        assert_eq!(entry.file_size, 1000);
        assert_eq!(entry.flags, block_flags::COMPRESSED);
    }

    #[test]
    fn test_block_table_add_entry() {
        let mut table = BlockTable::new(0);

        // Add some entries
        let entry1 = BlockEntry {
            offset: 100,
            compressed_size: 200,
            file_size: 300,
            flags: block_flags::COMPRESSED,
        };

        let entry2 = BlockEntry {
            offset: 400,
            compressed_size: 500,
            file_size: 600,
            flags: block_flags::ENCRYPTED,
        };

        let index1 = table.add_entry(entry1);
        let index2 = table.add_entry(entry2);

        assert_eq!(index1, 0);
        assert_eq!(index2, 1);
        assert_eq!(table.size(), 2);

        // Check the entries
        let stored_entry1 = table.get(index1).unwrap();
        assert_eq!(stored_entry1.offset, entry1.offset);
        assert_eq!(stored_entry1.compressed_size, entry1.compressed_size);
        assert_eq!(stored_entry1.file_size, entry1.file_size);
        assert_eq!(stored_entry1.flags, entry1.flags);

        let stored_entry2 = table.get(index2).unwrap();
        assert_eq!(stored_entry2.offset, entry2.offset);
        assert_eq!(stored_entry2.compressed_size, entry2.compressed_size);
        assert_eq!(stored_entry2.file_size, entry2.file_size);
        assert_eq!(stored_entry2.flags, entry2.flags);
    }

    #[test]
    fn test_block_table_read_write() {
        // Create a block table with some entries
        let mut original_table = BlockTable::new(0);

        original_table.add_entry(BlockEntry {
            offset: 100,
            compressed_size: 200,
            file_size: 300,
            flags: block_flags::COMPRESSED,
        });

        original_table.add_entry(BlockEntry {
            offset: 400,
            compressed_size: 500,
            file_size: 600,
            flags: block_flags::ENCRYPTED,
        });

        // Create a buffer to write to
        let mut buffer = Cursor::new(Vec::new());

        // Write the table to the buffer
        original_table.write_to(&mut buffer, 0).unwrap();

        // Reset cursor position
        buffer.set_position(0);

        // Create a new table to read into
        let mut new_table = BlockTable::new(0);

        // Read the table from the buffer
        new_table.read_from(&mut buffer, 0, 2).unwrap();

        // Verify the entries were read correctly
        assert_eq!(new_table.size(), 2);

        let entry1 = new_table.get(0).unwrap();
        assert_eq!(entry1.offset, 100);
        assert_eq!(entry1.compressed_size, 200);
        assert_eq!(entry1.file_size, 300);
        assert_eq!(entry1.flags, block_flags::COMPRESSED);

        let entry2 = new_table.get(1).unwrap();
        assert_eq!(entry2.offset, 400);
        assert_eq!(entry2.compressed_size, 500);
        assert_eq!(entry2.file_size, 600);
        assert_eq!(entry2.flags, block_flags::ENCRYPTED);
    }
}
