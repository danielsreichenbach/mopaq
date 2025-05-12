//! MPQ file handling
//! Provides functionality for reading files from MPQ archives

use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

use crate::archive::ReadSeek;
use crate::compression::{decompress_block, detect_compression_types};
use crate::crypto::{decrypt_block, detect_file_key, generate_file_key};
use crate::error::{Error, Result};
use crate::tables::block_table::{BlockEntry, block_flags};
use crate::tables::ext_table::ExtBlockEntry;

/// A file within an MPQ archive
pub struct MpqFile {
    /// Name of the file if known
    pub name: Option<String>,

    /// Block entry containing file metadata
    block: BlockEntry,

    /// Extended block entry (for v2+ archives)
    ext_block: Option<ExtBlockEntry>,

    /// Offset of the MPQ header within the archive
    header_offset: u64,

    /// Archive sector size
    sector_size: u32,

    /// Reader for accessing the archive data
    reader: Arc<Mutex<Box<dyn ReadSeek>>>,
}

impl MpqFile {
    /// Creates a new MPQ file
    pub fn new(
        name: String,
        block: &BlockEntry,
        ext_block: Option<&ExtBlockEntry>,
        header_offset: u64,
        sector_size: u32,
        reader: Arc<Mutex<Box<dyn ReadSeek>>>,
    ) -> Result<Self> {
        // Check if the file exists (not deleted)
        if !block.exists() {
            return Err(Error::FileNotFound(name));
        }

        // Copy block entry
        let block = block.clone();

        // Copy extended block entry if present
        let ext_block = ext_block.cloned();

        Ok(Self {
            name: Some(name),
            block,
            ext_block,
            header_offset,
            sector_size,
            reader,
        })
    }

    /// Gets the full 64-bit file offset
    pub fn offset_64(&self) -> u64 {
        if let Some(ext) = &self.ext_block {
            ((ext.offset_high as u64) << 32) | (self.block.offset as u64)
        } else {
            self.block.offset as u64
        }
    }

    /// Gets the full 64-bit compressed size
    pub fn compressed_size_64(&self) -> u64 {
        if let Some(ext) = &self.ext_block {
            ((ext.compressed_size_high as u64) << 32) | (self.block.compressed_size as u64)
        } else {
            self.block.compressed_size as u64
        }
    }

    /// Gets the full 64-bit file size
    pub fn file_size_64(&self) -> u64 {
        if let Some(ext) = &self.ext_block {
            ((ext.file_size_high as u64) << 32) | (self.block.file_size as u64)
        } else {
            self.block.file_size as u64
        }
    }

    /// Checks if this file is encrypted
    pub fn is_encrypted(&self) -> bool {
        self.block.is_encrypted()
    }

    /// Checks if this file is compressed
    pub fn is_compressed(&self) -> bool {
        self.block.is_compressed()
    }

    /// Checks if this file is stored as a single unit
    pub fn is_single_unit(&self) -> bool {
        self.block.is_single_unit()
    }

    /// Gets the file's encryption key if it's encrypted
    pub fn encryption_key(&self) -> Option<u32> {
        if !self.is_encrypted() {
            return None;
        }

        // Try to generate a key from the filename
        if let Some(ref name) = self.name {
            Some(generate_file_key(name))
        } else {
            // If no filename, try to generate a key from the file offset
            Some(crate::crypto::key_derivation::generate_key_from_offset(
                self.block.offset,
            ))
        }
    }

    /// Reads the file's raw data into a buffer
    pub fn read_raw_data(&self) -> Result<Vec<u8>> {
        // Calculate the absolute file offset
        let file_offset = self.header_offset + self.offset_64();

        // Get a lock on the reader
        let mut reader_guard = self
            .reader
            .lock()
            .map_err(|_| Error::Other("Failed to lock reader".to_string()))?;

        // Seek to the file position
        reader_guard
            .seek(SeekFrom::Start(file_offset))
            .map_err(|e| Error::IoError(e))?;

        // Read the compressed data
        let mut data = vec![0u8; self.compressed_size_64() as usize];
        reader_guard
            .read_exact(&mut data)
            .map_err(|e| Error::IoError(e))?;

        Ok(data)
    }

    /// Reads the file's data, decompressing and decrypting if necessary
    pub fn read_data(&self) -> Result<Vec<u8>> {
        // Read the raw data
        let mut data = self.read_raw_data()?;

        // If the file is encrypted, decrypt it
        if self.is_encrypted() {
            // Get the encryption key
            let key = if let Some(key) = self.encryption_key() {
                key
            } else {
                // Try to detect the key
                let encryption_header = &data[0..min(data.len(), 8)];
                detect_file_key(encryption_header, self.block.offset, &[])
                    .map_err(|e| Error::CryptoError(e))?
            };

            // Decrypt the data
            decrypt_block(&mut data, key).map_err(|e| Error::CryptoError(e))?;
        }

        // If the file is a single unit, handle it differently
        if self.is_single_unit() {
            // If compressed, decompress the whole file
            if self.is_compressed() {
                // Detect compression type from the first byte
                if data.is_empty() {
                    return Err(Error::InvalidSector {
                        context: "Empty data for compressed file".to_string(),
                        source: None,
                    });
                }

                // Decompress the data
                let file_size = self.file_size_64() as usize;
                data =
                    decompress_block(&data, file_size).map_err(|e| Error::CompressionError(e))?;
            }

            return Ok(data);
        }

        // Handle a file split into sectors
        if self.block.file_size > 0 {
            // Read and process sector data
            self.read_sectors(data)
        } else {
            // Empty file
            Ok(Vec::new())
        }
    }

    /// Reads a file split into sectors
    fn read_sectors(&self, mut data: Vec<u8>) -> Result<Vec<u8>> {
        // Calculate sector-related values
        let sectors_count = (self.block.file_size + self.sector_size - 1) / self.sector_size;
        let sector_count_plus_1 = sectors_count + 1;

        // A sector offset table is at the beginning of the file data
        // Each entry is 4 bytes
        let sector_table_size = (sector_count_plus_1 * 4) as usize;

        if data.len() < sector_table_size {
            return Err(Error::InvalidSector {
                context: format!(
                    "Data too small for sector table: {} < {}",
                    data.len(),
                    sector_table_size
                ),
                source: None,
            });
        }

        // Read the sector offset table
        let mut sector_offsets = Vec::with_capacity(sector_count_plus_1 as usize);
        for i in 0..sector_count_plus_1 as usize {
            let offset = u32::from_le_bytes([
                data[i * 4],
                data[i * 4 + 1],
                data[i * 4 + 2],
                data[i * 4 + 3],
            ]);
            sector_offsets.push(offset);
        }

        // Allocate the output buffer
        let mut result = Vec::with_capacity(self.block.file_size as usize);

        // Process each sector
        for i in 0..sectors_count as usize {
            let offset = sector_offsets[i] as usize;
            let next_offset = sector_offsets[i + 1] as usize;

            // Calculate the current sector's size
            let sector_size = next_offset - offset;

            if offset >= data.len() || offset + sector_size > data.len() {
                return Err(Error::InvalidSector {
                    context: format!(
                        "Invalid sector offset: {} + {} exceeds data length {}",
                        offset,
                        sector_size,
                        data.len()
                    ),
                    source: None,
                });
            }

            // Calculate expected decompressed size
            let expected_size = if i == sectors_count as usize - 1 {
                // Last sector might be smaller
                self.block.file_size as usize - (i * self.sector_size as usize)
            } else {
                self.sector_size as usize
            };

            // Extract the sector data
            let sector_data = &data[offset..offset + sector_size];

            // Check if sector is compressed
            if sector_size < expected_size {
                if sector_data.is_empty() {
                    // Empty sector - just add zeroes
                    result.extend(vec![0u8; expected_size]);
                } else {
                    // Decompress the sector
                    let decompressed = decompress_block(sector_data, expected_size)
                        .map_err(|e| Error::CompressionError(e))?;

                    // Add to result
                    result.extend(decompressed);
                }
            } else {
                // Sector not compressed, just add the raw data
                result.extend_from_slice(sector_data);
            }
        }

        Ok(result)
    }
}

// Helper functions
fn min<T: Ord>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
