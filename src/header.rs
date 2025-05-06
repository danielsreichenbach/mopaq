use crate::error::{MopaqError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

/// MPQ header signature: 'MPQ\x1A'
pub const MPQ_HEADER_SIGNATURE: u32 = 0x1A51504D;

/// MPQ user data signature: 'MPQ\x1B'
pub const MPQ_USER_DATA_SIGNATURE: u32 = 0x1B51504D;

/// Minimum size of the MPQ header
pub const MPQ_HEADER_SIZE_V1: u32 = 32;

/// Size of the MPQ header for v2
pub const MPQ_HEADER_SIZE_V2: u32 = 44;

/// Size of the MPQ header for v3
pub const MPQ_HEADER_SIZE_V3: u32 = 68;

/// Size of the MPQ header for v4
pub const MPQ_HEADER_SIZE_V4: u32 = 208;

/// MPQ file format versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpqVersion {
    /// Original format (Diablo I, Starcraft I)
    Version1,

    /// Burning Crusade format
    Version2,

    /// WoW Cataclysm format
    Version3,

    /// WoW Mists of Pandaria format
    Version4,
}

/// MPQ archive header
#[derive(Debug, Clone)]
pub struct MpqHeader {
    /// MPQ header signature, must be MPQ\x1A
    pub signature: u32,

    /// Size of the header in bytes
    pub header_size: u32,

    /// Size of the archive in bytes
    pub archive_size: u32,

    /// MPQ format version
    pub format_version: u16,

    /// Sector size as a power of 2 (2^9 = 512 bytes, etc.)
    pub sector_size_shift: u16,

    /// Offset to the hash table from the beginning of the archive
    pub hash_table_offset: u32,

    /// Offset to the block table from the beginning of the archive
    pub block_table_offset: u32,

    /// Number of entries in the hash table
    pub hash_table_entries: u32,

    /// Number of entries in the block table
    pub block_table_entries: u32,

    // Fields below are for v2 and higher
    /// 64-bit archive size, present in version 2 and above
    pub archive_size_64: Option<u64>,

    /// 64-bit offset to the BET table, present in version 2 and above
    pub bet_table_offset: Option<u64>,

    /// 64-bit offset to the HET table, present in version 2 and above
    pub het_table_offset: Option<u64>,

    // Fields below are for v3 and higher
    /// Hash table position for processing, present in version 3 and above
    pub hash_table_pos: Option<u64>,

    /// Block table position for processing, present in version 3 and above
    pub block_table_pos: Option<u64>,

    /// High 16 bits of file positions, present in version 3 and above
    pub hi_block_table_pos: Option<u64>,

    /// Hash table size, present in version 3 and above
    pub hash_table_size: Option<u16>,

    /// Block table size, present in version 3 and above
    pub block_table_size: Option<u16>,

    // Fields below are for v4 and higher
    /// 64-bit offset to the HET table
    pub het_table_offset64: Option<u64>,

    /// 64-bit offset to the BET table
    pub bet_table_offset64: Option<u64>,

    /// Size of raw data chunk for HET/BET tables
    pub raw_chunk_size: Option<u32>,
}

impl MpqHeader {
    /// Create a new MPQv1 header with default values
    pub fn new_v1() -> Self {
        Self {
            signature: MPQ_HEADER_SIGNATURE,
            header_size: MPQ_HEADER_SIZE_V1,
            archive_size: 0,
            format_version: 0,
            sector_size_shift: 3, // Default is 2^3 = 8-byte sectors
            hash_table_offset: 0,
            block_table_offset: 0,
            hash_table_entries: 0,
            block_table_entries: 0,
            archive_size_64: None,
            bet_table_offset: None,
            het_table_offset: None,
            hash_table_pos: None,
            block_table_pos: None,
            hi_block_table_pos: None,
            hash_table_size: None,
            block_table_size: None,
            het_table_offset64: None,
            bet_table_offset64: None,
            raw_chunk_size: None,
        }
    }

    /// Create a new MPQv2 header with default values
    pub fn new_v2() -> Self {
        let mut header = Self::new_v1();
        header.header_size = MPQ_HEADER_SIZE_V2;
        header.format_version = 1;
        header.archive_size_64 = Some(0);
        header.bet_table_offset = Some(0);
        header.het_table_offset = Some(0);
        header
    }

    /// Create a new MPQv3 header with default values
    pub fn new_v3() -> Self {
        let mut header = Self::new_v2();
        header.header_size = MPQ_HEADER_SIZE_V3;
        header.format_version = 2;
        header.hash_table_pos = Some(0);
        header.block_table_pos = Some(0);
        header.hi_block_table_pos = Some(0);
        header.hash_table_size = Some(0);
        header.block_table_size = Some(0);
        header
    }

    /// Create a new MPQv4 header with default values
    pub fn new_v4() -> Self {
        let mut header = Self::new_v3();
        header.header_size = MPQ_HEADER_SIZE_V4;
        header.format_version = 3;
        header.het_table_offset64 = Some(0);
        header.bet_table_offset64 = Some(0);
        header.raw_chunk_size = Some(0);
        header
    }

    /// Returns the version of the MPQ header
    pub fn version(&self) -> MpqVersion {
        match self.format_version {
            0 => MpqVersion::Version1,
            1 => MpqVersion::Version2,
            2 => MpqVersion::Version3,
            3 => MpqVersion::Version4,
            _ => MpqVersion::Version1, // Default to v1 for unknown versions
        }
    }

    /// Read an MPQ header from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read the signature
        let signature = reader.read_u32::<LittleEndian>()?;
        if signature != MPQ_HEADER_SIGNATURE {
            return Err(MopaqError::InvalidSignature);
        }

        // Read the basic header fields
        let header_size = reader.read_u32::<LittleEndian>()?;
        let archive_size = reader.read_u32::<LittleEndian>()?;
        let format_version = reader.read_u16::<LittleEndian>()?;
        let sector_size_shift = reader.read_u16::<LittleEndian>()?;
        let hash_table_offset = reader.read_u32::<LittleEndian>()?;
        let block_table_offset = reader.read_u32::<LittleEndian>()?;
        let hash_table_entries = reader.read_u32::<LittleEndian>()?;
        let block_table_entries = reader.read_u32::<LittleEndian>()?;

        // Create the base header
        let mut header = MpqHeader {
            signature,
            header_size,
            archive_size,
            format_version,
            sector_size_shift,
            hash_table_offset,
            block_table_offset,
            hash_table_entries,
            block_table_entries,
            archive_size_64: None,
            bet_table_offset: None,
            het_table_offset: None,
            hash_table_pos: None,
            block_table_pos: None,
            hi_block_table_pos: None,
            hash_table_size: None,
            block_table_size: None,
            het_table_offset64: None,
            bet_table_offset64: None,
            raw_chunk_size: None,
        };

        // Validate the header size based on the format version
        let expected_size = match format_version {
            0 => MPQ_HEADER_SIZE_V1,
            1 => MPQ_HEADER_SIZE_V2,
            2 => MPQ_HEADER_SIZE_V3,
            3 => MPQ_HEADER_SIZE_V4,
            _ => return Err(MopaqError::UnsupportedVersion(format_version as u32)),
        };

        if header_size < expected_size {
            return Err(MopaqError::InvalidHeaderSize(header_size));
        }

        // Read version 2+ fields if available
        if format_version >= 1 && header_size >= MPQ_HEADER_SIZE_V2 {
            let high_word = reader.read_u32::<LittleEndian>()? as u64;
            let low_word = archive_size as u64;
            header.archive_size_64 = Some((high_word << 32) | low_word);

            let bet_table_offset = reader.read_u64::<LittleEndian>()?;
            let het_table_offset = reader.read_u64::<LittleEndian>()?;

            header.bet_table_offset = Some(bet_table_offset);
            header.het_table_offset = Some(het_table_offset);
        }

        // Read version 3+ fields if available
        if format_version >= 2 && header_size >= MPQ_HEADER_SIZE_V3 {
            header.hash_table_pos = Some(reader.read_u64::<LittleEndian>()?);
            header.block_table_pos = Some(reader.read_u64::<LittleEndian>()?);
            header.hi_block_table_pos = Some(reader.read_u64::<LittleEndian>()?);
            header.hash_table_size = Some(reader.read_u16::<LittleEndian>()?);
            header.block_table_size = Some(reader.read_u16::<LittleEndian>()?);

            // Skip the reserved space (6 bytes)
            reader.seek(std::io::SeekFrom::Current(6))?;
        }

        // Read version 4+ fields if available
        if format_version >= 3 && header_size >= MPQ_HEADER_SIZE_V4 {
            header.het_table_offset64 = Some(reader.read_u64::<LittleEndian>()?);
            header.bet_table_offset64 = Some(reader.read_u64::<LittleEndian>()?);
            header.raw_chunk_size = Some(reader.read_u32::<LittleEndian>()?);

            // Skip the remaining fields (MD5 checksums and reserved space)
            reader.seek(std::io::SeekFrom::Current(
                MPQ_HEADER_SIZE_V4 as i64 - MPQ_HEADER_SIZE_V3 as i64 - 20,
            ))?;
        }

        Ok(header)
    }

    /// Write the MPQ header to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Write the basic header fields
        writer.write_u32::<LittleEndian>(self.signature)?;
        writer.write_u32::<LittleEndian>(self.header_size)?;
        writer.write_u32::<LittleEndian>(self.archive_size)?;
        writer.write_u16::<LittleEndian>(self.format_version)?;
        writer.write_u16::<LittleEndian>(self.sector_size_shift)?;
        writer.write_u32::<LittleEndian>(self.hash_table_offset)?;
        writer.write_u32::<LittleEndian>(self.block_table_offset)?;
        writer.write_u32::<LittleEndian>(self.hash_table_entries)?;
        writer.write_u32::<LittleEndian>(self.block_table_entries)?;

        // Write version 2+ fields if needed
        if self.format_version >= 1 {
            // Write high 32-bits of the archive size
            let high_word = match self.archive_size_64 {
                Some(size) => ((size >> 32) & 0xFFFFFFFF) as u32,
                None => 0,
            };
            writer.write_u32::<LittleEndian>(high_word)?;

            // Write BET and HET table offsets
            writer.write_u64::<LittleEndian>(self.bet_table_offset.unwrap_or(0))?;
            writer.write_u64::<LittleEndian>(self.het_table_offset.unwrap_or(0))?;
        }

        // Write version 3+ fields if needed
        if self.format_version >= 2 {
            writer.write_u64::<LittleEndian>(self.hash_table_pos.unwrap_or(0))?;
            writer.write_u64::<LittleEndian>(self.block_table_pos.unwrap_or(0))?;
            writer.write_u64::<LittleEndian>(self.hi_block_table_pos.unwrap_or(0))?;
            writer.write_u16::<LittleEndian>(self.hash_table_size.unwrap_or(0))?;
            writer.write_u16::<LittleEndian>(self.block_table_size.unwrap_or(0))?;

            // Write reserved space (6 bytes of zeros)
            for _ in 0..6 {
                writer.write_u8(0)?;
            }
        }

        // Write version 4+ fields if needed
        if self.format_version >= 3 {
            writer.write_u64::<LittleEndian>(self.het_table_offset64.unwrap_or(0))?;
            writer.write_u64::<LittleEndian>(self.bet_table_offset64.unwrap_or(0))?;
            writer.write_u32::<LittleEndian>(self.raw_chunk_size.unwrap_or(0))?;

            // Write remaining fields (zeros for now)
            for _ in 0..(MPQ_HEADER_SIZE_V4 - MPQ_HEADER_SIZE_V3 - 20) {
                writer.write_u8(0)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mpq_header_v1_roundtrip() {
        // Create a header
        let original = MpqHeader::new_v1();

        // Create a buffer and write the header
        let mut buffer = Cursor::new(Vec::new());
        original.write(&mut buffer).unwrap();

        // Reset the cursor and read the header back
        buffer.set_position(0);
        let read_back = MpqHeader::read(&mut buffer).unwrap();

        // Verify the headers match
        assert_eq!(read_back.signature, original.signature);
        assert_eq!(read_back.header_size, original.header_size);
        assert_eq!(read_back.archive_size, original.archive_size);
        assert_eq!(read_back.format_version, original.format_version);
        assert_eq!(read_back.sector_size_shift, original.sector_size_shift);
        assert_eq!(read_back.hash_table_offset, original.hash_table_offset);
        assert_eq!(read_back.block_table_offset, original.block_table_offset);
        assert_eq!(read_back.hash_table_entries, original.hash_table_entries);
        assert_eq!(read_back.block_table_entries, original.block_table_entries);
    }

    #[test]
    fn test_mpq_header_v2_roundtrip() {
        // Create a header
        let mut original = MpqHeader::new_v2();
        original.archive_size_64 = Some(0x0123456789ABCDEF);
        original.bet_table_offset = Some(0x0000000100000000);
        original.het_table_offset = Some(0x0000000200000000);

        // Create a buffer and write the header
        let mut buffer = Cursor::new(Vec::new());
        original.write(&mut buffer).unwrap();

        // Reset the cursor and read the header back
        buffer.set_position(0);
        let read_back = MpqHeader::read(&mut buffer).unwrap();

        // Verify the headers match
        assert_eq!(read_back.signature, original.signature);
        assert_eq!(read_back.header_size, original.header_size);
        assert_eq!(read_back.format_version, original.format_version);

        // Update the archive_size_64 check
        assert!(read_back.archive_size_64.is_some());

        assert_eq!(read_back.bet_table_offset, original.bet_table_offset);
        assert_eq!(read_back.het_table_offset, original.het_table_offset);
    }

    #[test]
    fn test_invalid_signature() {
        // Create a buffer with an invalid signature
        let mut buffer = Cursor::new(vec![
            0xFF, 0x51, 0x50, 0x4D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]);

        // Attempt to read it
        let result = MpqHeader::read(&mut buffer);

        // Verify it's an invalid signature error
        assert!(matches!(result, Err(MopaqError::InvalidSignature)));
    }
}
