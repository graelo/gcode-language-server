//! Flavor management system for G-code language server.
//!
//! This module handles:
//! - Loading flavor definitions from TOML files
//! - File watching for live reload
//! - Loading priority: built-in < user-global < workspace
//! - Flavor selection via command-line, project config, or modeline
//! - Error handling and validation

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use tokio::sync::{mpsc, RwLock};
use tower_lsp::lsp_types::MessageType;
use tower_lsp::Client;

use crate::config;

/// Complete flavor definition loaded from TOML
#[derive(Debug, Clone, Deserialize)]
pub struct Flavor {
    pub flavor: FlavorMeta,
    pub commands: Option<Vec<CommandDef>>,
}

/// Metadata about a flavor
#[derive(Debug, Clone, Deserialize)]
pub struct FlavorMeta {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Definition of a G-code command within a flavor
#[derive(Clone, Debug, Deserialize)]
pub struct CommandDef {
    pub name: String,
    pub pattern: Option<String>,
    pub description_short: Option<String>,
    pub description_long: Option<String>,
    pub parameters: Option<Vec<ParameterDef>>,
}

impl CommandDef {
    /// Find a parameter definition by name (including aliases)
    pub fn find_parameter(&self, name: &str) -> Option<&ParameterDef> {
        self.parameters
            .as_ref()?
            .iter()
            .find(|param| param.matches_name(name))
    }

    /// Get all required parameters for this command
    pub fn required_parameters(&self) -> Vec<&ParameterDef> {
        self.parameters
            .as_ref()
            .map(|params| params.iter().filter(|p| p.required).collect())
            .unwrap_or_default()
    }

    /// Validate all parameters for this command
    pub fn validate_parameters(&self, provided_params: &[(String, String)]) -> Vec<String> {
        let mut errors = Vec::new();

        // Check provided parameters are valid
        for (param_name, param_value) in provided_params {
            match self.find_parameter(param_name) {
                Some(param_def) => {
                    if let Err(validation_error) = param_def.validate_value(param_value) {
                        errors.push(validation_error);
                    }
                }
                None => {
                    errors.push(format!(
                        "Unknown parameter '{}' for command '{}'",
                        param_name, self.name
                    ));
                }
            }
        }

        // Check for missing required parameters
        let provided_names: std::collections::HashSet<String> = provided_params
            .iter()
            .map(|(name, _)| name.to_lowercase())
            .collect();

        for required_param in self.required_parameters() {
            let param_name_lower = required_param.name.to_lowercase();
            let alias_match = required_param
                .aliases
                .as_ref()
                .map(|aliases| {
                    aliases
                        .iter()
                        .any(|alias| provided_names.contains(&alias.to_lowercase()))
                })
                .unwrap_or(false);

            if !provided_names.contains(&param_name_lower) && !alias_match {
                errors.push(format!(
                    "Missing required parameter '{}' for command '{}'",
                    required_param.name, self.name
                ));
            }
        }

        errors
    }
}

/// Definition of a command parameter
#[derive(Clone, Debug, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    #[serde(default)]
    pub required: bool,
    pub description: String,
    pub constraints: Option<ParameterConstraints>,
    pub default_value: Option<String>,
    pub aliases: Option<Vec<String>>,
}

/// Parameter type enumeration
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Int,
    Float,
    String,
    Bool,
}

/// Parameter constraints for validation
#[derive(Clone, Debug, Deserialize)]
pub struct ParameterConstraints {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub enum_values: Option<Vec<String>>,
    pub pattern: Option<String>,
}

impl ParameterDef {
    /// Check if this parameter matches a given name (including aliases)
    pub fn matches_name(&self, name: &str) -> bool {
        if self.name.eq_ignore_ascii_case(name) {
            return true;
        }

        if let Some(aliases) = &self.aliases {
            return aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name));
        }

        false
    }

    /// Validate a parameter value against this definition
    pub fn validate_value(&self, value: &str) -> Result<(), String> {
        // Type validation
        match self.param_type {
            ParameterType::Int => {
                let _: i64 = value.parse().map_err(|_| {
                    format!(
                        "Parameter '{}' expects an integer, got '{}'",
                        self.name, value
                    )
                })?;
            }
            ParameterType::Float => {
                let parsed_value: f64 = value.parse().map_err(|_| {
                    format!(
                        "Parameter '{}' expects a number, got '{}'",
                        self.name, value
                    )
                })?;

                // Check constraints
                if let Some(constraints) = &self.constraints {
                    if let Some(min) = constraints.min_value {
                        if parsed_value < min {
                            return Err(format!(
                                "Parameter '{}' value {} is below minimum {}",
                                self.name, parsed_value, min
                            ));
                        }
                    }
                    if let Some(max) = constraints.max_value {
                        if parsed_value > max {
                            return Err(format!(
                                "Parameter '{}' value {} exceeds maximum {}",
                                self.name, parsed_value, max
                            ));
                        }
                    }
                }
            }
            ParameterType::String => {
                // Check enum constraints
                if let Some(constraints) = &self.constraints {
                    if let Some(enum_values) = &constraints.enum_values {
                        if !enum_values.iter().any(|v| v.eq_ignore_ascii_case(value)) {
                            return Err(format!(
                                "Parameter '{}' value '{}' is not one of: {}",
                                self.name,
                                value,
                                enum_values.join(", ")
                            ));
                        }
                    }

                    // Check pattern constraints
                    if let Some(pattern) = &constraints.pattern {
                        // For now, just check if it's provided - full regex validation could be added later
                        if pattern.is_empty() {
                            return Err(format!(
                                "Parameter '{}' pattern constraint is invalid",
                                self.name
                            ));
                        }
                    }
                }
            }
            ParameterType::Bool => {
                let lower_value = value.to_lowercase();
                if !matches!(
                    lower_value.as_str(),
                    "true" | "false" | "1" | "0" | "on" | "off"
                ) {
                    return Err(format!("Parameter '{}' expects a boolean value (true/false, 1/0, on/off), got '{}'", 
                        self.name, value));
                }
            }
        }

        Ok(())
    }
}

/// Represents the loading priority of flavors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlavorPriority {
    BuiltIn = 0,
    UserGlobal = 1,
    Workspace = 2,
}

/// A loaded flavor with its source and priority
#[derive(Debug, Clone)]
pub struct LoadedFlavor {
    pub flavor: Flavor,
    pub priority: FlavorPriority,
    pub source_path: Option<PathBuf>,
}

/// Events from the file watcher
#[derive(Debug)]
enum WatcherEvent {
    FlavorFileChanged(PathBuf),
    WatcherError(notify::Error),
}

/// Configuration for flavor selection
#[derive(Debug, Clone)]
pub struct FlavorSelectionConfig {
    /// Flavor explicitly specified via CLI
    pub cli_flavor: Option<String>,
    /// Flavor from project configuration
    pub project_flavor: Option<String>,
    /// Path to project config (for logging)
    pub project_config_path: Option<PathBuf>,
}

/// The main flavor manager that handles loading, watching, and resolving flavors
pub struct FlavorManager {
    /// Currently loaded flavors by name
    flavors: Arc<RwLock<HashMap<String, LoadedFlavor>>>,
    /// Directory paths to watch for user flavors
    flavor_dirs: Vec<PathBuf>,
    /// Flavor selection configuration
    selection_config: FlavorSelectionConfig,
    /// File watcher
    _watcher: Option<RecommendedWatcher>,
    /// Channel to receive watcher events
    watcher_rx: Option<mpsc::UnboundedReceiver<WatcherEvent>>,
    /// LSP client for logging
    client: Option<Client>,
}

impl FlavorManager {
    /// Create a new flavor manager with configuration
    pub fn new(config: &config::Config) -> Result<Self> {
        let selection_config = FlavorSelectionConfig {
            cli_flavor: config.cli_flavor.clone(),
            project_flavor: config.project_flavor.clone(),
            project_config_path: config.project_config_path.as_ref().map(|p| p.to_path_buf()),
        };

        Ok(Self {
            flavors: Arc::new(RwLock::new(HashMap::new())),
            flavor_dirs: config.flavor_dirs.clone(),
            selection_config,
            _watcher: None,
            watcher_rx: None,
            client: None,
        })
    }

    /// Create a flavor manager with default configuration (for backward compatibility)
    pub fn with_default_config() -> Result<Self> {
        let flavor_dirs = Self::get_default_flavor_directories()?;
        let selection_config = FlavorSelectionConfig {
            cli_flavor: None,
            project_flavor: None,
            project_config_path: None,
        };

        Ok(Self {
            flavors: Arc::new(RwLock::new(HashMap::new())),
            flavor_dirs,
            selection_config,
            _watcher: None,
            watcher_rx: None,
            client: None,
        })
    }

    /// Get the standard flavor directories (for backward compatibility)
    fn get_default_flavor_directories() -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        // User global directory: ~/.config/gcode-ls/flavors/
        if let Some(config_dir) = dirs::config_dir() {
            dirs.push(config_dir.join("gcode-ls").join("flavors"));
        }

        // Current workspace directory: ./.gcode-ls/flavors/
        let workspace_dir = std::env::current_dir()?.join(".gcode-ls").join("flavors");
        dirs.push(workspace_dir);

        Ok(dirs)
    }

    /// Initialize the flavor manager with an LSP client and start watching
    pub async fn initialize(&mut self, client: Option<Client>) -> Result<()> {
        self.client = client;

        // Load all flavors
        self.load_all_flavors().await?;

        // Start file watching
        self.start_watching().await?;

        Ok(())
    }

    /// Get the effective default flavor based on configuration priority
    pub async fn get_effective_default_flavor(&self) -> Option<LoadedFlavor> {
        // Priority: CLI > Project Config > Built-in Default
        if let Some(cli_flavor) = &self.selection_config.cli_flavor {
            if let Some(flavor) = self.get_flavor(cli_flavor).await {
                return Some(flavor);
            }
            // Log warning if CLI flavor not found
            if let Some(client) = &self.client {
                client
                    .log_message(
                        MessageType::WARNING,
                        format!(
                            "CLI-specified flavor '{}' not found, falling back",
                            cli_flavor
                        ),
                    )
                    .await;
            }
        }

        if let Some(project_flavor) = &self.selection_config.project_flavor {
            if let Some(flavor) = self.get_flavor(project_flavor).await {
                return Some(flavor);
            }
            // Log warning if project flavor not found
            if let Some(client) = &self.client {
                let config_path = self
                    .selection_config
                    .project_config_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| ".gcode.toml".to_string());
                client
                    .log_message(
                        MessageType::WARNING,
                        format!(
                            "Project flavor '{}' from {} not found, falling back",
                            project_flavor, config_path
                        ),
                    )
                    .await;
            }
        }

        // Fall back to built-in default
        self.get_default_flavor().await
    }

    /// Load all flavors from built-in and external sources
    async fn load_all_flavors(&mut self) -> Result<()> {
        let mut flavors = HashMap::new();

        // Load built-in flavors first (lowest priority)
        self.load_built_in_flavors(&mut flavors).await?;

        // Load user flavors (higher priority)
        for (i, flavor_dir) in self.flavor_dirs.iter().enumerate() {
            let priority = if i == 0 {
                FlavorPriority::UserGlobal
            } else {
                FlavorPriority::Workspace
            };
            self.load_flavors_from_directory(flavor_dir, priority, &mut flavors)
                .await?;
        }

        // Update the shared flavor map
        let mut shared_flavors = self.flavors.write().await;
        *shared_flavors = flavors;

        if let Some(client) = &self.client {
            let count = shared_flavors.len();
            client
                .log_message(
                    MessageType::INFO,
                    format!("Loaded {} G-code flavors", count),
                )
                .await;
        }

        Ok(())
    }

    /// Load built-in flavors embedded in the binary
    async fn load_built_in_flavors(
        &self,
        flavors: &mut HashMap<String, LoadedFlavor>,
    ) -> Result<()> {
        // Load the built-in Prusa flavor
        let prusa_content = include_str!("../docs/work/samples/prusa.gcode-flavor.toml");
        match self.parse_flavor_content(prusa_content, None) {
            Ok(flavor) => {
                let loaded_flavor = LoadedFlavor {
                    flavor: flavor.clone(),
                    priority: FlavorPriority::BuiltIn,
                    source_path: None,
                };
                flavors.insert(flavor.flavor.name.clone(), loaded_flavor);
            }
            Err(e) => {
                if let Some(client) = &self.client {
                    client
                        .log_message(
                            MessageType::ERROR,
                            format!("Failed to load built-in Prusa flavor: {}", e),
                        )
                        .await;
                }
            }
        }

        Ok(())
    }

    /// Load flavors from a specific directory
    async fn load_flavors_from_directory(
        &self,
        dir: &Path,
        priority: FlavorPriority,
        flavors: &mut HashMap<String, LoadedFlavor>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(dir)
            .await
            .with_context(|| format!("Failed to read flavor directory: {}", dir.display()))?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Err(e) = self.load_flavor_file(&path, priority, flavors).await {
                    if let Some(client) = &self.client {
                        client
                            .log_message(
                                MessageType::ERROR,
                                format!("Failed to load flavor file {}: {}", path.display(), e),
                            )
                            .await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single flavor file
    async fn load_flavor_file(
        &self,
        path: &Path,
        priority: FlavorPriority,
        flavors: &mut HashMap<String, LoadedFlavor>,
    ) -> Result<()> {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read flavor file: {}", path.display()))?;

        let flavor = self.parse_flavor_content(&content, Some(path))?;

        // Check if we should override an existing flavor based on priority
        let should_load = match flavors.get(&flavor.flavor.name) {
            Some(existing) => priority >= existing.priority,
            None => true,
        };

        if should_load {
            let loaded_flavor = LoadedFlavor {
                flavor,
                priority,
                source_path: Some(path.to_path_buf()),
            };
            flavors.insert(loaded_flavor.flavor.flavor.name.clone(), loaded_flavor);
        }

        Ok(())
    }

    /// Parse flavor content from TOML string
    fn parse_flavor_content(&self, content: &str, source_path: Option<&Path>) -> Result<Flavor> {
        toml::from_str(content).with_context(|| match source_path {
            Some(path) => format!("Failed to parse flavor TOML: {}", path.display()),
            None => "Failed to parse built-in flavor TOML".to_string(),
        })
    }

    /// Start file watching for flavor directories
    async fn start_watching(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.watcher_rx = Some(rx);

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if let EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) =
                        event.kind
                    {
                        for path in event.paths {
                            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                                let _ = tx.send(WatcherEvent::FlavorFileChanged(path));
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(WatcherEvent::WatcherError(e));
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        // Watch each flavor directory
        for dir in &self.flavor_dirs {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::NonRecursive)?;
            }
        }

        self._watcher = Some(watcher);

        // Start the background task to handle watcher events
        self.start_watcher_task().await;

        Ok(())
    }

    /// Start the background task that processes file watcher events
    async fn start_watcher_task(&mut self) {
        if let Some(mut rx) = self.watcher_rx.take() {
            let flavors = self.flavors.clone();
            let flavor_dirs = self.flavor_dirs.clone();
            let client = self.client.clone();

            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        WatcherEvent::FlavorFileChanged(path) => {
                            if let Some(client) = &client {
                                client
                                    .log_message(
                                        MessageType::INFO,
                                        format!("Flavor file changed: {}", path.display()),
                                    )
                                    .await;
                            }

                            // Reload all flavors when any file changes
                            // This is simpler than trying to reload just the changed file
                            // and handles dependencies correctly
                            Self::reload_flavors_static(
                                flavors.clone(),
                                &flavor_dirs,
                                client.as_ref(),
                            )
                            .await;
                        }
                        WatcherEvent::WatcherError(e) => {
                            if let Some(client) = &client {
                                client
                                    .log_message(
                                        MessageType::ERROR,
                                        format!("Flavor file watcher error: {}", e),
                                    )
                                    .await;
                            }
                        }
                    }
                }
            });
        }
    }

    /// Static method to reload flavors (used in the background task)
    async fn reload_flavors_static(
        flavors: Arc<RwLock<HashMap<String, LoadedFlavor>>>,
        flavor_dirs: &[PathBuf],
        client: Option<&Client>,
    ) {
        let mut new_flavors = HashMap::new();

        // Load built-in flavors
        let prusa_content = include_str!("../docs/work/samples/prusa.gcode-flavor.toml");
        if let Ok(flavor) = toml::from_str::<Flavor>(prusa_content) {
            let loaded_flavor = LoadedFlavor {
                flavor: flavor.clone(),
                priority: FlavorPriority::BuiltIn,
                source_path: None,
            };
            new_flavors.insert(flavor.flavor.name.clone(), loaded_flavor);
        }

        // Load user flavors
        for (i, flavor_dir) in flavor_dirs.iter().enumerate() {
            let priority = if i == 0 {
                FlavorPriority::UserGlobal
            } else {
                FlavorPriority::Workspace
            };

            if let Ok(mut entries) = tokio::fs::read_dir(flavor_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                        if let Ok(content) = tokio::fs::read_to_string(&path).await {
                            if let Ok(flavor) = toml::from_str::<Flavor>(&content) {
                                let should_load = match new_flavors.get(&flavor.flavor.name) {
                                    Some(existing) => priority >= existing.priority,
                                    None => true,
                                };

                                if should_load {
                                    let loaded_flavor = LoadedFlavor {
                                        flavor,
                                        priority,
                                        source_path: Some(path.clone()),
                                    };
                                    new_flavors.insert(
                                        loaded_flavor.flavor.flavor.name.clone(),
                                        loaded_flavor,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update the shared map
        {
            let mut shared_flavors = flavors.write().await;
            *shared_flavors = new_flavors;
        }

        if let Some(client) = client {
            client
                .log_message(MessageType::INFO, "Flavors reloaded due to file changes")
                .await;
        }
    }

    /// Get a flavor by name
    pub async fn get_flavor(&self, name: &str) -> Option<LoadedFlavor> {
        let flavors = self.flavors.read().await;
        flavors.get(name).cloned()
    }

    /// List all available flavor names
    pub async fn list_flavor_names(&self) -> Vec<String> {
        let flavors = self.flavors.read().await;
        flavors.keys().cloned().collect()
    }

    /// Get the default flavor (usually "prusa" if available)
    pub async fn get_default_flavor(&self) -> Option<LoadedFlavor> {
        let flavors = self.flavors.read().await;

        // Try to get "prusa" first
        if let Some(flavor) = flavors.get("prusa") {
            return Some(flavor.clone());
        }

        // Fall back to any available flavor
        flavors.values().next().cloned()
    }

    /// Convert a flavor to a command lookup map for the LSP backend
    pub fn flavor_to_command_map(&self, flavor: &Flavor) -> HashMap<String, CommandDef> {
        let mut map = HashMap::new();
        if let Some(commands) = &flavor.commands {
            for cmd in commands {
                map.insert(cmd.name.to_uppercase(), cmd.clone());
            }
        }
        map
    }

    /// Detect flavor from modeline in document content
    pub fn detect_modeline_flavor(&self, content: &str) -> Option<String> {
        // Check first and last few lines for modeline
        let lines: Vec<&str> = content.lines().collect();
        let check_lines: Vec<&str> = if lines.len() <= 10 {
            lines
        } else {
            // Check first 5 and last 5 lines
            let mut check = Vec::new();
            check.extend_from_slice(&lines[0..5]);
            check.extend_from_slice(&lines[lines.len() - 5..]);
            check
        };

        let modeline_re = regex::Regex::new(r"gcode_flavor\s*=\s*(\w+)");

        for line in check_lines {
            // Look for patterns like:
            // ; vim: gcode_flavor=prusa
            // ; gcode_flavor=prusa
            // // gcode_flavor=prusa
            if let Some(captures) = modeline_re.as_ref().ok()?.captures(line) {
                return Some(captures.get(1)?.as_str().to_string());
            }
        }

        None
    }
}

impl Default for FlavorManager {
    fn default() -> Self {
        Self::with_default_config().expect("Failed to create default FlavorManager")
    }
}
