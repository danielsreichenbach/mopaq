use crate::error::{MopaqError, Result};
use crate::header::{MPQ_USER_DATA_SIGNATURE, MpqHeader};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom, Write};

/// MPQ user data header
#[derive(Debug, Clone)]
pub struct MpqUserHeader {
    /// User data signature, must be MPQ\x1B
    pub signature: u32,

    /// Size of the user data
    pub user_data_size: u32,

    /// MPQ header offset
    pub mpq_header_offset: u32,

    /// User data header size
    pub user_header_size: u32,
}

impl MpqUserHeader {
    /// Create a new user header with default values
    pub fn new() -> Self {
        Self {
            signature: MPQ_USER_DATA_SIGNATURE,
            user_data_size: 0,
            mpq_header_offset: 0,
            user_header_size: 16, // Size of the user header structure
        }
    }

    /// Read a user header from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Remember the current position
        let start_pos = reader.seek(SeekFrom::Current(0))?;

        // Read the signature
        let signature = reader.read_u32::<LittleEndian>()?;
        if signature != MPQ_USER_DATA_SIGNATURE {
            // If not a user header, seek back to start
            reader.seek(SeekFrom::Start(start_pos))?;
            return Err(MopaqError::InvalidSignature);
        }

        // Read the user header fields
        let user_data_size = reader.read_u32::<LittleEndian>()?;
        let mpq_header_offset = reader.read_u32::<LittleEndian>()?;
        let user_header_size = reader.read_u32::<LittleEndian>()?;

        Ok(MpqUserHeader {
            signature,
            user_data_size,
            mpq_header_offset,
            user_header_size,
        })
    }

    /// Write the user header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.signature)?;
        writer.write_u32::<LittleEndian>(self.user_data_size)?;
        writer.write_u32::<LittleEndian>(self.mpq_header_offset)?;
        writer.write_u32::<LittleEndian>(self.user_header_size)?;
        Ok(())
    }
}

/// Helper function to read an MPQ header from a file, handling user headers
pub fn read_mpq_header<R: Read + Seek>(
    reader: &mut R,
) -> Result<(Option<MpqUserHeader>, MpqHeader)> {
    // Remember the current position
    let start_pos = reader.seek(SeekFrom::Current(0))?;

    // Try to read a user header first
    match MpqUserHeader::read(reader) {
        Ok(user_header) => {
            // We found a user header, so now seek to the MPQ header
            reader.seek(SeekFrom::Start(
                start_pos + user_header.mpq_header_offset as u64,
            ))?;

            // Read the actual MPQ header
            let mpq_header = MpqHeader::read(reader)?;

            Ok((Some(user_header), mpq_header))
        }
        Err(MopaqError::InvalidSignature) => {
            // No user header, try to read MPQ header directly from the current position
            reader.seek(SeekFrom::Start(start_pos))?;
            let mpq_header = MpqHeader::read(reader)?;

            Ok((None, mpq_header))
        }
        Err(e) => Err(e),
    }
}

/// Helper function to write an MPQ header to a file, with optional user header
pub fn write_mpq_header<W: Write + Seek>(
    writer: &mut W,
    user_header: Option<&MpqUserHeader>,
    mpq_header: &MpqHeader,
) -> Result<()> {
    match user_header {
        Some(user_header) => {
            // Write the user header
            user_header.write(writer)?;

            // Write user data if any
            if user_header.user_data_size > 0 {
                // In a real implementation, we would write the user data here
                // For now, just write zeros
                for _ in 0..user_header.user_data_size {
                    writer.write_u8(0)?;
                }
            }

            // Seek to the MPQ header position
            let current_pos = writer.seek(SeekFrom::Current(0))?;
            let target_pos = current_pos - user_header.user_data_size as u64 - 16
                + user_header.mpq_header_offset as u64;
            writer.seek(SeekFrom::Start(target_pos))?;

            // Write the MPQ header
            mpq_header.write(writer)?;
        }
        None => {
            // Write the MPQ header directly
            mpq_header.write(writer)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_user_header_roundtrip() {
        // Create a user header
        let mut original = MpqUserHeader::new();
        original.user_data_size = 32;
        original.mpq_header_offset = 48;

        // Create a buffer and write the header
        let mut buffer = Cursor::new(Vec::new());
        original.write(&mut buffer).unwrap();

        // Reset the cursor and read the header back
        buffer.set_position(0);
        let read_back = MpqUserHeader::read(&mut buffer).unwrap();

        // Verify the headers match
        assert_eq!(read_back.signature, original.signature);
        assert_eq!(read_back.user_data_size, original.user_data_size);
        assert_eq!(read_back.mpq_header_offset, original.mpq_header_offset);
        assert_eq!(read_back.user_header_size, original.user_header_size);
    }

    #[test]
    fn test_read_with_user_header() {
        // Create a user header and MPQ header
        let mut user_header = MpqUserHeader::new();
        user_header.user_data_size = 0;
        user_header.mpq_header_offset = 16;

        let mpq_header = MpqHeader::new_v1();

        // Create a buffer and write both headers
        let mut buffer = Cursor::new(Vec::new());
        user_header.write(&mut buffer).unwrap();
        mpq_header.write(&mut buffer).unwrap();

        // Reset the cursor and read the headers back
        buffer.set_position(0);
        let (read_user_header, read_mpq_header) = read_mpq_header(&mut buffer).unwrap();

        // Verify we got a user header
        assert!(read_user_header.is_some());
        let read_user_header = read_user_header.unwrap();

        // Verify the user header matches
        assert_eq!(read_user_header.signature, user_header.signature);
        assert_eq!(read_user_header.user_data_size, user_header.user_data_size);
        assert_eq!(
            read_user_header.mpq_header_offset,
            user_header.mpq_header_offset
        );
        assert_eq!(
            read_user_header.user_header_size,
            user_header.user_header_size
        );

        // Verify the MPQ header matches
        assert_eq!(read_mpq_header.signature, mpq_header.signature);
        assert_eq!(read_mpq_header.header_size, mpq_header.header_size);
    }
}
