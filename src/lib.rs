//! MPQ library for Rust
//! Provides functionality for reading MPQ (Mo'PaQ) archives used in Blizzard games
//!
//! This library allows you to:
//! - Open and extract files from MPQ archives
//! - Access file metadata and contents
//! - Handle encryption and compression

// Public modules
pub mod archive;
pub mod compression;
pub mod crypto;
pub mod error;
pub mod file;
pub mod header;
pub mod listfile;
pub mod tables;

// Re-export main types for convenience
pub use archive::MpqArchive;
pub use error::{Error, Result};
pub use file::MpqFile;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library information
pub const LIBRARY_INFO: &str = concat!(
    "mopaq v",
    env!("CARGO_PKG_VERSION"),
    " - ",
    "Rust MPQ library (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

/// Initialize the library
/// This is optional but can be used to perform any global setup
#[allow(unused_variables)]
pub fn init() -> Result<()> {
    // No initialization needed currently
    Ok(())
}

/// Convenience function to open an MPQ archive
pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<MpqArchive> {
    MpqArchive::open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
