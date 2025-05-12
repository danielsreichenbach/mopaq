//! Module for MPQ table structures (hash table, block table, etc.)

mod attributes;
mod bet_table;
pub mod block_table;
pub mod ext_table;
mod hash_table;
mod het_table;
mod lookup;

// Re-export public interfaces
pub use block_table::{BlockEntry, BlockTable};
pub use ext_table::ExtendedBlockTable;
pub use hash_table::{HashEntry, HashTable};
pub use lookup::{find_file, find_file_by_hash};

use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Seek, SeekFrom, Write};
use thiserror::Error;

/// Error types specific to MPQ table operations
#[derive(Error, Debug)]
pub enum TableError {
    #[error("I/O error: {0}")]
    IoError(#[from] IoError),

    #[error("Invalid table size: {0}")]
    InvalidSize(usize),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Table read error: {0}")]
    ReadError(String),

    #[error("Table write error: {0}")]
    WriteError(String),

    #[error("Invalid locale/platform: {0}/{1}")]
    InvalidLocalePlatform(u16, u16),
}

/// Common trait for MPQ tables
pub trait Table {
    /// Returns the number of entries in the table
    fn size(&self) -> usize;

    /// Returns true if the table is empty
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Reads the table from an MPQ archive
    fn read_from<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        offset: u64,
        size: usize,
    ) -> Result<(), TableError>;

    /// Writes the table to an MPQ archive
    fn write_to<W: Write + Seek>(&self, writer: &mut W, offset: u64) -> Result<(), TableError>;
}

/// Creates a properly sized hash table for the given number of files
/// The hash table size is always a power of 2 and at least double the file count
pub fn create_hash_table(file_count: usize) -> Result<HashTable, TableError> {
    // Calculate the next power of 2 that is at least double the file count
    let mut size = 2;
    while size < file_count * 2 {
        size *= 2;
    }

    HashTable::new(size)
}

/// Checks if a locale and platform combination is valid
pub fn is_valid_locale_platform(locale: u16, platform: u16) -> bool {
    // Check if locale matches any defined locale constant
    let valid_locale = match locale {
        LOCALE_NEUTRAL | LOCALE_ENGLISH => true,
        // Additional locale constants can be added here as they're defined
        LOCALE_CHINESE_TRADITIONAL => true, // Chinese Traditional
        LOCALE_CZECH => true,               // Czech
        LOCALE_GERMAN => true,              // German
        LOCALE_GREEK => true,               // Greek
        LOCALE_SPANISH => true,             // Spanish
        LOCALE_FINNISH => true,             // Finnish
        LOCALE_FRENCH => true,              // French
        LOCALE_HUNGARIAN => true,           // Hungarian
        LOCALE_ITALIAN => true,             // Italian
        LOCALE_JAPANESE => true,            // Japanese
        LOCALE_KOREAN => true,              // Korean
        LOCALE_DUTCH => true,               // Dutch
        LOCALE_POLISH => true,              // Polish
        LOCALE_PORTUGUESE => true,          // Portuguese
        LOCALE_RUSSIAN => true,             // Russian
        LOCALE_SLOVAK => true,              // Slovak
        LOCALE_TURKISH => true,             // Turkish
        _ => false,                         // All other locales are considered invalid
    };

    // Check if platform matches any defined platform constant
    let valid_platform = match platform {
        PLATFORM_NEUTRAL | PLATFORM_WINDOWS => true,
        // Additional platform constants can be added here
        PLATFORM_MAC => true,   // macOS
        PLATFORM_LINUX => true, // Linux
        _ => false,             // All other platforms are considered invalid
    };

    // Both locale and platform must be valid
    valid_locale && valid_platform
}

// Locale constants
pub const LOCALE_NEUTRAL: u16 = 0;
pub const LOCALE_ENGLISH: u16 = 0x409;
pub const LOCALE_CHINESE_TRADITIONAL: u16 = 0x0404;
pub const LOCALE_CZECH: u16 = 0x0405;
pub const LOCALE_GERMAN: u16 = 0x0407;
pub const LOCALE_GREEK: u16 = 0x0408;
pub const LOCALE_SPANISH: u16 = 0x040a;
pub const LOCALE_FINNISH: u16 = 0x040b;
pub const LOCALE_FRENCH: u16 = 0x040c;
pub const LOCALE_HUNGARIAN: u16 = 0x040e;
pub const LOCALE_ITALIAN: u16 = 0x0410;
pub const LOCALE_JAPANESE: u16 = 0x0411;
pub const LOCALE_KOREAN: u16 = 0x0412;
pub const LOCALE_DUTCH: u16 = 0x0413;
pub const LOCALE_POLISH: u16 = 0x0415;
pub const LOCALE_PORTUGUESE: u16 = 0x0416;
pub const LOCALE_RUSSIAN: u16 = 0x0419;
pub const LOCALE_SLOVAK: u16 = 0x041b;
pub const LOCALE_TURKISH: u16 = 0x041f;
// Add more as needed

// Platform constants
pub const PLATFORM_NEUTRAL: u16 = 0;
pub const PLATFORM_WINDOWS: u16 = 0x0100;
pub const PLATFORM_MAC: u16 = 0x0200;
pub const PLATFORM_LINUX: u16 = 0x0300;
// Add more as needed
