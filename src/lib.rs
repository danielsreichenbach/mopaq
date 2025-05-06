//! # Mopaq
//!
//! `mopaq` is a Rust library for handling World of Warcraft MPQ archives.
//! It provides functionality to read and write MPQ archives, including
//! support for user headers.

mod archive;
mod block_table;
mod error;
mod hash_table;
mod header;
mod user_header;
mod utils;

pub use archive::MpqArchive;
pub use block_table::{MpqBlockEntry, MpqBlockTable, block_flags, compression_type};
pub use error::{MopaqError, Result};
pub use hash_table::{MpqHashEntry, MpqHashTable, hash};
pub use header::{MPQ_HEADER_SIGNATURE, MPQ_USER_DATA_SIGNATURE, MpqHeader, MpqVersion};
pub use user_header::{MpqUserHeader, read_mpq_header, write_mpq_header};
pub use utils::{
    calculate_hash_table_size, calculate_sector_count, calculate_table_size, get_sector_size,
};
