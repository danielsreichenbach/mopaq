//! MPQ archive handling
//! Provides functionality for reading MPQ archives

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::file::MpqFile;
use crate::header::MpqHeader;
use crate::listfile::read_listfile;
use crate::tables::{BlockTable, ExtendedBlockTable, HashTable, find_file, find_file_by_hash};

/// Reader trait for abstracting over different input sources
pub trait ReadSeek: Read + Seek + Send + Sync {}
impl<T: Read + Seek + Send + Sync> ReadSeek for T {}

/// An MPQ archive
pub struct MpqArchive {
    /// Path to the archive, if opened from a file
    path: Option<PathBuf>,

    /// The archive header
    header: MpqHeader,

    /// Offset of the header within the file
    header_offset: u64,

    /// The hash table
    hash_table: HashTable,

    /// The block table
    block_table: BlockTable,

    /// The extended block table (v2+ archives)
    ext_block_table: Option<ExtendedBlockTable>,

    /// Reader for accessing the archive data
    reader: Arc<Mutex<Box<dyn ReadSeek>>>,

    /// Known filenames from (listfile) if available
    filenames: Vec<String>,

    /// Filename to hash mapping for quicker lookup
    filename_map: HashMap<String, (u32, u32)>, // (HashA, HashB)
}

impl MpqArchive {
    /// Opens an MPQ archive from a file path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .map_err(|_| Error::ArchiveOpenError(path.as_ref().to_path_buf()))?;

        let reader = Box::new(BufReader::new(file));
        Self::from_reader(reader, Some(path.as_ref().to_path_buf()))
    }

    /// Creates an MPQ archive from a reader
    pub fn from_reader<R>(reader: Box<R>, path: Option<PathBuf>) -> Result<Self>
    where
        R: ReadSeek + 'static,
    {
        let mut reader = reader;

        // Find and read the MPQ header
        let (header, header_offset) = MpqHeader::find_and_read(&mut reader, None)?;

        // Validate the header
        header.validate()?;

        // Read the hash table
        let mut hash_table = HashTable::new(header.hash_table_entries as usize)?;
        let hash_table_offset = header_offset + header.hash_table_offset_64();
        hash_table.read_from(
            &mut reader,
            hash_table_offset,
            header.hash_table_entries as usize,
        )?;

        // Read the block table
        let mut block_table = BlockTable::new(header.block_table_entries as usize);
        let block_table_offset = header_offset + header.block_table_offset_64();
        block_table.read_from(
            &mut reader,
            block_table_offset,
            header.block_table_entries as usize,
        )?;

        // Read the extended block table if present (v2+)
        let ext_block_table = if header.format_version >= 2 {
            let mut ext_table = ExtendedBlockTable::new(header.block_table_entries as usize);
            let ext_offset = match header.format_version {
                2 => (header.ext_block_table_offset_high as u64) << 32,
                _ => 0, // In later versions, we'd need to handle this differently
            };

            if ext_offset > 0 {
                ext_table.read_from(
                    &mut reader,
                    header_offset + ext_offset,
                    header.block_table_entries as usize,
                )?;
                Some(ext_table)
            } else {
                None
            }
        } else {
            None
        };

        // Create the archive
        let mut archive = Self {
            path,
            header,
            header_offset,
            hash_table,
            block_table,
            ext_block_table,
            reader: Arc::new(Mutex::new(reader)),
            filenames: Vec::new(),
            filename_map: HashMap::new(),
        };

        // Try to load the listfile
        archive.load_listfile().ok(); // Ignore errors

        Ok(archive)
    }

    /// Gets a reference to the archive's header
    pub fn header(&self) -> &MpqHeader {
        &self.header
    }

    /// Gets the sector size for this archive
    pub fn sector_size(&self) -> u32 {
        self.header.sector_size()
    }

    /// Opens a file from the archive by name
    pub fn open_file(&self, filename: &str) -> Result<MpqFile> {
        // Look up the file in the hash and block tables
        let hash_a;
        let hash_b;

        // Try to use cached hash values if available
        if let Some(&(a, b)) = self.filename_map.get(filename) {
            hash_a = a;
            hash_b = b;
        } else {
            // Calculate hash values for the filename
            let (_, a, b) = crate::crypto::hash::compute_file_hashes(filename);
            hash_a = a;
            hash_b = b;
        }

        // Find the file in the tables
        let file_entry = find_file_by_hash(
            &self.hash_table,
            &self.block_table,
            self.ext_block_table.as_ref(),
            hash_a,
            hash_b,
            0, // Default locale
        )
        .map_err(|_| Error::FileNotFound(filename.to_string()))?;

        // Create the MPQ file
        MpqFile::new(
            filename.to_string(),
            file_entry.block,
            file_entry.ext,
            self.header_offset,
            self.sector_size(),
            Arc::clone(&self.reader),
        )
    }

    /// Loads the (listfile) if present
    fn load_listfile(&mut self) -> Result<()> {
        // Try to open the (listfile)
        match self.open_file("(listfile)") {
            Ok(file) => {
                // Read the listfile
                let data = file.read_data()?;
                let filenames = read_listfile(&data)?;

                // Build the filename map for quick lookup
                let mut filename_map = HashMap::new();
                for filename in &filenames {
                    let (_, hash_a, hash_b) = crate::crypto::hash::compute_file_hashes(filename);
                    filename_map.insert(filename.clone(), (hash_a, hash_b));
                }

                self.filenames = filenames;
                self.filename_map = filename_map;

                Ok(())
            }
            Err(_) => {
                // No listfile, which is okay
                Ok(())
            }
        }
    }

    /// Gets a list of known filenames in the archive
    pub fn filenames(&self) -> &[String] {
        &self.filenames
    }

    /// Gets the total number of files in the archive
    pub fn file_count(&self) -> usize {
        // Count non-empty entries in the block table
        let mut count = 0;
        for i in 0..self.block_table.size() {
            if let Some(entry) = self.block_table.get(i) {
                if entry.exists() {
                    count += 1;
                }
            }
        }
        count
    }

    /// Extracts a file to the specified path
    pub fn extract_file<P: AsRef<Path>>(&self, filename: &str, path: P) -> Result<()> {
        let file = self.open_file(filename)?;
        let data = file.read_data()?;

        std::fs::write(path, data).map_err(|e| Error::IoError(e))
    }

    /// Extracts all files to the specified directory
    pub fn extract_all<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();

        // Create directory if it doesn't exist
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(|e| Error::IoError(e))?;
        }

        // Extract each file
        for filename in &self.filenames {
            let dest_path = dir.join(filename);

            // Create parent directories if needed
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| Error::IoError(e))?;
                }
            }

            // Extract the file
            self.extract_file(filename, dest_path)?;
        }

        Ok(())
    }

    /// Checks if a file exists in the archive
    pub fn has_file(&self, filename: &str) -> bool {
        if let Some(&(hash_a, hash_b)) = self.filename_map.get(filename) {
            // Use cached hash values
            self.hash_table.find_entry(hash_a, hash_b, 0).is_some()
        } else {
            // Calculate hash values
            let (_, hash_a, hash_b) = crate::crypto::hash::compute_file_hashes(filename);
            self.hash_table.find_entry(hash_a, hash_b, 0).is_some()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    // Helper to create a mock archive for testing
    fn create_mock_archive() -> Vec<u8> {
        // This would create a minimal valid MPQ archive
        // For a real test, you'd need a pre-made MPQ file
        vec![0; 1024]
    }

    #[test]
    fn test_archive_from_reader() {
        // Skip this test until we have a proper mock archive
        // let data = create_mock_archive();
        // let reader = Box::new(Cursor::new(data));
        // let archive = MpqArchive::from_reader(reader, None);
        // assert!(archive.is_ok());
    }
}
