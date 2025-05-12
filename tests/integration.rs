//! Integration tests

#[cfg(test)]
mod tests {
    use mopaq::header::*;
    use std::io::Cursor;

    // Helper function to create a test MPQ v1 header
    fn create_test_header_v1() -> Vec<u8> {
        let mut header = vec![0u8; MpqHeaderSize::V1Size as usize];

        // Set signature "MPQ\x1A"
        header[0..4].copy_from_slice(&MPQ_SIGNATURE.to_le_bytes());

        // Header size (32 bytes for v1)
        header[4..8].copy_from_slice(&(MpqHeaderSize::V1Size as u32).to_le_bytes());

        // Archive size (1MB for test)
        header[8..12].copy_from_slice(&(1024 * 1024u32).to_le_bytes());

        // Format version (1)
        header[12..14].copy_from_slice(&(1u16).to_le_bytes());

        // Sector size shift (3 for 4KB sectors)
        header[14..16].copy_from_slice(&(3u16).to_le_bytes());

        // Hash table offset (1000)
        header[16..20].copy_from_slice(&(1000u32).to_le_bytes());

        // Block table offset (2000)
        header[20..24].copy_from_slice(&(2000u32).to_le_bytes());

        // Hash table entries (10)
        header[24..28].copy_from_slice(&(10u32).to_le_bytes());

        // Block table entries (10)
        header[28..32].copy_from_slice(&(10u32).to_le_bytes());

        header
    }

    #[test]
    fn test_read_v1_header() {
        let header_data = create_test_header_v1();
        let mut cursor = Cursor::new(header_data);

        let header = MpqHeader::read_from(&mut cursor).expect("Failed to read header");

        assert_eq!(header.signature, MPQ_SIGNATURE);
        assert_eq!(header.header_size, MpqHeaderSize::V1Size as u32);
        assert_eq!(header.archive_size, 1024 * 1024);
        assert_eq!(header.format_version, 1);
        assert_eq!(header.sector_size_shift, 3);
        assert_eq!(header.hash_table_offset, 1000);
        assert_eq!(header.block_table_offset, 2000);
        assert_eq!(header.hash_table_entries, 10);
        assert_eq!(header.block_table_entries, 10);

        // Derived values
        assert_eq!(header.sector_size(), 4096);
        assert_eq!(header.hash_table_offset_64(), 1000);
        assert_eq!(header.block_table_offset_64(), 2000);
        assert_eq!(header.archive_size_64(), 1024 * 1024);
    }

    #[test]
    fn test_invalid_signature() {
        let mut header_data = create_test_header_v1();
        // Corrupt the signature
        header_data[0] = 0x00;

        let mut cursor = Cursor::new(header_data);

        let result = MpqHeader::read_from(&mut cursor);
        assert!(result.is_err());

        if let Err(HeaderError::InvalidSignature) = result {
            // Expected error
        } else {
            panic!("Expected InvalidSignature error");
        }
    }

    #[test]
    fn test_validate_header() {
        let header_data = create_test_header_v1();
        let mut cursor = Cursor::new(header_data);

        let header = MpqHeader::read_from(&mut cursor).expect("Failed to read header");

        // Should validate successfully
        assert!(header.validate().is_ok());

        // Test invalid header
        let mut invalid_header = header;
        invalid_header.hash_table_offset = 2_000_000; // Beyond archive size

        assert!(invalid_header.validate().is_err());
    }

    /// Create a test user data header
    fn create_test_user_data_header(mpq_header_offset: u32) -> Vec<u8> {
        let mut header = vec![0u8; 12]; // Correct size is 12 bytes

        // Set signature "MPQ\x1B"
        header[0..4].copy_from_slice(&MPQ_USER_DATA_SIGNATURE.to_le_bytes());

        // User data size (100 bytes)
        header[4..8].copy_from_slice(&(100u32).to_le_bytes());

        // MPQ header offset
        header[8..12].copy_from_slice(&mpq_header_offset.to_le_bytes());

        header
    }

    #[test]
    fn test_read_user_data_header() {
        let header_data = create_test_user_data_header(512);
        let mut cursor = Cursor::new(header_data);

        let user_data_header =
            MpqUserDataHeader::read_from(&mut cursor).expect("Failed to read user data header");

        assert_eq!(user_data_header.signature, MPQ_USER_DATA_SIGNATURE);
        assert_eq!(user_data_header.user_data_size, 100);
        assert_eq!(user_data_header.mpq_header_offset, 512);
    }

    #[test]
    fn test_find_mpq_header_with_user_data() {
        // Create a file with:
        // 1. User data header at offset 0
        // 2. Some dummy user data
        // 3. MPQ header at offset 512

        let mpq_header_offset = 512;
        let user_data_header = create_test_user_data_header(mpq_header_offset);
        let mpq_header = create_test_header_v1();

        // Create the file content
        let mut file_content = vec![0u8; 1024];

        // Place user data header at the beginning
        file_content[0..12].copy_from_slice(&user_data_header); // Corrected size

        // Fill user data section with dummy data (0xAA)
        for i in 12..mpq_header_offset as usize {
            file_content[i] = 0xAA;
        }

        // Place MPQ header at offset 512
        file_content[mpq_header_offset as usize..(mpq_header_offset as usize + mpq_header.len())]
            .copy_from_slice(&mpq_header);

        let mut cursor = Cursor::new(file_content);

        // Find and read the MPQ header
        let (header, offset) = MpqHeader::find_and_read(&mut cursor, None)
            .expect("Failed to find and read MPQ header");

        // Verify we found the correct header at the right offset
        assert_eq!(offset, mpq_header_offset as u64);
        assert_eq!(header.signature, MPQ_SIGNATURE);
    }

    #[test]
    fn test_find_mpq_header_at_512_boundary() {
        // Create a file with:
        // 1. 512 bytes of random data
        // 2. MPQ header at offset 512

        let mpq_header = create_test_header_v1();

        // Create the file content
        let mut file_content = vec![0u8; 1024];

        // Fill first 512 bytes with dummy data (0xAA)
        for i in 0..512 {
            file_content[i] = 0xAA;
        }

        // Place MPQ header at offset 512
        file_content[512..(512 + mpq_header.len())].copy_from_slice(&mpq_header);

        let mut cursor = Cursor::new(file_content);

        // Find and read the MPQ header
        let (header, offset) = MpqHeader::find_and_read(&mut cursor, None)
            .expect("Failed to find and read MPQ header");

        // Verify we found the correct header at the right offset
        assert_eq!(offset, 512);
        assert_eq!(header.signature, MPQ_SIGNATURE);
    }

    #[test]
    fn test_no_header_found() {
        // Create a file with just random data, no MPQ headers
        let file_content = vec![0xAA; 1024];
        let mut cursor = Cursor::new(file_content);

        // Try to find an MPQ header
        let result = MpqHeader::find_and_read(&mut cursor, None);

        // Should return HeaderNotFound error
        assert!(matches!(result, Err(HeaderError::HeaderNotFound)));
    }
}
