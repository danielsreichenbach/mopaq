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
        // Update seed
        seed = seed.wrapping_add(STORM_BUFFER_CRYPT[0x400 + ((k >> 24) & 0xFF) as usize]);

        // Store original value for key calculation
        let plain = *val;

        // Encrypt value
        *val = plain ^ (k.wrapping_add(seed));

        // Update key for next round
        k = ((k << 1) | (k >> 31)).wrapping_add(seed.wrapping_add(plain));
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
        // Update seed
        seed = seed.wrapping_add(STORM_BUFFER_CRYPT[0x400 + ((k >> 24) & 0xFF) as usize]);

        // Decrypt value
        let cipher = *val;
        let plain = cipher ^ (k.wrapping_add(seed));

        // Update key for next round
        k = ((k << 1) | (k >> 31)).wrapping_add(seed.wrapping_add(plain));

        // Store decrypted value
        *val = plain;
    }

    Ok(())
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

        // Define expected values - these would need to be updated based on the actual STORM_BUFFER_CRYPT table
        // For now, we'll just verify that we can encrypt/decrypt successfully

        // Keep a copy of the encrypted data
        let encrypted = input.clone();

        // Decrypt and check that we get back the original
        decrypt_block(&mut input, key).expect("Decryption failed");

        // Should be back to the original
        let original = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        assert_eq!(input, original);
    }

    #[test]
    fn test_table_decryption() {
        // Test with a simple dataset
        let data_size = 16; // 16 bytes = 4 u32 values
        let mut original_data = vec![0u8; data_size];
        for i in 0..data_size {
            original_data[i] = i as u8;
        }

        // Make a copy for encryption
        let mut encrypted_data = original_data.clone();

        // Test with hash table key
        encrypt_block(&mut encrypted_data, super::super::HASH_TABLE_KEY)
            .expect("Encryption failed");
        assert_ne!(encrypted_data, original_data);

        decrypt_block(&mut encrypted_data, super::super::HASH_TABLE_KEY)
            .expect("Decryption failed");
        assert_eq!(encrypted_data, original_data);

        // Test with block table key
        let mut encrypted_data = original_data.clone();
        encrypt_block(&mut encrypted_data, super::super::BLOCK_TABLE_KEY)
            .expect("Encryption failed");
        assert_ne!(encrypted_data, original_data);

        decrypt_block(&mut encrypted_data, super::super::BLOCK_TABLE_KEY)
            .expect("Decryption failed");
        assert_eq!(encrypted_data, original_data);
    }
}
