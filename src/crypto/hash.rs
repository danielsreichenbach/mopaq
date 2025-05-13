//! Hash functions for MPQ archives

use super::constants::STORM_BUFFER_CRYPT;
use std::path::Path;

/// Hash types used in MPQ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    /// Hash used to get the index in the hash table (also known as HashTableOffset)
    TableOffset = 0,
    /// First hash for file verification (also known as HashNameA)
    NameA = 1,
    /// Second hash for file verification (also known as HashNameB)
    NameB = 2,
    /// Hash used for file key calculation (also known as HashFileKey)
    FileKey = 3,
}

/// Calculates a hash value for a string based on the specified hash type
///
/// This is the core hashing function used in MPQ archives.
///
/// # Arguments
///
/// * `input` - The string to hash
/// * `hash_type` - The type of hash to calculate (0-3)
///
/// # Returns
///
/// The calculated hash value
pub fn hash_string(input: &str, hash_type: HashType) -> u32 {
    let mut seed1: u32 = 0x7FED_7FED;
    let mut seed2: u32 = 0xEEEE_EEEE;

    // Convert to uppercase for case-insensitive hash
    let uppercase = input.to_uppercase();

    for ch in uppercase.bytes() {
        // Use the lookup table to get a value based on the character and hash type
        let index = (hash_type as usize * 0x100) + (ch as usize);

        // IMPORTANT: Fixed calculation to match StormLib exactly
        seed1 = STORM_BUFFER_CRYPT[index].wrapping_add(seed1.wrapping_mul(2).wrapping_add(seed2));
        seed2 = (ch as u32)
            .wrapping_add(seed1)
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }

    seed1
}

/// Computes the three hash values used for file lookup
///
/// # Arguments
///
/// * `filename` - The filename to hash
///
/// # Returns
///
/// A tuple of (TableOffset, NameA, NameB) hash values
pub fn compute_file_hashes(filename: &str) -> (u32, u32, u32) {
    // Split the filename at the last path separator
    let file_part = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(filename);

    // Calculate the three hash values
    let hash_table_offset = hash_string(file_part, HashType::TableOffset);
    let hash_name_a = hash_string(file_part, HashType::NameA);
    let hash_name_b = hash_string(file_part, HashType::NameB);

    (hash_table_offset, hash_name_a, hash_name_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        // Test with known hash values from StormLib
        assert_eq!(
            hash_string("arr\\units.dat", HashType::TableOffset),
            0xF4E6C69D
        );
        assert_eq!(
            hash_string("unit\\neutral\\acritter.grp", HashType::TableOffset),
            0xA26067F3
        );

        // Test case 1: "(listfile)"
        assert_eq!(hash_string("(listfile)", HashType::TableOffset), 0x47F3D35A);
        assert_eq!(hash_string("(listfile)", HashType::NameA), 0x5F3DE859);
        assert_eq!(hash_string("(listfile)", HashType::NameB), 0x600F8C95);

        // Test case 2: "(attributes)"
        assert_eq!(
            hash_string("(attributes)", HashType::TableOffset),
            0xD38437CB
        );
        assert_eq!(hash_string("(attributes)", HashType::NameA), 0x07973B89);
        assert_eq!(hash_string("(attributes)", HashType::NameB), 0xA9FD618C);

        // Test case 3: "war3map.j"
        assert_eq!(hash_string("war3map.j", HashType::TableOffset), 0xFA393EE6);
        assert_eq!(hash_string("war3map.j", HashType::NameA), 0xD5B0C549);
        assert_eq!(hash_string("war3map.j", HashType::NameB), 0x87A5FFC8);
    }

    #[test]
    fn test_compute_file_hashes() {
        // Test with a filename with path
        let filename = "path\\to\\file.txt";
        let (offset, hash_a, hash_b) = compute_file_hashes(filename);

        // Should use only the filename part
        assert_eq!(offset, hash_string("file.txt", HashType::TableOffset));
        assert_eq!(hash_a, hash_string("file.txt", HashType::NameA));
        assert_eq!(hash_b, hash_string("file.txt", HashType::NameB));

        // Test with a plain filename
        let filename = "file.txt";
        let (offset, hash_a, hash_b) = compute_file_hashes(filename);

        assert_eq!(offset, hash_string("file.txt", HashType::TableOffset));
        assert_eq!(hash_a, hash_string("file.txt", HashType::NameA));
        assert_eq!(hash_b, hash_string("file.txt", HashType::NameB));
    }

    #[test]
    fn test_case_insensitivity() {
        // Hash values should be the same regardless of case
        let lowercase = hash_string("file.txt", HashType::TableOffset);
        let uppercase = hash_string("FILE.TXT", HashType::TableOffset);
        let mixed_case = hash_string("FiLe.TxT", HashType::TableOffset);

        assert_eq!(lowercase, uppercase);
        assert_eq!(lowercase, mixed_case);
    }
}
