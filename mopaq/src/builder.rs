//! Archive builder for creating MPQ archives

use crate::{
    compression::{compress, flags as compression_flags},
    crypto::{encrypt_block, hash_string, hash_type},
    header::FormatVersion,
    tables::{BlockEntry, BlockTable, HashEntry, HashTable},
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
    /// Sector size
    sector_size: usize,
    /// File position in archive
    file_pos: u32,
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

    /// Add a file from disk
    pub fn add_file<P: AsRef<Path>>(mut self, path: P, archive_name: &str) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Path(path.as_ref().to_path_buf()),
            archive_name: archive_name.to_string(),
            compression: self.default_compression,
            encrypt: false,
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
            // TODO: Implement HET/BET table creation
            log::warn!(
                "HET/BET table creation not yet implemented, falling back to classic tables"
            );
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

        // Write all files and populate tables
        for (block_index, pending_file) in self.pending_files.iter().enumerate() {
            let file_pos = writer.stream_position()? as u32;

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

            // Add to block table
            let block_entry = BlockEntry {
                file_pos,
                compressed_size: compressed_size as u32,
                file_size: file_data.len() as u32,
                flags: flags | BlockEntry::FLAG_EXISTS,
            };

            // Get mutable reference and update
            if let Some(entry) = block_table.get_mut(block_index) {
                *entry = block_entry;
            } else {
                return Err(Error::invalid_format("Block index out of bounds"));
            }
        }

        // Write hash table
        let hash_table_pos = writer.stream_position()? as u32;
        self.write_hash_table(writer, &hash_table)?;

        // Write block table
        let block_table_pos = writer.stream_position()? as u32;
        self.write_block_table(writer, &block_table)?;

        // Calculate archive size
        let archive_size = writer.stream_position()? as u32;

        // Write header at the beginning
        writer.seek(SeekFrom::Start(0))?;
        self.write_header(
            writer,
            archive_size,
            hash_table_pos,
            block_table_pos,
            hash_table_size,
            block_table_size,
        )?;

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
            sector_size,
            file_pos,
        } = params;
        let mut flags = 0u32;

        // For small files or if single unit is requested, write as single unit
        let is_single_unit = file_data.len() <= *sector_size;

        if is_single_unit {
            flags |= BlockEntry::FLAG_SINGLE_UNIT;

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
                    // For non-zlib compression, prepend the compression type byte
                    if *compression != compression_flags::ZLIB {
                        let mut final_data = Vec::with_capacity(1 + compressed.len());
                        final_data.push(*compression);
                        final_data.extend_from_slice(&compressed);
                        final_data
                    } else {
                        // Zlib can be stored without type byte for compatibility
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

            Ok((final_data.len(), flags))
        } else {
            // Multi-sector file
            let sector_count = file_data.len().div_ceil(*sector_size);

            // Reserve space for sector offset table
            let offset_table_size = (sector_count + 1) * 4;
            let data_start = offset_table_size;

            let mut sector_offsets = vec![0u32; sector_count + 1];
            let mut sector_data = Vec::new();

            // Process each sector
            for (i, offset) in sector_offsets.iter_mut().enumerate().take(sector_count) {
                let sector_start = i * *sector_size;
                let sector_end = ((i + 1) * *sector_size).min(file_data.len());
                let sector_bytes = &file_data[sector_start..sector_end];

                *offset = (data_start + sector_data.len()) as u32;

                // Compress sector if needed
                let compressed_sector = if *compression != 0 && !sector_bytes.is_empty() {
                    // Check if compression actually helps
                    let compressed = compress(sector_bytes, *compression)?;
                    if compressed.len() < sector_bytes.len() {
                        flags |= BlockEntry::FLAG_COMPRESS;
                        compressed
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

            // Encrypt if needed
            if *encrypt {
                flags |= BlockEntry::FLAG_ENCRYPTED;
                let key =
                    self.calculate_file_key(archive_name, *file_pos, file_data.len() as u32, flags);

                // Encrypt sector offset table
                let offset_key = key.wrapping_sub(1);
                self.encrypt_data_u32(&mut sector_offsets, offset_key);

                // Encrypt each sector
                let mut encrypted_sectors = Vec::new();
                for (i, offset_pair) in sector_offsets.windows(2).enumerate() {
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

            // Write sector data
            writer.write_all(&sector_data)?;

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
                    platform: 0,
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

    /// Write the MPQ header
    fn write_header<W: Write>(
        &self,
        writer: &mut W,
        archive_size: u32,
        hash_table_pos: u32,
        block_table_pos: u32,
        hash_table_size: u32,
        block_table_size: u32,
    ) -> Result<()> {
        // Write signature
        writer.write_u32_le(crate::signatures::MPQ_ARCHIVE)?;

        // Write header size
        writer.write_u32_le(self.version.header_size())?;

        // Write archive size
        writer.write_u32_le(archive_size)?;

        // Write format version
        writer.write_u16_le(self.version as u16)?;

        // Write block size
        writer.write_u16_le(self.block_size)?;

        // Write table positions and sizes
        writer.write_u32_le(hash_table_pos)?;
        writer.write_u32_le(block_table_pos)?;
        writer.write_u32_le(hash_table_size)?;
        writer.write_u32_le(block_table_size)?;

        // Write version-specific fields
        match self.version {
            FormatVersion::V1 => {
                // No additional fields
            }
            FormatVersion::V2 => {
                // Hi-block table position (not used in new archives)
                writer.write_u64_le(0)?;

                // High 16 bits of positions (not needed for new archives)
                writer.write_u16_le(0)?; // hash_table_pos_hi
                writer.write_u16_le(0)?; // block_table_pos_hi
            }
            FormatVersion::V3 => {
                // V2 fields
                writer.write_u64_le(0)?; // hi_block_table_pos
                writer.write_u16_le(0)?; // hash_table_pos_hi
                writer.write_u16_le(0)?; // block_table_pos_hi

                // V3 fields
                writer.write_u64_le(archive_size as u64)?; // archive_size_64
                writer.write_u64_le(0)?; // bet_table_pos
                writer.write_u64_le(0)?; // het_table_pos
            }
            FormatVersion::V4 => {
                // TODO: Implement V4 header with MD5 checksums
                return Err(Error::invalid_format("V4 format not yet implemented"));
            }
        }

        Ok(())
    }

    /// Calculate file encryption key
    fn calculate_file_key(&self, filename: &str, file_pos: u32, file_size: u32, flags: u32) -> u32 {
        let base_key = hash_string(filename, hash_type::FILE_KEY);

        if flags & BlockEntry::FLAG_FIX_KEY != 0 {
            (base_key.wrapping_add(file_pos)) ^ file_size
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
}

impl Default for ArchiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}
