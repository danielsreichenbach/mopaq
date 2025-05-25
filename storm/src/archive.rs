//! MPQ archive handling

use crate::{Error, Result};
use std::path::Path;

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
    pub fn version(mut self, _version: crate::FormatVersion) -> Self {
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
    // TODO: Add fields
}

impl Archive {
    /// Open an existing MPQ archive
    pub fn open<P: AsRef<Path>>(_path: P) -> Result<Self> {
        todo!("Implement archive opening")
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
