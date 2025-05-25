//! # MoPaQ - MPQ Archive Library
//!
//! A high-performance, safe Rust implementation of the MPQ (Mo'PaQ) archive format
//! used by Blizzard Entertainment games.
//!
//! ## About the Name
//!
//! MoPaQ is named after the original format name "Mo'PaQ" (Mike O'Brien Pack),
//! which was later shortened to MPQ. This library provides the core MPQ functionality,
//! while `storm-ffi` provides StormLib compatibility.
//!
//! ## Features
//!
//! - Support for all MPQ format versions (v1-v4)
//! - Full compatibility with StormLib API through FFI
//! - Multiple compression algorithms (zlib, bzip2, LZMA, etc.)
//! - Strong security with signature verification
//! - Memory-mapped I/O support for performance
//! - Comprehensive error handling
//!
//! ## Example
//!
//! ```no_run
//! use mopaq::{Archive, OpenOptions};
//!
//! # fn main() -> Result<(), mopaq::Error> {
//! // Open an existing MPQ archive
//! let archive = Archive::open("example.mpq")?;
//!
//! // List files in the archive
//! for entry in archive.list()? {
//!     println!("{}", entry.name);
//! }
//!
//! // Extract a specific file
//! let data = archive.read_file("war3map.j")?;
//! # Ok(())
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

pub mod archive;
pub mod compression;
pub mod crypto;
pub mod error;
pub mod hash;
pub mod io;
pub mod tables;

// Re-export commonly used types
pub use archive::{Archive, OpenOptions};
pub use error::{Error, Result};

/// MPQ format version constants
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatVersion {
    /// Original MPQ format (32-byte header)
    V1 = 0,
    /// The Burning Crusade format (44-byte header)
    V2 = 1,
    /// Cataclysm Beta format (68-byte header)
    V3 = 2,
    /// Cataclysm+ format (208-byte header)
    V4 = 3,
}

/// MPQ signature constants
pub mod signatures {
    /// Standard MPQ archive signature ('MPQ\x1A')
    pub const MPQ_ARCHIVE: u32 = 0x1A51504D;

    /// MPQ user data signature ('MPQ\x1B')
    pub const MPQ_USERDATA: u32 = 0x1B51504D;

    /// HET table signature ('HET\x1A')
    pub const HET_TABLE: u32 = 0x1A544548;

    /// BET table signature ('BET\x1A')
    pub const BET_TABLE: u32 = 0x1A544542;

    /// Strong signature magic ('NGIS')
    pub const STRONG_SIGNATURE: [u8; 4] = *b"NGIS";
}

/// Block size calculation
#[inline]
pub fn calculate_sector_size(block_size_shift: u16) -> usize {
    512 << block_size_shift
}

/// Check if a value is a power of two
#[inline]
pub fn is_power_of_two(value: u32) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_size_calculation() {
        assert_eq!(calculate_sector_size(0), 512);
        assert_eq!(calculate_sector_size(3), 4096);
        assert_eq!(calculate_sector_size(8), 131072);
    }

    #[test]
    fn test_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(1024));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(1023));
    }
}
