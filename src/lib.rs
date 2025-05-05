//! A Rust library for reading and writing World of Warcraft MPQ archives
//!
//! This library implements the MPQ (Mo'PQ or Mike O'Brien Pack) archive format
//! used by Blizzard Entertainment games, including World of Warcraft.

pub mod archive;
pub mod config;
pub mod error;
pub mod hash_table;
pub mod header;

pub use archive::MpqArchive;
pub use config::MpqConfig;
pub use error::{MpqError, Result};
pub use hash_table::{MpqHashEntry, MpqHashTable};
pub use header::{MpqHeader, MpqUserDataHeader};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
