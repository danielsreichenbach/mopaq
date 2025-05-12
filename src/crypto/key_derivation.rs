//! Key generation from filename/position

//! Key derivation functions for MPQ archives

use super::CryptoError;
use super::CryptoResult;
use super::hash::{HashType, hash_string};

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
    // File key is just the hash with HashType::FileKey
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
    // Exact StormLib formula for sector key generation
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
    // Formula from StormLib for generating a key from a file offset
    let adjusted_offset = file_offset / 0x1000;
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
    // The first decrypted DWORD should be equal to 'MPQ\x1A'
    let expected_header = 0x1A51504D; // 'MPQ\x1A' in little-endian

    // Try filename-based keys first
    for &filename in known_filenames {
        let key = generate_file_key(filename);
        if verify_encryption_key(encryption_header, key, expected_header) {
            return Ok(key);
        }
    }

    // Try offset-based key
    let key = generate_key_from_offset(file_offset);
    if verify_encryption_key(encryption_header, key, expected_header) {
        return Ok(key);
    }

    // No valid key found
    Err(CryptoError::InvalidKey(format!(
        "Could not detect encryption key for file at offset {:#x}",
        file_offset
    )))
}

/// Verifies if a key correctly decrypts the encryption header
fn verify_encryption_key(encryption_header: &[u8], key: u32, expected_value: u32) -> bool {
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

    if signature_value != expected_value {
        return false;
    }

    // Try to decrypt using the key
    let mut seed = 0xEEEEEEEEu32;
    seed = seed
        .wrapping_add(super::constants::STORM_BUFFER_CRYPT[0x400 + ((key >> 24) & 0xFF) as usize]);

    let encrypted = header_u32[0];
    let decrypted = encrypted ^ (key.wrapping_add(seed));

    // For valid keys in MPQ files, the decrypted value is often 0 or a small value
    // This is a heuristic check that matches StormLib's approach
    (decrypted & 0xFFFFFF00) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_file_key() {
        // Test with known values
        assert_eq!(generate_file_key("(listfile)"), 0xFE1A5969);
        assert_eq!(generate_file_key("(attributes)"), 0x9473FB7C);
        assert_eq!(generate_file_key("war3map.j"), 0xA11F1C6A);
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
        // Test with known values
        assert_eq!(generate_key_from_offset(0x00000000), 0x00000000);
        assert_eq!(generate_key_from_offset(0x00001000), 0x00000001);
        assert_eq!(generate_key_from_offset(0x00012000), 0x00010012);
        assert_eq!(generate_key_from_offset(0x00123000), 0x00120123);
        assert_eq!(generate_key_from_offset(0x01234000), 0x01230234);
    }

    #[test]
    fn test_detect_file_key() {
        // Create a mock encryption header:
        // - First 4 bytes are encrypted with a known key
        // - Second 4 bytes are the MPQ signature

        // Use the key for "(listfile)"
        let key = generate_file_key("(listfile)");

        // Create a header with first 4 bytes set to zeros
        let mut header = [0x00, 0x00, 0x00, 0x00, 0x4D, 0x50, 0x51, 0x1A]; // Second part is 'MPQ\x1A'

        // Encrypt the first 4 bytes
        let header_u32 =
            unsafe { std::slice::from_raw_parts_mut(header.as_mut_ptr() as *mut u32, 2) };

        let mut seed = 0xEEEEEEEEu32;
        let mut encrypt_key = key;

        seed = seed.wrapping_add(
            super::super::constants::STORM_BUFFER_CRYPT
                [0x400 + ((encrypt_key >> 24) & 0xFF) as usize],
        );

        let plain = header_u32[0];
        header_u32[0] = plain ^ (encrypt_key.wrapping_add(seed));

        // Now try to detect the key
        let known_filenames = ["unknown.txt", "(listfile)", "something.txt"];

        let detected_key =
            detect_file_key(&header, 0, &known_filenames).expect("Failed to detect key");

        // Should match the original key
        assert_eq!(detected_key, key);

        // Test with no matching keys
        let known_filenames = ["unknown.txt", "something.txt"];

        let result = detect_file_key(&header, 0, &known_filenames);
        assert!(result.is_err());
    }
}
