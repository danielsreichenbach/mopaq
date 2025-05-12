//! MPQ header structures and parsing

use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Seek, SeekFrom, Write};
use thiserror::Error;

/// MPQ standard header signature: "MPQ\x1A"
pub const MPQ_SIGNATURE: u32 = 0x1A51504D;

/// MPQ user data signature: "MPQ\x1B"
pub const MPQ_USER_DATA_SIGNATURE: u32 = 0x1B51504D;

/// Error types specific to MPQ header operations
#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("I/O error: {0}")]
    IoError(#[from] IoError),

    #[error("Invalid MPQ signature")]
    InvalidSignature,

    #[error("Unsupported MPQ version: {0}")]
    UnsupportedVersion(u16),

    #[error("Invalid header size: expected {expected}, got {actual}")]
    InvalidHeaderSize { expected: u32, actual: u32 },

    #[error("Header not found within search limit")]
    HeaderNotFound,

    #[error("Invalid sector size shift: {0}")]
    InvalidSectorSizeShift(u16),

    #[error("Hash table offset out of bounds")]
    HashTableOffsetOutOfBounds,

    #[error("Block table offset out of bounds")]
    BlockTableOffsetOutOfBounds,

    #[error("Unreasonable hash table entry count: {0}")]
    UnreasonableHashTableSize(u32),

    #[error("Unreasonable block table entry count: {0}")]
    UnreasonableBlockTableSize(u32),
}

/// Represents the header of an MPQ archive
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MpqHeader {
    /// Magic signature, should be "MPQ\x1A" (0x1A51504D)
    pub signature: u32,
    /// Size of the header in bytes
    pub header_size: u32,
    /// Size of the archive in bytes
    pub archive_size: u32,
    /// Format version
    pub format_version: u16,
    /// Sector size shift value: sector_size = 512 * 2^sector_size_shift
    /// Typically 3 for 4KB sectors (512 * 2^3 = 4096 bytes)
    pub sector_size_shift: u16,
    /// Offset to the hash table from archive start
    pub hash_table_offset: u32,
    /// Offset to the block table from archive start
    pub block_table_offset: u32,
    /// Number of entries in the hash table
    pub hash_table_entries: u32,
    /// Number of entries in the block table
    pub block_table_entries: u32,

    // Fields below are only present in v2+ headers
    /// High 16 bits of the extended block table offset (v2+)
    pub ext_block_table_offset_high: u16,
    /// High 16 bits of the hash table offset (v2+)
    pub hash_table_offset_high: u16,
    /// High 16 bits of the block table offset (v2+)
    pub block_table_offset_high: u16,

    // Fields below are only present in v3+ headers
    /// Version of the archive (v3+)
    pub archive_version: u8,
    /// Version of the MPQ format (v3+)
    pub mpq_version: u8,

    // Fields below are only present in v4+ headers
    /// High 16 bits of the archive size (v4+)
    pub archive_size_high: u16,
    /// Offset to the BET table (v4+)
    pub bet_table_offset: u64,
    /// Offset to the HET table (v4+)
    pub het_table_offset: u64,
}

/// Represents the size of MPQ headers for different versions
pub enum MpqHeaderSize {
    V1Size = 32,  // Basic header size (v1)
    V2Size = 44,  // Header size for v2 (with extended fields)
    V3Size = 68,  // Header size for v3 (with HET/BET tables support)
    V4Size = 208, // Header size for v4 (with additional data)
}

impl MpqHeader {
    /// Creates a new empty MPQ header with default values for v1
    pub fn new_v1() -> Self {
        Self {
            signature: MPQ_SIGNATURE,
            header_size: MpqHeaderSize::V1Size as u32,
            archive_size: 0,
            format_version: 0,
            sector_size_shift: 9, // Default to 512 * 2^9 = 4096 bytes
            hash_table_offset: 0,
            block_table_offset: 0,
            hash_table_entries: 0,
            block_table_entries: 0,

            // Default values for v2+ fields
            ext_block_table_offset_high: 0,
            hash_table_offset_high: 0,
            block_table_offset_high: 0,

            // Default values for v3+ fields
            archive_version: 0,
            mpq_version: 0,

            // Default values for v4+ fields
            archive_size_high: 0,
            bet_table_offset: 0,
            het_table_offset: 0,
        }
    }

    /// Creates a new header for a specified version
    pub fn new(version: u16) -> Result<Self, HeaderError> {
        match version {
            0 | 1 => Ok(Self::new_v1()),
            2 => {
                let mut header = Self::new_v1();
                header.format_version = 2;
                header.header_size = MpqHeaderSize::V2Size as u32;
                Ok(header)
            }
            3 => {
                let mut header = Self::new_v1();
                header.format_version = 3;
                header.header_size = MpqHeaderSize::V3Size as u32;
                header.archive_version = 1;
                header.mpq_version = 3;
                Ok(header)
            }
            4 => {
                let mut header = Self::new_v1();
                header.format_version = 4;
                header.header_size = MpqHeaderSize::V4Size as u32;
                header.archive_version = 1;
                header.mpq_version = 4;
                Ok(header)
            }
            _ => Err(HeaderError::UnsupportedVersion(version)),
        }
    }

    /// Returns the sector size in bytes
    /// Calculated as 512 * 2^sector_size_shift
    /// Common values:
    /// - shift=0: 512 bytes
    /// - shift=1: 1024 bytes
    /// - shift=2: 2048 bytes
    /// - shift=3: 4096 bytes (4KB, most common)
    pub fn sector_size(&self) -> u32 {
        512 << self.sector_size_shift
    }

    /// Returns the 64-bit hash table offset
    pub fn hash_table_offset_64(&self) -> u64 {
        if self.format_version >= 2 {
            ((self.hash_table_offset_high as u64) << 32) | (self.hash_table_offset as u64)
        } else {
            self.hash_table_offset as u64
        }
    }

    /// Returns the 64-bit block table offset
    pub fn block_table_offset_64(&self) -> u64 {
        if self.format_version >= 2 {
            ((self.block_table_offset_high as u64) << 32) | (self.block_table_offset as u64)
        } else {
            self.block_table_offset as u64
        }
    }

    /// Returns the 64-bit archive size
    pub fn archive_size_64(&self) -> u64 {
        if self.format_version >= 4 {
            ((self.archive_size_high as u64) << 32) | (self.archive_size as u64)
        } else {
            self.archive_size as u64
        }
    }

    /// Read an MPQ header from a reader at the current position
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self, HeaderError> {
        let mut header = Self::new_v1();

        // Read basic header (v1 size)
        let mut buffer = [0u8; MpqHeaderSize::V1Size as usize];
        reader.read_exact(&mut buffer)?;

        // Parse signature and initial fields
        header.signature = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

        // Validate signature
        if header.signature != MPQ_SIGNATURE {
            return Err(HeaderError::InvalidSignature);
        }

        // Parse the rest of v1 header
        header.header_size = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        header.archive_size = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);
        header.format_version = u16::from_le_bytes([buffer[12], buffer[13]]);
        header.sector_size_shift = u16::from_le_bytes([buffer[14], buffer[15]]);
        header.hash_table_offset =
            u32::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]);
        header.block_table_offset =
            u32::from_le_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]);
        header.hash_table_entries =
            u32::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]);
        header.block_table_entries =
            u32::from_le_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]);

        // Read extended header fields based on format version
        match header.format_version {
            0 | 1 => {
                // v1 format, we've already read everything
            }
            2 => {
                // v2 format adds 12 more bytes
                if header.header_size < MpqHeaderSize::V2Size as u32 {
                    return Err(HeaderError::InvalidHeaderSize {
                        expected: MpqHeaderSize::V2Size as u32,
                        actual: header.header_size,
                    });
                }

                let mut ext_buffer =
                    [0u8; MpqHeaderSize::V2Size as usize - MpqHeaderSize::V1Size as usize];
                reader.read_exact(&mut ext_buffer)?;

                header.ext_block_table_offset_high =
                    u16::from_le_bytes([ext_buffer[0], ext_buffer[1]]);
                header.hash_table_offset_high = u16::from_le_bytes([ext_buffer[2], ext_buffer[3]]);
                header.block_table_offset_high = u16::from_le_bytes([ext_buffer[4], ext_buffer[5]]);
                // Remaining bytes are reserved
            }
            3 => {
                // v3 format adds more fields
                if header.header_size < MpqHeaderSize::V3Size as u32 {
                    return Err(HeaderError::InvalidHeaderSize {
                        expected: MpqHeaderSize::V3Size as u32,
                        actual: header.header_size,
                    });
                }

                let mut ext_buffer =
                    [0u8; MpqHeaderSize::V3Size as usize - MpqHeaderSize::V1Size as usize];
                reader.read_exact(&mut ext_buffer)?;

                // Parse v2 fields
                header.ext_block_table_offset_high =
                    u16::from_le_bytes([ext_buffer[0], ext_buffer[1]]);
                header.hash_table_offset_high = u16::from_le_bytes([ext_buffer[2], ext_buffer[3]]);
                header.block_table_offset_high = u16::from_le_bytes([ext_buffer[4], ext_buffer[5]]);

                // Parse v3 specific fields
                header.archive_version = ext_buffer[6];
                header.mpq_version = ext_buffer[7];
                // Additional v3 fields would be parsed here...
            }
            4 => {
                // v4 format with HET/BET tables
                if header.header_size < MpqHeaderSize::V4Size as u32 {
                    return Err(HeaderError::InvalidHeaderSize {
                        expected: MpqHeaderSize::V4Size as u32,
                        actual: header.header_size,
                    });
                }

                let mut ext_buffer =
                    [0u8; MpqHeaderSize::V4Size as usize - MpqHeaderSize::V1Size as usize];
                reader.read_exact(&mut ext_buffer)?;

                // Parse v2 and v3 fields...
                header.ext_block_table_offset_high =
                    u16::from_le_bytes([ext_buffer[0], ext_buffer[1]]);
                header.hash_table_offset_high = u16::from_le_bytes([ext_buffer[2], ext_buffer[3]]);
                header.block_table_offset_high = u16::from_le_bytes([ext_buffer[4], ext_buffer[5]]);
                header.archive_version = ext_buffer[6];
                header.mpq_version = ext_buffer[7];

                // Parse v4 specific fields
                header.archive_size_high = u16::from_le_bytes([ext_buffer[8], ext_buffer[9]]);

                // Parse HET and BET table offsets (these are at specific offsets in the v4 header)
                let het_offset_pos = 16; // Position in ext_buffer where HET offset starts
                let bet_offset_pos = 24; // Position in ext_buffer where BET offset starts

                header.het_table_offset = u64::from_le_bytes([
                    ext_buffer[het_offset_pos],
                    ext_buffer[het_offset_pos + 1],
                    ext_buffer[het_offset_pos + 2],
                    ext_buffer[het_offset_pos + 3],
                    ext_buffer[het_offset_pos + 4],
                    ext_buffer[het_offset_pos + 5],
                    ext_buffer[het_offset_pos + 6],
                    ext_buffer[het_offset_pos + 7],
                ]);

                header.bet_table_offset = u64::from_le_bytes([
                    ext_buffer[bet_offset_pos],
                    ext_buffer[bet_offset_pos + 1],
                    ext_buffer[bet_offset_pos + 2],
                    ext_buffer[bet_offset_pos + 3],
                    ext_buffer[bet_offset_pos + 4],
                    ext_buffer[bet_offset_pos + 5],
                    ext_buffer[bet_offset_pos + 6],
                    ext_buffer[bet_offset_pos + 7],
                ]);
            }
            _ => {
                return Err(HeaderError::UnsupportedVersion(header.format_version));
            }
        }

        Ok(header)
    }

    /// Writes the header to a writer
    pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> Result<(), HeaderError> {
        // Write basic header fields
        writer.write_all(&self.signature.to_le_bytes())?;
        writer.write_all(&self.header_size.to_le_bytes())?;
        writer.write_all(&self.archive_size.to_le_bytes())?;
        writer.write_all(&self.format_version.to_le_bytes())?;
        writer.write_all(&self.sector_size_shift.to_le_bytes())?;
        writer.write_all(&self.hash_table_offset.to_le_bytes())?;
        writer.write_all(&self.block_table_offset.to_le_bytes())?;
        writer.write_all(&self.hash_table_entries.to_le_bytes())?;
        writer.write_all(&self.block_table_entries.to_le_bytes())?;

        // Write extended fields based on format version
        match self.format_version {
            0 | 1 => {
                // No extended fields for v1
            }
            2 => {
                // Write v2 fields
                writer.write_all(&self.ext_block_table_offset_high.to_le_bytes())?;
                writer.write_all(&self.hash_table_offset_high.to_le_bytes())?;
                writer.write_all(&self.block_table_offset_high.to_le_bytes())?;

                // Write padding to reach full v2 header size
                let padding_size =
                    MpqHeaderSize::V2Size as usize - MpqHeaderSize::V1Size as usize - 6;
                writer.write_all(&vec![0u8; padding_size])?;
            }
            3 => {
                // Write v2 and v3 fields
                // ... (similar to above, with appropriate fields and padding)
            }
            4 => {
                // Write v2, v3, and v4 fields
                // ... (similar to above, with appropriate fields and padding)
            }
            _ => return Err(HeaderError::UnsupportedVersion(self.format_version)),
        }

        Ok(())
    }

    /// Validates the MPQ header
    pub fn validate(&self) -> Result<(), HeaderError> {
        // Check signature
        if self.signature != MPQ_SIGNATURE {
            return Err(HeaderError::InvalidSignature);
        }

        // Check header size based on format version
        let expected_size = match self.format_version {
            0 | 1 => MpqHeaderSize::V1Size as u32,
            2 => MpqHeaderSize::V2Size as u32,
            3 => MpqHeaderSize::V3Size as u32,
            4 => MpqHeaderSize::V4Size as u32,
            _ => return Err(HeaderError::UnsupportedVersion(self.format_version)),
        };

        if self.header_size < expected_size {
            return Err(HeaderError::InvalidHeaderSize {
                expected: expected_size,
                actual: self.header_size,
            });
        }

        // Validate sector size (must be a power of 2 and at least 512 bytes)
        if self.sector_size_shift < 1 {
            return Err(HeaderError::InvalidSectorSizeShift(self.sector_size_shift));
        }

        // Check if hash and block table offsets are valid
        if self.format_version < 2 {
            // For v1, offsets should be within the 32-bit archive size
            if self.hash_table_offset >= self.archive_size {
                return Err(HeaderError::HashTableOffsetOutOfBounds);
            }

            if self.block_table_offset >= self.archive_size {
                return Err(HeaderError::BlockTableOffsetOutOfBounds);
            }
        } else {
            // For v2+, check 64-bit offsets
            let archive_size = self.archive_size_64();

            if self.hash_table_offset_64() >= archive_size {
                return Err(HeaderError::HashTableOffsetOutOfBounds);
            }

            if self.block_table_offset_64() >= archive_size {
                return Err(HeaderError::BlockTableOffsetOutOfBounds);
            }
        }

        // Validate table entries (must be reasonable numbers)
        // We set arbitrary limits here to catch obviously corrupted values
        const MAX_REASONABLE_TABLE_SIZE: u32 = 1_000_000;

        if self.hash_table_entries > MAX_REASONABLE_TABLE_SIZE {
            return Err(HeaderError::UnreasonableHashTableSize(
                self.hash_table_entries,
            ));
        }

        if self.block_table_entries > MAX_REASONABLE_TABLE_SIZE {
            return Err(HeaderError::UnreasonableBlockTableSize(
                self.block_table_entries,
            ));
        }

        Ok(())
    }

    /// Search for an MPQ header or user data header within a file
    ///
    /// This function searches for either:
    /// 1. A standard MPQ header with signature 0x1A51504D
    /// 2. A user data header with signature 0x1B51504D that points to the MPQ header
    ///
    /// The search is performed at 512-byte boundaries.
    /// Returns the MPQ header and its offset within the file.
    pub fn find_and_read<R: Read + Seek>(
        reader: &mut R,
        search_limit: Option<u64>,
    ) -> Result<(Self, u64), HeaderError> {
        // Default search limit is 512KB
        let limit = search_limit.unwrap_or(512 * 1024);

        // Start at the beginning of the file
        reader.seek(SeekFrom::Start(0))?;

        // Check file size
        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        // We'll search at 512-byte boundaries
        let mut position = 0;
        let mut signature_buffer = [0u8; 4];

        while position < limit && position < file_size {
            // Seek to the current 512-byte boundary
            reader.seek(SeekFrom::Start(position))?;

            // Read the potential signature
            if reader.read_exact(&mut signature_buffer).is_err() {
                // Reached end of file
                break;
            }

            let signature = u32::from_le_bytes(signature_buffer);

            match signature {
                // Found standard MPQ header
                MPQ_SIGNATURE => {
                    // Go back to start of signature
                    reader.seek(SeekFrom::Start(position))?;

                    // Try to read the header
                    match Self::read_from(reader) {
                        Ok(header) => {
                            // Validate the header
                            if header.validate().is_ok() {
                                return Ok((header, position));
                            }
                        }
                        Err(_) => {
                            // Invalid header, continue search
                        }
                    }
                }

                // Found user data header
                MPQ_USER_DATA_SIGNATURE => {
                    // Go back to start of signature
                    reader.seek(SeekFrom::Start(position))?;

                    // Try to read the user data header
                    match MpqUserDataHeader::read_from(reader) {
                        Ok(user_data_header) => {
                            // Calculate the offset to the MPQ header
                            let mpq_header_offset =
                                position + user_data_header.mpq_header_offset as u64;

                            // Seek to the MPQ header
                            reader.seek(SeekFrom::Start(mpq_header_offset))?;

                            // Read the MPQ header
                            match Self::read_from(reader) {
                                Ok(header) => {
                                    // Validate the header
                                    if header.validate().is_ok() {
                                        return Ok((header, mpq_header_offset));
                                    }
                                }
                                Err(_) => {
                                    // Invalid header, continue search
                                }
                            }
                        }
                        Err(_) => {
                            // Invalid user data header, continue search
                        }
                    }
                }

                // Not an MPQ signature, continue search
                _ => {}
            }

            // Move to the next 512-byte boundary
            position += 512;
        }

        // No valid MPQ header found
        Err(HeaderError::HeaderNotFound)
    }
}

/// MPQ User Data Header structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MpqUserDataHeader {
    /// Signature, should be "MPQ\x1B" (0x1B51504D)
    pub signature: u32,
    /// Size of user data that follows this header
    pub user_data_size: u32,
    /// Offset to the MPQ header, relative to the beginning of this user data header
    pub mpq_header_offset: u32,
}

impl MpqUserDataHeader {
    /// Read a user data header from a reader
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self, HeaderError> {
        let mut header = Self {
            signature: 0,
            user_data_size: 0,
            mpq_header_offset: 0,
        };

        // Read the user data header fields (12 bytes total)
        let mut buffer = [0u8; 12];
        reader.read_exact(&mut buffer)?;

        // Parse fields
        header.signature = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

        // Validate signature
        if header.signature != MPQ_USER_DATA_SIGNATURE {
            return Err(HeaderError::InvalidSignature);
        }

        header.user_data_size = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        header.mpq_header_offset =
            u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);

        Ok(header)
    }
}
