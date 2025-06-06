//! Storm CLI - Command-line tool for working with MPQ archives
//!
//! The binary is named `storm-cli` to avoid conflicts with the `storm` library crate.

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

mod commands;
mod config;
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

    /// Path to config file
    #[arg(global = true, short = 'C', long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Archive operations
    #[command(subcommand)]
    Archive(ArchiveCommands),

    /// File operations
    #[command(subcommand)]
    File(FileCommands),

    /// Table operations
    #[command(subcommand)]
    Table(TableCommands),

    /// Hash utilities
    #[command(subcommand)]
    Hash(HashCommands),

    /// Cryptography utilities
    #[command(subcommand)]
    Crypto(CryptoCommands),

    /// Generate shell completion scripts
    #[command(about = "Generate completion scripts for your shell")]
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Configuration management
    #[command(about = "Manage storm-cli configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., default_compression, default_version)
        key: String,

        /// Value to set
        value: String,
    },

    /// Reset configuration to defaults
    Reset,

    /// Show configuration file path
    Path,
}

#[derive(Subcommand)]
enum ArchiveCommands {
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

    /// Show detailed archive information
    Info {
        /// Path to the MPQ archive
        archive: String,
    },

    /// Verify archive integrity
    Verify {
        /// Path to the MPQ archive
        archive: String,

        /// Check CRC values
        #[arg(long)]
        check_crc: bool,

        /// Check file contents
        #[arg(long)]
        check_contents: bool,
    },

    /// List files in an archive (alias for 'file list')
    List {
        /// Path to the MPQ archive
        archive: String,

        /// Show all entries even without filenames
        #[arg(short, long)]
        all: bool,

        /// Filter by pattern (glob or regex)
        #[arg(short = 'p', long)]
        pattern: Option<String>,

        /// Use regex instead of glob pattern
        #[arg(short = 'r', long)]
        regex: bool,

        /// Show file name hashes
        #[arg(long)]
        show_hashes: bool,
    },

    /// Analyze compression methods used in an archive
    Analyze {
        /// Path to the MPQ archive
        archive: String,

        /// Show compression method for each file
        #[arg(short, long)]
        detailed: bool,

        /// Group results by file extension
        #[arg(short = 'e', long)]
        by_extension: bool,

        /// Show only files using unsupported compression methods
        #[arg(short = 'u', long)]
        unsupported_only: bool,

        /// Show compression ratio statistics
        #[arg(short = 's', long)]
        show_stats: bool,
    },
}

#[derive(Subcommand)]
enum FileCommands {
    /// List files in an archive
    List {
        /// Path to the MPQ archive
        archive: String,

        /// Show all entries even without filenames
        #[arg(short, long)]
        all: bool,

        /// Filter by pattern (glob or regex)
        #[arg(short = 'p', long)]
        pattern: Option<String>,

        /// Use regex instead of glob pattern
        #[arg(short = 'r', long)]
        regex: bool,

        /// Show file name hashes
        #[arg(long)]
        show_hashes: bool,
    },

    /// Extract files from an archive
    Extract {
        /// Path to the MPQ archive
        archive: String,

        /// File to extract (if not specified, extracts all)
        file: Option<String>,

        /// Target directory or file for extraction
        #[arg(short = 't', long = "target-directory")]
        target_directory: Option<String>,

        /// Preserve directory structure
        #[arg(short = 'p', long)]
        preserve_path: bool,
    },

    /// Add files to an existing archive
    Add {
        /// Path to the MPQ archive
        archive: String,

        /// Files to add
        #[arg(required = true)]
        files: Vec<String>,

        /// Compression method
        #[arg(short = 'c', long, value_enum)]
        compression: Option<CompressionMethod>,

        /// Archive path for the file
        #[arg(short = 'p', long)]
        path: Option<String>,
    },

    /// Remove files from an archive
    Remove {
        /// Path to the MPQ archive
        archive: String,

        /// Files to remove
        #[arg(required = true)]
        files: Vec<String>,
    },

    /// Find files in an archive
    Find {
        /// Path to the MPQ archive
        archive: String,

        /// Pattern to search for (glob or regex)
        pattern: String,

        /// Use regex instead of glob pattern
        #[arg(short = 'r', long)]
        regex: bool,

        /// Case insensitive search
        #[arg(short = 'i', long)]
        ignore_case: bool,
    },

    /// Show detailed file information
    Info {
        /// Path to the MPQ archive
        archive: String,

        /// File to inspect
        file: String,
    },
}

#[derive(Subcommand)]
enum TableCommands {
    /// Display table contents
    Show {
        /// Path to the MPQ archive
        archive: String,

        /// Table type (hash, block, het, bet)
        #[arg(short = 't', long)]
        table_type: Option<TableType>,

        /// Limit number of entries shown
        #[arg(short, long, default_value = "20")]
        limit: Option<usize>,

        /// Show only occupied entries
        #[arg(long)]
        occupied_only: bool,
    },

    /// Analyze table structure and efficiency
    Analyze {
        /// Path to the MPQ archive
        archive: String,

        /// Include detailed statistics
        #[arg(short = 'd', long)]
        detailed: bool,
    },
}

#[derive(Subcommand)]
enum HashCommands {
    /// Generate hash values for a filename
    Generate {
        /// Filename to hash
        filename: String,

        /// Hash type (table-offset, name-a, name-b, file-key, key2-mix)
        #[arg(short = 't', long)]
        hash_type: Option<HashType>,

        /// Generate all hash types
        #[arg(short, long)]
        all: bool,
    },

    /// Compare hash values for two filenames
    Compare {
        /// First filename
        filename1: String,

        /// Second filename
        filename2: String,
    },

    /// Generate Jenkins hash (for HET tables)
    Jenkins {
        /// Filename to hash
        filename: String,
    },
}

#[derive(Subcommand)]
enum CryptoCommands {
    /// Test cryptographic functions
    Test {
        /// Run specific test
        #[arg(short = 't', long)]
        test: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum CompressionMethod {
    None,
    Zlib,
    Bzip2,
    Lzma,
    Sparse,
    Pkware,
    AdpcmMono,
    AdpcmStereo,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum TableType {
    Hash,
    Block,
    Het,
    Bet,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum HashType {
    TableOffset,
    NameA,
    NameB,
    FileKey,
    Key2Mix,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config if specified
    let config = if let Some(config_path) = &cli.config {
        config::load_config(Some(config_path))?
    } else {
        config::load_config(None)?
    };

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
        Commands::Archive(cmd) => match cmd {
            ArchiveCommands::Create {
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
                let mut options = commands::archive::CreateOptions {
                    version: if let Some(v) = version {
                        match v {
                            1 => FormatVersion::V1,
                            2 => FormatVersion::V2,
                            3 => FormatVersion::V3,
                            4 => FormatVersion::V4,
                            _ => unreachable!(),
                        }
                    } else if let Some(v) = config.default_version {
                        match v {
                            1 => FormatVersion::V1,
                            2 => FormatVersion::V2,
                            3 => FormatVersion::V3,
                            4 => FormatVersion::V4,
                            _ => FormatVersion::V1, // fallback to V1 for invalid values
                        }
                    } else {
                        FormatVersion::V1
                    },
                    compression: if let Some(comp) = compression {
                        match comp {
                            CompressionMethod::None => 0,
                            CompressionMethod::Zlib => mopaq::compression::flags::ZLIB as u16,
                            CompressionMethod::Bzip2 => mopaq::compression::flags::BZIP2 as u16,
                            CompressionMethod::Lzma => mopaq::compression::flags::LZMA as u16,
                            CompressionMethod::Sparse => mopaq::compression::flags::SPARSE as u16,
                            CompressionMethod::Pkware => mopaq::compression::flags::PKWARE as u16,
                            CompressionMethod::AdpcmMono => {
                                mopaq::compression::flags::ADPCM_MONO as u16
                            }
                            CompressionMethod::AdpcmStereo => {
                                mopaq::compression::flags::ADPCM_STEREO as u16
                            }
                        }
                    } else if let Some(comp_str) = &config.default_compression {
                        match comp_str.as_str() {
                            "none" => 0,
                            "zlib" => mopaq::compression::flags::ZLIB as u16,
                            "bzip2" => mopaq::compression::flags::BZIP2 as u16,
                            "lzma" => mopaq::compression::flags::LZMA as u16,
                            "sparse" => mopaq::compression::flags::SPARSE as u16,
                            "pkware" => mopaq::compression::flags::PKWARE as u16,
                            "adpcm-mono" => mopaq::compression::flags::ADPCM_MONO as u16,
                            "adpcm-stereo" => mopaq::compression::flags::ADPCM_STEREO as u16,
                            _ => mopaq::compression::flags::ZLIB as u16, // fallback to zlib
                        }
                    } else {
                        mopaq::compression::flags::ZLIB as u16
                    },
                    block_size: block_size.or(config.default_block_size).unwrap_or(3),
                    listfile: if no_listfile {
                        ListfileOption::None
                    } else if let Some(lf) = listfile {
                        ListfileOption::External(lf.into())
                    } else {
                        ListfileOption::Generate
                    },
                    recursive: !no_recursive,
                    follow_symlinks,
                    ..Default::default()
                };

                if !ignore_patterns.is_empty() {
                    options.ignore_patterns.extend(ignore_patterns);
                }

                commands::archive::create(&archive, &source, options)?;
            }
            ArchiveCommands::Info { archive } => {
                commands::archive::info(&archive)?;
            }
            ArchiveCommands::Verify {
                archive,
                check_crc,
                check_contents,
            } => {
                commands::archive::verify(&archive, check_crc, check_contents)?;
            }
            ArchiveCommands::List {
                archive,
                all,
                pattern,
                regex,
                show_hashes,
            } => {
                // Delegate to the file list command
                commands::file::list(&archive, all, pattern.as_deref(), regex, show_hashes)?;
            }

            ArchiveCommands::Analyze {
                archive,
                detailed,
                by_extension,
                unsupported_only,
                show_stats,
            } => {
                commands::archive::analyze(
                    &archive,
                    detailed,
                    by_extension,
                    unsupported_only,
                    show_stats,
                )?;
            }
        },

        Commands::File(cmd) => match cmd {
            FileCommands::List {
                archive,
                all,
                pattern,
                regex,
                show_hashes,
            } => {
                commands::file::list(&archive, all, pattern.as_deref(), regex, show_hashes)?;
            }
            FileCommands::Extract {
                archive,
                file,
                target_directory,
                preserve_path,
            } => {
                commands::file::extract(
                    &archive,
                    file.as_deref(),
                    target_directory.as_deref(),
                    preserve_path,
                )?;
            }
            FileCommands::Add {
                archive,
                files,
                compression,
                path,
            } => {
                let comp = compression.map(|c| match c {
                    CompressionMethod::None => 0u16,
                    CompressionMethod::Zlib => mopaq::compression::flags::ZLIB as u16,
                    CompressionMethod::Bzip2 => mopaq::compression::flags::BZIP2 as u16,
                    CompressionMethod::Lzma => mopaq::compression::flags::LZMA as u16,
                    CompressionMethod::Sparse => mopaq::compression::flags::SPARSE as u16,
                    CompressionMethod::Pkware => mopaq::compression::flags::PKWARE as u16,
                    CompressionMethod::AdpcmMono => mopaq::compression::flags::ADPCM_MONO as u16,
                    CompressionMethod::AdpcmStereo => {
                        mopaq::compression::flags::ADPCM_STEREO as u16
                    }
                });
                commands::file::add(&archive, &files, comp, path.as_deref())?;
            }
            FileCommands::Remove { archive, files } => {
                commands::file::remove(&archive, &files)?;
            }
            FileCommands::Find {
                archive,
                pattern,
                regex,
                ignore_case,
            } => {
                commands::file::find(&archive, &pattern, regex, ignore_case)?;
            }
            FileCommands::Info { archive, file } => {
                commands::file::info(&archive, &file)?;
            }
        },

        Commands::Table(cmd) => match cmd {
            TableCommands::Show {
                archive,
                table_type,
                limit,
                occupied_only,
            } => {
                commands::table::show(&archive, table_type, limit, occupied_only)?;
            }
            TableCommands::Analyze { archive, detailed } => {
                commands::table::analyze(&archive, detailed)?;
            }
        },

        Commands::Hash(cmd) => match cmd {
            HashCommands::Generate {
                filename,
                hash_type,
                all,
            } => {
                commands::hash::generate(&filename, hash_type, all)?;
            }
            HashCommands::Compare {
                filename1,
                filename2,
            } => {
                commands::hash::compare(&filename1, &filename2)?;
            }
            HashCommands::Jenkins { filename } => {
                commands::hash::jenkins(&filename)?;
            }
        },

        Commands::Crypto(cmd) => match cmd {
            CryptoCommands::Test { test } => {
                commands::crypto::test(test.as_deref())?;
            }
        },

        Commands::Completion { shell } => {
            // Generate completion script for the specified shell
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
        }

        Commands::Config { command } => {
            handle_config_command(command, &cli.config)?;
        }
    }

    Ok(())
}

/// Handle config commands
fn handle_config_command(
    command: ConfigCommands,
    config_path_override: &Option<PathBuf>,
) -> Result<()> {
    use colored::Colorize;

    // Determine config path
    let config_path = if let Some(path) = config_path_override {
        path.clone()
    } else {
        // Use default config location
        if let Some(home) = dirs::home_dir() {
            let storm_config = home.join(".storm-cli").join("config.toml");
            let xdg_config = home.join(".config").join("storm-cli").join("config.toml");

            // Prefer XDG location for new configs
            if storm_config.exists() {
                storm_config
            } else {
                xdg_config
            }
        } else {
            anyhow::bail!("Could not determine home directory");
        }
    };

    match command {
        ConfigCommands::Show => {
            let config = config::load_config(config_path_override.as_ref())?;
            println!("{}", "Current Configuration:".green().bold());
            println!(
                "  Default compression: {}",
                config
                    .default_compression
                    .as_deref()
                    .unwrap_or("zlib")
                    .cyan()
            );
            println!(
                "  Default version: {}",
                config.default_version.unwrap_or(1).to_string().cyan()
            );
            println!(
                "  Default block size: {}",
                config.default_block_size.unwrap_or(3).to_string().cyan()
            );
            println!(
                "  Default output: {}",
                config.default_output.as_deref().unwrap_or("text").cyan()
            );

            if let Some(aliases) = &config.aliases {
                if !aliases.is_empty() {
                    println!("\n{}:", "Aliases".green().bold());
                    for (alias, command) in aliases {
                        println!("  {} = {}", alias.yellow(), command);
                    }
                }
            }
        }

        ConfigCommands::Set { key, value } => {
            let mut config = config::load_config(config_path_override.as_ref())?;

            match key.as_str() {
                "default_compression" => {
                    // Validate compression method
                    match value.as_str() {
                        "none" | "zlib" | "bzip2" | "lzma" | "sparse" | "pkware" | "adpcm-mono" | "adpcm-stereo" => {
                            config.default_compression = Some(value.clone());
                        }
                        _ => anyhow::bail!(
                            "Invalid compression method. Valid values: none, zlib, bzip2, lzma, sparse, pkware, adpcm-mono, adpcm-stereo"
                        ),
                    }
                }
                "default_version" => {
                    // Validate version
                    match value.parse::<u16>() {
                        Ok(v) if (1..=4).contains(&v) => {
                            config.default_version = Some(v);
                        }
                        _ => anyhow::bail!("Invalid version. Valid values: 1, 2, 3, 4"),
                    }
                }
                "default_block_size" => {
                    // Validate block size
                    match value.parse::<u16>() {
                        Ok(bs) if bs <= 23 => {
                            config.default_block_size = Some(bs);
                        }
                        _ => anyhow::bail!("Invalid block size. Valid range: 0-23"),
                    }
                }
                "default_output" => {
                    // Validate output format
                    match value.as_str() {
                        "text" | "json" | "csv" => {
                            config.default_output = Some(value.clone());
                        }
                        _ => anyhow::bail!("Invalid output format. Valid values: text, json, csv"),
                    }
                }
                _ => anyhow::bail!("Unknown configuration key: {}", key),
            }

            // Save the updated config
            config::save_config(&config, &config_path)?;
            println!(
                "{} {} = {}",
                "Set".green().bold(),
                key.cyan(),
                value.yellow()
            );
            println!("Configuration saved to: {}", config_path.display());
        }

        ConfigCommands::Reset => {
            let default_config = config::Config::default();
            config::save_config(&default_config, &config_path)?;
            println!("{}", "Configuration reset to defaults".green().bold());
            println!("Configuration saved to: {}", config_path.display());
        }

        ConfigCommands::Path => {
            println!(
                "{} {}",
                "Configuration file path:".green().bold(),
                config_path.display()
            );
            if config_path.exists() {
                println!("Status: {}", "exists".green());
            } else {
                println!(
                    "Status: {} (will be created when you set a value)",
                    "does not exist".yellow()
                );
            }
        }
    }

    Ok(())
}
