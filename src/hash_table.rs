use crate::error::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

/// MPQ hash table entry
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MpqHashEntry {
    /// File path hash part A
    pub name_1: u32,

    /// File path hash part B
    pub name_2: u32,

    /// Language of the file
    pub locale: u16,

    /// Platform the file is used for
    pub platform: u16,

    /// Index into the block table
    pub block_index: u32,
}

/// Special values for hash table entries
pub mod hash_entry {
    /// Empty hash table entry
    pub const EMPTY: u32 = 0xFFFFFFFF;

    /// Deleted hash table entry
    pub const DELETED: u32 = 0xFFFFFFFE;

    /// Index mask
    pub const INDEX_MASK: u32 = 0x0FFFFFFF;
}

impl MpqHashEntry {
    /// Create a new empty hash entry
    pub fn new_empty() -> Self {
        Self {
            name_1: hash_entry::EMPTY,
            name_2: hash_entry::EMPTY,
            locale: 0,
            platform: 0,
            block_index: hash_entry::EMPTY,
        }
    }

    /// Create a new deleted hash entry
    pub fn new_deleted() -> Self {
        Self {
            name_1: hash_entry::DELETED,
            name_2: hash_entry::DELETED,
            locale: 0,
            platform: 0,
            block_index: hash_entry::DELETED,
        }
    }

    /// Check if this entry is empty
    pub fn is_empty(&self) -> bool {
        self.name_1 == hash_entry::EMPTY && self.block_index == hash_entry::EMPTY
    }

    /// Check if this entry is deleted
    pub fn is_deleted(&self) -> bool {
        self.name_1 == hash_entry::DELETED && self.block_index == hash_entry::DELETED
    }

    /// Check if this entry is valid (not empty and not deleted)
    pub fn is_valid(&self) -> bool {
        !self.is_empty() && !self.is_deleted()
    }

    /// Read a hash entry from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let name_1 = reader.read_u32::<LittleEndian>()?;
        let name_2 = reader.read_u32::<LittleEndian>()?;
        let locale = reader.read_u16::<LittleEndian>()?;
        let platform = reader.read_u16::<LittleEndian>()?;
        let block_index = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            name_1,
            name_2,
            locale,
            platform,
            block_index,
        })
    }

    /// Write a hash entry to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.name_1)?;
        writer.write_u32::<LittleEndian>(self.name_2)?;
        writer.write_u16::<LittleEndian>(self.locale)?;
        writer.write_u16::<LittleEndian>(self.platform)?;
        writer.write_u32::<LittleEndian>(self.block_index)?;
        Ok(())
    }
}

/// Table of hash entries
#[derive(Debug, Clone)]
pub struct MpqHashTable {
    /// The entries in the hash table
    pub entries: Vec<MpqHashEntry>,
}

impl MpqHashTable {
    /// Create a new hash table with the given number of entries
    pub fn new(entry_count: usize) -> Self {
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(MpqHashEntry::new_empty());
        }
        Self { entries }
    }

    /// Read a hash table from a reader
    pub fn read<R: Read + Seek>(reader: &mut R, entry_count: usize) -> Result<Self> {
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(MpqHashEntry::read(reader)?);
        }
        Ok(Self { entries })
    }

    /// Write a hash table to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        for entry in &self.entries {
            entry.write(writer)?;
        }
        Ok(())
    }

    /// Find a file in the hash table using its hashes
    pub fn find_entry(
        &self,
        name_a: u32,
        name_b: u32,
        locale: u16,
    ) -> Option<(usize, &MpqHashEntry)> {
        let mask = (self.entries.len() - 1) as u32;
        let mut start_index = name_a & mask;
        let mut i = start_index;

        loop {
            let entry = &self.entries[i as usize];

            if entry.is_empty() {
                // We found an empty entry, so the file doesn't exist
                return None;
            }

            if entry.name_1 == name_a
                && entry.name_2 == name_b
                && (entry.locale == locale || locale == 0 || entry.locale == 0)
            {
                // We found a matching entry
                return Some((i as usize, entry));
            }

            // Move to the next entry
            i = (i + 1) & mask;
            if i == start_index {
                // We've gone all the way around the table
                return None;
            }
        }
    }

    /// Add a file to the hash table
    pub fn add_entry(
        &mut self,
        name_a: u32,
        name_b: u32,
        locale: u16,
        platform: u16,
        block_index: u32,
    ) -> Result<()> {
        // First, check if the file already exists
        if let Some((idx, _)) = self.find_entry(name_a, name_b, locale) {
            // Replace the entry
            self.entries[idx] = MpqHashEntry {
                name_1: name_a,
                name_2: name_b,
                locale,
                platform,
                block_index,
            };
            return Ok(());
        }

        // Find an empty or deleted slot
        let mask = (self.entries.len() - 1) as u32;
        let mut start_index = name_a & mask;
        let mut i = start_index;

        loop {
            let entry = &self.entries[i as usize];

            if entry.is_empty() || entry.is_deleted() {
                // Found an empty or deleted slot
                self.entries[i as usize] = MpqHashEntry {
                    name_1: name_a,
                    name_2: name_b,
                    locale,
                    platform,
                    block_index,
                };
                return Ok(());
            }

            // Move to the next entry
            i = (i + 1) & mask;
            if i == start_index {
                // We've gone all the way around the table
                return Err(crate::error::MopaqError::TableFull);
            }
        }
    }
}

/// MPQ hash functions
pub mod hash {
    /// MPQ hash types
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HashType {
        /// Hash used for table offset
        TableOffset,
        /// Hash used for file name A
        NameA,
        /// Hash used for file name B
        NameB,
    }

    /// Encryption table for MPQ hashing
    static CRYPT_TABLE: [u32; 0x500] = generate_crypt_table();

    /// Generate the MPQ hashing table
    const fn generate_crypt_table() -> [u32; 0x500] {
        let mut table = [0u32; 0x500];
        let mut seed: u32 = 0x0010_0001;

        let mut i = 0;
        while i < 0x100 {
            let mut j = 0;
            while j < 5 {
                let index = i + j * 0x100;
                seed = (seed * 125 + 3) % 0x002A_AAAB;
                let t1 = (seed & 0xFFFF) << 0x10;
                seed = (seed * 125 + 3) % 0x002A_AAAB;
                let t2 = seed & 0xFFFF;

                table[index] = t1 | t2;

                j += 1;
            }
            i += 1;
        }

        table
    }

    /// Hash a string using the MPQ hash function
    pub fn hash_string(string: &str, hash_type: HashType) -> u32 {
        let string = string.to_uppercase();
        let mut seed1: u32 = 0x7FED_7FED;
        let mut seed2: u32 = 0xEEEE_EEEE;

        for ch in string.bytes() {
            let ch = ch as usize;
            let value = match hash_type {
                HashType::TableOffset => CRYPT_TABLE[ch],
                HashType::NameA => CRYPT_TABLE[0x100 + ch],
                HashType::NameB => CRYPT_TABLE[0x200 + ch],
            };

            // This is the correct MPQ hash algorithm
            seed1 = value ^ (seed1.wrapping_add(seed2));
            seed2 = ch as u32 + seed1 + seed2 + (seed2 << 5) + 3;
        }

        seed1
    }

    /// Generate all three hashes for a file name
    pub fn hash_filename(filename: &str) -> (u32, u32, u32) {
        (
            hash_string(filename, HashType::TableOffset),
            hash_string(filename, HashType::NameA),
            hash_string(filename, HashType::NameB),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_hash_entry_roundtrip() {
        // Create a hash entry
        let original = MpqHashEntry {
            name_1: 0x12345678,
            name_2: 0x87654321,
            locale: 0x0000,
            platform: 0x0000,
            block_index: 0x00000001,
        };

        // Create a buffer and write the entry
        let mut buffer = Cursor::new(Vec::new());
        original.write(&mut buffer).unwrap();

        // Reset the cursor and read the entry back
        buffer.set_position(0);
        let read_back = MpqHashEntry::read(&mut buffer).unwrap();

        // Verify the entries match
        assert_eq!(read_back.name_1, original.name_1);
        assert_eq!(read_back.name_2, original.name_2);
        assert_eq!(read_back.locale, original.locale);
        assert_eq!(read_back.platform, original.platform);
        assert_eq!(read_back.block_index, original.block_index);
    }

    #[test]
    fn test_hash_table_roundtrip() {
        // Create a hash table
        let mut original = MpqHashTable::new(4);
        original.entries[0] = MpqHashEntry {
            name_1: 0x12345678,
            name_2: 0x87654321,
            locale: 0x0000,
            platform: 0x0000,
            block_index: 0x00000001,
        };

        // Create a buffer and write the table
        let mut buffer = Cursor::new(Vec::new());
        original.write(&mut buffer).unwrap();

        // Reset the cursor and read the table back
        buffer.set_position(0);
        let read_back = MpqHashTable::read(&mut buffer, 4).unwrap();

        // Verify the tables match
        assert_eq!(read_back.entries.len(), original.entries.len());
        assert_eq!(read_back.entries[0].name_1, original.entries[0].name_1);
        assert_eq!(read_back.entries[0].name_2, original.entries[0].name_2);
        assert_eq!(
            read_back.entries[0].block_index,
            original.entries[0].block_index
        );
    }

    #[test]
    fn test_hash_string() {
        let file = "(listfile)";

        let hash_a = hash::hash_string(file, hash::HashType::TableOffset);

        // Expected hash values (from StormLib test cases)
        assert_eq!(hash_a, 0x5F3DE859);
    }

    #[test]
    fn test_find_entry() {
        // Create a hash table
        let mut hash_table = MpqHashTable::new(4);

        // Add an entry
        hash_table.entries[0] = MpqHashEntry {
            name_1: 0x12345678,
            name_2: 0x87654321,
            locale: 0x0000,
            platform: 0x0000,
            block_index: 0x00000001,
        };

        // Find the entry
        let found = hash_table.find_entry(0x12345678, 0x87654321, 0x0000);
        assert!(found.is_some());
        let (idx, entry) = found.unwrap();
        assert_eq!(idx, 0);
        assert_eq!(entry.block_index, 0x00000001);

        // Try to find a non-existent entry
        let not_found = hash_table.find_entry(0x11111111, 0x22222222, 0x0000);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_add_entry() {
        // Create a hash table
        let mut hash_table = MpqHashTable::new(4);

        // Add an entry
        hash_table
            .add_entry(0x12345678, 0x87654321, 0x0000, 0x0000, 0x00000001)
            .unwrap();

        // Find the entry
        let found = hash_table.find_entry(0x12345678, 0x87654321, 0x0000);
        assert!(found.is_some());
        let (idx, entry) = found.unwrap();
        assert_eq!(entry.block_index, 0x00000001);

        // Update the entry
        hash_table
            .add_entry(0x12345678, 0x87654321, 0x0000, 0x0000, 0x00000002)
            .unwrap();

        // Find the updated entry
        let found = hash_table.find_entry(0x12345678, 0x87654321, 0x0000);
        assert!(found.is_some());
        let (_, entry) = found.unwrap();
        assert_eq!(entry.block_index, 0x00000002);
    }
}
