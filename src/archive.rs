use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::config::MpqConfig;
use crate::error::{MpqError, Result};
use crate::header::{MPQ_HEADER_MAGIC, MPQ_USER_DATA_MAGIC, MpqHeader, MpqUserDataHeader};

/// Structure representing an MPQ archive
pub struct MpqArchive {
    /// The file handle for the MPQ archive
    file: File,

    /// The MPQ header
    pub header: MpqHeader,

    /// The MPQ user data header, if present
    pub user_header: Option<MpqUserDataHeader>,

    /// The offset of the MPQ header in the file
    pub header_offset: u64,

    /// Configuration for the archive
    pub config: MpqConfig,
}

impl MpqArchive {
    /// Open an existing MPQ archive for reading with default configuration
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = MpqConfig::load()?;
        Self::open_with_config(path, config)
    }

    /// Open an existing MPQ archive for reading with custom configuration
    pub fn open_with_config<P: AsRef<Path>>(path: P, config: MpqConfig) -> Result<Self> {
        let file = File::open(path)?;
        Self::open_from_file_with_config(file, config)
    }

    /// Open an MPQ archive from an already opened file with default configuration
    pub fn open_from_file(file: File) -> Result<Self> {
        let config = MpqConfig::load()?;
        Self::open_from_file_with_config(file, config)
    }

    /// Open an MPQ archive from an already opened file with custom configuration
    pub fn open_from_file_with_config(mut file: File, config: MpqConfig) -> Result<Self> {
        // Check for user header first
        let mut header_offset = 0;
        let mut user_header = None;

        // Read the first 4 bytes to check for a signature
        let mut signature = [0u8; 4];
        file.read_exact(&mut signature)?;
        file.seek(SeekFrom::Start(0))?;

        let magic = u32::from_le_bytes(signature);

        if magic == MPQ_USER_DATA_MAGIC {
            // We found a user header
            let user_data_header = MpqUserDataHeader::read(&mut file)?;
            header_offset = user_data_header.header_offset as u64;
            user_header = Some(user_data_header);

            // Skip user data and position at the main header
            file.seek(SeekFrom::Start(header_offset))?;
        } else if magic != MPQ_HEADER_MAGIC {
            // Try some common offsets
            // MPQ archives often start at offset 0x200 or other common offsets
            for &offset in &[0x200, 0x400, 0x800, 0x1000] {
                file.seek(SeekFrom::Start(offset))?;
                let mut sig = [0u8; 4];
                if file.read_exact(&mut sig).is_ok() && u32::from_le_bytes(sig) == MPQ_HEADER_MAGIC
                {
                    header_offset = offset;
                    file.seek(SeekFrom::Start(header_offset))?;
                    break;
                }
            }
        }

        // At this point, we should be positioned at the MPQ header
        let header = match MpqHeader::read(&mut file) {
            Ok(header) => header,
            Err(e) => return Err(MpqError::InvalidHeader(e.to_string())),
        };

        Ok(MpqArchive {
            file,
            header,
            user_header,
            header_offset,
            config,
        })
    }

    /// Create a new MPQ archive with default configuration
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = MpqConfig::load()?;
        Self::create_with_config(path, config)
    }

    /// Create a new MPQ archive with custom configuration
    pub fn create_with_config<P: AsRef<Path>>(path: P, config: MpqConfig) -> Result<Self> {
        let file = File::create(path)?;
        Self::create_from_file_with_config(file, config)
    }

    /// Create an MPQ archive from an already created file with default configuration
    pub fn create_from_file(file: File) -> Result<Self> {
        let config = MpqConfig::load()?;
        Self::create_from_file_with_config(file, config)
    }

    /// Create an MPQ archive from an already created file with custom configuration
    pub fn create_from_file_with_config(mut file: File, config: MpqConfig) -> Result<Self> {
        // Create a new header with default values from the config
        let mut header = MpqHeader::new(config.default_format_version);
        header.sector_size_shift = config.default_sector_size_shift;

        // Write the header
        file.seek(SeekFrom::Start(0))?;
        header.write(&mut file)?;

        Ok(MpqArchive {
            file,
            header,
            user_header: None,
            header_offset: 0,
            config,
        })
    }

    /// Create a new MPQ archive with user data and default configuration
    pub fn create_with_user_data<P: AsRef<Path>>(path: P, user_data: &[u8]) -> Result<Self> {
        let config = MpqConfig::load()?;
        Self::create_with_user_data_and_config(path, user_data, config)
    }

    /// Create a new MPQ archive with user data and custom configuration
    pub fn create_with_user_data_and_config<P: AsRef<Path>>(
        path: P,
        user_data: &[u8],
        config: MpqConfig,
    ) -> Result<Self> {
        let mut file = File::create(path)?;

        // Calculate the header offset (user header size + user data size, aligned to 512 bytes)
        let user_data_size = user_data.len() as u32;
        let header_offset = ((16 + user_data_size + 511) / 512) * 512;

        // Create the user header
        let user_header = MpqUserDataHeader::new(user_data_size, header_offset);

        // Write the user header
        user_header.write(&mut file)?;

        // Write the user data
        file.write_all(user_data)?;

        // Pad to align to 512 bytes
        let padding_size = header_offset as usize - 16 - user_data.len();
        let padding = vec![0u8; padding_size];
        file.write_all(&padding)?;

        // Create and write the main header
        let mut header = MpqHeader::new(config.default_format_version);
        header.sector_size_shift = config.default_sector_size_shift;
        header.write(&mut file)?;

        Ok(MpqArchive {
            file,
            header,
            user_header: Some(user_header),
            header_offset: header_offset as u64,
            config,
        })
    }

    /// Get the sector size in bytes
    pub fn sector_size(&self) -> u32 {
        1 << self.header.sector_size_shift
    }

    /// Flush any changes to disk
    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        Ok(())
    }

    /// Update the header in the file
    pub fn update_header(&mut self) -> Result<()> {
        self.file.seek(SeekFrom::Start(self.header_offset))?;
        self.header.write(&mut self.file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Write};
    use tempfile::tempdir;

    #[test]
    fn test_create_and_open_archive() {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        let archive_path = dir.path().join("test.mpq");

        // Create a new archive
        {
            let archive = MpqArchive::create(&archive_path).unwrap();
            assert_eq!(archive.header.format_version, 1);
            assert!(archive.user_header.is_none());
        }

        // Open the archive
        {
            let archive = MpqArchive::open(&archive_path).unwrap();
            assert_eq!(archive.header.format_version, 1);
            assert!(archive.user_header.is_none());
        }
    }

    #[test]
    fn test_create_and_open_archive_with_user_data() {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        let archive_path = dir.path().join("test_user_data.mpq");

        // Create a new archive with user data
        let user_data = b"This is some user data for testing";
        {
            let archive = MpqArchive::create_with_user_data(&archive_path, user_data).unwrap();

            assert_eq!(archive.header.format_version, 1);
            assert!(archive.user_header.is_some());
            assert_eq!(
                archive.user_header.as_ref().unwrap().user_data_size,
                user_data.len() as u32
            );
        }

        // Open the archive and check the user data
        {
            let archive = MpqArchive::open(&archive_path).unwrap();
            assert_eq!(archive.header.format_version, 1);
            assert!(archive.user_header.is_some());

            let user_header = archive.user_header.as_ref().unwrap();
            assert_eq!(user_header.user_data_size, user_data.len() as u32);

            // We could also read the user data here, but we'll leave that for a more complete implementation
        }
    }
}
