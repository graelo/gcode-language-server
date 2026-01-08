//! Configuration management for the G-code language server.
//!
//! Handles:
//! - Command-line argument parsing
//! - Flavor directory configuration

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments for the G-code language server
#[derive(Debug, Parser)]
#[command(name = "gcode-language-server")]
#[command(about = "Language server for G-code files")]
#[command(version)]
pub struct Args {
    /// Explicitly specify the G-code flavor to use
    #[arg(long, help = "G-code flavor to use (e.g., 'prusa', 'marlin')")]
    pub flavor: Option<String>,

    /// Custom flavor directory to search for flavor files
    #[arg(long, help = "Directory containing flavor TOML files")]
    pub flavor_dir: Option<PathBuf>,

    /// Log level for the language server
    #[arg(
        long,
        default_value = "info",
        help = "Log level (trace, debug, info, warn, error)"
    )]
    pub log_level: String,
}

/// Combined configuration from all sources
#[derive(Debug, Clone)]
pub struct Config {
    /// Flavor name explicitly set via command line
    pub cli_flavor: Option<String>,
    /// Custom flavor directories to search
    pub flavor_dirs: Vec<PathBuf>,
    /// Log level
    pub log_level: String,
}

impl Config {
    /// Create configuration from command-line arguments
    pub fn from_args_and_env() -> Result<Self> {
        Self::from_args(Args::parse())
    }

    /// Create configuration from explicit arguments (useful for testing)
    pub fn from_args(args: Args) -> Result<Self> {
        // Determine flavor directories
        let mut flavor_dirs = Vec::new();

        // Add user-specified directory if provided
        if let Some(custom_dir) = args.flavor_dir {
            flavor_dirs.push(custom_dir);
        }

        // Add default user config directory
        if let Some(config_dir) = dirs::config_dir() {
            flavor_dirs.push(config_dir.join("gcode-ls").join("flavors"));
        }

        Ok(Config {
            cli_flavor: args.flavor,
            flavor_dirs,
            log_level: args.log_level,
        })
    }

    /// Get the effective flavor name from CLI arguments
    pub fn get_effective_flavor(&self) -> Option<String> {
        self.cli_flavor.clone()
    }
}
