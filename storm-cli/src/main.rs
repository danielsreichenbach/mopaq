//! Storm CLI - Command-line tool for working with MPQ archives
//!
//! The binary is named `storm-cli` to avoid conflicts with the `storm` library crate.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "storm-cli")]
#[command(about = "Command-line tool for working with MPQ archives", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List files in an archive
    List {
        /// Path to the MPQ archive
        archive: String,
    },
    /// Extract files from an archive
    Extract {
        /// Path to the MPQ archive
        archive: String,
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,
    },
    /// Create a new archive
    Create {
        /// Path to the new MPQ archive
        archive: String,
        /// Directory containing files to add
        source: String,
    },
    /// Verify archive integrity
    Verify {
        /// Path to the MPQ archive
        archive: String,
    },
    /// Debug commands
    #[command(subcommand)]
    Debug(DebugCommands),
}

#[derive(Subcommand)]
enum DebugCommands {
    /// Show detailed archive information
    Info {
        /// Path to the MPQ archive
        archive: String,
    },
    /// Test crypto functions
    Crypto,
    /// Generate hash values for a filename
    Hash {
        /// Filename to hash
        filename: String,
        /// Hash type (table-offset, name-a, name-b, file-key, key2-mix, or 0-4)
        #[arg(short = 't', long)]
        hash_type: Option<String>,
        /// Generate all hash types
        #[arg(short, long)]
        all: bool,
        /// Generate Jenkins hash (for HET tables)
        #[arg(short, long)]
        jenkins: bool,
    },
    /// Compare hash values for two filenames
    HashCompare {
        /// First filename
        filename1: String,
        /// Second filename
        filename2: String,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::List { archive } => {
            println!("Listing files in: {}", archive);
            // TODO: Implement listing
        }
        Commands::Extract { archive, output } => {
            println!("Extracting {} to {}", archive, output);
            // TODO: Implement extraction
        }
        Commands::Create { archive, source } => {
            println!("Creating {} from {}", archive, source);
            // TODO: Implement creation
        }
        Commands::Verify { archive } => {
            println!("Verifying {}", archive);
            // TODO: Implement verification
        }
        Commands::Debug(debug_cmd) => match debug_cmd {
            DebugCommands::Info { archive } => {
                commands::debug::info(&archive)?;
            }
            DebugCommands::Crypto => {
                commands::debug::crypto()?;
            }
            DebugCommands::Hash {
                filename,
                hash_type,
                all,
                jenkins,
            } => {
                commands::debug::hash(&filename, hash_type.as_deref(), all, jenkins)?;
            }
            DebugCommands::HashCompare {
                filename1,
                filename2,
            } => {
                commands::debug::hash_compare(&filename1, &filename2)?;
            }
        },
    }
    Ok(())
}
