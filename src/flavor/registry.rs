//! Flavor Registry
//!
//! Simple in-memory registry - much cleaner than the complex async version.

use super::schema::{CommandDef, Flavor};
use std::collections::HashMap;

/// Simple in-memory flavor registry
#[derive(Debug, Clone)]
pub struct FlavorRegistry {
    flavors: HashMap<String, Flavor>,
    active_flavor: Option<String>,
}

impl Default for FlavorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FlavorRegistry {
    pub fn new() -> Self {
        Self {
            flavors: HashMap::new(),
            active_flavor: None,
        }
    }

    /// Add a flavor to the registry
    pub fn add_flavor(&mut self, flavor: Flavor) {
        self.flavors.insert(flavor.name.clone(), flavor);
    }

    /// Set the active flavor
    pub fn set_active_flavor(&mut self, name: &str) -> bool {
        if self.flavors.contains_key(name) {
            self.active_flavor = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Get the currently active flavor
    pub fn get_active_flavor(&self) -> Option<&Flavor> {
        self.active_flavor
            .as_ref()
            .and_then(|name| self.flavors.get(name))
    }

    /// List all available flavors
    pub fn list_flavors(&self) -> Vec<&str> {
        self.flavors.keys().map(|s| s.as_str()).collect()
    }

    /// Get command definition from active flavor
    pub fn get_command(&self, name: &str) -> Option<&CommandDef> {
        self.get_active_flavor()?.commands.get(name)
    }

    /// Add comprehensive embedded Prusa flavor with rich command definitions
    pub fn add_embedded_prusa_flavor(&mut self) {
        use crate::flavor::schema::{Flavor, FlavorFile};

        // Load embedded TOML content
        let embedded_toml = include_str!("../../resources/flavors/prusa.gcode-flavor.toml");

        // Parse the embedded TOML into a FlavorFile
        match toml::from_str::<FlavorFile>(embedded_toml) {
            Ok(flavor_file) => {
                let flavor = Flavor::from(flavor_file);
                self.add_flavor(flavor);
            }
            Err(e) => {
                // Fallback to minimal flavor if parsing fails
                log::warn!(
                    "Failed to parse embedded Prusa flavor: {}. Using minimal fallback.",
                    e
                );
                self.add_minimal_prusa_flavor();
            }
        }
    }

    /// Add minimal fallback Prusa flavor in case embedded TOML parsing fails
    fn add_minimal_prusa_flavor(&mut self) {
        use crate::flavor::schema::{CommandDef, Flavor};
        use std::collections::HashMap;

        let mut commands = HashMap::new();

        commands.insert(
            "G0".to_string(),
            CommandDef {
                name: "G0".to_string(),
                description_short: Some("Rapid positioning".to_string()),
                description_long: Some(
                    "Move to position at rapid rate without extrusion".to_string(),
                ),
                parameters: None,
                constraints: None,
            },
        );

        commands.insert(
            "G1".to_string(),
            CommandDef {
                name: "G1".to_string(),
                description_short: Some("Linear move".to_string()),
                description_long: Some("Linear move with extrusion".to_string()),
                parameters: None,
                constraints: None,
            },
        );

        let flavor = Flavor {
            name: "prusa".to_string(),
            version: Some("minimal-fallback".to_string()),
            description: Some("Minimal fallback Prusa flavor".to_string()),
            commands,
        };

        self.add_flavor(flavor);
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

        for line in check_lines {
            // Look for patterns like:
            // ; vim: gcode_flavor=prusa
            // ; gcode_flavor=prusa
            // // gcode_flavor=prusa
            if let Some(flavor_name) = extract_flavor_from_modeline(line) {
                // Verify the flavor exists in registry
                if self.flavors.contains_key(&flavor_name) {
                    return Some(flavor_name);
                }
            }
        }

        None
    }
}

/// Extract flavor name from a modeline string
fn extract_flavor_from_modeline(line: &str) -> Option<String> {
    // Simple pattern matching for gcode_flavor=name
    if let Some(start) = line.find("gcode_flavor=") {
        let flavor_part = &line[start + 13..]; // Skip "gcode_flavor="
        let end = flavor_part
            .find(|c: char| c.is_whitespace() || c == ';' || c == '#')
            .unwrap_or(flavor_part.len());
        let flavor_name = &flavor_part[..end];

        if !flavor_name.is_empty()
            && flavor_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Some(flavor_name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flavor::schema::{CommandDef, Flavor, FlavorFile, FlavorMeta};

    #[test]
    fn test_registry_creation() {
        let registry = FlavorRegistry::new();
        assert!(registry.list_flavors().is_empty());
        assert!(registry.get_active_flavor().is_none());
    }

    #[test]
    fn test_add_and_activate_flavor() {
        let mut registry = FlavorRegistry::new();

        let file = FlavorFile {
            flavor: FlavorMeta {
                name: "test".to_string(),
                version: None,
                description: None,
            },
            commands: vec![],
        };
        let flavor = Flavor::from(file);

        registry.add_flavor(flavor);
        assert!(registry.set_active_flavor("test"));

        assert!(registry.get_active_flavor().is_some());
        assert_eq!(registry.get_active_flavor().unwrap().name, "test");
    }

    #[test]
    fn test_get_command() {
        let mut registry = FlavorRegistry::new();

        let file = FlavorFile {
            flavor: FlavorMeta {
                name: "test".to_string(),
                version: None,
                description: None,
            },
            commands: vec![CommandDef {
                name: "G1".to_string(),
                description_short: Some("Linear move".to_string()),
                description_long: None,
                parameters: None,
                constraints: None,
            }],
        };
        let flavor = Flavor::from(file);

        registry.add_flavor(flavor);
        assert!(registry.set_active_flavor("test"));

        let command = registry.get_command("G1");
        assert!(command.is_some());
        assert_eq!(command.unwrap().name, "G1");
        assert_eq!(
            command.unwrap().description_short,
            Some("Linear move".to_string())
        );
    }

    #[test]
    fn test_nonexistent_flavor() {
        let mut registry = FlavorRegistry::new();
        assert!(!registry.set_active_flavor("nonexistent"));
        assert!(registry.get_command("G1").is_none());
    }
}
