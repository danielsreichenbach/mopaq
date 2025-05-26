//! MPQ archive handling
//!
//! This module provides the main Archive type for reading MPQ files.
//! It supports:
//! - All MPQ versions (v1-v4)
//! - File extraction with decompression
//! - Sector CRC validation
//! - Encryption/decryption
//! - Multi-sector and single-unit files

use crate::{
    compression,
    crypto::{decrypt_block, decrypt_dword},
    hash::{hash_string, hash_type},
    header::{self, MpqHeader, UserDataHeader},
    special_files,
    tables::{BlockTable, HashTable, HiBlockTable},
    Error, Result,
};
use byteorder::{LittleEndian, ReadBytesExt};
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
    pub fn list(&mut self) -> Result<Vec<FileEntry>> {
        // Try to find and read (listfile)
        if let Some(listfile_info) = self.find_file("(listfile)")? {
            // Read the listfile
            let listfile_data = self.read_file("(listfile)")?;

            // Parse the listfile
            let filenames = special_files::parse_listfile(&listfile_data)?;

            let mut entries = Vec::new();

            // Look up each file in the hash table
            for filename in filenames {
                if let Some(file_info) = self.find_file(&filename)? {
                    // Get the block entry for size information
                    if let Some(block_table) = &self.block_table {
                        if let Some(block_entry) = block_table.get(file_info.block_index) {
                            entries.push(FileEntry {
                                name: filename,
                                size: block_entry.file_size as u64,
                                compressed_size: block_entry.compressed_size as u64,
                                flags: block_entry.flags,
                            });
                        }
                    }
                } else {
                    // File is in listfile but not found in archive
                    log::warn!(
                        "File '{}' listed in (listfile) but not found in archive",
                        filename
                    );
                }
            }

            Ok(entries)
        } else {
            // No listfile, we'll need to enumerate entries without names
            log::info!("No (listfile) found, enumerating anonymous entries");

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

        // Get the block entry
        let block_table = self
            .block_table
            .as_ref()
            .ok_or_else(|| Error::invalid_format("Block table not loaded"))?;
        let block_entry = block_table
            .get(file_info.block_index)
            .ok_or_else(|| Error::block_table("Invalid block index"))?;

        // Calculate encryption key if needed
        let key = if file_info.is_encrypted() {
            let base_key = hash_string(name, hash_type::FILE_KEY);
            if file_info.has_fix_key() {
                // Apply FIX_KEY modification
                let file_pos = (file_info.file_pos - self.archive_offset) as u32;
                (base_key.wrapping_add(file_pos)) ^ block_entry.file_size
            } else {
                base_key
            }
        } else {
            0
        };

        // Read the file data
        self.reader.seek(SeekFrom::Start(file_info.file_pos))?;

        if file_info.is_single_unit() || !file_info.is_compressed() {
            // Single unit or uncompressed file - read directly
            let mut data = vec![0u8; file_info.compressed_size as usize];
            self.reader.read_exact(&mut data)?;

            // Decrypt if needed
            if file_info.is_encrypted() {
                decrypt_file_data(&mut data, key);
            }

            // Validate CRC if present for single unit files
            if file_info.has_sector_crc() && file_info.is_single_unit() {
                // For single unit files, there's one CRC after the data
                let mut crc_bytes = [0u8; 4];
                self.reader.read_exact(&mut crc_bytes)?;
                let expected_crc = u32::from_le_bytes(crc_bytes);

                // CRC is calculated on the decompressed data
                let data_to_check = if file_info.is_compressed() {
                    // We need to decompress first to check CRC
                    let compression_type = data[0];
                    let compressed_data = &data[1..];
                    let decompressed = compression::decompress(
                        compressed_data,
                        compression_type,
                        file_info.file_size as usize,
                    )?;
                    decompressed
                } else {
                    data.clone()
                };

                let actual_crc = crc32fast::hash(&data_to_check);
                if actual_crc != expected_crc {
                    return Err(Error::ChecksumMismatch {
                        file: name.to_string(),
                        expected: expected_crc,
                        actual: actual_crc,
                    });
                }

                log::debug!("Single unit file CRC validated: 0x{:08X}", actual_crc);
            }

            // Decompress if needed
            if file_info.is_compressed() {
                // Get compression type from first byte
                let compression_type = data[0];
                let compressed_data = &data[1..];

                compression::decompress(
                    compressed_data,
                    compression_type,
                    file_info.file_size as usize,
                )
            } else {
                Ok(data)
            }
        } else {
            // Multi-sector compressed file
            self.read_sectored_file(&file_info, key)
        }
    }

    /// Read a file that is split into sectors
    fn read_sectored_file(&mut self, file_info: &FileInfo, key: u32) -> Result<Vec<u8>> {
        let sector_size = self.header.sector_size();
        let sector_count = ((file_info.file_size as usize + sector_size - 1) / sector_size) as u32;

        // Read sector offset table
        let offset_table_size = (sector_count + 1) * 4;
        let mut offset_data = vec![0u8; offset_table_size as usize];
        self.reader.read_exact(&mut offset_data)?;

        // Decrypt sector offset table if needed
        if file_info.is_encrypted() {
            let offset_key = key.wrapping_sub(1);
            decrypt_file_data(&mut offset_data, offset_key);
        }

        // Parse sector offsets
        let mut sector_offsets = Vec::with_capacity((sector_count + 1) as usize);
        let mut cursor = std::io::Cursor::new(&offset_data);
        for _ in 0..=sector_count {
            sector_offsets.push(cursor.read_u32::<LittleEndian>()?);
        }

        // Read CRC table if present
        let mut sector_crcs = None;
        if file_info.has_sector_crc() {
            let crc_table_size = sector_count * 4;
            let mut crc_data = vec![0u8; crc_table_size as usize];
            self.reader.read_exact(&mut crc_data)?;

            // CRC table is not encrypted
            let mut crcs = Vec::with_capacity(sector_count as usize);
            let mut cursor = std::io::Cursor::new(&crc_data);
            for _ in 0..sector_count {
                crcs.push(cursor.read_u32::<LittleEndian>()?);
            }
            sector_crcs = Some(crcs);

            log::debug!("Read {} sector CRCs for file", sector_count);
        }

        // Read and decompress each sector
        let mut decompressed_data = Vec::with_capacity(file_info.file_size as usize);

        for i in 0..sector_count as usize {
            let sector_start = sector_offsets[i] as usize;
            let sector_end = sector_offsets[i + 1] as usize;
            let sector_size_compressed = sector_end - sector_start;

            // Calculate expected decompressed size for this sector
            let remaining = file_info.file_size as usize - decompressed_data.len();
            let expected_size = remaining.min(sector_size);

            // Read sector data
            let mut sector_data = vec![0u8; sector_size_compressed];
            self.reader.read_exact(&mut sector_data)?;

            // Decrypt sector if needed
            if file_info.is_encrypted() {
                let sector_key = key.wrapping_add(i as u32);
                decrypt_file_data(&mut sector_data, sector_key);
            }

            // Decompress sector
            let decompressed_sector =
                if file_info.is_compressed() && sector_size_compressed < expected_size {
                    // Sector is compressed
                    let compression_type = sector_data[0];
                    let compressed = &sector_data[1..];
                    compression::decompress(compressed, compression_type, expected_size)?
                } else {
                    // Sector is not compressed (or compression didn't help)
                    sector_data[..expected_size].to_vec()
                };

            // Validate CRC if present
            if let Some(ref crcs) = sector_crcs {
                let expected_crc = crcs[i];
                let actual_crc = crc32fast::hash(&decompressed_sector);

                if actual_crc != expected_crc {
                    return Err(Error::ChecksumMismatch {
                        file: format!("sector {}", i),
                        expected: expected_crc,
                        actual: actual_crc,
                    });
                }

                log::trace!("Sector {} CRC validated: 0x{:08X}", i, actual_crc);
            }

            decompressed_data.extend_from_slice(&decompressed_sector);
        }

        Ok(decompressed_data)
    }

    /// Add a file to the archive
    pub fn add_file(&mut self, _name: &str, _data: &[u8]) -> Result<()> {
        todo!("Implement file addition")
    }
}

/// Decrypt file data in-place
fn decrypt_file_data(data: &mut [u8], key: u32) {
    if data.is_empty() || key == 0 {
        return;
    }

    // Process full u32 chunks
    let chunks = data.len() / 4;
    if chunks > 0 {
        // Create a properly aligned u32 slice
        let mut u32_data = Vec::with_capacity(chunks);

        // Copy data as u32 values (little-endian)
        for i in 0..chunks {
            let offset = i * 4;
            let value = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            u32_data.push(value);
        }

        // Decrypt the u32 data
        decrypt_block(&mut u32_data, key);

        // Copy back to byte array
        for (i, &value) in u32_data.iter().enumerate() {
            let offset = i * 4;
            let bytes = value.to_le_bytes();
            data[offset] = bytes[0];
            data[offset + 1] = bytes[1];
            data[offset + 2] = bytes[2];
            data[offset + 3] = bytes[3];
        }
    }

    // Handle remaining bytes if not aligned to 4
    let remainder = data.len() % 4;
    if remainder > 0 {
        let offset = chunks * 4;

        // Read remaining bytes into a u32 (padding with zeros)
        let mut last_bytes = [0u8; 4];
        for i in 0..remainder {
            last_bytes[i] = data[offset + i];
        }
        let last_dword = u32::from_le_bytes(last_bytes);

        // Decrypt with adjusted key
        let decrypted = decrypt_dword(last_dword, key.wrapping_add(chunks as u32));

        // Write back only the remainder bytes
        let decrypted_bytes = decrypted.to_le_bytes();
        for i in 0..remainder {
            data[offset + i] = decrypted_bytes[i];
        }
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

    /// Check if the file is stored as a single unit
    pub fn is_single_unit(&self) -> bool {
        use crate::tables::BlockEntry;
        (self.flags & BlockEntry::FLAG_SINGLE_UNIT) != 0
    }

    /// Check if the file has sector CRCs
    pub fn has_sector_crc(&self) -> bool {
        use crate::tables::BlockEntry;
        (self.flags & BlockEntry::FLAG_SECTOR_CRC) != 0
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

    #[test]
    fn test_decrypt_file_data() {
        let mut data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let original = data.clone();

        // Encrypt
        decrypt_file_data(&mut data, 0xDEADBEEF);
        assert_ne!(data, original);

        // Decrypt (same operation)
        decrypt_file_data(&mut data, 0xDEADBEEF);
        assert_eq!(data, original);
    }

    #[test]
    fn test_crc_calculation() {
        // Test that we're using the correct CRC algorithm (CRC-32)
        let test_data = b"Hello, World!";
        let crc = crc32fast::hash(test_data);

        // This is the expected CRC-32 value for "Hello, World!"
        assert_eq!(crc, 0xEC4AC3D0);
    }
}
