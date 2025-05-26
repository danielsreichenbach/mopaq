//! MPQ archive handling

use crate::{
    header::{self, MpqHeader, UserDataHeader},
    tables::{BlockTable, HashTable, HiBlockTable},
    Error, Result,
};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Options for opening MPQ archives
#[derive(Debug, Clone)]
pub struct OpenOptions {
    /// Load tables immediately when opening
    pub load_tables: bool,
}

impl OpenOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self { load_tables: true }
    }

    /// Set whether to load tables immediately
    pub fn load_tables(mut self, load: bool) -> Self {
        self.load_tables = load;
        self
    }

    /// Set the MPQ version for new archives
    pub fn version(mut self, _version: crate::header::FormatVersion) -> Self {
        // TODO: Implement when creating archives
        self
    }

    /// Create a new MPQ archive
    pub fn create<P: AsRef<Path>>(self, _path: P) -> Result<Archive> {
        todo!("Implement archive creation")
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// An MPQ archive
#[derive(Debug)]
pub struct Archive {
    /// Path to the archive file
    path: PathBuf,
    /// Archive file reader
    reader: BufReader<File>,
    /// Offset where the MPQ data starts in the file
    archive_offset: u64,
    /// Optional user data header
    user_data: Option<UserDataHeader>,
    /// MPQ header
    header: MpqHeader,
    /// Hash table (optional, loaded on demand)
    hash_table: Option<HashTable>,
    /// Block table (optional, loaded on demand)
    block_table: Option<BlockTable>,
    /// Hi-block table for v2+ archives (optional)
    hi_block_table: Option<HiBlockTable>,
}

impl Archive {
    /// Open an existing MPQ archive
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_options(path, OpenOptions::default())
    }

    /// Open an archive with specific options
    pub fn open_with_options<P: AsRef<Path>>(path: P, options: OpenOptions) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);

        // Find and read the MPQ header
        let (archive_offset, user_data, header) = header::find_header(&mut reader)?;

        let mut archive = Archive {
            path,
            reader,
            archive_offset,
            user_data,
            header,
            hash_table: None,
            block_table: None,
            hi_block_table: None,
        };

        // Load tables if requested
        if options.load_tables {
            archive.load_tables()?;
        }

        Ok(archive)
    }

    /// Load hash and block tables
    pub fn load_tables(&mut self) -> Result<()> {
        // Load hash table
        let hash_table_offset = self.archive_offset + self.header.get_hash_table_pos();
        self.hash_table = Some(HashTable::read(
            &mut self.reader,
            hash_table_offset,
            self.header.hash_table_size,
        )?);

        // Load block table
        let block_table_offset = self.archive_offset + self.header.get_block_table_pos();
        self.block_table = Some(BlockTable::read(
            &mut self.reader,
            block_table_offset,
            self.header.block_table_size,
        )?);

        // Load hi-block table if present (v2+)
        if let Some(hi_block_pos) = self.header.hi_block_table_pos {
            if hi_block_pos != 0 {
                let hi_block_offset = self.archive_offset + hi_block_pos;
                self.hi_block_table = Some(HiBlockTable::read(
                    &mut self.reader,
                    hi_block_offset,
                    self.header.block_table_size,
                )?);
            }
        }

        Ok(())
    }

    /// Get the archive header
    pub fn header(&self) -> &MpqHeader {
        &self.header
    }

    /// Get the user data header if present
    pub fn user_data(&self) -> Option<&UserDataHeader> {
        self.user_data.as_ref()
    }

    /// Get the archive offset in the file
    pub fn archive_offset(&self) -> u64 {
        self.archive_offset
    }

    /// Get the path to the archive
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the hash table
    pub fn hash_table(&self) -> Option<&HashTable> {
        self.hash_table.as_ref()
    }

    /// Get the block table
    pub fn block_table(&self) -> Option<&BlockTable> {
        self.block_table.as_ref()
    }

    /// Find a file in the archive
    pub fn find_file(&self, filename: &str) -> Result<Option<FileInfo>> {
        let hash_table = self
            .hash_table
            .as_ref()
            .ok_or_else(|| Error::invalid_format("Hash table not loaded"))?;
        let block_table = self
            .block_table
            .as_ref()
            .ok_or_else(|| Error::invalid_format("Block table not loaded"))?;

        // Try to find the file with default locale
        if let Some((hash_index, hash_entry)) = hash_table.find_file(filename, 0) {
            let block_entry = block_table
                .get(hash_entry.block_index as usize)
                .ok_or_else(|| Error::block_table("Invalid block index"))?;

            // Calculate full file position for v2+ archives
            let file_pos = if let Some(hi_block) = &self.hi_block_table {
                let high_bits = hi_block.get_file_pos_high(hash_entry.block_index as usize);
                (high_bits << 32) | (block_entry.file_pos as u64)
            } else {
                block_entry.file_pos as u64
            };

            Ok(Some(FileInfo {
                filename: filename.to_string(),
                hash_index,
                block_index: hash_entry.block_index as usize,
                file_pos: self.archive_offset + file_pos,
                compressed_size: block_entry.compressed_size as u64,
                file_size: block_entry.file_size as u64,
                flags: block_entry.flags,
                locale: hash_entry.locale,
            }))
        } else {
            Ok(None)
        }
    }

    /// List files in the archive
    pub fn list(&self) -> Result<Vec<FileEntry>> {
        // For now, we can only list files if we have a (listfile)
        // In the future, we could try to enumerate all valid entries

        // Try to find and read (listfile)
        if let Some(listfile_info) = self.find_file("(listfile)")? {
            // TODO: Read and parse the listfile
            // For now, return empty list
            Ok(vec![])
        } else {
            // No listfile, we'll need to enumerate entries
            // This is less reliable but can still work
            let hash_table = self
                .hash_table
                .as_ref()
                .ok_or_else(|| Error::invalid_format("Hash table not loaded"))?;
            let block_table = self
                .block_table
                .as_ref()
                .ok_or_else(|| Error::invalid_format("Block table not loaded"))?;

            let mut entries = Vec::new();

            // Scan hash table for valid entries
            for (i, hash_entry) in hash_table.entries().iter().enumerate() {
                if hash_entry.is_valid() {
                    if let Some(block_entry) = block_table.get(hash_entry.block_index as usize) {
                        if block_entry.exists() {
                            entries.push(FileEntry {
                                name: format!("file_{:04}.dat", i), // Unknown name
                                size: block_entry.file_size as u64,
                                compressed_size: block_entry.compressed_size as u64,
                                flags: block_entry.flags,
                            });
                        }
                    }
                }
            }

            Ok(entries)
        }
    }

    /// Read a file from the archive
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>> {
        let file_info = self
            .find_file(name)?
            .ok_or_else(|| Error::FileNotFound(name.to_string()))?;

        // Seek to file position
        self.reader.seek(SeekFrom::Start(file_info.file_pos))?;

        // TODO: Handle compression and encryption
        if file_info.is_compressed() || file_info.is_encrypted() {
            return Err(Error::invalid_format(
                "Compression/encryption not yet implemented",
            ));
        }

        // For now, just read uncompressed, unencrypted files
        let mut data = vec![0u8; file_info.file_size as usize];
        self.reader.read_exact(&mut data)?;

        Ok(data)
    }

    /// Add a file to the archive
    pub fn add_file(&mut self, _name: &str, _data: &[u8]) -> Result<()> {
        todo!("Implement file addition")
    }
}

/// Information about a file in the archive
#[derive(Debug)]
pub struct FileInfo {
    /// File name
    pub filename: String,
    /// Index in hash table
    pub hash_index: usize,
    /// Index in block table
    pub block_index: usize,
    /// Absolute file position in archive file
    pub file_pos: u64,
    /// Compressed size
    pub compressed_size: u64,
    /// Uncompressed size
    pub file_size: u64,
    /// File flags
    pub flags: u32,
    /// File locale
    pub locale: u16,
}

impl FileInfo {
    /// Check if the file is compressed
    pub fn is_compressed(&self) -> bool {
        use crate::tables::BlockEntry;
        (self.flags & (BlockEntry::FLAG_IMPLODE | BlockEntry::FLAG_COMPRESS)) != 0
    }

    /// Check if the file is encrypted
    pub fn is_encrypted(&self) -> bool {
        use crate::tables::BlockEntry;
        (self.flags & BlockEntry::FLAG_ENCRYPTED) != 0
    }

    /// Check if the file has fixed key encryption
    pub fn has_fix_key(&self) -> bool {
        use crate::tables::BlockEntry;
        (self.flags & BlockEntry::FLAG_FIX_KEY) != 0
    }
}

/// Information about a file in the archive (for listing)
#[derive(Debug)]
pub struct FileEntry {
    /// File name
    pub name: String,
    /// Uncompressed size
    pub size: u64,
    /// Compressed size
    pub compressed_size: u64,
    /// File flags
    pub flags: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_options() {
        let opts = OpenOptions::new().load_tables(false);

        assert!(!opts.load_tables);
    }

    #[test]
    fn test_file_info_flags() {
        use crate::tables::BlockEntry;

        let info = FileInfo {
            filename: "test.txt".to_string(),
            hash_index: 0,
            block_index: 0,
            file_pos: 0,
            compressed_size: 100,
            file_size: 200,
            flags: BlockEntry::FLAG_COMPRESS | BlockEntry::FLAG_ENCRYPTED,
            locale: 0,
        };

        assert!(info.is_compressed());
        assert!(info.is_encrypted());
        assert!(!info.has_fix_key());
    }
}
