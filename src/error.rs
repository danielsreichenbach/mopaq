use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MpqError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid MPQ header: {0}")]
    InvalidHeader(String),

    #[error("Invalid MPQ user header")]
    InvalidUserHeader,

    #[error("Unsupported MPQ version: {0}")]
    UnsupportedVersion(u16),

    #[error("Archive is corrupted: {0}")]
    CorruptedArchive(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("MPQ feature not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, MpqError>;
