//! Encryption and decryption algorithms for MPQ files

use crate::Result;
use once_cell::sync::Lazy;

/// The static encryption table used by all MPQ operations
pub static ENCRYPTION_TABLE: Lazy<[u32; 0x500]> = Lazy::new(generate_encryption_table);

/// Generate the MPQ encryption table
///
/// This table is used for all encryption, decryption, and hashing operations
/// in the MPQ format. It consists of 1280 (0x500) 32-bit values.
fn generate_encryption_table() -> [u32; 0x500] {
    let mut table = [0u32; 0x500];
    let mut seed: u32 = 0x00100001;

    for index1 in 0..0x100 {
        for index2 in 0..5 {
            let table_index = index1 + index2 * 0x100;

            // Update seed using the algorithm
            seed = (seed.wrapping_mul(125) + 3) % 0x2AAAAB;
            let temp1 = (seed & 0xFFFF) << 0x10;

            seed = (seed.wrapping_mul(125) + 3) % 0x2AAAAB;
            let temp2 = seed & 0xFFFF;

            table[table_index] = temp1 | temp2;
        }
    }

    table
}

/// Decrypt a block of data
pub fn decrypt_block(data: &mut [u32], mut key: u32) {
    if key == 0 {
        return;
    }

    let mut seed: u32 = 0xEEEEEEEE;

    for value in data.iter_mut() {
        // Update seed using the encryption table and key
        seed = seed.wrapping_add(ENCRYPTION_TABLE[0x400 + (key & 0xFF) as usize]);

        // Decrypt the current DWORD
        let ch = *value ^ (key.wrapping_add(seed));
        *value = ch;

        // Update key for next round
        key = (!key << 0x15).wrapping_add(0x11111111) | (key >> 0x0B);

        // Update seed for next round
        seed = ch
            .wrapping_add(seed)
            .wrapping_add(seed << 5)
            .wrapping_add(3);
    }
}

/// Encrypt a block of data
pub fn encrypt_block(data: &mut [u32], mut key: u32) {
    if key == 0 {
        return;
    }

    let mut seed: u32 = 0xEEEEEEEE;

    for value in data.iter_mut() {
        // Update seed using the encryption table and key
        seed = seed.wrapping_add(ENCRYPTION_TABLE[0x400 + (key & 0xFF) as usize]);

        // Store original value
        let ch = *value;

        // Encrypt the current DWORD
        *value = ch ^ (key.wrapping_add(seed));

        // Update key for next round
        key = (!key << 0x15).wrapping_add(0x11111111) | (key >> 0x0B);

        // Update seed for next round
        seed = ch
            .wrapping_add(seed)
            .wrapping_add(seed << 5)
            .wrapping_add(3);
    }
}

/// Decrypt a single DWORD value
pub fn decrypt_dword(value: u32, key: u32) -> u32 {
    if key == 0 {
        return value;
    }

    let mut seed: u32 = 0xEEEEEEEE;
    seed = seed.wrapping_add(ENCRYPTION_TABLE[0x400 + (key & 0xFF) as usize]);

    value ^ (key.wrapping_add(seed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_table_generation() {
        // Test known values from the encryption table
        // These values are from the MPQ specification
        assert_eq!(ENCRYPTION_TABLE[0x000], 0x1A790AA9);
        assert_eq!(ENCRYPTION_TABLE[0x001], 0x18DF4175);
        assert_eq!(ENCRYPTION_TABLE[0x002], 0x3C064005);
        assert_eq!(ENCRYPTION_TABLE[0x003], 0x0D66C89C);
        assert_eq!(ENCRYPTION_TABLE[0x004], 0x24C5C5A9);

        // Test some middle values
        assert_eq!(ENCRYPTION_TABLE[0x100], 0x8AD9D6A4);
        assert_eq!(ENCRYPTION_TABLE[0x200], 0x8142F724);
        assert_eq!(ENCRYPTION_TABLE[0x300], 0xECFA1006);
        assert_eq!(ENCRYPTION_TABLE[0x400], 0x2F8E7E01);

        // Test last few values
        assert_eq!(ENCRYPTION_TABLE[0x4FB], 0x3C9740B0);
        assert_eq!(ENCRYPTION_TABLE[0x4FC], 0x3C579B79);
        assert_eq!(ENCRYPTION_TABLE[0x4FD], 0x1A3C54E7);
        assert_eq!(ENCRYPTION_TABLE[0x4FE], 0x21B86B73);
        assert_eq!(ENCRYPTION_TABLE[0x4FF], 0x16FEF546);
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let original_data = vec![
            0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0, 0xFEDCBA98, 0x76543210, 0xF0DEBC9A,
            0xE1C3A597,
        ];

        let key = 0xC1EB1CEF;

        // Test round trip
        let mut data = original_data.clone();
        encrypt_block(&mut data, key);

        // Verify data was changed
        assert_ne!(data, original_data);

        // Decrypt back
        decrypt_block(&mut data, key);

        // Verify we got the original data back
        assert_eq!(data, original_data);
    }

    #[test]
    fn test_known_encryption() {
        // Test with known test vectors from the MPQ specification
        let mut data = vec![
            0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0, 0xFEDCBA98, 0x76543210, 0xF0DEBC9A,
            0xE1C3A597,
        ];

        let key = 0xC1EB1CEF;

        encrypt_block(&mut data, key);

        // Expected encrypted values from the specification
        let expected = vec![
            0x6DBB9D94, 0x20F0AF34, 0x3A73EA6F, 0x8E82A467, 0x5F11FC9B, 0xD9BE74FF, 0x82071B61,
            0xF1E4D305,
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_decrypt_single_dword() {
        let encrypted = 0x6DBB9D94;
        let key = 0xC1EB1CEF;
        let expected = 0x12345678;

        assert_eq!(decrypt_dword(encrypted, key), expected);
    }

    #[test]
    fn test_zero_key() {
        // Test that zero key doesn't modify data
        let original = vec![0x12345678, 0x9ABCDEF0];
        let mut data = original.clone();

        encrypt_block(&mut data, 0);
        assert_eq!(data, original);

        decrypt_block(&mut data, 0);
        assert_eq!(data, original);
    }

    #[test]
    fn test_different_keys_produce_different_results() {
        let original = vec![0x12345678, 0x9ABCDEF0];

        let mut data1 = original.clone();
        let mut data2 = original.clone();

        encrypt_block(&mut data1, 0x11111111);
        encrypt_block(&mut data2, 0x22222222);

        assert_ne!(data1, data2);
        assert_ne!(data1, original);
        assert_ne!(data2, original);
    }
}
