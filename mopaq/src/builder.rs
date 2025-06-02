//! Archive builder for creating MPQ archives

use crate::{
    compression::{compress, flags as compression_flags},
    crypto::{encrypt_block, hash_string, hash_type, jenkins_hash},
    header::FormatVersion,
    tables::{BetHeader, BlockEntry, BlockTable, HashEntry, HashTable, HetHeader, HiBlockTable},
    Error, Result,
};
use std::fs::{self};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Helper trait for writing little-endian integers
trait WriteLittleEndian: Write {
    fn write_u16_le(&mut self, value: u16) -> Result<()> {
        self.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_u32_le(&mut self, value: u32) -> Result<()> {
        self.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_u64_le(&mut self, value: u64) -> Result<()> {
        self.write_all(&value.to_le_bytes())?;
        Ok(())
    }
}

impl<W: Write> WriteLittleEndian for W {}

/// File to be added to the archive
#[derive(Debug)]
struct PendingFile {
    /// Source path or data
    source: FileSource,
    /// Target filename in archive
    archive_name: String,
    /// Compression method to use
    compression: u8,
    /// Whether to encrypt the file
    encrypt: bool,
    /// Whether to use FIX_KEY encryption (adjusts key by block position)
    use_fix_key: bool,
    /// Locale code
    locale: u16,
}

#[derive(Debug)]
enum FileSource {
    Path(PathBuf),
    Data(Vec<u8>),
}

/// Parameters for writing a file to the archive
struct FileWriteParams<'a> {
    /// File data to write
    file_data: &'a [u8],
    /// Archive name for the file
    archive_name: &'a str,
    /// Compression method
    compression: u8,
    /// Whether to encrypt
    encrypt: bool,
    /// Whether to use FIX_KEY encryption
    use_fix_key: bool,
    /// Sector size
    sector_size: usize,
    /// File position in archive (64-bit for large archives)
    file_pos: u64,
}

/// Parameters for writing the MPQ header
struct HeaderWriteParams {
    archive_size: u64,
    hash_table_pos: u64,
    block_table_pos: u64,
    hash_table_size: u32,
    block_table_size: u32,
    hi_block_table_pos: Option<u64>,
    het_table_pos: Option<u64>,
    bet_table_pos: Option<u64>,
    _het_table_size: Option<u64>,
    _bet_table_size: Option<u64>,
}

/// Options for listfile generation
#[derive(Debug, Clone)]
pub enum ListfileOption {
    /// Automatically generate listfile from added files
    Generate,
    /// Use external listfile
    External(PathBuf),
    /// Don't include a listfile
    None,
}

/// Builder for creating new MPQ archives
#[derive(Debug)]
pub struct ArchiveBuilder {
    /// Target MPQ version
    version: FormatVersion,
    /// Block size (sector size = 512 * 2^block_size)
    block_size: u16,
    /// Files to be added
    pending_files: Vec<PendingFile>,
    /// Listfile option
    listfile_option: ListfileOption,
    /// Default compression method
    default_compression: u8,
    /// Whether to generate sector CRCs for files
    generate_crcs: bool,
}

impl ArchiveBuilder {
    /// Create a new archive builder
    pub fn new() -> Self {
        Self {
            version: FormatVersion::V1,
            block_size: 3, // Default 4KB sectors
            pending_files: Vec::new(),
            listfile_option: ListfileOption::Generate,
            default_compression: compression_flags::ZLIB,
            generate_crcs: false,
        }
    }

    /// Set the MPQ format version
    pub fn version(mut self, version: FormatVersion) -> Self {
        self.version = version;
        self
    }

    /// Set the block size (sector size = 512 * 2^block_size)
    pub fn block_size(mut self, block_size: u16) -> Self {
        self.block_size = block_size;
        self
    }

    /// Set the default compression method
    pub fn default_compression(mut self, compression: u8) -> Self {
        self.default_compression = compression;
        self
    }

    /// Set the listfile option
    pub fn listfile_option(mut self, option: ListfileOption) -> Self {
        self.listfile_option = option;
        self
    }

    /// Enable or disable sector CRC generation
    pub fn generate_crcs(mut self, generate: bool) -> Self {
        self.generate_crcs = generate;
        self
    }

    /// Add a file from disk
    pub fn add_file<P: AsRef<Path>>(mut self, path: P, archive_name: &str) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Path(path.as_ref().to_path_buf()),
            archive_name: archive_name.to_string(),
            compression: self.default_compression,
            encrypt: false,
            use_fix_key: false,
            locale: 0, // Neutral locale
        });
        self
    }

    /// Add a file from disk with specific options
    pub fn add_file_with_options<P: AsRef<Path>>(
        mut self,
        path: P,
        archive_name: &str,
        compression: u8,
        encrypt: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Path(path.as_ref().to_path_buf()),
            archive_name: archive_name.to_string(),
            compression,
            encrypt,
            use_fix_key: false,
            locale,
        });
        self
    }

    /// Add a file from memory
    pub fn add_file_data(mut self, data: Vec<u8>, archive_name: &str) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Data(data),
            archive_name: archive_name.to_string(),
            compression: self.default_compression,
            encrypt: false,
            use_fix_key: false,
            locale: 0,
        });
        self
    }

    /// Add a file from memory with specific options
    pub fn add_file_data_with_options(
        mut self,
        data: Vec<u8>,
        archive_name: &str,
        compression: u8,
        encrypt: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Data(data),
            archive_name: archive_name.to_string(),
            compression,
            encrypt,
            use_fix_key: false,
            locale,
        });
        self
    }

    /// Add a file with full encryption options including FIX_KEY support
    pub fn add_file_with_encryption<P: AsRef<Path>>(
        mut self,
        path: P,
        archive_name: &str,
        compression: u8,
        use_fix_key: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Path(path.as_ref().to_path_buf()),
            archive_name: archive_name.to_string(),
            compression,
            encrypt: true,
            use_fix_key,
            locale,
        });
        self
    }

    /// Add file data with full encryption options including FIX_KEY support
    pub fn add_file_data_with_encryption(
        mut self,
        data: Vec<u8>,
        archive_name: &str,
        compression: u8,
        use_fix_key: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Data(data),
            archive_name: archive_name.to_string(),
            compression,
            encrypt: true,
            use_fix_key,
            locale,
        });
        self
    }

    /// Calculate optimal hash table size based on file count
    fn calculate_hash_table_size(&self) -> u32 {
        let file_count = self.pending_files.len()
            + match &self.listfile_option {
                ListfileOption::Generate | ListfileOption::External(_) => 1,
                ListfileOption::None => 0,
            };

        // Use 2x the file count for good performance, minimum 16
        let optimal_size = (file_count * 2).max(16) as u32;

        // Round up to next power of 2
        optimal_size.next_power_of_two()
    }

    /// Build the archive and write to the specified path
    pub fn build<P: AsRef<Path>>(mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Create a temporary file in the same directory
        let temp_file = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;

        // Add listfile if needed
        self.prepare_listfile()?;

        // Write the archive to the temp file
        {
            let mut writer = BufWriter::new(temp_file.as_file());
            self.write_archive(&mut writer)?;
            writer.flush()?;
        }

        // Atomically rename temp file to final destination
        temp_file.persist(path).map_err(|e| Error::Io(e.error))?;

        Ok(())
    }

    /// Prepare the listfile based on the option
    fn prepare_listfile(&mut self) -> Result<()> {
        match &self.listfile_option {
            ListfileOption::Generate => {
                // Generate listfile content from pending files
                let mut content = String::new();
                for file in &self.pending_files {
                    content.push_str(&file.archive_name);
                    content.push('\r');
                    content.push('\n');
                }

                // Add the listfile itself
                content.push_str("(listfile)\r\n");

                self.pending_files.push(PendingFile {
                    source: FileSource::Data(content.into_bytes()),
                    archive_name: "(listfile)".to_string(),
                    compression: self.default_compression,
                    encrypt: false,
                    use_fix_key: false,
                    locale: 0,
                });
            }
            ListfileOption::External(path) => {
                // Read external listfile
                let data = fs::read(path)?;

                self.pending_files.push(PendingFile {
                    source: FileSource::Data(data),
                    archive_name: "(listfile)".to_string(),
                    compression: self.default_compression,
                    encrypt: false,
                    use_fix_key: false,
                    locale: 0,
                });
            }
            ListfileOption::None => {}
        }

        Ok(())
    }

    /// Write the complete archive
    fn write_archive<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // For v3+, we should create HET/BET tables instead of/in addition to hash/block
        let use_het_bet = self.version >= FormatVersion::V3;

        if use_het_bet {
            return self.write_archive_with_het_bet(writer);
        }

        let hash_table_size = self.calculate_hash_table_size();
        let block_table_size = self.pending_files.len() as u32;

        // Calculate sector size
        let sector_size = crate::calculate_sector_size(self.block_size);

        // Reserve space for header (we'll write it at the end)
        let header_size = self.version.header_size();
        writer.seek(SeekFrom::Start(header_size as u64))?;

        // Build tables and write files
        let mut hash_table = HashTable::new(hash_table_size as usize)?;
        let mut block_table = BlockTable::new(block_table_size as usize)?;
        let mut hi_block_table = if self.version >= FormatVersion::V2 {
            Some(HiBlockTable::new(block_table_size as usize))
        } else {
            None
        };

        // Write all files and populate tables
        for (block_index, pending_file) in self.pending_files.iter().enumerate() {
            let file_pos = writer.stream_position()?;

            // Read file data
            let file_data = match &pending_file.source {
                FileSource::Path(path) => fs::read(path)?,
                FileSource::Data(data) => data.clone(),
            };

            // Write file and get sizes
            let params = FileWriteParams {
                file_data: &file_data,
                archive_name: &pending_file.archive_name,
                compression: pending_file.compression,
                encrypt: pending_file.encrypt,
                use_fix_key: pending_file.use_fix_key,
                sector_size,
                file_pos,
            };
            let (compressed_size, flags) = self.write_file(writer, &params)?;

            // Add to hash table
            self.add_to_hash_table(
                &mut hash_table,
                &pending_file.archive_name,
                block_index as u32,
                pending_file.locale,
            )?;

            // Add to block table and hi-block table if needed
            let block_entry = BlockEntry {
                file_pos: file_pos as u32, // Low 32 bits
                compressed_size: compressed_size as u32,
                file_size: file_data.len() as u32,
                flags: flags | BlockEntry::FLAG_EXISTS,
            };

            // Store high 16 bits in hi-block table if needed
            if let Some(ref mut hi_table) = hi_block_table {
                let high_bits = (file_pos >> 32) as u16;
                hi_table.set(block_index, high_bits);
            }

            // Get mutable reference and update
            if let Some(entry) = block_table.get_mut(block_index) {
                *entry = block_entry;
            } else {
                return Err(Error::invalid_format("Block index out of bounds"));
            }
        }

        // Write hash table
        let hash_table_pos = writer.stream_position()?;
        self.write_hash_table(writer, &hash_table)?;

        // Write block table
        let block_table_pos = writer.stream_position()?;
        self.write_block_table(writer, &block_table)?;

        // Write hi-block table if needed
        let hi_block_table_pos = if let Some(ref hi_table) = hi_block_table {
            if hi_table.is_needed() {
                let pos = writer.stream_position()?;
                self.write_hi_block_table(writer, hi_table)?;
                Some(pos)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate archive size
        let archive_size = writer.stream_position()?;

        // Write header at the beginning
        writer.seek(SeekFrom::Start(0))?;
        let header_params = HeaderWriteParams {
            archive_size,
            hash_table_pos,
            block_table_pos,
            hash_table_size,
            block_table_size,
            hi_block_table_pos,
            het_table_pos: None,
            bet_table_pos: None,
            _het_table_size: None,
            _bet_table_size: None,
        };
        self.write_header(writer, &header_params)?;

        Ok(())
    }

    /// Write archive with HET/BET tables (v3+)
    fn write_archive_with_het_bet<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        let block_table_size = self.pending_files.len() as u32;

        // Calculate sector size
        let sector_size = crate::calculate_sector_size(self.block_size);

        // Reserve space for header (we'll write it at the end)
        let header_size = self.version.header_size();
        writer.seek(SeekFrom::Start(header_size as u64))?;

        // We'll still need block table data for file information
        let mut block_table = BlockTable::new(block_table_size as usize)?;
        let mut hi_block_table = Some(HiBlockTable::new(block_table_size as usize));

        // Write all files and populate block table
        for (block_index, pending_file) in self.pending_files.iter().enumerate() {
            let file_pos = writer.stream_position()?;

            // Read file data
            let file_data = match &pending_file.source {
                FileSource::Path(path) => fs::read(path)?,
                FileSource::Data(data) => data.clone(),
            };

            // Write file and get sizes
            let params = FileWriteParams {
                file_data: &file_data,
                archive_name: &pending_file.archive_name,
                compression: pending_file.compression,
                encrypt: pending_file.encrypt,
                use_fix_key: pending_file.use_fix_key,
                sector_size,
                file_pos,
            };
            let (compressed_size, flags) = self.write_file(writer, &params)?;

            // Add to block table
            let block_entry = BlockEntry {
                file_pos: file_pos as u32, // Low 32 bits
                compressed_size: compressed_size as u32,
                file_size: file_data.len() as u32,
                flags: flags | BlockEntry::FLAG_EXISTS,
            };

            // Store high 16 bits in hi-block table
            if let Some(ref mut hi_table) = hi_block_table {
                let high_bits = (file_pos >> 32) as u16;
                hi_table.set(block_index, high_bits);
            }

            // Update block table entry
            if let Some(entry) = block_table.get_mut(block_index) {
                *entry = block_entry;
            } else {
                return Err(Error::invalid_format("Block index out of bounds"));
            }
        }

        // Create HET table
        let het_table_pos = writer.stream_position()?;
        let (het_data, _het_header) = self.create_het_table()?;
        let het_table_size = het_data.len() as u64;
        self.write_het_table(writer, &het_data, true)?;

        // Create BET table
        let bet_table_pos = writer.stream_position()?;
        let (bet_data, _bet_header) = self.create_bet_table(&block_table)?;
        let bet_table_size = bet_data.len() as u64;
        self.write_bet_table(writer, &bet_data, true)?;

        // For compatibility, also write classic tables
        let hash_table_size = self.calculate_hash_table_size();
        let mut hash_table = HashTable::new(hash_table_size as usize)?;

        // Populate hash table
        for (block_index, pending_file) in self.pending_files.iter().enumerate() {
            self.add_to_hash_table(
                &mut hash_table,
                &pending_file.archive_name,
                block_index as u32,
                pending_file.locale,
            )?;
        }

        // Write hash table
        let hash_table_pos = writer.stream_position()?;
        self.write_hash_table(writer, &hash_table)?;

        // Write block table
        let block_table_pos = writer.stream_position()?;
        self.write_block_table(writer, &block_table)?;

        // Write hi-block table if needed
        let hi_block_table_pos = if let Some(ref hi_table) = hi_block_table {
            if hi_table.is_needed() {
                let pos = writer.stream_position()?;
                self.write_hi_block_table(writer, hi_table)?;
                Some(pos)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate archive size
        let archive_size = writer.stream_position()?;

        // Write header at the beginning
        writer.seek(SeekFrom::Start(0))?;
        let header_params = HeaderWriteParams {
            archive_size,
            hash_table_pos,
            block_table_pos,
            hash_table_size,
            block_table_size: self.pending_files.len() as u32,
            hi_block_table_pos,
            het_table_pos: Some(het_table_pos),
            bet_table_pos: Some(bet_table_pos),
            _het_table_size: Some(het_table_size),
            _bet_table_size: Some(bet_table_size),
        };
        self.write_header(writer, &header_params)?;

        Ok(())
    }

    /// Write a single file to the archive
    fn write_file<W: Write>(
        &self,
        writer: &mut W,
        params: &FileWriteParams<'_>,
    ) -> Result<(usize, u32)> {
        let FileWriteParams {
            file_data,
            archive_name,
            compression,
            encrypt,
            use_fix_key,
            sector_size,
            file_pos,
        } = params;
        let mut flags = 0u32;

        // For small files or if single unit is requested, write as single unit
        let is_single_unit = file_data.len() <= *sector_size;

        if is_single_unit {
            flags |= BlockEntry::FLAG_SINGLE_UNIT;

            // Set CRC flag early if enabled (needed for encryption key calculation)
            if self.generate_crcs {
                flags |= BlockEntry::FLAG_SECTOR_CRC;
            }

            // Compress if needed
            let compressed_data = if *compression != 0 && !file_data.is_empty() {
                log::debug!(
                    "Compressing {} with method 0x{:02X}",
                    archive_name,
                    compression
                );
                let compressed = compress(file_data, *compression)?;

                // Only use compression if it actually reduces size
                if compressed.len() < file_data.len() {
                    log::debug!(
                        "Compression successful: {} -> {} bytes",
                        file_data.len(),
                        compressed.len()
                    );
                    flags |= BlockEntry::FLAG_COMPRESS;
                    // For encrypted files, always prepend compression type byte
                    // For unencrypted files, zlib can be stored without type byte for compatibility
                    if *encrypt || *compression != compression_flags::ZLIB {
                        let mut final_data = Vec::with_capacity(1 + compressed.len());
                        final_data.push(*compression);
                        final_data.extend_from_slice(&compressed);
                        final_data
                    } else {
                        // Zlib can be stored without type byte for compatibility (unencrypted only)
                        compressed
                    }
                } else {
                    // Don't compress if it doesn't help
                    log::debug!("Compression not beneficial, storing uncompressed");
                    file_data.to_vec()
                }
            } else {
                file_data.to_vec()
            };

            // Encrypt if needed
            let final_data = if *encrypt {
                flags |= BlockEntry::FLAG_ENCRYPTED;
                if *use_fix_key {
                    flags |= BlockEntry::FLAG_FIX_KEY;
                }
                let key =
                    self.calculate_file_key(archive_name, *file_pos, file_data.len() as u32, flags);
                let mut encrypted = compressed_data;
                self.encrypt_data(&mut encrypted, key);
                encrypted
            } else {
                compressed_data
            };

            // Write the data
            writer.write_all(&final_data)?;

            // Write CRC if enabled
            if self.generate_crcs {
                let crc = crc32fast::hash(file_data);
                writer.write_u32_le(crc)?;
                log::debug!(
                    "Generated CRC for single unit file {}: 0x{:08X}",
                    archive_name,
                    crc
                );
            }

            // Return compressed size (NOT including CRC)
            Ok((final_data.len(), flags))
        } else {
            // Multi-sector file
            let sector_count = file_data.len().div_ceil(*sector_size);

            // Set CRC flag early if enabled (needed for encryption key calculation)
            if self.generate_crcs {
                flags |= BlockEntry::FLAG_SECTOR_CRC;
            }

            // Reserve space for sector offset table and CRC table if enabled
            let offset_table_size = (sector_count + 1) * 4;
            let crc_table_size = if self.generate_crcs {
                sector_count * 4
            } else {
                0
            };
            let data_start = offset_table_size + crc_table_size;

            let mut sector_offsets = vec![0u32; sector_count + 1];
            let mut sector_data = Vec::new();
            let mut sector_crcs = if self.generate_crcs {
                Vec::with_capacity(sector_count)
            } else {
                Vec::new()
            };

            // Process each sector
            for (i, offset) in sector_offsets.iter_mut().enumerate().take(sector_count) {
                let sector_start = i * *sector_size;
                let sector_end = ((i + 1) * *sector_size).min(file_data.len());
                let sector_bytes = &file_data[sector_start..sector_end];

                *offset = (data_start + sector_data.len()) as u32;

                // Calculate CRC for uncompressed sector if enabled
                if self.generate_crcs {
                    let crc = crc32fast::hash(sector_bytes);
                    sector_crcs.push(crc);
                }

                // Compress sector if needed
                let compressed_sector = if *compression != 0 && !sector_bytes.is_empty() {
                    // Check if compression actually helps
                    let compressed = compress(sector_bytes, *compression)?;
                    if compressed.len() < sector_bytes.len() {
                        flags |= BlockEntry::FLAG_COMPRESS;
                        // For encrypted files, always prepend compression type byte to each sector
                        if *encrypt {
                            let mut final_sector = Vec::with_capacity(1 + compressed.len());
                            final_sector.push(*compression);
                            final_sector.extend_from_slice(&compressed);
                            final_sector
                        } else {
                            compressed
                        }
                    } else {
                        sector_bytes.to_vec()
                    }
                } else {
                    sector_bytes.to_vec()
                };

                sector_data.extend_from_slice(&compressed_sector);
            }

            // Set last offset
            sector_offsets[sector_count] = (data_start + sector_data.len()) as u32;

            // Log CRC generation if enabled
            if self.generate_crcs {
                log::debug!(
                    "Generated {} sector CRCs for file {}, first few: {:?}",
                    sector_count,
                    archive_name,
                    &sector_crcs[..5.min(sector_crcs.len())]
                );
            }

            // Encrypt if needed
            if *encrypt {
                flags |= BlockEntry::FLAG_ENCRYPTED;
                if *use_fix_key {
                    flags |= BlockEntry::FLAG_FIX_KEY;
                }
                let key =
                    self.calculate_file_key(archive_name, *file_pos, file_data.len() as u32, flags);

                // Save original offsets for sector encryption
                let original_offsets = sector_offsets.clone();

                // Encrypt sector offset table
                let offset_key = key.wrapping_sub(1);
                self.encrypt_data_u32(&mut sector_offsets, offset_key);

                // Encrypt each sector using the original (unencrypted) offsets
                let mut encrypted_sectors = Vec::new();
                for (i, offset_pair) in original_offsets.windows(2).enumerate() {
                    let start = (offset_pair[0] - data_start as u32) as usize;
                    let end = (offset_pair[1] - data_start as u32) as usize;

                    let mut sector = sector_data[start..end].to_vec();
                    let sector_key = key.wrapping_add(i as u32);
                    self.encrypt_data(&mut sector, sector_key);
                    encrypted_sectors.extend_from_slice(&sector);
                }

                sector_data = encrypted_sectors;
            }

            // Write sector offset table
            for offset in &sector_offsets {
                writer.write_u32_le(*offset)?;
            }

            // Write CRC table if enabled
            if self.generate_crcs {
                for crc in &sector_crcs {
                    writer.write_u32_le(*crc)?;
                }
            }

            // Write sector data
            writer.write_all(&sector_data)?;

            // Return size NOT including CRC table (offset table + sector data only)
            let total_size = offset_table_size + sector_data.len();
            Ok((total_size, flags))
        }
    }

    /// Add a file to the hash table
    fn add_to_hash_table(
        &self,
        hash_table: &mut HashTable,
        filename: &str,
        block_index: u32,
        locale: u16,
    ) -> Result<()> {
        let table_offset = hash_string(filename, hash_type::TABLE_OFFSET);
        let name_a = hash_string(filename, hash_type::NAME_A);
        let name_b = hash_string(filename, hash_type::NAME_B);

        let table_size = hash_table.size() as u32;
        let mut index = table_offset & (table_size - 1);

        // Linear probing to find empty slot
        loop {
            let entry = hash_table
                .get_mut(index as usize)
                .ok_or_else(|| Error::invalid_format("Hash table index out of bounds"))?;

            if entry.is_empty() {
                // Found empty slot
                *entry = HashEntry {
                    name_1: name_a,
                    name_2: name_b,
                    locale,
                    platform: 0, // Always 0 - platform codes are vestigial
                    block_index,
                };
                break;
            }

            // Check for duplicate
            if entry.name_1 == name_a && entry.name_2 == name_b && entry.locale == locale {
                return Err(Error::invalid_format(format!(
                    "Duplicate file in archive: {}",
                    filename
                )));
            }

            // Move to next slot
            index = (index + 1) & (table_size - 1);
        }

        Ok(())
    }

    /// Write the hash table
    fn write_hash_table<W: Write>(&self, writer: &mut W, hash_table: &HashTable) -> Result<()> {
        // Convert to bytes for encryption
        let mut table_data = Vec::new();
        for entry in hash_table.entries() {
            table_data.write_u32_le(entry.name_1)?;
            table_data.write_u32_le(entry.name_2)?;
            table_data.write_u16_le(entry.locale)?;
            table_data.write_u16_le(entry.platform)?;
            table_data.write_u32_le(entry.block_index)?;
        }

        // Encrypt the table
        let key = hash_string("(hash table)", hash_type::FILE_KEY);
        self.encrypt_data(&mut table_data, key);

        // Write encrypted table
        writer.write_all(&table_data)?;

        Ok(())
    }

    /// Write the block table
    fn write_block_table<W: Write>(&self, writer: &mut W, block_table: &BlockTable) -> Result<()> {
        // Convert to bytes for encryption
        let mut table_data = Vec::new();
        for entry in block_table.entries() {
            table_data.write_u32_le(entry.file_pos)?;
            table_data.write_u32_le(entry.compressed_size)?;
            table_data.write_u32_le(entry.file_size)?;
            table_data.write_u32_le(entry.flags)?;
        }

        // Encrypt the table
        let key = hash_string("(block table)", hash_type::FILE_KEY);
        self.encrypt_data(&mut table_data, key);

        // Write encrypted table
        writer.write_all(&table_data)?;

        Ok(())
    }

    /// Write the hi-block table
    fn write_hi_block_table<W: Write>(
        &self,
        writer: &mut W,
        hi_block_table: &HiBlockTable,
    ) -> Result<()> {
        // Hi-block table is not encrypted
        for &entry in hi_block_table.entries() {
            writer.write_u16_le(entry)?;
        }
        Ok(())
    }

    /// Write the MPQ header
    fn write_header<W: Write>(&self, writer: &mut W, params: &HeaderWriteParams) -> Result<()> {
        // Write signature
        writer.write_u32_le(crate::signatures::MPQ_ARCHIVE)?;

        // Write header size
        writer.write_u32_le(self.version.header_size())?;

        // Write archive size (32-bit for v1, deprecated in v2+)
        writer.write_u32_le(params.archive_size.min(u32::MAX as u64) as u32)?;

        // Write format version
        writer.write_u16_le(self.version as u16)?;

        // Write block size
        writer.write_u16_le(self.block_size)?;

        // Write table positions and sizes (low 32 bits)
        writer.write_u32_le(params.hash_table_pos as u32)?;
        writer.write_u32_le(params.block_table_pos as u32)?;
        writer.write_u32_le(params.hash_table_size)?;
        writer.write_u32_le(params.block_table_size)?;

        // Write version-specific fields
        match self.version {
            FormatVersion::V1 => {
                // No additional fields
            }
            FormatVersion::V2 => {
                // Hi-block table position
                writer.write_u64_le(params.hi_block_table_pos.unwrap_or(0))?;

                // High 16 bits of positions
                writer.write_u16_le((params.hash_table_pos >> 32) as u16)?; // hash_table_pos_hi
                writer.write_u16_le((params.block_table_pos >> 32) as u16)?; // block_table_pos_hi
            }
            FormatVersion::V3 => {
                // V2 fields
                writer.write_u64_le(params.hi_block_table_pos.unwrap_or(0))?; // hi_block_table_pos
                writer.write_u16_le((params.hash_table_pos >> 32) as u16)?; // hash_table_pos_hi
                writer.write_u16_le((params.block_table_pos >> 32) as u16)?; // block_table_pos_hi

                // V3 fields
                writer.write_u64_le(params.archive_size)?; // archive_size_64
                writer.write_u64_le(params.bet_table_pos.unwrap_or(0))?; // bet_table_pos
                writer.write_u64_le(params.het_table_pos.unwrap_or(0))?; // het_table_pos
            }
            FormatVersion::V4 => {
                // TODO: Implement V4 header with MD5 checksums
                return Err(Error::invalid_format("V4 format not yet implemented"));
            }
        }

        Ok(())
    }

    /// Calculate file encryption key
    fn calculate_file_key(&self, filename: &str, file_pos: u64, file_size: u32, flags: u32) -> u32 {
        let base_key = hash_string(filename, hash_type::FILE_KEY);

        if flags & BlockEntry::FLAG_FIX_KEY != 0 {
            // For FIX_KEY, use only the low 32 bits of the file position
            (base_key.wrapping_add(file_pos as u32)) ^ file_size
        } else {
            base_key
        }
    }

    /// Encrypt data in place
    pub fn encrypt_data(&self, data: &mut [u8], key: u32) {
        if data.is_empty() || key == 0 {
            return;
        }

        // Process full u32 chunks
        let (chunks, remainder) = data.split_at_mut((data.len() / 4) * 4);

        // Convert chunks to u32 values, encrypt, and write back
        let mut u32_buffer = Vec::with_capacity(chunks.len() / 4);
        for chunk in chunks.chunks_exact(4) {
            u32_buffer.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }

        encrypt_block(&mut u32_buffer, key);

        // Write encrypted u32s back to bytes
        for (i, &encrypted) in u32_buffer.iter().enumerate() {
            let bytes = encrypted.to_le_bytes();
            chunks[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }

        // Handle remaining bytes
        if !remainder.is_empty() {
            let mut last_dword = [0u8; 4];
            last_dword[..remainder.len()].copy_from_slice(remainder);

            let mut last_u32 = u32::from_le_bytes(last_dword);
            encrypt_block(
                std::slice::from_mut(&mut last_u32),
                key.wrapping_add((chunks.len() / 4) as u32),
            );

            let encrypted_bytes = last_u32.to_le_bytes();
            remainder.copy_from_slice(&encrypted_bytes[..remainder.len()]);
        }
    }
    /// Encrypt u32 data in place
    fn encrypt_data_u32(&self, data: &mut [u32], key: u32) {
        encrypt_block(data, key);
    }

    /// Create HET table data
    fn create_het_table(&self) -> Result<(Vec<u8>, HetHeader)> {
        // Calculate required sizes
        let max_file_count = self.pending_files.len() as u32;
        let hash_table_entries = (max_file_count * 2).next_power_of_two();

        log::debug!(
            "Creating HET table: {} files, {} hash entries",
            max_file_count,
            hash_table_entries
        );

        // Calculate bit sizes
        let hash_entry_size = Self::calculate_bits_needed(hash_table_entries as u64);
        let index_size = Self::calculate_bits_needed(max_file_count as u64);

        log::debug!(
            "HET bit sizes: hash_entry_size={}, index_size={}",
            hash_entry_size,
            index_size
        );

        // Calculate table sizes
        // The hash_table_size in the header seems to be calculated differently
        // Based on the reading code: hash_table_entries = hash_table_size * 8 / hash_entry_size
        // So: hash_table_size = hash_table_entries * hash_entry_size / 8
        let hash_table_size = (hash_table_entries * hash_entry_size).div_ceil(8);

        // But the actual data needs more space for hash + index
        let total_entry_bits = hash_entry_size + index_size;
        let actual_hash_table_size = (hash_table_entries * total_entry_bits).div_ceil(8);
        let total_index_size = hash_table_entries * index_size;
        let index_size_extra = 0; // No extra bits for now

        // Create header
        let header = HetHeader {
            signature: 0x1A544548, // "HET\x1A"
            version: 1,
            data_size: 0,  // Will be calculated later
            table_size: 0, // Will be calculated later
            max_file_count,
            hash_table_size,
            hash_entry_size,
            total_index_size,
            index_size_extra,
            index_size,
            block_table_size: 0, // Not used in our implementation
        };

        // Create hash table and file indices
        let mut hash_table = vec![0u8; actual_hash_table_size as usize];
        let file_indices_size = (total_index_size as usize).div_ceil(8); // Round up to bytes
        let file_indices = vec![0u8; file_indices_size];

        log::debug!(
            "HET table sizes: hash_table_size={}, actual_hash_table_size={}, file_indices_size={}",
            hash_table_size,
            actual_hash_table_size,
            file_indices_size
        );

        // Build file hash map to track which hash slots map to which files
        let mut file_map: Vec<Option<u32>> = vec![None; hash_table_entries as usize];

        // Process each file
        for (file_index, pending_file) in self.pending_files.iter().enumerate() {
            let hash = jenkins_hash(&pending_file.archive_name);
            let hash_mask = (1u64 << hash_entry_size) - 1;
            let table_index = (hash & (hash_table_entries as u64 - 1)) as usize;

            // Linear probing for collision resolution
            let mut current_index = table_index;
            loop {
                if file_map[current_index].is_none() {
                    file_map[current_index] = Some(file_index as u32);

                    // Write hash entry (stores hash + file index in upper bits)
                    let hash_entry = (hash & hash_mask) | ((file_index as u64) << hash_entry_size);
                    // The actual entry size is hash_entry_size for the hash + index_size for the file index
                    self.write_bit_entry(
                        &mut hash_table,
                        current_index,
                        hash_entry,
                        hash_entry_size + index_size,
                    )?;

                    break;
                }

                current_index = (current_index + 1) % hash_table_entries as usize;
                if current_index == table_index {
                    return Err(Error::invalid_format("HET table full"));
                }
            }
        }

        // Combine all data
        let header_size = std::mem::size_of::<HetHeader>();
        let data_size = hash_table_size + file_indices_size as u32;
        let table_size = header_size as u32 + data_size;

        // Update header with final sizes
        let mut final_header = header;
        final_header.data_size = data_size;
        final_header.table_size = table_size;

        // Serialize header and combine with table data
        let mut result = Vec::with_capacity(table_size as usize);
        result.write_u32_le(final_header.signature)?;
        result.write_u32_le(final_header.version)?;
        result.write_u32_le(final_header.data_size)?;
        result.write_u32_le(final_header.table_size)?;
        result.write_u32_le(final_header.max_file_count)?;
        result.write_u32_le(final_header.hash_table_size)?;
        result.write_u32_le(final_header.hash_entry_size)?;
        result.write_u32_le(final_header.total_index_size)?;
        result.write_u32_le(final_header.index_size_extra)?;
        result.write_u32_le(final_header.index_size)?;
        result.write_u32_le(final_header.block_table_size)?;

        result.extend_from_slice(&hash_table);
        result.extend_from_slice(&file_indices);

        Ok((result, final_header))
    }

    /// Write a bit-packed entry to a byte array
    fn write_bit_entry(
        &self,
        data: &mut [u8],
        index: usize,
        value: u64,
        bit_size: u32,
    ) -> Result<()> {
        let bit_offset = index * bit_size as usize;
        let byte_offset = bit_offset / 8;
        let bit_shift = bit_offset % 8;

        // Calculate how many bytes we actually need
        let bits_needed = bit_shift + bit_size as usize;
        let bytes_needed = bits_needed.div_ceil(8);

        if byte_offset + bytes_needed > data.len() {
            log::error!(
                "Bit entry out of bounds: index={}, bit_size={}, bit_offset={}, byte_offset={}, bytes_needed={}, data.len()={}", 
                index, bit_size, bit_offset, byte_offset, bytes_needed, data.len()
            );
            return Err(Error::invalid_format("Bit entry out of bounds"));
        }

        // Read existing bits
        let mut existing = 0u64;
        for i in 0..bytes_needed {
            if byte_offset + i < data.len() {
                existing |= (data[byte_offset + i] as u64) << (i * 8);
            }
        }

        // Clear the bits we're about to write
        let value_mask = if bit_size >= 64 {
            u64::MAX
        } else {
            (1u64 << bit_size) - 1
        };
        let mask = value_mask << bit_shift;
        existing &= !mask;

        // Write the new value
        existing |= (value & value_mask) << bit_shift;

        // Write back
        for i in 0..bytes_needed {
            if byte_offset + i < data.len() {
                data[byte_offset + i] = (existing >> (i * 8)) as u8;
            }
        }

        Ok(())
    }

    /// Calculate the number of bits needed to represent a value
    fn calculate_bits_needed(max_value: u64) -> u32 {
        if max_value == 0 {
            1
        } else {
            (64 - max_value.leading_zeros()).max(1)
        }
    }

    /// Write HET table to the archive
    fn write_het_table<W: Write>(&self, writer: &mut W, data: &[u8], encrypt: bool) -> Result<()> {
        if encrypt {
            let mut encrypted = data.to_vec();
            let key = hash_string("(hash table)", hash_type::FILE_KEY);
            self.encrypt_data(&mut encrypted, key);
            writer.write_all(&encrypted)?;
        } else {
            writer.write_all(data)?;
        }
        Ok(())
    }

    /// Create BET table data
    fn create_bet_table(&self, block_table: &BlockTable) -> Result<(Vec<u8>, BetHeader)> {
        let file_count = self.pending_files.len() as u32;

        // Analyze block table to determine optimal bit widths
        let mut max_file_pos = 0u64;
        let mut max_file_size = 0u64;
        let mut max_compressed_size = 0u64;
        let mut unique_flags = std::collections::HashSet::new();

        for i in 0..file_count as usize {
            if let Some(entry) = block_table.get(i) {
                max_file_pos = max_file_pos.max(entry.file_pos as u64);
                max_file_size = max_file_size.max(entry.file_size as u64);
                max_compressed_size = max_compressed_size.max(entry.compressed_size as u64);
                unique_flags.insert(entry.flags);
            }
        }

        // Calculate bit counts for each field
        let bit_count_file_pos = Self::calculate_bits_needed(max_file_pos);
        let bit_count_file_size = Self::calculate_bits_needed(max_file_size);
        let bit_count_cmp_size = Self::calculate_bits_needed(max_compressed_size);
        let bit_count_flag_index = Self::calculate_bits_needed(unique_flags.len() as u64 - 1);
        let bit_count_unknown = 0; // Not used

        // Calculate bit positions
        let bit_index_file_pos = 0;
        let bit_index_file_size = bit_index_file_pos + bit_count_file_pos;
        let bit_index_cmp_size = bit_index_file_size + bit_count_file_size;
        let bit_index_flag_index = bit_index_cmp_size + bit_count_cmp_size;
        let bit_index_unknown = bit_index_flag_index + bit_count_flag_index;

        // Calculate table entry size
        let table_entry_size = bit_index_unknown + bit_count_unknown;

        // Create flag array
        let mut flag_array: Vec<u32> = unique_flags.into_iter().collect();
        flag_array.sort();
        let flag_count = flag_array.len() as u32;

        // Create flag index map
        let mut flag_index_map = std::collections::HashMap::new();
        for (index, &flags) in flag_array.iter().enumerate() {
            flag_index_map.insert(flags, index as u32);
        }

        // Calculate table sizes
        let file_table_bits = file_count * table_entry_size;
        let file_table_size = file_table_bits.div_ceil(8); // Round up to bytes

        // BET hash information (simplified - we'll use 64-bit hashes)
        let bet_hash_size = 64;
        let total_bet_hash_size = file_count * bet_hash_size;
        let bet_hash_size_extra = 0;
        let bet_hash_array_size = total_bet_hash_size.div_ceil(8);

        // Create header
        let header = BetHeader {
            signature: 0x1A544542, // "BET\x1A"
            version: 1,
            data_size: 0,  // Will be calculated later
            table_size: 0, // Will be calculated later
            file_count,
            unknown_08: 0x10,
            table_entry_size,
            bit_index_file_pos,
            bit_index_file_size,
            bit_index_cmp_size,
            bit_index_flag_index,
            bit_index_unknown,
            bit_count_file_pos,
            bit_count_file_size,
            bit_count_cmp_size,
            bit_count_flag_index,
            bit_count_unknown,
            total_bet_hash_size,
            bet_hash_size_extra,
            bet_hash_size,
            bet_hash_array_size,
            flag_count,
        };

        // Create file table
        let mut file_table = vec![0u8; file_table_size as usize];

        // Create BET hashes
        let mut bet_hashes = Vec::with_capacity(file_count as usize);

        // Fill tables
        for (i, pending_file) in self.pending_files.iter().enumerate() {
            if let Some(entry) = block_table.get(i) {
                // Get flag index
                let flag_index = flag_index_map.get(&entry.flags).unwrap();

                // Pack entry data
                let mut entry_bits = 0u64;
                entry_bits |= (entry.file_pos as u64) << bit_index_file_pos;
                entry_bits |= (entry.file_size as u64) << bit_index_file_size;
                entry_bits |= (entry.compressed_size as u64) << bit_index_cmp_size;
                entry_bits |= (*flag_index as u64) << bit_index_flag_index;

                // Write to file table
                self.write_bit_entry(&mut file_table, i, entry_bits, table_entry_size)?;

                // Generate BET hash (Jenkins hash of filename)
                let hash = jenkins_hash(&pending_file.archive_name);
                bet_hashes.push(hash);
            }
        }

        // Calculate final sizes
        let header_size = std::mem::size_of::<BetHeader>();
        let flag_array_size = flag_count * 4;
        let data_size = flag_array_size + file_table_size + bet_hash_array_size;
        let table_size = header_size as u32 + data_size;

        // Update header with final sizes
        let mut final_header = header;
        final_header.data_size = data_size;
        final_header.table_size = table_size;

        // Serialize everything
        let mut result = Vec::with_capacity(table_size as usize);

        // Write header
        result.write_u32_le(final_header.signature)?;
        result.write_u32_le(final_header.version)?;
        result.write_u32_le(final_header.data_size)?;
        result.write_u32_le(final_header.table_size)?;
        result.write_u32_le(final_header.file_count)?;
        result.write_u32_le(final_header.unknown_08)?;
        result.write_u32_le(final_header.table_entry_size)?;
        result.write_u32_le(final_header.bit_index_file_pos)?;
        result.write_u32_le(final_header.bit_index_file_size)?;
        result.write_u32_le(final_header.bit_index_cmp_size)?;
        result.write_u32_le(final_header.bit_index_flag_index)?;
        result.write_u32_le(final_header.bit_index_unknown)?;
        result.write_u32_le(final_header.bit_count_file_pos)?;
        result.write_u32_le(final_header.bit_count_file_size)?;
        result.write_u32_le(final_header.bit_count_cmp_size)?;
        result.write_u32_le(final_header.bit_count_flag_index)?;
        result.write_u32_le(final_header.bit_count_unknown)?;
        result.write_u32_le(final_header.total_bet_hash_size)?;
        result.write_u32_le(final_header.bet_hash_size_extra)?;
        result.write_u32_le(final_header.bet_hash_size)?;
        result.write_u32_le(final_header.bet_hash_array_size)?;
        result.write_u32_le(final_header.flag_count)?;

        // Write flag array
        for &flags in &flag_array {
            result.write_u32_le(flags)?;
        }

        // Write file table
        result.extend_from_slice(&file_table);

        // Write BET hashes (bit-packed)
        let mut hash_bytes = vec![0u8; bet_hash_array_size as usize];
        for (i, &hash) in bet_hashes.iter().enumerate() {
            self.write_bit_entry(&mut hash_bytes, i, hash, bet_hash_size)?;
        }
        result.extend_from_slice(&hash_bytes);

        Ok((result, final_header))
    }

    /// Write BET table to the archive
    fn write_bet_table<W: Write>(&self, writer: &mut W, data: &[u8], encrypt: bool) -> Result<()> {
        if encrypt {
            let mut encrypted = data.to_vec();
            let key = hash_string("(block table)", hash_type::FILE_KEY);
            self.encrypt_data(&mut encrypted, key);
            writer.write_all(&encrypted)?;
        } else {
            writer.write_all(data)?;
        }
        Ok(())
    }
}

impl Default for ArchiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}
