//! Cryptographic constants for MPQ archives

/// The STORM_BUFFER_CRYPT table
/// This table is used for both hashing and encryption
pub static STORM_BUFFER_CRYPT: [u32; 0x500] = generate_crypt_table();

/// Key for hash table encryption
pub const HASH_TABLE_KEY: u32 = 0xC3AF3770;

/// Key for block table encryption
pub const BLOCK_TABLE_KEY: u32 = 0xEC83B3A3;

/// Key for extended block table encryption
pub const MPQ_EXTENDED_BLOCK_TABLE_KEY: u32 = 0x39525245;

/// Generate the MPQ hashing table
const fn generate_crypt_table() -> [u32; 0x500] {
    let mut table = [0u32; 0x500];
    let mut seed: u32 = 0x0010_0001;

    let mut i = 0;
    while i < 0x100 {
        let mut j = 0;
        while j < 5 {
            let index = i + j * 0x100;
            seed = (seed * 125 + 3) % 0x002A_AAAB;
            let t1 = (seed & 0xFFFF) << 0x10;
            seed = (seed * 125 + 3) % 0x002A_AAAB;
            let t2 = seed & 0xFFFF;

            table[index] = t1 | t2;

            j += 1;
        }
        i += 1;
    }

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storm_buffer_crypt_table() {
        // Known correct values from StormLib
        // Format: (index, expected_value)
        let test_cases = [
            // Hash type 0 (TableOffset) section
            (0x000, 0x55C6_36E2),
            (0x001, 0x02BE_0170),
            (0x002, 0x584B_71D4),
            (0x010, 0x848F_645E),
            // Test middle of first section (hash type 0)
            (0x080, 0xB676_6CEF),
            (0x081, 0x0678_2877),
            (0x082, 0x155C_6DD0),
            (0x08F, 0x42B0_8FEE),
            // Test end of first section (hash type 0)
            (0x0F0, 0xDD84_7EBA),
            (0x0F1, 0x883D_305D),
            (0x0F2, 0x25F1_3152),
            (0x0FF, 0x708C_9EEC),
            // Hash type 1 (NameA) section
            (0x100, 0x76F8_C1B1),
            (0x101, 0xB394_59D2),
            (0x102, 0x3F1E_26D9),
            (0x10F, 0x426E_6FB0),
            // Hash type 2 (NameB) section
            (0x200, 0x3DF6_965D),
            (0x201, 0x30C1_237B),
            (0x202, 0xF7F6_686A),
            (0x20F, 0x88EE_3168),
            // Hash type 3 (FileKey) section
            (0x300, 0x15F2_61D3),
            (0x301, 0xA84C_2D0D),
            (0x302, 0x50F1_85A6),
            (0x30F, 0x74FE_755A),
            // Encryption section
            (0x400, 0x193A_A698),
            (0x401, 0x5496_F7D5),
            (0x402, 0x4208_931B),
            (0x40F, 0xA248_9278),
        ];

        // Verify each test case
        for &(index, expected) in &test_cases {
            let actual = STORM_BUFFER_CRYPT[index];
            assert_eq!(
                actual, expected,
                "STORM_BUFFER_CRYPT[{:#x}] = {:#010x}, expected {:#010x}",
                index, actual, expected
            );
        }

        // Verify table size
        assert_eq!(
            STORM_BUFFER_CRYPT.len(),
            0x500,
            "STORM_BUFFER_CRYPT table should have 0x500 entries"
        );

        // Calculate a simple checksum of the entire table as an additional validation
        let table_checksum: u32 = STORM_BUFFER_CRYPT
            .iter()
            .fold(0u32, |acc, &val| acc.wrapping_add(val));
        // This checksum value should be calculated from StormLib's table
        const EXPECTED_CHECKSUM: u32 = 0x5EE0_21AE; // Replace with actual checksum from StormLib

        assert_eq!(
            table_checksum, EXPECTED_CHECKSUM,
            "STORM_BUFFER_CRYPT table checksum incorrect: {:#010x}, expected {:#010x}",
            table_checksum, EXPECTED_CHECKSUM
        );
    }
}
