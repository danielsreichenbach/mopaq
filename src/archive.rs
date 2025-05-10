use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use memmap2::{Mmap, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::block_table::{MpqBlockEntry, MpqBlockTable, block_flags};
use crate::error::{MopaqError, Result};
use crate::hash_table::{MpqHashEntry, MpqHashTable, hash};
use crate::header::{MPQ_HEADER_SIGNATURE, MpqHeader, MpqVersion};
use crate::user_header::{MpqUserHeader, read_mpq_header, write_mpq_header};
use crate::utils::{calculate_hash_table_size, get_sector_size};

/// MPQ archive
pub struct MpqArchive {
    /// The file handle
    file: File,

    /// Memory mapping of the file (if used)
    mmap: Option<Mmap>,

    /// The user header, if any
    user_header: Option<MpqUserHeader>,

    /// The MPQ header
    header: MpqHeader,

    /// The hash table
    hash_table: MpqHashTable,

    /// The block table
    block_table: MpqBlockTable,
}

impl MpqArchive {
    /// Open an existing MPQ archive
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Open the file
        let file = File::open(path)?;

        // Create a memory mapping
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        // Create a cursor to read from the mapping
        let mut cursor = std::io::Cursor::new(&mmap[..]);

        // Read the headers
        let (user_header, header) = read_mpq_header(&mut cursor)?;

        // Read the hash table
        cursor.seek(SeekFrom::Start(header.hash_table_offset as u64))?;
        let hash_table = MpqHashTable::read(&mut cursor, header.hash_table_entries as usize)?;

        // Read the block table
        cursor.seek(SeekFrom::Start(header.block_table_offset as u64))?;
        let block_table = MpqBlockTable::read(&mut cursor, header.block_table_entries as usize)?;

        Ok(Self {
            file,
            mmap: Some(mmap),
            user_header,
            header,
            hash_table,
            block_table,
        })
    }

    /// Create a new MPQ archive
    pub fn create<P: AsRef<Path>>(path: P, version: MpqVersion) -> Result<Self> {
        // Create the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        // Create the header based on the requested version
        let header = match version {
            MpqVersion::Version1 => MpqHeader::new_v1(),
            MpqVersion::Version2 => MpqHeader::new_v2(),
            MpqVersion::Version3 => MpqHeader::new_v3(),
            MpqVersion::Version4 => MpqHeader::new_v4(),
        };

        // Create empty tables
        let hash_table = MpqHashTable::new(4); // Minimum size
        let block_table = MpqBlockTable::new(0);

        // Create the archive
        let mut archive = Self {
            file,
            mmap: None,
            user_header: None,
            header,
            hash_table,
            block_table,
        };

        // Initialize the file structure
        archive.initialize()?;

        Ok(archive)
    }

    /// Initialize a new MPQ archive
    fn initialize(&mut self) -> Result<()> {
        // Calculate the offset of the hash table
        let mut offset = self.header.header_size as u64;

        // Update the header
        self.header.hash_table_offset = offset as u32;
        self.header.hash_table_entries = self.hash_table.entries.len() as u32;

        // Calculate the offset of the block table
        offset += (self.hash_table.entries.len() * 16) as u64;
        self.header.block_table_offset = offset as u32;
        self.header.block_table_entries = self.block_table.entries.len() as u32;

        // Calculate the total size of the archive
        offset += (self.block_table.entries.len() * 16) as u64;
        self.header.archive_size = offset as u32;
        if let Some(size_64) = self.header.archive_size_64.as_mut() {
            *size_64 = offset;
        }

        // Seek to the beginning of the file
        self.file.seek(SeekFrom::Start(0))?;

        // Write the header
        write_mpq_header(&mut self.file, self.user_header.as_ref(), &self.header)?;

        // Write the hash table
        self.file
            .seek(SeekFrom::Start(self.header.hash_table_offset as u64))?;
        self.hash_table.write(&mut self.file)?;

        // Write the block table
        self.file
            .seek(SeekFrom::Start(self.header.block_table_offset as u64))?;
        self.block_table.write(&mut self.file)?;

        Ok(())
    }

    /// Get the MPQ header
    pub fn header(&self) -> &MpqHeader {
        &self.header
    }

    /// Get the user header, if any
    pub fn user_header(&self) -> Option<&MpqUserHeader> {
        self.user_header.as_ref()
    }

    /// Get the hash table
    pub fn hash_table(&self) -> &MpqHashTable {
        &self.hash_table
    }

    /// Get the block table
    pub fn block_table(&self) -> &MpqBlockTable {
        &self.block_table
    }

    /// Find a file in the archive by name
    pub fn find_file(&self, filename: &str) -> Option<usize> {
        // Hash the filename
        let (_, name_a, name_b) = hash::hash_filename(filename);

        // Look up the file in the hash table
        self.hash_table
            .find_entry(name_a, name_b, 0)
            .map(|(_, entry)| (entry.block_index & 0x0FFFFFFF) as usize)
            .filter(|&idx| idx < self.block_table.entries.len())
    }

    /// Get the sector size for this archive
    pub fn sector_size(&self) -> u32 {
        get_sector_size(self.header.sector_size_shift)
    }

    /// Add a file to the archive
    pub fn add_file<P: AsRef<Path>>(&mut self, filepath: P, internal_name: &str) -> Result<()> {
        // If memory mapping is active, drop it as we'll be modifying the file
        self.mmap = None;

        // Open the file to add
        let mut file = File::open(&filepath)?;
        let file_size = file.metadata()?.len() as u32;

        // Hash the internal name
        let (offset_hash, name_a, name_b) = hash::hash_filename(internal_name);

        // Read the file content
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)?;

        // For simplicity, store uncompressed for now
        let block_entry = MpqBlockEntry {
            file_pos: self.header.archive_size,
            c_size: file_size,
            f_size: file_size,
            flags: block_flags::EXISTS | block_flags::SINGLE_UNIT,
        };

        // Add the block entry
        let block_index = self.block_table.entries.len();
        self.block_table.entries.push(block_entry);

        // Add the hash entry
        self.hash_table
            .add_entry(name_a, name_b, 0, 0, block_index as u32)?;

        // Seek to the end of the file and write the file data
        self.file
            .seek(SeekFrom::Start(self.header.archive_size as u64))?;
        self.file.write_all(&file_data)?;

        // Update the header
        self.header.archive_size += file_size;
        if let Some(size_64) = self.header.archive_size_64.as_mut() {
            *size_64 += file_size as u64;
        }
        self.header.block_table_entries = self.block_table.entries.len() as u32;

        // Update the tables
        self.file.seek(SeekFrom::Start(0))?;
        write_mpq_header(&mut self.file, self.user_header.as_ref(), &self.header)?;

        self.file
            .seek(SeekFrom::Start(self.header.hash_table_offset as u64))?;
        self.hash_table.write(&mut self.file)?;

        self.file
            .seek(SeekFrom::Start(self.header.block_table_offset as u64))?;
        self.block_table.write(&mut self.file)?;

        // Flush to ensure all data is written
        self.file.flush()?;

        Ok(())
    }

    /// Extract a file from the archive by name
    pub fn extract_file<P: AsRef<Path>>(&self, internal_name: &str, output_path: P) -> Result<()> {
        // Find the file
        let block_index = self
            .find_file(internal_name)
            .ok_or_else(|| MopaqError::FileNotFound(internal_name.to_string()))?;

        // Get the block entry
        let block_entry = &self.block_table.entries[block_index];

        // Check if the file exists
        if !block_entry.exists() {
            return Err(MopaqError::FileNotFound(internal_name.to_string()));
        }

        // For simplicity, we'll only support single unit files for now
        if !block_entry.is_single_unit() {
            return Err(MopaqError::UnsupportedFeature(
                "Multi-sector files".to_string(),
            ));
        }

        // Check for unsupported features
        if block_entry.is_encrypted() {
            return Err(MopaqError::UnsupportedFeature(
                "Encrypted files".to_string(),
            ));
        }
        if block_entry.is_compressed() {
            return Err(MopaqError::UnsupportedFeature(
                "Compressed files".to_string(),
            ));
        }

        // Create the output file
        let mut output_file = File::create(output_path)?;

        // Prepare to read the data
        let data_offset = block_entry.file_pos as u64;
        let data_size = block_entry.c_size as usize;

        // Buffer to hold the file data
        let mut buffer = vec![0u8; data_size];

        match &self.mmap {
            Some(mmap) => {
                // Use memory mapping
                if data_offset as usize + data_size <= mmap.len() {
                    let data = &mmap[data_offset as usize..(data_offset as usize + data_size)];
                    output_file.write_all(data)?;
                } else {
                    return Err(MopaqError::InvalidArchiveSize(
                        data_offset + data_size as u64,
                    ));
                }
            }
            None => {
                // Use file I/O
                let mut file = &self.file;
                let mut file_clone = file.try_clone()?; // Clone to avoid borrowing issues
                file_clone.seek(SeekFrom::Start(data_offset))?;
                file_clone.read_exact(&mut buffer)?;
                output_file.write_all(&buffer)?;
            }
        }

        Ok(())
    }

    /// List all files in the archive
    pub fn list_files(&self) -> Vec<String> {
        let mut result = Vec::new();

        // Since the hash table doesn't store filenames, we can only return
        // entries that are marked as valid and exist
        for (i, entry) in self.hash_table.entries.iter().enumerate() {
            if entry.is_valid() {
                let block_idx = entry.block_index as usize;
                if block_idx < self.block_table.entries.len() {
                    let block = &self.block_table.entries[block_idx];
                    if block.exists() {
                        // We don't have actual filenames, so we'll use a placeholder
                        // In a real implementation, we would need to store file names separately
                        result.push(format!("File#{}", i));
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, Write};
    use tempfile::tempdir;

    #[test]
    fn test_create_archive() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.mpq");

        // Create a new archive
        let archive = MpqArchive::create(&path, MpqVersion::Version1).unwrap();

        // Verify the header
        assert_eq!(archive.header.signature, MPQ_HEADER_SIGNATURE);
        assert_eq!(archive.header.format_version, 0);
        assert_eq!(archive.header.sector_size_shift, 3);

        // Verify the hash table
        assert_eq!(archive.hash_table.entries.len(), 4);

        // Verify the block table
        assert_eq!(archive.block_table.entries.len(), 0);
    }

    #[test]
    fn test_add_and_extract_file() {
        let dir = tempdir().unwrap();
        let archive_path = dir.path().join("test.mpq");
        let test_file_path = dir.path().join("test.txt");
        let output_path = dir.path().join("output.txt");

        // Create a test file with known content
        let test_content = b"This is a test file";
        {
            let mut test_file = File::create(&test_file_path).unwrap();
            test_file.write_all(test_content).unwrap();
            test_file.flush().unwrap();
        }

        // Step 1: Create a new archive and add the file
        {
            let mut archive = MpqArchive::create(&archive_path, MpqVersion::Version1).unwrap();
            archive.add_file(&test_file_path, "test.txt").unwrap();
            // Archive is dropped here, closing the file
        }

        // Step 2: Reopen the archive for extraction
        {
            let archive = MpqArchive::open(&archive_path).unwrap();
            archive.extract_file("test.txt", &output_path).unwrap();
        }

        // Read the extracted file
        let mut output_content = Vec::new();
        {
            let mut output_file = File::open(&output_path).unwrap();
            output_file.read_to_end(&mut output_content).unwrap();
        }

        // Verify the content
        assert_eq!(output_content, test_content);
    }
}
