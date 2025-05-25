//! Hash algorithms for MPQ file name hashing

use crate::crypto::ENCRYPTION_TABLE;

/// Hash types for MPQ operations
pub mod hash_type {
    pub const TABLE_OFFSET: u32 = 0;
    pub const NAME_A: u32 = 1;
    pub const NAME_B: u32 = 2;
    pub const FILE_KEY: u32 = 3;
    pub const KEY2_MIX: u32 = 4;
}

/// ASCII uppercase conversion table
pub const ASCII_TO_UPPER: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = i as u8;
        i += 1;
    }
    // Convert lowercase to uppercase (a-z to A-Z)
    let mut i = b'a';
    while i <= b'z' {
        table[i as usize] = i - 32;
        i += 1;
    }
    table
};

/// ASCII lowercase conversion table
pub const ASCII_TO_LOWER: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = i as u8;
        i += 1;
    }
    // Convert uppercase to lowercase (A-Z to a-z)
    let mut i = b'A';
    while i <= b'Z' {
        table[i as usize] = i + 32;
        i += 1;
    }
    table
};

/// Hash a string using the MPQ hash algorithm
pub fn hash_string(filename: &str, hash_type: u32) -> u32 {
    let mut seed1: u32 = 0x7FED7FED;
    let mut seed2: u32 = 0xEEEEEEEE;

    for &byte in filename.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to uppercase using the table
        ch = ASCII_TO_UPPER[ch as usize];

        // Update the hash
        let table_idx = (hash_type * 0x100 + ch as u32) as usize;
        seed1 = ENCRYPTION_TABLE[table_idx] ^ (seed1.wrapping_add(seed2));
        seed2 = ch as u32 + seed1 + seed2 + (seed2 << 5) + 3;
    }

    seed1
}

/// Jenkins hash function for HET tables
pub fn jenkins_hash(filename: &str) -> u64 {
    let mut hash: u64 = 0;

    for &byte in filename.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to lowercase using the table
        ch = ASCII_TO_LOWER[ch as usize];

        // Jenkins one-at-a-time hash algorithm
        hash = hash.wrapping_add(ch as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }

    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_tables() {
        // Test uppercase conversion
        assert_eq!(ASCII_TO_UPPER[b'a' as usize], b'A');
        assert_eq!(ASCII_TO_UPPER[b'z' as usize], b'Z');
        assert_eq!(ASCII_TO_UPPER[b'A' as usize], b'A');
        assert_eq!(ASCII_TO_UPPER[b'0' as usize], b'0');

        // Test lowercase conversion
        assert_eq!(ASCII_TO_LOWER[b'A' as usize], b'a');
        assert_eq!(ASCII_TO_LOWER[b'Z' as usize], b'z');
        assert_eq!(ASCII_TO_LOWER[b'a' as usize], b'a');
        assert_eq!(ASCII_TO_LOWER[b'0' as usize], b'0');
    }

    #[test]
    fn test_path_separator_normalization() {
        // Both paths should produce the same hash
        let hash1 = hash_string("path/to/file.txt", hash_type::TABLE_OFFSET);
        let hash2 = hash_string("path\\to\\file.txt", hash_type::TABLE_OFFSET);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_case_insensitivity() {
        // Different cases should produce the same hash
        let hash1 = hash_string("File.txt", hash_type::TABLE_OFFSET);
        let hash2 = hash_string("FILE.TXT", hash_type::TABLE_OFFSET);
        assert_eq!(hash1, hash2);
    }
}
