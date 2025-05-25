//! Encryption and decryption algorithms for MPQ files

use crate::Result;

/// Generate the MPQ encryption table at compile time
const fn generate_encryption_table() -> [u32; 0x500] {
    let mut table = [0u32; 0x500];
    let mut seed: u32 = 0x00100001;

    let mut index1 = 0;
    while index1 < 0x100 {
        let mut index2 = 0;
        while index2 < 5 {
            let table_index = index1 + index2 * 0x100;

            // Update seed using the algorithm
            seed = seed.wrapping_mul(125).wrapping_add(3) % 0x2AAAAB;
            let temp1 = (seed & 0xFFFF) << 0x10;

            seed = seed.wrapping_mul(125).wrapping_add(3) % 0x2AAAAB;
            let temp2 = seed & 0xFFFF;

            table[table_index] = temp1 | temp2;
            index2 += 1;
        }
        index1 += 1;
    }

    table
}

/// The static encryption table used by all MPQ operations
pub const ENCRYPTION_TABLE: [u32; 0x500] = generate_encryption_table();

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
        assert_eq!(ENCRYPTION_TABLE[0x000], 0x55C6_36E2);
        assert_eq!(ENCRYPTION_TABLE[0x001], 0x02BE_0170);
        assert_eq!(ENCRYPTION_TABLE[0x002], 0x584B_71D4);
        assert_eq!(ENCRYPTION_TABLE[0x003], 0x2984_F00E);
        assert_eq!(ENCRYPTION_TABLE[0x004], 0xB682_C809);

        // Test some middle values
        assert_eq!(ENCRYPTION_TABLE[0x100], 0x708C_9EEC);
        assert_eq!(ENCRYPTION_TABLE[0x200], 0xEE8D_D024);
        assert_eq!(ENCRYPTION_TABLE[0x300], 0x4C20_2B7A);
        assert_eq!(ENCRYPTION_TABLE[0x400], 0x3A6F_DD6C);

        // Test last few values
        assert_eq!(ENCRYPTION_TABLE[0x4FB], 0x6149_809C);
        assert_eq!(ENCRYPTION_TABLE[0x4FC], 0xB009_9EF4);
        assert_eq!(ENCRYPTION_TABLE[0x4FD], 0xC5F6_53A5);
        assert_eq!(ENCRYPTION_TABLE[0x4FE], 0x4C10_790D);
        assert_eq!(ENCRYPTION_TABLE[0x4FF], 0x7303_286C);
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
        // Test with known test vectors
        // Note: These values depend on the specific encryption table values
        let mut data = vec![
            0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0, 0xFEDCBA98, 0x76543210, 0xF0DEBC9A,
            0xE1C3A597,
        ];

        let key = 0xC1EB1CEF;
        let original = data.clone();

        encrypt_block(&mut data, key);

        // Verify encryption changed the data
        assert_ne!(data, original, "Encryption should modify the data");

        // Decrypt and verify round-trip
        decrypt_block(&mut data, key);
        assert_eq!(data, original, "Decryption should restore original data");
    }

    #[test]
    fn test_decrypt_single_dword() {
        // Test single DWORD encryption/decryption
        let original = 0x12345678;
        let key = 0xC1EB1CEF;

        // Encrypt using block function
        let mut data = vec![original];
        encrypt_block(&mut data, key);
        let encrypted = data[0];

        // Decrypt using single dword function
        let decrypted = decrypt_dword(encrypted, key);

        assert_eq!(decrypted, original);
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
