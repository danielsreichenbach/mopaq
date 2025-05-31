//! Configuration file support

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default compression method
    pub default_compression: Option<String>,

    /// Default MPQ version
    pub default_version: Option<u16>,

    /// Default block size
    pub default_block_size: Option<u16>,

    /// Command aliases
    pub aliases: Option<std::collections::HashMap<String, String>>,

    /// Default output format
    pub default_output: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_compression: Some("zlib".to_string()),
            default_version: Some(1),
            default_block_size: Some(3),
            aliases: None,
            default_output: Some("text".to_string()),
        }
    }
}

/// Load configuration from file or defaults
pub fn load_config(path: Option<&PathBuf>) -> Result<Config> {
    let config_path = if let Some(p) = path {
        p.clone()
    } else {
        // Try default locations
        if let Some(home) = dirs::home_dir() {
            let storm_config = home.join(".storm-cli").join("config.toml");
            if storm_config.exists() {
                storm_config
            } else {
                let config_dir = home.join(".config").join("storm-cli").join("config.toml");
                if config_dir.exists() {
                    config_dir
                } else {
                    // Return default config if no file found
                    return Ok(Config::default());
                }
            }
        } else {
            return Ok(Config::default());
        }
    };

    if config_path.exists() {
        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

/// Save configuration to file
#[allow(dead_code)]
pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    let contents = toml::to_string_pretty(config)?;

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}
