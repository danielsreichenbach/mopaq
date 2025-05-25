//! MPQ archive handling

use crate::{
    header::{self, MpqHeader, UserDataHeader},
    Error, Result,
};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};

/// Options for opening MPQ archives
#[derive(Debug, Clone)]
pub struct OpenOptions {
    // TODO: Add fields
}

impl OpenOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self {}
    }

    /// Set the MPQ version for new archives
    pub fn version(mut self, _version: crate::header::FormatVersion) -> Self {
        // TODO: Implement
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
}

impl Archive {
    /// Open an existing MPQ archive
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);

        // Find and read the MPQ header
        let (archive_offset, user_data, header) = header::find_header(&mut reader)?;

        Ok(Archive {
            path,
            reader,
            archive_offset,
            user_data,
            header,
        })
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

    /// List files in the archive
    pub fn list(&self) -> Result<Vec<FileEntry>> {
        todo!("Implement file listing")
    }

    /// Read a file from the archive
    pub fn read_file(&self, _name: &str) -> Result<Vec<u8>> {
        todo!("Implement file reading")
    }

    /// Add a file to the archive
    pub fn add_file(&mut self, _name: &str, _data: &[u8]) -> Result<()> {
        todo!("Implement file addition")
    }
}

/// Information about a file in the archive
#[derive(Debug)]
pub struct FileEntry {
    /// File name
    pub name: String,
    /// Uncompressed size
    pub size: u64,
}
