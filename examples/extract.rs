use mopaq::{Error, MpqArchive};
use std::env;
use std::path::Path;

/// Simple example to extract all files from an MPQ archive
fn main() -> Result<(), Error> {
    // Get the archive path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <archive.mpq> [output_directory]", args[0]);
        std::process::exit(1);
    }

    // Get the archive path and output directory
    let archive_path = &args[1];
    let output_dir = if args.len() >= 3 {
        Path::new(&args[2])
    } else {
        Path::new("output")
    };

    // Print basic info
    println!("MPQ Extractor");
    println!("Archive: {}", archive_path);
    println!("Output directory: {}", output_dir.display());

    // Open the archive
    println!("Opening archive...");
    let archive = MpqArchive::open(archive_path)?;

    // Display archive information
    println!("Archive version: {}", archive.header().format_version);
    println!("Archive size: {} bytes", archive.header().archive_size_64());
    println!("Number of files: {}", archive.file_count());

    // Extract all files
    println!("Extracting files...");
    archive.extract_all(output_dir)?;

    // Print file names if listfile is available
    let filenames = archive.filenames();
    if !filenames.is_empty() {
        println!("\nExtracted files:");
        for filename in filenames {
            println!("  {}", filename);
        }
    }

    println!("\nExtraction complete!");
    Ok(())
}
