use clap::{Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use mopaq::{MpqArchive, MpqVersion, block_flags};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mopaq")]
#[command(author = "Daniel S. Reichenbach <daniel@kogito.network>")]
#[command(version = "0.1.0")]
#[command(about = "MPQ archive utility", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display information about an MPQ archive
    Info {
        /// The path to the MPQ archive
        #[arg(value_parser)]
        path: PathBuf,

        /// Show extended information
        #[arg(short, long)]
        extended: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Info { path, extended } => {
            if !path.exists() {
                eprintln!("Error: File not found: {}", path.display());
                std::process::exit(1);
            }

            // Create a multi-progress bar for better styling
            let mp = MultiProgress::new();

            // Create a spinner for the loading animation
            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb.set_message(format!("Opening MPQ archive: {}", path.display()));

            // Open the archive
            let result = MpqArchive::open(path);

            // Small delay for visual effect
            std::thread::sleep(std::time::Duration::from_millis(500));

            match result {
                Ok(archive) => {
                    pb.finish_with_message(format!(
                        "✓ Successfully opened MPQ archive: {}",
                        path.display()
                    ));

                    // Print header information
                    print_header_info(&archive, *extended);
                }
                Err(err) => {
                    pb.finish_with_message(format!("✗ Error opening MPQ archive: {}", err));
                }
            }
        }
    }

    Ok(())
}

fn print_header_info(archive: &MpqArchive, extended: bool) {
    let header = archive.header();
    let user_header = archive.user_header();

    println!("\n{}", style_section("MPQ HEADER"));
    println!("│ Signature:          0x{:08X}", header.signature);
    println!("│ Header Size:        {} bytes", header.header_size);
    println!("│ Archive Size:       {} bytes", header.archive_size);

    if let Some(size_64) = header.archive_size_64 {
        println!("│ Archive Size (64):  {} bytes", size_64);
    }

    println!("│ Format Version:     v{}", header.format_version + 1);
    println!(
        "│ Sector Size:        {} bytes",
        512 << header.sector_size_shift
    );
    println!("│ Hash Table Entries: {}", header.hash_table_entries);
    println!("│ Block Table Entries: {}", header.block_table_entries);

    if let Some(user_hdr) = user_header {
        println!("\n{}", style_section("USER HEADER INFORMATION"));
        println!("│ Signature:          0x{:08X}", user_hdr.signature);
        println!("│ User Data Size:     {} bytes", user_hdr.user_data_size);
        println!("│ MPQ Header Offset:  {} bytes", user_hdr.mpq_header_offset);
        println!("│ User Header Size:   {} bytes", user_hdr.user_header_size);
    }

    if extended {
        println!("\n{}", style_section("EXTENDED INFORMATION"));
        println!("│ Hash Table Offset:  0x{:08X}", header.hash_table_offset);
        println!("│ Block Table Offset: 0x{:08X}", header.block_table_offset);

        if header.format_version >= 1 {
            println!(
                "│ BET Table Offset:   0x{:016X}",
                header.bet_table_offset.unwrap_or(0)
            );
            println!(
                "│ HET Table Offset:   0x{:016X}",
                header.het_table_offset.unwrap_or(0)
            );
        }

        if header.format_version >= 2 {
            println!(
                "│ Hash Table Pos:     0x{:016X}",
                header.hash_table_pos.unwrap_or(0)
            );
            println!(
                "│ Block Table Pos:    0x{:016X}",
                header.block_table_pos.unwrap_or(0)
            );
            println!(
                "│ Hi Block Table Pos: 0x{:016X}",
                header.hi_block_table_pos.unwrap_or(0)
            );
            println!(
                "│ Hash Table Size:    {}",
                header.hash_table_size.unwrap_or(0)
            );
            println!(
                "│ Block Table Size:   {}",
                header.block_table_size.unwrap_or(0)
            );
        }

        println!("\n{}", style_section("FILE LISTING"));
        let files = archive.list_files();
        if files.is_empty() {
            println!("│ No files found.");
        } else {
            for (i, file) in files.iter().enumerate() {
                println!("│ {}. {}", i + 1, file);
            }
        }
    }

    println!("\n");
}

fn style_section(title: &str) -> String {
    format!(
        "┌─────────────────────────────────────────────────────┐\n│ \x1b[1;32m{:^51}\x1b[0m │\n└─────────────────────────────────────────────────────┘",
        title
    )
}
