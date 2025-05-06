use std::io;
use thiserror::Error;

/// Errors that can occur when working with MPQ archives
#[derive(Error, Debug)]
pub enum MopaqError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid MPQ signature")]
    InvalidSignature,

    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),

    #[error("Invalid header size: {0}")]
    InvalidHeaderSize(u32),

    #[error("Invalid archive size: {0}")]
    InvalidArchiveSize(u64),

    #[error("Invalid user data position")]
    InvalidUserDataPosition,

    #[error("The hash or block table is full")]
    TableFull,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("The file is corrupted")]
    CorruptedFile,
}

/// A Result type specialized for MPQ operations
pub type Result<T> = std::result::Result<T, MopaqError>;
