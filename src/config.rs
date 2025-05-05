use directories::ProjectDirs;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{MpqError, Result};

/// Configuration for the MPQ library
#[derive(Debug, Clone)]
pub struct MpqConfig {
    /// Default format version for newly created archives
    pub default_format_version: u16,

    /// Default sector size shift for newly created archives
    pub default_sector_size_shift: u16,

    /// Whether to use compression by default
    pub use_compression: bool,

    /// Whether to encrypt file names by default
    pub encrypt_file_names: bool,

    /// Whether to verify checksums when opening archives
    pub verify_checksums: bool,
}

impl Default for MpqConfig {
    fn default() -> Self {
        MpqConfig {
            default_format_version: 1,
            default_sector_size_shift: 3, // 8 KB sectors
            use_compression: true,
            encrypt_file_names: true,
            verify_checksums: true,
        }
    }
}

impl MpqConfig {
    /// Load configuration from the standard XDG config path
    pub fn load() -> Result<Self> {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        // Return default config if no config file exists
        Ok(Self::default())
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Parse the configuration (simple key=value format for now)
        let mut config = Self::default();

        for line in contents.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "default_format_version" => {
                        if let Ok(v) = value.parse::<u16>() {
                            config.default_format_version = v;
                        }
                    }
                    "default_sector_size_shift" => {
                        if let Ok(v) = value.parse::<u16>() {
                            config.default_sector_size_shift = v;
                        }
                    }
                    "use_compression" => {
                        config.use_compression = value.to_lowercase() == "true";
                    }
                    "encrypt_file_names" => {
                        config.encrypt_file_names = value.to_lowercase() == "true";
                    }
                    "verify_checksums" => {
                        config.verify_checksums = value.to_lowercase() == "true";
                    }
                    _ => {
                        // Ignore unknown keys
                    }
                }
            }
        }

        Ok(config)
    }

    /// Save configuration to the standard XDG config path
    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            // Create the directory if it doesn't exist
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            return self.save_to_file(&path);
        }

        Err(MpqError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine configuration path",
        )))
    }

    /// Save configuration to a specific file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = fs::File::create(path)?;

        writeln!(file, "# MPQ library configuration")?;
        writeln!(
            file,
            "default_format_version={}",
            self.default_format_version
        )?;
        writeln!(
            file,
            "default_sector_size_shift={}",
            self.default_sector_size_shift
        )?;
        writeln!(file, "use_compression={}", self.use_compression)?;
        writeln!(file, "encrypt_file_names={}", self.encrypt_file_names)?;
        writeln!(file, "verify_checksums={}", self.verify_checksums)?;

        Ok(())
    }

    /// Get the standard XDG config path
    pub fn config_path() -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("rs", "", "wow_mpq") {
            Some(proj_dirs.config_dir().join("config.txt"))
        } else {
            None
        }
    }
}
