//! Storm CLI - Command-line tool for working with MPQ archives
//!
//! The binary is named `storm-cli` to avoid conflicts with the `storm` library crate.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::sync::OnceLock;

mod commands;
mod output;

// Global context for commands to access
pub static GLOBAL_OPTS: OnceLock<GlobalOptions> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct GlobalOptions {
    pub output: OutputFormat,
    pub verbose: u8,
    pub quiet: bool,
    pub no_color: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Parser)]
#[command(name = "storm-cli")]
#[command(about = "Command-line tool for working with MPQ archives", long_about = None)]
#[command(version)]
struct Cli {
    /// Output format
    #[arg(global = true, short = 'o', long, value_enum, default_value = "text")]
    output: OutputFormat,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(global = true, short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output except errors
    #[arg(global = true, short = 'q', long, conflicts_with = "verbose")]
    quiet: bool,

    /// Disable colored output
    #[arg(global = true, long)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List files in an archive
    List {
        /// Path to the MPQ archive
        archive: String,
        /// Show all entries even without filenames
        #[arg(short, long)]
        all: bool,
    },
    /// Find a specific file in an archive
    Find {
        /// Path to the MPQ archive
        archive: String,
        /// Filename to search for
        filename: String,
    },
    /// Extract files from an archive
    Extract {
        /// Path to the MPQ archive
        archive: String,
        /// Target directory
        #[arg(short, long, default_value = ".")]
        target: String,
        /// Specific file to extract (if not specified, extracts all)
        #[arg(short, long)]
        file: Option<String>,
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
    /// Display table contents
    Tables {
        /// Path to the MPQ archive
        archive: String,
        /// Table type (hash, block) or index number
        #[arg(short = 't', long)]
        table_type: Option<String>,
        /// Limit number of entries shown
        #[arg(short, long, default_value = "20")]
        limit: Option<usize>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up colored output based on flags
    if cli.no_color || cli.output != OutputFormat::Text {
        colored::control::set_override(false);
    }

    // Configure logging based on verbosity
    let log_level = match (cli.quiet, cli.verbose) {
        (true, _) => "error",
        (false, 0) => "warn",
        (false, 1) => "info",
        (false, 2) => "debug",
        (false, _) => "trace",
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    // Store global options for commands to access
    let global_opts = GlobalOptions {
        output: cli.output,
        verbose: cli.verbose,
        quiet: cli.quiet,
        no_color: cli.no_color,
    };

    GLOBAL_OPTS
        .set(global_opts)
        .expect("Failed to set global options");

    // Execute command
    match cli.command {
        Commands::List { archive, all } => {
            // Pass the global verbose value instead
            let verbose = cli.verbose > 0;
            commands::list::list(&archive, verbose, all)?;
        }
        Commands::Find { archive, filename } => {
            let verbose = cli.verbose > 0;
            commands::find::find(&archive, &filename, verbose)?;
        }
        Commands::Extract {
            archive,
            target,
            file,
        } => {
            commands::extract::extract(&archive, &target, file.as_deref())?;
        }
        Commands::Create { archive, source } => {
            println!("Creating {} from {}", archive, source);
            // TODO: Implement creation
        }
        Commands::Verify { archive } => {
            let verbose = cli.verbose > 0;
            commands::verify::verify(&archive, verbose)?;
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
            DebugCommands::Tables {
                archive,
                table_type,
                limit,
            } => {
                commands::debug::tables(&archive, table_type.as_deref(), limit)?;
            }
        },
    }

    Ok(())
}
