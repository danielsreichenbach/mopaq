use mopaq::compression::flags as compression_flags;
use mopaq::{Archive, ArchiveBuilder, FormatVersion};
use std::fs;

fn main() -> mopaq::Result<()> {
    // Create a test archive with HET/BET table compression enabled
    let builder = ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .compress_tables(true)
        .table_compression(compression_flags::ZLIB)
        .add_file_data(b"Hello, World!".to_vec(), "test.txt")
        .add_file_data(b"This is another test file.".to_vec(), "test2.txt")
        .add_file_data(b"A third file for good measure.".to_vec(), "test3.txt");

    println!("Creating archive with compressed HET/BET tables...");
    builder.build("test_compressed_tables.mpq")?;

    // Verify we can read the archive back
    println!("Opening archive to verify compressed tables can be read...");
    let mut archive = Archive::open("test_compressed_tables.mpq")?;

    // List files to ensure HET/BET tables work
    let files = archive.list()?;
    println!("Found {} files:", files.len());
    for file in files {
        println!("  - {} ({} bytes)", file.name, file.size);
    }

    // Test reading a file to ensure data integrity
    let data = archive.read_file("test.txt")?;
    let content =
        String::from_utf8(data).map_err(|_| mopaq::Error::invalid_format("Invalid UTF-8"))?;
    println!("Read file content: '{}'", content);

    if content == "Hello, World!" {
        println!("✅ HET/BET table compression test PASSED!");
    } else {
        println!("❌ HET/BET table compression test FAILED!");
    }

    // Clean up
    fs::remove_file("test_compressed_tables.mpq").ok();

    Ok(())
}
