//! Encryption and decryption algorithms for MPQ files

use crate::Result;

/// The encryption table used by MPQ archives
pub struct EncryptionTable {
    table: [u32; 0x500],
}

impl EncryptionTable {
    /// Generate the static encryption table
    pub fn new() -> Self {
        let mut table = [0u32; 0x500];
        // TODO: Implement encryption table generation
        Self { table }
    }

    /// Get the table data
    pub fn data(&self) -> &[u32; 0x500] {
        &self.table
    }
}

impl Default for EncryptionTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Decrypt a block of data
pub fn decrypt_block(_data: &mut [u32], _key: u32, _table: &EncryptionTable) {
    todo!("Implement block decryption")
}

/// Encrypt a block of data
pub fn encrypt_block(_data: &mut [u32], _key: u32, _table: &EncryptionTable) {
    todo!("Implement block encryption")
}
