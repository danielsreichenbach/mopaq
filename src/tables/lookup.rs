//! File lookup algorithms

use super::block_table::BlockEntry;
use super::ext_table::ExtBlockEntry;
use super::hash_table::HashEntry;
use super::{BlockTable, ExtendedBlockTable, HashTable, TableError};
use crate::crypto::hash::{HashType, compute_file_hashes};

/// Combined block entry with extended table support
pub struct CombinedEntry<'a> {
    /// Reference to the block entry
    pub block: &'a BlockEntry,
    /// Reference to the extended block entry (if available)
    pub ext: Option<&'a ExtBlockEntry>,
}

impl CombinedEntry<'_> {
    /// Get the full 64-bit file offset
    pub fn offset_64(&self) -> u64 {
        if let Some(ext) = self.ext {
            ((ext.offset_high as u64) << 32) | (self.block.offset as u64)
        } else {
            self.block.offset as u64
        }
    }

    /// Get the full 64-bit compressed size
    pub fn compressed_size_64(&self) -> u64 {
        if let Some(ext) = self.ext {
            ((ext.compressed_size_high as u64) << 32) | (self.block.compressed_size as u64)
        } else {
            self.block.compressed_size as u64
        }
    }

    /// Get the full 64-bit file size
    pub fn file_size_64(&self) -> u64 {
        if let Some(ext) = self.ext {
            ((ext.file_size_high as u64) << 32) | (self.block.file_size as u64)
        } else {
            self.block.file_size as u64
        }
    }
}

/// Finds a file in the MPQ archive by its filename
pub fn find_file<'a>(
    hash_table: &HashTable,
    block_table: &'a BlockTable,
    ext_table: Option<&'a ExtendedBlockTable>,
    filename: &str,
    locale: u16,
) -> Result<CombinedEntry<'a>, TableError> {
    // Calculate file hashes
    let (_, hash_a, hash_b) = compute_file_hashes(filename);

    // Find the file by hash
    find_file_by_hash(hash_table, block_table, ext_table, hash_a, hash_b, locale)
}

/// Finds a file in the MPQ archive by its hash values
pub fn find_file_by_hash<'a>(
    hash_table: &HashTable,
    block_table: &'a BlockTable,
    ext_table: Option<&'a ExtendedBlockTable>,
    hash_a: u32,
    hash_b: u32,
    locale: u16,
) -> Result<CombinedEntry<'a>, TableError> {
    // Find the hash entry
    let (_, hash_entry) = hash_table
        .find_entry(hash_a, hash_b, locale)
        .ok_or_else(|| {
            TableError::FileNotFound(format!("Hash A: {:#x}, Hash B: {:#x}", hash_a, hash_b))
        })?;

    // Get the block entry
    let block_index = hash_entry.block_index as usize;
    let block_entry = block_table
        .get(block_index)
        .ok_or_else(|| TableError::ReadError(format!("Invalid block index: {}", block_index)))?;

    // Get the extended block entry if available
    let ext_entry = ext_table.and_then(|table| table.get(block_index));

    Ok(CombinedEntry {
        block: block_entry,
        ext: ext_entry,
    })
}

/// Finds a file's hash entry and block entry by its filename
pub fn find_file_entry<'a>(
    hash_table: &'a HashTable,
    block_table: &'a BlockTable,
    filename: &str,
    locale: u16,
) -> Result<(&'a HashEntry, &'a BlockEntry), TableError> {
    // Calculate file hashes
    let (_, hash_a, hash_b) = compute_file_hashes(filename);

    // Find the hash entry
    let (_, hash_entry) = hash_table
        .find_entry(hash_a, hash_b, locale)
        .ok_or_else(|| TableError::FileNotFound(filename.to_string()))?;

    // Get the block entry
    let block_index = hash_entry.block_index as usize;
    let block_entry = block_table
        .get(block_index)
        .ok_or_else(|| TableError::ReadError(format!("Invalid block index: {}", block_index)))?;

    Ok((hash_entry, block_entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::block_table::BlockEntry;
    use crate::tables::block_table::block_flags;
    use crate::tables::hash_table::HashEntry;

    #[test]
    fn test_find_file() {
        // Create hash table and block table
        let mut hash_table = HashTable::new(16).unwrap();
        let mut block_table = BlockTable::new(0);

        // Add a file to the block table
        let block_index = block_table.add_entry(BlockEntry {
            offset: 1000,
            compressed_size: 500,
            file_size: 1000,
            flags: block_flags::COMPRESSED,
        });

        // Calculate hashes for the test filename
        let filename = "test.txt";
        let (_, hash_a, hash_b) = compute_file_hashes(filename);

        // Add the file to the hash table
        hash_table
            .add_entry(hash_a, hash_b, 0, 0, block_index as u32)
            .unwrap();

        // Try to find the file
        let result = find_file(&hash_table, &block_table, None, filename, 0);
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.block.offset, 1000);
        assert_eq!(entry.block.compressed_size, 500);
        assert_eq!(entry.block.file_size, 1000);
        assert_eq!(entry.block.flags, block_flags::COMPRESSED);
        assert!(entry.ext.is_none());

        // Try to find a non-existent file
        let result = find_file(&hash_table, &block_table, None, "nonexistent.txt", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_combined_entry() {
        // Create block and ext entries
        let block = BlockEntry {
            offset: 0x12345678,
            compressed_size: 0x23456789,
            file_size: 0x3456789A,
            flags: block_flags::COMPRESSED,
        };

        let ext = ExtBlockEntry {
            offset_high: 0x9ABC,
            compressed_size_high: 0xDEF0,
            file_size_high: 0x1234,
        };

        // Test without extended entry
        let entry1 = CombinedEntry {
            block: &block,
            ext: None,
        };

        assert_eq!(entry1.offset_64(), 0x12345678);
        assert_eq!(entry1.compressed_size_64(), 0x23456789);
        assert_eq!(entry1.file_size_64(), 0x3456789A);

        // Test with extended entry
        let entry2 = CombinedEntry {
            block: &block,
            ext: Some(&ext),
        };

        assert_eq!(entry2.offset_64(), 0x9ABC00000000 | 0x12345678);
        assert_eq!(entry2.compressed_size_64(), 0xDEF000000000 | 0x23456789);
        assert_eq!(entry2.file_size_64(), 0x123400000000 | 0x3456789A);
    }
}
