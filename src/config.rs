//! Configuration management for the G-code language server.
//!
//! Handles:
//! - Command-line argument parsing
//! - Project configuration file (.gcode.toml) loading
//! - Hierarchical configuration search

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;

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
    #[arg(long, default_value = "info", help = "Log level (trace, debug, info, warn, error)")]
    pub log_level: String,
}

/// Project configuration loaded from .gcode.toml
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectSettings,
}

/// Project settings section
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectSettings {
    pub default_flavor: Option<String>,
    pub settings: Option<AdditionalSettings>,
}

/// Additional optional settings
#[derive(Debug, Clone, Deserialize)]
pub struct AdditionalSettings {
    pub enable_diagnostics: Option<bool>,
    pub completion_style: Option<String>,
}

/// Combined configuration from all sources
#[derive(Debug, Clone)]
pub struct Config {
    /// Flavor name explicitly set via command line
    pub cli_flavor: Option<String>,
    /// Flavor name from project configuration
    pub project_flavor: Option<String>,
    /// Custom flavor directories to search
    pub flavor_dirs: Vec<PathBuf>,
    /// Project configuration file path (if found)
    pub project_config_path: Option<PathBuf>,
    /// Log level
    pub log_level: String,
}

impl Config {
    /// Create configuration from command-line arguments and project config search
    pub fn from_args_and_env() -> Result<Self> {
        let args = Args::parse();
        
        // Search for project configuration
        let (project_config, project_config_path) = Self::search_project_config()?;
        let project_flavor = project_config
            .as_ref()
            .and_then(|c| c.project.default_flavor.clone());

        // Determine flavor directories
        let mut flavor_dirs = Vec::new();
        
        // Add user-specified directory if provided
        if let Some(custom_dir) = args.flavor_dir {
            flavor_dirs.push(custom_dir);
        }
        
        // Add default directories
        if let Some(config_dir) = dirs::config_dir() {
            flavor_dirs.push(config_dir.join("gcode-ls").join("flavors"));
        }
        
        // Add workspace directory
        let workspace_dir = std::env::current_dir()?.join(".gcode-ls").join("flavors");
        flavor_dirs.push(workspace_dir);

        Ok(Config {
            cli_flavor: args.flavor,
            project_flavor,
            flavor_dirs,
            project_config_path,
            log_level: args.log_level,
        })
    }

    /// Search for .gcode.toml starting from current directory going up
    fn search_project_config() -> Result<(Option<ProjectConfig>, Option<PathBuf>)> {
        let mut current = std::env::current_dir()?;
        
        loop {
            let config_path = current.join(".gcode.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)
                    .with_context(|| format!("Failed to read project config: {}", config_path.display()))?;
                
                let config: ProjectConfig = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse project config: {}", config_path.display()))?;
                
                return Ok((Some(config), Some(config_path)));
            }
            
            // Move to parent directory
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                // Reached filesystem root
                break;
            }
        }
        
        Ok((None, None))
    }

    /// Get the effective flavor name based on priority:
    /// CLI argument > project config > fallback to None (will use default)
    pub fn get_effective_flavor(&self) -> Option<String> {
        self.cli_flavor.clone().or(self.project_flavor.clone())
    }

    /// Check if a project configuration was found
    pub fn has_project_config(&self) -> bool {
        self.project_config_path.is_some()
    }

    /// Get project config path for logging/debugging
    pub fn project_config_path(&self) -> Option<&Path> {
        self.project_config_path.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_project_config_parsing() {
        let config_content = r#"
[project]
default_flavor = "marlin"

[project.settings]
enable_diagnostics = true
completion_style = "detailed"
"#;
        
        let config: ProjectConfig = toml::from_str(config_content).unwrap();
        assert_eq!(config.project.default_flavor.as_deref(), Some("marlin"));
        assert_eq!(config.project.settings.as_ref().unwrap().enable_diagnostics, Some(true));
        assert_eq!(config.project.settings.as_ref().unwrap().completion_style.as_deref(), Some("detailed"));
    }

    #[tokio::test]
    async fn test_hierarchical_search() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("project");
        let subdir = project_root.join("subdir").join("deep");
        
        fs::create_dir_all(&subdir).unwrap();
        
        // Create config in project root
        let config_path = project_root.join(".gcode.toml");
        fs::write(&config_path, r#"
[project]
default_flavor = "test_flavor"
"#).unwrap();
        
        // Change to subdirectory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&subdir).unwrap();
        
        // Search should find the config
        let (config, path) = Config::search_project_config().unwrap();
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
        
        assert!(config.is_some());
        assert_eq!(config.unwrap().project.default_flavor.as_deref(), Some("test_flavor"));
        // Use canonicalize to handle symlinks in temp directories on macOS
        assert_eq!(path.unwrap().canonicalize().unwrap(), config_path.canonicalize().unwrap());
    }
}
