//! Key derivation functions for MPQ archives

use super::constants::STORM_BUFFER_CRYPT;
use super::hash::{HashType, hash_string};
use super::{CryptoError, CryptoResult};

/// Generates a file key from a filename
///
/// This key is used for encrypting/decrypting file contents.
///
/// # Arguments
///
/// * `filename` - The name of the file to generate the key for
///
/// # Returns
///
/// The generated file key
pub fn generate_file_key(filename: &str) -> u32 {
    hash_string(filename, HashType::FileKey)
}

/// Generates a key for a specific file sector
///
/// # Arguments
///
/// * `base_key` - The base file key
/// * `offset` - The offset of the sector in the file
///
/// # Returns
///
/// The generated sector key
pub fn generate_sector_key(base_key: u32, offset: u32) -> u32 {
    (base_key + offset) ^ offset
}

/// Generates a file key from the file offset
///
/// This is used when the filename is unknown.
///
/// # Arguments
///
/// * `file_offset` - The offset of the file in the MPQ archive
///
/// # Returns
///
/// The generated file key
pub fn generate_key_from_offset(file_offset: u32) -> u32 {
    // Offset is divided by 0x1000 as per MPQ specification
    let adjusted_offset = file_offset / 0x1000;

    // Algorithm from StormLib: filekey = (adjusted_offset & 0xFFFF) + ((adjusted_offset & 0xFFFF0000) >> 5)
    (adjusted_offset & 0xFFFF) + ((adjusted_offset & 0xFFFF0000) >> 5)
}

/// Attempts to detect the encryption key used for a file
///
/// This function tries various key sources to find the one that properly decrypts
/// the file's encryption header.
///
/// # Arguments
///
/// * `encryption_header` - The first 8 bytes of the file (encryption header)
/// * `file_offset` - The offset of the file in the MPQ archive
/// * `known_filenames` - A list of potential filenames to try
///
/// # Returns
///
/// The detected encryption key if found, or an error if not
pub fn detect_file_key(
    encryption_header: &[u8],
    file_offset: u32,
    known_filenames: &[&str],
) -> CryptoResult<u32> {
    // Ensure the encryption header is at least 8 bytes
    if encryption_header.len() < 8 {
        return Err(CryptoError::BufferSizeError(format!(
            "Encryption header is too small: {} bytes",
            encryption_header.len()
        )));
    }

    // Check if the file is actually encrypted by checking the encryption header
    // A valid MPQ file encryption header has the first 4 bytes encrypted and the next 4 bytes decrypted
    // The first decrypted DWORD should typically be 'MPQ\x1A' or a small number

    // Try filename-based keys first (these are most common)
    for &filename in known_filenames {
        let key = generate_file_key(filename);
        if verify_encryption_key(encryption_header, key) {
            return Ok(key);
        }
    }

    // Try offset-based key
    let key = generate_key_from_offset(file_offset);
    if verify_encryption_key(encryption_header, key) {
        return Ok(key);
    }

    // No valid key found
    Err(CryptoError::InvalidKey(format!(
        "Could not detect encryption key for file at offset {:#x}",
        file_offset
    )))
}

/// Verifies if a key correctly decrypts the encryption header
fn verify_encryption_key(encryption_header: &[u8], key: u32) -> bool {
    // Ensure the encryption header is at least 8 bytes
    if encryption_header.len() < 8 {
        return false;
    }

    // Create a copy of the header to decrypt
    let mut header_copy = [0u8; 8];
    header_copy.copy_from_slice(&encryption_header[0..8]);

    // Cast to u32 slices for processing
    let header_u32 =
        unsafe { std::slice::from_raw_parts_mut(header_copy.as_mut_ptr() as *mut u32, 2) };

    // In MPQ files, the first 4 bytes are encrypted, and the next 4 bytes
    // are the expected signature (typically MPQ\x1A)
    let signature_value = header_u32[1];

    // Some expected values for decrypted headers
    const MPQ_SIGNATURE: u32 = 0x1A51504D; // 'MPQ\x1A' in little-endian

    // If the second DWORD is the MPQ signature, this is a good indicator
    if signature_value == MPQ_SIGNATURE {
        // Try to decrypt using the key
        let mut seed = 0xEEEEEEEEu32;
        seed = seed.wrapping_add(STORM_BUFFER_CRYPT[0x400 + ((key >> 24) & 0xFF) as usize]);

        let encrypted = header_u32[0];
        let decrypted = encrypted ^ (key.wrapping_add(seed));

        // For valid keys in MPQ files, the decrypted value is often 0 or a small value
        // This is a heuristic check
        (decrypted & 0xFFFFFF00) == 0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_file_key() {
        // Test with known filenames
        // These values may need to be adjusted based on your STORM_BUFFER_CRYPT table

        // Check that different filenames produce different keys
        let key1 = generate_file_key("(listfile)");
        let key2 = generate_file_key("(attributes)");
        let key3 = generate_file_key("replay.rep");

        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);

        // Check that keys are consistent
        assert_eq!(generate_file_key("test.txt"), generate_file_key("test.txt"));
    }

    #[test]
    fn test_generate_sector_key() {
        let base_key = 0x12345678;

        // Test with different offsets
        assert_eq!(generate_sector_key(base_key, 0), base_key);
        assert_eq!(
            generate_sector_key(base_key, 0x1000),
            0x12345678 ^ 0x1000 + 0x1000
        );
        assert_eq!(
            generate_sector_key(base_key, 0x2000),
            0x12345678 ^ 0x2000 + 0x2000
        );
    }

    #[test]
    fn test_generate_key_from_offset() {
        // Test with known offsets
        assert_eq!(generate_key_from_offset(0x00000000), 0);
        assert_eq!(generate_key_from_offset(0x00001000), 1);
        assert_eq!(generate_key_from_offset(0x00010000), 0x800);
        assert_eq!(generate_key_from_offset(0x00011000), 0x801);
    }

    #[test]
    fn test_detect_file_key() {
        // This is a more complex test that would need a proper header
        // We'll create a simple mock test

        // Define a test key
        let test_key = 0x12345678;

        // Create a mock encryption header that would decrypt properly with the test key
        // In a real MPQ, this would be more complex
        let mut mock_header = [0u8; 8];

        // Set the second DWORD to the MPQ signature
        mock_header[4] = 0x4D; // 'M'
        mock_header[5] = 0x50; // 'P'
        mock_header[6] = 0x51; // 'Q'
        mock_header[7] = 0x1A; // '\x1A'

        // Set the first DWORD to a value that would decrypt to 0 with our test key
        // This is a simplification for the test
        // In a real implementation, you'd need to actually encrypt a value

        // For this test, just check that too-small headers are rejected
        let result = detect_file_key(&mock_header[0..4], 0, &["test.txt"]);
        assert!(result.is_err());

        // For a full test, we'd need to create properly encrypted headers
        // This would require implementing the actual encryption logic first
    }
}
