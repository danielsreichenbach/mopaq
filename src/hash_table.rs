use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// MPQ hash table entry structure
#[derive(Debug, Clone, Copy)]
pub struct MpqHashEntry {
    /// File path hash A
    pub hash_a: u32,

    /// File path hash B
    pub hash_b: u32,

    /// Language of the file
    pub locale: u16,

    /// Platform the file is used for
    pub platform: u16,

    /// Block table index
    pub block_index: u32,
}

impl MpqHashEntry {
    /// Create a new empty hash entry
    pub fn empty() -> Self {
        MpqHashEntry {
            hash_a: 0xFFFFFFFF,
            hash_b: 0xFFFFFFFF,
            locale: 0xFFFF,
            platform: 0xFFFF,
            block_index: 0xFFFFFFFF,
        }
    }

    /// Check if this entry is empty
    pub fn is_empty(&self) -> bool {
        self.hash_a == 0xFFFFFFFF && self.hash_b == 0xFFFFFFFF
    }

    /// Check if this entry is deleted
    pub fn is_deleted(&self) -> bool {
        self.hash_a == 0xFFFFFFFF && self.hash_b != 0xFFFFFFFF
    }

    /// Read a hash entry from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let hash_a = reader.read_u32::<LittleEndian>()?;
        let hash_b = reader.read_u32::<LittleEndian>()?;
        let locale = reader.read_u16::<LittleEndian>()?;
        let platform = reader.read_u16::<LittleEndian>()?;
        let block_index = reader.read_u32::<LittleEndian>()?;

        Ok(MpqHashEntry {
            hash_a,
            hash_b,
            locale,
            platform,
            block_index,
        })
    }

    /// Write a hash entry to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<LittleEndian>(self.hash_a)?;
        writer.write_u32::<LittleEndian>(self.hash_b)?;
        writer.write_u16::<LittleEndian>(self.locale)?;
        writer.write_u16::<LittleEndian>(self.platform)?;
        writer.write_u32::<LittleEndian>(self.block_index)?;

        Ok(())
    }
}

/// MPQ hash table
#[derive(Debug, Clone)]
pub struct MpqHashTable {
    /// The entries in the hash table
    pub entries: Vec<MpqHashEntry>,
}

impl MpqHashTable {
    /// Create a new hash table with a given size
    pub fn new(size: usize) -> Self {
        // Size must be a power of 2
        let size = size.next_power_of_two();

        let mut entries = Vec::with_capacity(size);
        for _ in 0..size {
            entries.push(MpqHashEntry::empty());
        }

        MpqHashTable { entries }
    }

    /// Read a hash table from a reader
    pub fn read<R: Read + Seek>(reader: &mut R, offset: u64, entries: u32) -> io::Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;

        let mut hash_table = MpqHashTable {
            entries: Vec::with_capacity(entries as usize),
        };

        for _ in 0..entries {
            hash_table.entries.push(MpqHashEntry::read(reader)?);
        }

        Ok(hash_table)
    }

    /// Write the hash table to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W, offset: u64) -> io::Result<()> {
        writer.seek(SeekFrom::Start(offset))?;

        for entry in &self.entries {
            entry.write(writer)?;
        }

        Ok(())
    }

    /// Get the size of the hash table in bytes
    pub fn size(&self) -> u32 {
        (self.entries.len() * std::mem::size_of::<MpqHashEntry>()) as u32
    }

    /// Get the number of entries in the hash table
    pub fn len(&self) -> u32 {
        self.entries.len() as u32
    }

    /// Check if the hash table is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Calculate the hash A of a file name
    pub fn hash_string_a(string: &str) -> u32 {
        let mut hash: u32 = 0x1505;

        for c in string.to_uppercase().bytes() {
            hash = ((hash << 5) + hash) + c as u32;
        }

        hash
    }

    /// Calculate the hash B of a file name
    pub fn hash_string_b(string: &str) -> u32 {
        let mut hash: u32 = 0x0;

        for c in string.to_uppercase().bytes() {
            hash = ((hash << 5) + hash) ^ (c as u32);
        }

        hash
    }

    /// Calculate the hash table index for a file name
    pub fn hash_string_table_offset(string: &str, table_size: u32) -> u32 {
        let mut hash: u32 = 0;

        for c in string.to_uppercase().bytes() {
            hash = ((hash << 5) + hash) + c as u32;
        }

        hash % table_size
    }

    /// Find a file in the hash table
    pub fn find_file(&self, file_path: &str) -> Option<(usize, &MpqHashEntry)> {
        if self.entries.is_empty() {
            return None;
        }

        let hash_a = Self::hash_string_a(file_path);
        let hash_b = Self::hash_string_b(file_path);
        let table_size = self.entries.len() as u32;
        let start_index = Self::hash_string_table_offset(file_path, table_size) as usize;

        // Search the hash table
        let mut index = start_index;
        loop {
            let entry = &self.entries[index];

            if entry.is_empty() {
                // Empty slot, file not found
                return None;
            }

            if !entry.is_deleted() && entry.hash_a == hash_a && entry.hash_b == hash_b {
                // Found the file
                return Some((index, entry));
            }

            // Move to the next slot
            index = (index + 1) % self.entries.len();

            // If we've checked all slots, file not found
            if index == start_index {
                return None;
            }
        }
    }

    /// Add a file to the hash table
    pub fn add_file(
        &mut self,
        file_path: &str,
        locale: u16,
        platform: u16,
        block_index: u32,
    ) -> bool {
        if self.entries.is_empty() {
            return false;
        }

        let hash_a = Self::hash_string_a(file_path);
        let hash_b = Self::hash_string_b(file_path);
        let table_size = self.entries.len() as u32;
        let start_index = Self::hash_string_table_offset(file_path, table_size) as usize;

        // Search for an empty or deleted slot
        let mut index = start_index;
        let mut first_deleted = None;

        loop {
            let entry = &mut self.entries[index];

            if entry.is_empty() {
                // Empty slot, use it
                *entry = MpqHashEntry {
                    hash_a,
                    hash_b,
                    locale,
                    platform,
                    block_index,
                };
                return true;
            }

            if entry.is_deleted() && first_deleted.is_none() {
                // Remember the first deleted slot
                first_deleted = Some(index);
            }

            if !entry.is_deleted() && entry.hash_a == hash_a && entry.hash_b == hash_b {
                // File already exists, update it
                entry.locale = locale;
                entry.platform = platform;
                entry.block_index = block_index;
                return true;
            }

            // Move to the next slot
            index = (index + 1) % self.entries.len();

            // If we've checked all slots, use the first deleted slot if available
            if index == start_index {
                if let Some(deleted_index) = first_deleted {
                    self.entries[deleted_index] = MpqHashEntry {
                        hash_a,
                        hash_b,
                        locale,
                        platform,
                        block_index,
                    };
                    return true;
                }

                // Hash table is full
                return false;
            }
        }
    }

    /// Remove a file from the hash table
    pub fn remove_file(&mut self, file_path: &str) -> bool {
        if let Some((index, _)) = self.find_file(file_path) {
            // Mark the entry as deleted
            self.entries[index].hash_a = 0xFFFFFFFF;
            self.entries[index].hash_b = 0xFFFFFFFE; // Not empty, but deleted
            return true;
        }

        false
    }
}
