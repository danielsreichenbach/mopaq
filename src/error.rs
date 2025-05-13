//! Error types for the MPQ library

use std::io;
use std::path::PathBuf;
use thiserror::Error;

use crate::compression::CompressionError;
use crate::crypto::CryptoError;
use crate::header::HeaderError;
use crate::tables::TableError;

/// Primary error type for MPQ operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Header error: {0}")]
    HeaderError(#[from] HeaderError),

    #[error("Table error: {0}")]
    TableError(#[from] TableError),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] CryptoError),

    #[error("Compression error: {0}")]
    CompressionError(#[from] CompressionError),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid archive: {0}")]
    InvalidArchive(String),

    #[error("Failed to open archive at {0}")]
    ArchiveOpenError(PathBuf),

    #[error("Failed to read file {0} from archive")]
    FileReadError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Invalid sector size: {0}")]
    InvalidSectorSize(u32),

    #[error("Invalid file sector: {context}")]
    InvalidSector {
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("{0}")]
    Other(String),
}

/// Result type for MPQ operations
pub type Result<T> = std::result::Result<T, Error>;
