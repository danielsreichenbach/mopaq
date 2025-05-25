//! Storm CLI - Command-line tool for working with MPQ archives

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "storm")]
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
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::List { archive } => {
            println!("Listing files in: {}", archive);
            // TODO: Implement listing
        },
        Commands::Extract { archive, output } => {
            println!("Extracting {} to {}", archive, output);
            // TODO: Implement extraction
        },
        Commands::Create { archive, source } => {
            println!("Creating {} from {}", archive, source);
            // TODO: Implement creation
        },
        Commands::Verify { archive } => {
            println!("Verifying {}", archive);
            // TODO: Implement verification
        },
    }

    Ok(())
}
