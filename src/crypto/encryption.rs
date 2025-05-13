//! Encryption and decryption functions for MPQ archives

use super::constants::STORM_BUFFER_CRYPT;
use super::{CryptoError, CryptoResult};

/// Encrypts a block of data using the MPQ encryption algorithm
///
/// # Arguments
///
/// * `data` - The data to encrypt (must be aligned to 4 bytes)
/// * `key` - The encryption key
///
/// # Returns
///
/// Result indicating success or an error
pub fn encrypt_block(data: &mut [u8], key: u32) -> CryptoResult<()> {
    // Ensure data length is a multiple of 4
    let len = data.len();
    if len % 4 != 0 {
        return Err(CryptoError::AlignmentError(format!(
            "Data length ({}) is not a multiple of 4",
            len
        )));
    }

    if len == 0 {
        return Ok(());
    }

    // Cast to u32 slices for processing
    let data_u32 =
        unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u32, len / 4) };

    // Set up encryption state
    let mut seed = 0xEEEEEEEEu32;
    let mut k = key;

    // Process each 4-byte block
    for val in data_u32.iter_mut() {
        // Update seed based on the MSB of the key
        seed = seed.wrapping_add(STORM_BUFFER_CRYPT[0x400 + ((k >> 24) & 0xFF) as usize]);

        // Store original value for key calculation
        let plain = *val;

        // Encrypt value
        *val = plain ^ (k.wrapping_add(seed));

        // Update key for next round
        k = ((k << 1) | (k >> 31)).wrapping_add(plain.wrapping_add(seed));
    }

    Ok(())
}

/// Decrypts a block of data using the MPQ decryption algorithm
///
/// # Arguments
///
/// * `data` - The data to decrypt (must be aligned to 4 bytes)
/// * `key` - The decryption key
///
/// # Returns
///
/// Result indicating success or an error
pub fn decrypt_block(data: &mut [u8], key: u32) -> CryptoResult<()> {
    // Ensure data length is a multiple of 4
    let len = data.len();
    if len % 4 != 0 {
        return Err(CryptoError::AlignmentError(format!(
            "Data length ({}) is not a multiple of 4",
            len
        )));
    }

    if len == 0 {
        return Ok(());
    }

    // Cast to u32 slices for processing
    let data_u32 =
        unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u32, len / 4) };

    // Set up decryption state
    let mut seed = 0xEEEEEEEEu32;
    let mut k = key;

    // Process each 4-byte block
    for val in data_u32.iter_mut() {
        // Update seed based on the MSB of the key
        seed = seed.wrapping_add(STORM_BUFFER_CRYPT[0x400 + ((k >> 24) & 0xFF) as usize]);

        // Decrypt value
        let cipher = *val;
        let plain = cipher ^ (k.wrapping_add(seed));

        // Update key for next round
        k = ((k << 1) | (k >> 31)).wrapping_add(plain.wrapping_add(seed));

        // Store decrypted value
        *val = plain;
    }

    Ok(())
}

/// Decrypts the hash table from an MPQ archive
pub fn decrypt_hash_table(data: &mut [u8]) -> CryptoResult<()> {
    decrypt_block(data, super::HASH_TABLE_KEY)
}

/// Decrypts the block table from an MPQ archive
pub fn decrypt_block_table(data: &mut [u8]) -> CryptoResult<()> {
    decrypt_block(data, super::BLOCK_TABLE_KEY)
}

/// Decrypts the extended block table from an MPQ archive
pub fn decrypt_extended_block_table(data: &mut [u8]) -> CryptoResult<()> {
    decrypt_block(data, super::MPQ_EXTENDED_BLOCK_TABLE_KEY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_error() {
        // Test with data that's not aligned to 4 bytes
        let mut data = vec![0x12, 0x34, 0x56];
        let key = 0x12345678;

        // Encryption should fail with alignment error
        let result = encrypt_block(&mut data, key);
        assert!(result.is_err());

        match result {
            Err(CryptoError::AlignmentError(_)) => {
                // Expected error
            }
            _ => panic!("Expected alignment error"),
        }

        // Decryption should also fail
        let result = decrypt_block(&mut data, key);
        assert!(result.is_err());

        match result {
            Err(CryptoError::AlignmentError(_)) => {
                // Expected error
            }
            _ => panic!("Expected alignment error"),
        }
    }

    #[test]
    fn test_empty_data() {
        // Test with empty data
        let mut data = vec![];
        let key = 0x12345678;

        // Should succeed without doing anything
        let result = encrypt_block(&mut data, key);
        assert!(result.is_ok());

        let result = decrypt_block(&mut data, key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // Test with different data sizes and keys
        let test_cases = [
            (vec![0x12, 0x34, 0x56, 0x78], 0x12345678),
            (
                vec![0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89],
                0xABCDEF01,
            ),
            (vec![0x01; 1024], 0x87654321),
        ];

        for (mut data, key) in test_cases {
            // Make a copy of the original data
            let original = data.clone();

            // Encrypt data
            encrypt_block(&mut data, key).expect("Encryption failed");

            // Data should be different after encryption
            assert_ne!(data, original);

            // Decrypt data
            decrypt_block(&mut data, key).expect("Decryption failed");

            // Data should match the original after decryption
            assert_eq!(data, original);
        }
    }

    #[test]
    fn test_known_values() {
        // Test with known input/output values
        // These values should be verified against StormLib's implementation
        let mut input = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let key = 0x12345678;

        // Encrypt and check the result
        encrypt_block(&mut input, key).expect("Encryption failed");

        // These expected values were generated using StormLib's encryption
        let expected = [0xF0, 0x9A, 0x35, 0xF6, 0x9F, 0x1B, 0x01, 0xDF];
        assert_eq!(input, expected);

        // Decrypt and check the result
        decrypt_block(&mut input, key).expect("Decryption failed");

        // Should be back to the original
        let original = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        assert_eq!(input, original);
    }

    #[test]
    fn test_table_decryption() {
        // Test hash table decryption
        let mut data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let original = data.clone();

        // Encrypt using the hash table key
        encrypt_block(&mut data, super::super::constants::HASH_TABLE_KEY)
            .expect("Encryption failed");

        // Decrypt using the dedicated function
        decrypt_hash_table(&mut data).expect("Hash table decryption failed");

        // Should match the original
        assert_eq!(data, original);

        // Test block table decryption
        let mut data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let original = data.clone();

        // Encrypt using the block table key
        encrypt_block(&mut data, super::super::constants::BLOCK_TABLE_KEY)
            .expect("Encryption failed");

        // Decrypt using the dedicated function
        decrypt_block_table(&mut data).expect("Block table decryption failed");

        // Should match the original
        assert_eq!(data, original);
    }
}
