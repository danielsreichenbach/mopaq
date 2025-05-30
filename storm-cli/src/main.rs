//! Storm CLI - Command-line tool for working with MPQ archives
//!
//! The binary is named `storm-cli` to avoid conflicts with the `storm` library crate.

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use std::io;
use std::sync::OnceLock;

mod commands;
mod output;

use mopaq::{FormatVersion, ListfileOption};

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
#[command(
    name = "storm-cli",
    about = "Command-line tool for working with MPQ archives",
    long_about = None,
    after_help = "EXAMPLES:
    # List files in an archive
    storm-cli list game.mpq

    # Extract all files
    storm-cli extract game.mpq -t extracted/

    # Extract specific file
    storm-cli extract game.mpq -f war3map.j

    # Create new archive
    storm-cli create new.mpq source_folder/

    # Find a file
    storm-cli find game.mpq \"*.mdx\"

    # Verify archive integrity
    storm-cli verify game.mpq

    # Generate shell completions
    storm-cli completion bash > ~/.bash_completion.d/storm-cli.bash
    storm-cli completion zsh > ~/.zsh/completions/_storm-cli
    storm-cli completion fish > ~/.config/fish/completions/storm-cli.fish
    storm-cli completion powershell > $PROFILE\\storm-cli.ps1

SHELL COMPLETION:
    To enable tab completion, run:

    Bash:
        storm-cli completion bash > ~/.bash_completion.d/storm-cli.bash
        source ~/.bash_completion.d/storm-cli.bash

    Zsh:
        storm-cli completion zsh > ~/.zsh/completions/_storm-cli
        # Add to ~/.zshrc: fpath=(~/.zsh/completions $fpath)

    Fish:
        storm-cli completion fish > ~/.config/fish/completions/storm-cli.fish

    PowerShell:
        storm-cli completion powershell >> $PROFILE"
)]
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
    /// Create a new MPQ archive
    Create {
        /// Path to the new MPQ archive
        archive: String,

        /// Source file or directory
        source: String,

        /// MPQ format version (1-4)
        #[arg(short = 'V', long, value_parser = clap::value_parser!(u16).range(1..=4))]
        version: Option<u16>,

        /// Compression method
        #[arg(short = 'c', long, value_enum)]
        compression: Option<CompressionMethod>,

        /// Block size (0-23, sector size = 512 * 2^n)
        #[arg(short = 'b', long, value_parser = clap::value_parser!(u16).range(0..=23))]
        block_size: Option<u16>,

        /// Don't include a (listfile)
        #[arg(long)]
        no_listfile: bool,

        /// Include external listfile
        #[arg(long, conflicts_with = "no_listfile")]
        listfile: Option<String>,

        /// Don't recurse into subdirectories
        #[arg(long)]
        no_recursive: bool,

        /// Follow symbolic links
        #[arg(long)]
        follow_symlinks: bool,

        /// Additional patterns to ignore (can be used multiple times)
        #[arg(short = 'i', long = "ignore")]
        ignore_patterns: Vec<String>,
    },
    /// Verify archive integrity
    Verify {
        /// Path to the MPQ archive
        archive: String,
    },
    /// Generate shell completion scripts
    #[command(about = "Generate completion scripts for your shell")]
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
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
        /// Table type (hash, block, het, bet) or index number
        #[arg(short = 't', long)]
        table_type: Option<String>,
        /// Limit number of entries shown
        #[arg(short, long, default_value = "20")]
        limit: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum CompressionMethod {
    None,
    Zlib,
    Bzip2,
    Lzma,
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
        Commands::Create {
            archive,
            source,
            version,
            compression,
            block_size,
            no_listfile,
            listfile,
            no_recursive,
            follow_symlinks,
            ignore_patterns,
        } => {
            let mut options = commands::create::CreateOptions::default();

            // Set version
            if let Some(v) = version {
                options.version = match v {
                    1 => FormatVersion::V1,
                    2 => FormatVersion::V2,
                    3 => FormatVersion::V3,
                    4 => FormatVersion::V4,
                    _ => unreachable!(),
                };
            }

            // Set compression
            if let Some(comp) = compression {
                options.compression = match comp {
                    CompressionMethod::None => 0,
                    CompressionMethod::Zlib => mopaq::compression::flags::ZLIB,
                    CompressionMethod::Bzip2 => mopaq::compression::flags::BZIP2,
                    CompressionMethod::Lzma => mopaq::compression::flags::LZMA,
                };
            }

            // Set block size
            if let Some(bs) = block_size {
                options.block_size = bs;
            }

            // Set listfile option
            options.listfile = if no_listfile {
                ListfileOption::None
            } else if let Some(lf) = listfile {
                ListfileOption::External(lf.into())
            } else {
                ListfileOption::Generate
            };

            // Set other options
            options.recursive = !no_recursive;
            options.follow_symlinks = follow_symlinks;
            if !ignore_patterns.is_empty() {
                options.ignore_patterns.extend(ignore_patterns);
            }

            commands::create::create(&archive, &source, options)?;
        }
        Commands::Verify { archive } => {
            let verbose = cli.verbose > 0;
            commands::verify::verify(&archive, verbose)?;
        }
        Commands::Completion { shell } => {
            // Generate completion script for the specified shell
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
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
