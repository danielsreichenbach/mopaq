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
    builder::ArchiveBuilder,
    compression,
    crypto::{decrypt_block, decrypt_dword, hash_string, hash_type},
    header::{self, MpqHeader, UserDataHeader},
    special_files,
    tables::{BetTable, BlockTable, HashTable, HetTable, HiBlockTable},
    Error, Result,
};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Helper trait for reading little-endian integers
trait ReadLittleEndian: Read {
    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

impl<R: Read> ReadLittleEndian for R {}

/// Detailed information about an MPQ archive
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    /// Path to the archive file
    pub path: PathBuf,
    /// Total file size in bytes
    pub file_size: u64,
    /// Archive offset (if MPQ data starts after user data)
    pub archive_offset: u64,
    /// MPQ format version
    pub format_version: header::FormatVersion,
    /// Number of files in the archive
    pub file_count: usize,
    /// Maximum file capacity (hash table size)
    pub max_file_count: u32,
    /// Sector size in bytes
    pub sector_size: usize,
    /// Archive is encrypted
    pub is_encrypted: bool,
    /// Archive has digital signature
    pub has_signature: bool,
    /// Signature status (if applicable)
    pub signature_status: SignatureStatus,
    /// Hash table information
    pub hash_table_info: TableInfo,
    /// Block table information
    pub block_table_info: TableInfo,
    /// HET table information (v3+)
    pub het_table_info: Option<TableInfo>,
    /// BET table information (v3+)
    pub bet_table_info: Option<TableInfo>,
    /// Hi-block table information (v2+)
    pub hi_block_table_info: Option<TableInfo>,
    /// Has (attributes) file
    pub has_attributes: bool,
    /// Has (listfile) file
    pub has_listfile: bool,
    /// User data information
    pub user_data_info: Option<UserDataInfo>,
    /// MD5 checksums status (v4)
    pub md5_status: Option<Md5Status>,
}

/// Information about a table in the archive
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Table size in entries (None if table failed to load)
    pub size: Option<u32>,
    /// Table offset in archive
    pub offset: u64,
    /// Compressed size (if applicable)
    pub compressed_size: Option<u64>,
    /// Whether the table failed to load
    pub failed_to_load: bool,
}

/// User data information
#[derive(Debug, Clone)]
pub struct UserDataInfo {
    /// User data header size
    pub header_size: u32,
    /// User data size
    pub data_size: u32,
}

/// Digital signature status
#[derive(Debug, Clone, PartialEq)]
pub enum SignatureStatus {
    /// No signature present
    None,
    /// Weak signature present and valid
    WeakValid,
    /// Weak signature present but invalid
    WeakInvalid,
    /// Strong signature present and valid
    StrongValid,
    /// Strong signature present but invalid
    StrongInvalid,
    /// Strong signature present but no public key available
    StrongNoKey,
}

/// MD5 checksum verification status for v4 archives
#[derive(Debug, Clone)]
pub struct Md5Status {
    /// Hash table MD5 valid
    pub hash_table_valid: bool,
    /// Block table MD5 valid
    pub block_table_valid: bool,
    /// Hi-block table MD5 valid
    pub hi_block_table_valid: bool,
    /// HET table MD5 valid
    pub het_table_valid: bool,
    /// BET table MD5 valid
    pub bet_table_valid: bool,
    /// MPQ header MD5 valid
    pub header_valid: bool,
}

/// Options for opening MPQ archives
#[derive(Debug, Clone)]
pub struct OpenOptions {
    /// Load tables immediately when opening
    pub load_tables: bool,
    /// Version for new archives
    version: Option<crate::header::FormatVersion>,
}

impl OpenOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self {
            load_tables: true,
            version: None,
        }
    }

    /// Set whether to load tables immediately
    pub fn load_tables(mut self, load: bool) -> Self {
        self.load_tables = load;
        self
    }

    /// Set the MPQ version for new archives
    pub fn version(mut self, version: crate::header::FormatVersion) -> Self {
        self.version = Some(version);
        self
    }

    /// Open an existing MPQ archive
    pub fn open<P: AsRef<Path>>(self, path: P) -> Result<Archive> {
        Archive::open_with_options(path, self)
    }

    /// Create a new MPQ archive
    pub fn create<P: AsRef<Path>>(self, path: P) -> Result<Archive> {
        let path = path.as_ref();

        // Create an empty archive with the specified version
        let builder =
            ArchiveBuilder::new().version(self.version.unwrap_or(crate::header::FormatVersion::V1));

        // Build the empty archive
        builder.build(path)?;

        // Open the newly created archive
        Self::new().load_tables(self.load_tables).open(path)
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
    /// HET table for v3+ archives
    het_table: Option<HetTable>,
    /// BET table for v3+ archives
    bet_table: Option<BetTable>,
    /// File attributes from (attributes) file
    attributes: Option<special_files::Attributes>,
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
            bet_table: None,
            het_table: None,
            attributes: None,
        };

        // Load tables if requested
        if options.load_tables {
            archive.load_tables()?;
        }

        Ok(archive)
    }

    /// Load hash and block tables
    pub fn load_tables(&mut self) -> Result<()> {
        // For v3+ archives, check for HET/BET tables first
        if self.header.format_version >= header::FormatVersion::V3 {
            // Try to load HET table
            if let Some(het_pos) = self.header.het_table_pos {
                if het_pos != 0 {
                    let het_size = self
                        .header
                        .v4_data
                        .as_ref()
                        .map(|v4| v4.het_table_size_64)
                        .unwrap_or(0);

                    if het_size > 0 {
                        log::debug!("Loading HET table from offset 0x{:X}", het_pos);

                        // HET table key is based on table name
                        let key = hash_string("(hash table)", hash_type::FILE_KEY);

                        match HetTable::read(
                            &mut self.reader,
                            self.archive_offset + het_pos,
                            het_size,
                            key,
                        ) {
                            Ok(het) => {
                                let file_count = het.header.max_file_count;
                                log::info!("Loaded HET table with {} max files", file_count);
                                self.het_table = Some(het);
                            }
                            Err(e) => {
                                log::warn!("Failed to load HET table: {}", e);
                            }
                        }
                    }
                }
            }

            // Try to load BET table
            if let Some(bet_pos) = self.header.bet_table_pos {
                if bet_pos != 0 {
                    let bet_size = self
                        .header
                        .v4_data
                        .as_ref()
                        .map(|v4| v4.bet_table_size_64)
                        .unwrap_or(0);

                    if bet_size > 0 {
                        log::debug!("Loading BET table from offset 0x{:X}", bet_pos);

                        // BET table key is based on table name
                        let key = hash_string("(block table)", hash_type::FILE_KEY);

                        match BetTable::read(
                            &mut self.reader,
                            self.archive_offset + bet_pos,
                            bet_size,
                            key,
                        ) {
                            Ok(bet) => {
                                let file_count = bet.header.file_count;
                                log::info!("Loaded BET table with {} files", file_count);
                                self.bet_table = Some(bet);
                            }
                            Err(e) => {
                                log::warn!("Failed to load BET table: {}", e);
                            }
                        }
                    }
                }
            }
        }

        // Check if we have valid HET/BET tables with actual entries
        let has_valid_het_bet = match (&self.het_table, &self.bet_table) {
            (Some(het), Some(bet)) => {
                // Tables are valid if they have entries
                het.header.max_file_count > 0 && bet.header.file_count > 0
            }
            _ => false,
        };

        // Only load hash/block tables if:
        // 1. We don't have valid HET/BET tables, OR
        // 2. The hash table size is non-zero (indicating they exist and may be needed for compatibility)
        if !has_valid_het_bet || self.header.hash_table_size > 0 {
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
        } else {
            log::info!("Skipping hash/block table loading - valid HET/BET tables present");
        }

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

        // Load attributes if present
        self.load_attributes()?;

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

    /// Get detailed information about the archive
    pub fn get_info(&mut self) -> Result<ArchiveInfo> {
        // Ensure tables are loaded
        if self.hash_table.is_none() && self.het_table.is_none() {
            self.load_tables()?;
        }

        // Get file size
        let file_size = self.reader.get_ref().metadata()?.len();

        // Count files
        let file_count = if let Some(bet) = &self.bet_table {
            bet.header.file_count as usize
        } else if let Some(block_table) = &self.block_table {
            // Count non-empty entries in block table
            block_table
                .entries()
                .iter()
                .filter(|entry| entry.file_size != 0)
                .count()
        } else {
            0
        };

        // Get max file count
        let max_file_count = if let Some(het) = &self.het_table {
            het.header.max_file_count
        } else {
            self.header.hash_table_size
        };

        // Check for special files
        let has_listfile = self.find_file("(listfile)")?.is_some();
        let has_signature = self.find_file("(signature)")?.is_some();
        let has_attributes = self.attributes.is_some() || self.find_file("(attributes)")?.is_some();

        // Determine encryption status
        let is_encrypted = if let Some(block_table) = &self.block_table {
            use crate::tables::BlockEntry;
            block_table
                .entries()
                .iter()
                .any(|entry| (entry.flags & BlockEntry::FLAG_ENCRYPTED) != 0)
        } else {
            false
        };

        // TODO: Implement signature verification
        let signature_status = if has_signature {
            SignatureStatus::StrongNoKey // Placeholder
        } else {
            SignatureStatus::None
        };

        // Build table info
        let hash_table_info = TableInfo {
            size: Some(self.header.hash_table_size),
            offset: self.header.get_hash_table_pos(),
            compressed_size: self.header.v4_data.as_ref().map(|v4| v4.hash_table_size_64),
            failed_to_load: self.hash_table.is_none() && self.header.hash_table_size > 0,
        };

        let block_table_info = TableInfo {
            size: Some(self.header.block_table_size),
            offset: self.header.get_block_table_pos(),
            compressed_size: self
                .header
                .v4_data
                .as_ref()
                .map(|v4| v4.block_table_size_64),
            failed_to_load: self.block_table.is_none() && self.header.block_table_size > 0,
        };

        let het_table_info = self.header.het_table_pos.and_then(|pos| {
            if pos == 0 {
                return None;
            }

            let compressed_size = self.header.v4_data.as_ref().map(|v4| v4.het_table_size_64);

            Some(TableInfo {
                size: self.het_table.as_ref().map(|het| het.header.max_file_count),
                offset: pos,
                compressed_size,
                failed_to_load: self.het_table.is_none() && compressed_size.unwrap_or(0) > 0,
            })
        });

        let bet_table_info = self.header.bet_table_pos.and_then(|pos| {
            if pos == 0 {
                return None;
            }

            let compressed_size = self.header.v4_data.as_ref().map(|v4| v4.bet_table_size_64);

            Some(TableInfo {
                size: self.bet_table.as_ref().map(|bet| bet.header.file_count),
                offset: pos,
                compressed_size,
                failed_to_load: self.bet_table.is_none() && compressed_size.unwrap_or(0) > 0,
            })
        });

        let hi_block_table_info = self.header.hi_block_table_pos.and_then(|pos| {
            if pos == 0 {
                return None;
            }

            Some(TableInfo {
                size: if self.hi_block_table.is_some() {
                    Some(self.header.block_table_size)
                } else {
                    None
                },
                offset: pos,
                compressed_size: self
                    .header
                    .v4_data
                    .as_ref()
                    .map(|v4| v4.hi_block_table_size_64),
                failed_to_load: self.hi_block_table.is_none(),
            })
        });

        let user_data_info = self.user_data.as_ref().map(|ud| UserDataInfo {
            header_size: ud.user_data_header_size,
            data_size: ud.user_data_size,
        });

        // TODO: Implement MD5 verification for v4 archives
        let md5_status = if self.header.v4_data.is_some() {
            Some(Md5Status {
                hash_table_valid: true, // Placeholder
                block_table_valid: true,
                hi_block_table_valid: true,
                het_table_valid: true,
                bet_table_valid: true,
                header_valid: true,
            })
        } else {
            None
        };

        Ok(ArchiveInfo {
            path: self.path.clone(),
            file_size,
            archive_offset: self.archive_offset,
            format_version: self.header.format_version,
            file_count,
            max_file_count,
            sector_size: self.header.sector_size(),
            is_encrypted,
            has_signature,
            signature_status,
            hash_table_info,
            block_table_info,
            het_table_info,
            bet_table_info,
            hi_block_table_info,
            has_attributes,
            has_listfile,
            user_data_info,
            md5_status,
        })
    }

    /// Get the hash table
    pub fn hash_table(&self) -> Option<&HashTable> {
        self.hash_table.as_ref()
    }

    /// Get the block table
    pub fn block_table(&self) -> Option<&BlockTable> {
        self.block_table.as_ref()
    }

    /// Get HET table reference
    pub fn het_table(&self) -> Option<&HetTable> {
        self.het_table.as_ref()
    }

    /// Get BET table reference
    pub fn bet_table(&self) -> Option<&BetTable> {
        self.bet_table.as_ref()
    }

    /// Find a file in the archive
    pub fn find_file(&self, filename: &str) -> Result<Option<FileInfo>> {
        // For v3+ archives, prioritize HET/BET tables if they exist and are valid
        if let (Some(het), Some(bet)) = (&self.het_table, &self.bet_table) {
            // Check if tables have actual entries
            if het.header.max_file_count > 0 && bet.header.file_count > 0 {
                if let Some(file_index) = het.find_file(filename) {
                    if let Some(bet_info) = bet.get_file_info(file_index) {
                        return Ok(Some(FileInfo {
                            filename: filename.to_string(),
                            hash_index: 0, // Not applicable for HET/BET
                            block_index: file_index as usize,
                            file_pos: self.archive_offset + bet_info.file_pos,
                            compressed_size: bet_info.compressed_size,
                            file_size: bet_info.file_size,
                            flags: bet_info.flags,
                            locale: 0, // HET/BET don't store locale separately
                        }));
                    }
                }

                // If HET/BET tables are valid but file not found, only fall back if hash tables exist
                // Some v3+ archives may have both table types for compatibility
                if self.hash_table.is_none() || self.block_table.is_none() {
                    return Ok(None);
                }
            }
        }

        // Fall back to traditional hash/block tables if:
        // 1. HET/BET tables don't exist
        // 2. HET/BET tables are empty/invalid
        // 3. File wasn't found in HET/BET but hash/block tables exist
        self.find_file_classic(filename)
    }

    /// Classic file lookup using hash/block tables
    fn find_file_classic(&self, filename: &str) -> Result<Option<FileInfo>> {
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
        if let Some(_listfile_info) = self.find_file("(listfile)")? {
            // Read the listfile
            let listfile_data = self.read_file("(listfile)")?;

            // Parse the listfile
            let filenames = special_files::parse_listfile(&listfile_data)?;

            let mut entries = Vec::new();

            // Look up each file
            for filename in filenames {
                if let Some(file_info) = self.find_file(&filename)? {
                    entries.push(FileEntry {
                        name: filename,
                        size: file_info.file_size,
                        compressed_size: file_info.compressed_size,
                        flags: file_info.flags,
                    });
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

            let mut entries = Vec::new();

            // For v3+ archives, prioritize HET/BET tables if they exist and are valid
            if let (Some(het), Some(bet)) = (&self.het_table, &self.bet_table) {
                if het.header.max_file_count > 0 && bet.header.file_count > 0 {
                    log::info!("Enumerating files using HET/BET tables");

                    // Enumerate using BET table
                    for i in 0..bet.header.file_count {
                        if let Some(bet_info) = bet.get_file_info(i) {
                            // Only include files that actually exist
                            if bet_info.flags & crate::tables::BlockEntry::FLAG_EXISTS != 0 {
                                entries.push(FileEntry {
                                    name: format!("file_{:08}.dat", i), // Unknown name with file index
                                    size: bet_info.file_size,
                                    compressed_size: bet_info.compressed_size,
                                    flags: bet_info.flags,
                                });
                            }
                        }
                    }

                    // If we enumerated from HET/BET successfully, return early
                    if !entries.is_empty() {
                        return Ok(entries);
                    }
                }
            }

            // Fall back to classic hash/block tables
            let hash_table = self
                .hash_table
                .as_ref()
                .ok_or_else(|| Error::invalid_format("No tables loaded for enumeration"))?;
            let block_table = self
                .block_table
                .as_ref()
                .ok_or_else(|| Error::invalid_format("No block table loaded"))?;

            log::info!("Enumerating files using hash/block tables");

            // Scan hash table for valid entries
            for (i, hash_entry) in hash_table.entries().iter().enumerate() {
                if hash_entry.is_valid() {
                    if let Some(block_entry) = block_table.get(hash_entry.block_index as usize) {
                        if block_entry.exists() {
                            entries.push(FileEntry {
                                name: format!("file_{:08}.dat", i), // Unknown name with hash index
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

        // For v3+ archives with HET/BET tables, we already have all the info we need in FileInfo
        // For classic archives, we need to get additional info from the block table
        let (file_size_for_key, actual_file_size) =
            if self.het_table.is_some() && self.bet_table.is_some() {
                // Using HET/BET tables - FileInfo already has all the data
                (file_info.file_size as u32, file_info.file_size)
            } else {
                // Using classic tables - need block entry for accurate sizes
                let block_table = self
                    .block_table
                    .as_ref()
                    .ok_or_else(|| Error::invalid_format("Block table not loaded"))?;
                let block_entry = block_table
                    .get(file_info.block_index)
                    .ok_or_else(|| Error::block_table("Invalid block index"))?;
                (block_entry.file_size, block_entry.file_size as u64)
            };

        // Calculate encryption key if needed
        let key = if file_info.is_encrypted() {
            let base_key = hash_string(name, hash_type::FILE_KEY);
            if file_info.has_fix_key() {
                // Apply FIX_KEY modification
                let file_pos = (file_info.file_pos - self.archive_offset) as u32;
                (base_key.wrapping_add(file_pos)) ^ file_size_for_key
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
                log::debug!(
                    "Decrypting file data: key=0x{:08X}, size={}",
                    key,
                    data.len()
                );
                if data.len() <= 64 {
                    log::debug!("Before decrypt: {:02X?}", &data);
                }
                decrypt_file_data(&mut data, key);
                if data.len() <= 64 {
                    log::debug!("After decrypt: {:02X?}", &data);
                }
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
                    compression::decompress(
                        compressed_data,
                        compression_type,
                        actual_file_size as usize,
                    )?
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
                // Encrypted files always have compression type byte
                // For unencrypted WoW MPQ files, try direct zlib first
                if file_info.is_encrypted() {
                    // Encrypted files always have compression type byte
                    if !data.is_empty() {
                        let compression_type = data[0];
                        let compressed_data = &data[1..];
                        log::debug!(
                            "Decompressing encrypted file: type=0x{:02X}, compressed_size={}, expected_size={}",
                            compression_type,
                            compressed_data.len(),
                            actual_file_size
                        );
                        compression::decompress(
                            compressed_data,
                            compression_type,
                            actual_file_size as usize,
                        )
                    } else {
                        Err(Error::compression("Empty compressed data"))
                    }
                } else {
                    // For unencrypted files, try direct zlib first (common in WoW MPQs)
                    match compression::decompress(
                        &data,
                        compression::flags::ZLIB,
                        actual_file_size as usize,
                    ) {
                        Ok(decompressed) => Ok(decompressed),
                        Err(e) => {
                            // If direct zlib fails, try with compression type byte
                            log::debug!(
                                "Direct zlib decompression failed: {}, trying with type byte",
                                e
                            );

                            if !data.is_empty() {
                                let compression_type = data[0];
                                let compressed_data = &data[1..];
                                compression::decompress(
                                    compressed_data,
                                    compression_type,
                                    actual_file_size as usize,
                                )
                            } else {
                                Err(Error::compression("Empty compressed data"))
                            }
                        }
                    }
                }
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
        let sector_count = (file_info.file_size as usize).div_ceil(sector_size);

        log::debug!(
            "Reading sectored file: {} sectors of {} bytes each",
            sector_count,
            sector_size
        );

        // Read sector offset table
        self.reader.seek(SeekFrom::Start(file_info.file_pos))?;
        let offset_table_size = (sector_count + 1) * 4;
        let mut offset_data = vec![0u8; offset_table_size];
        self.reader.read_exact(&mut offset_data)?;

        // Decrypt sector offset table if needed
        if file_info.is_encrypted() {
            let offset_key = key.wrapping_sub(1);
            decrypt_file_data(&mut offset_data, offset_key);
        }

        // Parse sector offsets
        let mut sector_offsets = Vec::with_capacity(sector_count + 1);
        let mut cursor = std::io::Cursor::new(&offset_data);
        for _ in 0..=sector_count {
            sector_offsets.push(cursor.read_u32_le()?);
        }

        log::debug!(
            "Sector offsets: first={}, last={}",
            sector_offsets.first().copied().unwrap_or(0),
            sector_offsets.last().copied().unwrap_or(0)
        );

        // Check if we have sector CRCs
        let mut sector_crcs = None;
        if file_info.has_sector_crc() {
            // The first sector offset tells us where the data starts
            // If it's large enough to accommodate a CRC table, then CRCs are present
            let first_data_offset = sector_offsets[0] as usize;
            let expected_crc_table_start = offset_table_size;
            let expected_crc_table_size = sector_count * 4;

            if first_data_offset >= expected_crc_table_start + expected_crc_table_size {
                // CRC table follows the offset table
                let mut crc_data = vec![0u8; expected_crc_table_size];
                self.reader.read_exact(&mut crc_data)?;

                // CRC table is not encrypted
                let mut crcs = Vec::with_capacity(sector_count);
                let mut cursor = std::io::Cursor::new(&crc_data);
                for _ in 0..sector_count {
                    crcs.push(cursor.read_u32_le()?);
                }

                // Log before moving
                log::debug!(
                    "Read {} sector CRCs, first few: {:?}",
                    sector_count,
                    &crcs[..5.min(crcs.len())]
                );

                sector_crcs = Some(crcs);
            } else {
                log::debug!(
                    "File has SECTOR_CRC flag but insufficient space for CRC table (offset_table_size={}, first_data_offset={}, needed={}). This is common in some MPQ implementations.",
                    offset_table_size,
                    first_data_offset,
                    expected_crc_table_start + expected_crc_table_size
                );
            }
        }

        // Read and decompress each sector
        let mut decompressed_data = Vec::with_capacity(file_info.file_size as usize);

        for i in 0..sector_count {
            let sector_start = sector_offsets[i] as u64;
            let sector_end = sector_offsets[i + 1] as u64;

            if sector_end < sector_start {
                return Err(Error::invalid_format(format!(
                    "Invalid sector offsets: start={}, end={} for sector {}",
                    sector_start, sector_end, i
                )));
            }

            let sector_size_compressed = (sector_end - sector_start) as usize;

            // Calculate expected decompressed size for this sector
            let remaining = file_info.file_size as usize - decompressed_data.len();
            let expected_size = remaining.min(sector_size);

            // Seek to sector data - offsets are absolute from file position
            self.reader
                .seek(SeekFrom::Start(file_info.file_pos + sector_start))?;

            // Read sector data
            let mut sector_data = vec![0u8; sector_size_compressed];
            self.reader.read_exact(&mut sector_data)?;

            if i == 0 {
                log::debug!(
                    "First sector: offset={}, size={}, first 16 bytes: {:02X?}",
                    sector_start,
                    sector_size_compressed,
                    &sector_data[..16.min(sector_data.len())]
                );
            }

            // Decrypt sector if needed
            if file_info.is_encrypted() {
                let sector_key = key.wrapping_add(i as u32);
                decrypt_file_data(&mut sector_data, sector_key);
            }

            // Decompress sector
            let decompressed_sector = if file_info.is_compressed()
                && sector_size_compressed < expected_size
            {
                // Encrypted sectors always have compression type byte
                if file_info.is_encrypted() {
                    if !sector_data.is_empty() {
                        let compression_type = sector_data[0];
                        let compressed_data = &sector_data[1..];
                        compression::decompress(compressed_data, compression_type, expected_size)?
                    } else {
                        return Err(Error::compression("Empty compressed sector data"));
                    }
                } else {
                    // For unencrypted sectors, check for compression type byte
                    if !sector_data.is_empty() && sector_data[0] == compression::flags::ZLIB {
                        let compressed = &sector_data[1..];
                        compression::decompress(
                            compressed,
                            compression::flags::ZLIB,
                            expected_size,
                        )?
                    } else {
                        // Try raw zlib
                        compression::decompress(
                            &sector_data,
                            compression::flags::ZLIB,
                            expected_size,
                        )?
                    }
                }
            } else {
                // Sector is not compressed
                sector_data[..expected_size.min(sector_data.len())].to_vec()
            };

            // Validate CRC if present
            if let Some(ref crcs) = sector_crcs {
                let expected_crc = crcs[i];
                let actual_crc = crc32fast::hash(&decompressed_sector);

                if actual_crc != expected_crc {
                    log::error!(
                        "CRC mismatch for sector {}: expected {:08x}, got {:08x}",
                        i,
                        expected_crc,
                        actual_crc
                    );
                    // For now, just log the error and continue
                    // Some MPQ files have incorrect CRCs
                }
            }

            decompressed_data.extend_from_slice(&decompressed_sector);
        }

        Ok(decompressed_data)
    }

    /// Load attributes from the (attributes) file if present
    pub fn load_attributes(&mut self) -> Result<()> {
        // Check if attributes are already loaded
        if self.attributes.is_some() {
            return Ok(());
        }

        // Try to read the (attributes) file
        match self.read_file("(attributes)") {
            Ok(data) => {
                // Get block count for parsing
                let block_count = if let Some(ref block_table) = self.block_table {
                    block_table.entries().len()
                } else if let Some(ref bet_table) = self.bet_table {
                    bet_table.header.file_count as usize
                } else {
                    return Err(Error::invalid_format(
                        "No block/BET table available for attributes",
                    ));
                };

                // Parse attributes
                let attributes = special_files::Attributes::parse(&data.into(), block_count)?;
                self.attributes = Some(attributes);

                log::info!("Loaded (attributes) file with {} entries", block_count);
                Ok(())
            }
            Err(Error::FileNotFound(_)) => {
                log::debug!("No (attributes) file found in archive");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Get attributes for a specific file by block index
    pub fn get_file_attributes(
        &self,
        block_index: usize,
    ) -> Option<&special_files::FileAttributes> {
        self.attributes.as_ref()?.get_file_attributes(block_index)
    }

    /// Get all loaded attributes
    pub fn attributes(&self) -> Option<&special_files::Attributes> {
        self.attributes.as_ref()
    }

    /// Add a file to the archive
    pub fn add_file(&mut self, _name: &str, _data: &[u8]) -> Result<()> {
        Err(Error::invalid_format(
            "In-place file addition not yet implemented. Use ArchiveBuilder to create new archives."
        ))
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
        last_bytes[..remainder].copy_from_slice(&data[offset..(remainder + offset)]);
        let last_dword = u32::from_le_bytes(last_bytes);

        // Decrypt with adjusted key
        let decrypted = decrypt_dword(last_dword, key.wrapping_add(chunks as u32));

        // Write back only the remainder bytes
        let decrypted_bytes = decrypted.to_le_bytes();
        data[offset..(remainder + offset)].copy_from_slice(&decrypted_bytes[..remainder]);
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

impl FileEntry {
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

    /// Check if the file uses fixed key encryption
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

    /// Check if the file exists
    pub fn exists(&self) -> bool {
        use crate::tables::BlockEntry;
        (self.flags & BlockEntry::FLAG_EXISTS) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encrypt_block;

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

        // For testing, we need an encrypt function
        fn encrypt_test_data(data: &mut [u8], key: u32) {
            if data.is_empty() || key == 0 {
                return;
            }

            // Convert to u32 for encryption
            let chunks = data.len() / 4;
            if chunks > 0 {
                let mut u32_data = Vec::with_capacity(chunks);
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

                encrypt_block(&mut u32_data, key);

                for (i, &value) in u32_data.iter().enumerate() {
                    let offset = i * 4;
                    let bytes = value.to_le_bytes();
                    data[offset] = bytes[0];
                    data[offset + 1] = bytes[1];
                    data[offset + 2] = bytes[2];
                    data[offset + 3] = bytes[3];
                }
            }
        }

        // Encrypt
        encrypt_test_data(&mut data, 0xDEADBEEF);
        assert_ne!(data, original, "Data should be changed after encryption");

        // Decrypt
        decrypt_file_data(&mut data, 0xDEADBEEF);
        assert_eq!(data, original, "Data should be restored after decryption");
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
