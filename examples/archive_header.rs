use mopaq::{MpqArchive, MpqVersion, Result};

fn main() -> Result<()> {
    // Open an existing WoW MPQ archive
    let archive = MpqArchive::open("path/to/your/wow.mpq")?;
    println!("Archive opened successfully!");

    // Print some basic information
    println!("Format version: {}", archive.header().format_version);
    println!("Contains user header: {}", archive.user_header().is_some());

    // Create a new MPQ archive (format version 1)
    let new_archive = MpqArchive::create("path/to/new.mpq", MpqVersion::Version1)?;
    println!("Created a new archive!");

    Ok(())
}
