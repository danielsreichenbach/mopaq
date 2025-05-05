use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Magic number used to identify MPQ archives: "MPQ\x1A"
pub const MPQ_HEADER_MAGIC: u32 = 0x1A51504D;

/// Magic number for user header: "MPQ\x1B"
pub const MPQ_USER_DATA_MAGIC: u32 = 0x1B51504D;

/// MPQ format version 0 header size (used in vanilla WoW)
pub const MPQ_HEADER_SIZE_V0: u32 = 32;

/// MPQ format version 1 header size (used in "The Burning Crusade" expansion)
pub const MPQ_HEADER_SIZE_V1: u32 = 44;

/// MPQ format version 2 header size
pub const MPQ_HEADER_SIZE_V2: u32 = 68;

/// MPQ format version 3 header size
pub const MPQ_HEADER_SIZE_V3: u32 = 72;

/// MPQ format version 4 header size
pub const MPQ_HEADER_SIZE_V4: u32 = 80;

/// Structure representing an MPQ archive header
#[derive(Debug, Clone)]
pub struct MpqHeader {
    /// 'MPQ\x1A' signature
    pub magic: u32,

    /// Size of the header in bytes
    pub header_size: u32,

    /// Size of the archive in bytes
    pub archive_size: u32,

    /// MPQ format version
    pub format_version: u16,

    /// Sector size shift (power of 2)
    pub sector_size_shift: u16,

    /// Offset to the hash table
    pub hash_table_offset: u32,

    /// Offset to the block table
    pub block_table_offset: u32,

    /// Number of entries in the hash table
    pub hash_table_entries: u32,

    /// Number of entries in the block table
    pub block_table_entries: u32,

    // Fields below are only present in format version 1 and above
    /// High bits of the hash table offset (v1+)
    pub hash_table_offset_high: Option<u32>,

    /// High bits of the block table offset (v1+)
    pub block_table_offset_high: Option<u32>,

    // Fields below are only present in format version 2 and above
    /// Offset to the extended block table (v2+)
    pub extended_block_table_offset: Option<u64>,

    /// High bits of the hash table entries (v2+)
    pub hash_table_entries_high: Option<u16>,

    /// High bits of the block table entries (v2+)
    pub block_table_entries_high: Option<u16>,

    // Fields below are only present in format version 3 and above
    /// 64-bit archive size (v3+)
    pub archive_size_64: Option<u64>,

    // Fields below are only present in format version 4 and above
    /// Offset to the BET table (v4+)
    pub bet_table_offset: Option<u64>,

    /// Offset to the HET table (v4+)
    pub het_table_offset: Option<u64>,
}

/// Structure representing an MPQ user data header
#[derive(Debug, Clone)]
pub struct MpqUserDataHeader {
    /// 'MPQ\x1B' signature
    pub magic: u32,

    /// Size of the user data
    pub user_data_size: u32,

    /// Size of the MPQ archive header
    pub header_offset: u32,

    /// Size of the user data header
    pub user_data_header_size: u32,
}

impl MpqHeader {
    /// Creates a new MPQ header with default values
    pub fn new(format_version: u16) -> Self {
        let header_size = match format_version {
            0 => MPQ_HEADER_SIZE_V0,
            1 => MPQ_HEADER_SIZE_V1,
            2 => MPQ_HEADER_SIZE_V2,
            3 => MPQ_HEADER_SIZE_V3,
            4 => MPQ_HEADER_SIZE_V4,
            _ => MPQ_HEADER_SIZE_V0, // Default to v0 if unknown
        };

        MpqHeader {
            magic: MPQ_HEADER_MAGIC,
            header_size,
            archive_size: 0,
            format_version,
            sector_size_shift: 3, // Default sector size: 2^3 = 8 KB
            hash_table_offset: 0,
            block_table_offset: 0,
            hash_table_entries: 0,
            block_table_entries: 0,
            hash_table_offset_high: if format_version >= 1 { Some(0) } else { None },
            block_table_offset_high: if format_version >= 1 { Some(0) } else { None },
            extended_block_table_offset: if format_version >= 2 { Some(0) } else { None },
            hash_table_entries_high: if format_version >= 2 { Some(0) } else { None },
            block_table_entries_high: if format_version >= 2 { Some(0) } else { None },
            archive_size_64: if format_version >= 3 { Some(0) } else { None },
            bet_table_offset: if format_version >= 4 { Some(0) } else { None },
            het_table_offset: if format_version >= 4 { Some(0) } else { None },
        }
    }

    /// Read an MPQ header from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?;

        if magic != MPQ_HEADER_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid MPQ header magic: 0x{:08X}", magic),
            ));
        }

        let header_size = reader.read_u32::<LittleEndian>()?;
        let archive_size = reader.read_u32::<LittleEndian>()?;
        let format_version = reader.read_u16::<LittleEndian>()?;
        let sector_size_shift = reader.read_u16::<LittleEndian>()?;
        let hash_table_offset = reader.read_u32::<LittleEndian>()?;
        let block_table_offset = reader.read_u32::<LittleEndian>()?;
        let hash_table_entries = reader.read_u32::<LittleEndian>()?;
        let block_table_entries = reader.read_u32::<LittleEndian>()?;

        // Read version 1+ fields if present
        let (hash_table_offset_high, block_table_offset_high) =
            if header_size >= MPQ_HEADER_SIZE_V1 && format_version >= 1 {
                (
                    Some(reader.read_u32::<LittleEndian>()?),
                    Some(reader.read_u32::<LittleEndian>()?),
                )
            } else {
                (None, None)
            };

        // Read version 2+ fields if present
        let (extended_block_table_offset, hash_table_entries_high, block_table_entries_high) =
            if header_size >= MPQ_HEADER_SIZE_V2 && format_version >= 2 {
                let hi = reader.read_u16::<LittleEndian>()?;
                let lo = reader.read_u16::<LittleEndian>()?;
                let extended_offset = ((hi as u64) << 32) | (lo as u64);

                (
                    Some(extended_offset),
                    Some(reader.read_u16::<LittleEndian>()?),
                    Some(reader.read_u16::<LittleEndian>()?),
                )
            } else {
                (None, None, None)
            };

        // Read version 3+ fields if present
        let archive_size_64 = if header_size >= MPQ_HEADER_SIZE_V3 && format_version >= 3 {
            let hi = reader.read_u32::<LittleEndian>()?;
            let lo = reader.read_u32::<LittleEndian>()?;
            Some(((hi as u64) << 32) | (lo as u64))
        } else {
            None
        };

        // Read version 4+ fields if present
        let (bet_table_offset, het_table_offset) =
            if header_size >= MPQ_HEADER_SIZE_V4 && format_version >= 4 {
                (
                    Some(reader.read_u64::<LittleEndian>()?),
                    Some(reader.read_u64::<LittleEndian>()?),
                )
            } else {
                (None, None)
            };

        Ok(MpqHeader {
            magic,
            header_size,
            archive_size,
            format_version,
            sector_size_shift,
            hash_table_offset,
            block_table_offset,
            hash_table_entries,
            block_table_entries,
            hash_table_offset_high,
            block_table_offset_high,
            extended_block_table_offset,
            hash_table_entries_high,
            block_table_entries_high,
            archive_size_64,
            bet_table_offset,
            het_table_offset,
        })
    }

    /// Write the MPQ header to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<LittleEndian>(self.magic)?;
        writer.write_u32::<LittleEndian>(self.header_size)?;
        writer.write_u32::<LittleEndian>(self.archive_size)?;
        writer.write_u16::<LittleEndian>(self.format_version)?;
        writer.write_u16::<LittleEndian>(self.sector_size_shift)?;
        writer.write_u32::<LittleEndian>(self.hash_table_offset)?;
        writer.write_u32::<LittleEndian>(self.block_table_offset)?;
        writer.write_u32::<LittleEndian>(self.hash_table_entries)?;
        writer.write_u32::<LittleEndian>(self.block_table_entries)?;

        // Write version 1+ fields if present
        if self.format_version >= 1 {
            writer.write_u32::<LittleEndian>(self.hash_table_offset_high.unwrap_or(0))?;
            writer.write_u32::<LittleEndian>(self.block_table_offset_high.unwrap_or(0))?;
        }

        // Write version 2+ fields if present
        if self.format_version >= 2 {
            let extended_offset = self.extended_block_table_offset.unwrap_or(0);
            writer.write_u16::<LittleEndian>((extended_offset >> 32) as u16)?;
            writer.write_u16::<LittleEndian>(extended_offset as u16)?;
            writer.write_u16::<LittleEndian>(self.hash_table_entries_high.unwrap_or(0))?;
            writer.write_u16::<LittleEndian>(self.block_table_entries_high.unwrap_or(0))?;
        }

        // Write version 3+ fields if present
        if self.format_version >= 3 {
            let archive_size_64 = self.archive_size_64.unwrap_or(0);
            writer.write_u32::<LittleEndian>((archive_size_64 >> 32) as u32)?;
            writer.write_u32::<LittleEndian>(archive_size_64 as u32)?;
        }

        // Write version 4+ fields if present
        if self.format_version >= 4 {
            writer.write_u64::<LittleEndian>(self.bet_table_offset.unwrap_or(0))?;
            writer.write_u64::<LittleEndian>(self.het_table_offset.unwrap_or(0))?;
        }

        Ok(())
    }
}

impl MpqUserDataHeader {
    /// Creates a new MPQ user data header
    pub fn new(user_data_size: u32, header_offset: u32) -> Self {
        MpqUserDataHeader {
            magic: MPQ_USER_DATA_MAGIC,
            user_data_size,
            header_offset,
            user_data_header_size: 16, // Fixed size for user data header
        }
    }

    /// Read an MPQ user data header from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?;

        if magic != MPQ_USER_DATA_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid MPQ user data magic: 0x{:08X}", magic),
            ));
        }

        let user_data_size = reader.read_u32::<LittleEndian>()?;
        let header_offset = reader.read_u32::<LittleEndian>()?;
        let user_data_header_size = reader.read_u32::<LittleEndian>()?;

        Ok(MpqUserDataHeader {
            magic,
            user_data_size,
            header_offset,
            user_data_header_size,
        })
    }

    /// Write the MPQ user data header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u32::<LittleEndian>(self.magic)?;
        writer.write_u32::<LittleEndian>(self.user_data_size)?;
        writer.write_u32::<LittleEndian>(self.header_offset)?;
        writer.write_u32::<LittleEndian>(self.user_data_header_size)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mpq_header_read_write() {
        // Create a header
        let header = MpqHeader::new(1);

        // Write it to a buffer
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        header.write(&mut cursor).unwrap();

        // Read it back
        cursor.set_position(0);
        let read_header = MpqHeader::read(&mut cursor).unwrap();

        // Check that the header was read correctly
        assert_eq!(read_header.magic, MPQ_HEADER_MAGIC);
        assert_eq!(read_header.header_size, MPQ_HEADER_SIZE_V1);
        assert_eq!(read_header.format_version, 1);
    }

    #[test]
    fn test_mpq_user_header_read_write() {
        // Create a user header
        let user_header = MpqUserDataHeader::new(1024, 0x200);

        // Write it to a buffer
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        user_header.write(&mut cursor).unwrap();

        // Read it back
        cursor.set_position(0);
        let read_user_header = MpqUserDataHeader::read(&mut cursor).unwrap();

        // Check that the user header was read correctly
        assert_eq!(read_user_header.magic, MPQ_USER_DATA_MAGIC);
        assert_eq!(read_user_header.user_data_size, 1024);
        assert_eq!(read_user_header.header_offset, 0x200);
        assert_eq!(read_user_header.user_data_header_size, 16);
    }

    #[test]
    fn test_invalid_header_magic() {
        // Create a buffer with an invalid magic number
        let mut buffer = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut cursor = Cursor::new(&buffer);

        // Try to read a header
        let result = MpqHeader::read(&mut cursor);

        // Check that it fails
        assert!(result.is_err());
    }
}
