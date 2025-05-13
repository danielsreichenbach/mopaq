//! Cryptographic functionality for MPQ archives
//! Includes hash functions, encryption/decryption, and key generation

mod constants;
mod encryption;
pub mod hash;
pub mod key_derivation;

// Re-export public interfaces
pub use constants::{
    BLOCK_TABLE_KEY, HASH_TABLE_KEY, MPQ_EXTENDED_BLOCK_TABLE_KEY, STORM_BUFFER_CRYPT,
};
pub use encryption::{decrypt_block, encrypt_block};
pub use hash::{HashType, compute_file_hashes, hash_string};
pub use key_derivation::{detect_file_key, generate_file_key, generate_sector_key};

use std::io::Error as IoError;
use thiserror::Error;

/// Error types specific to MPQ cryptographic operations
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("I/O error: {0}")]
    IoError(#[from] IoError),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Data alignment error: {0}")]
    AlignmentError(String),

    #[error("Buffer size error: {0}")]
    BufferSizeError(String),
}

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;
