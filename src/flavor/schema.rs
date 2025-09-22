//! Flavor Schema Types
//!
//! Clean, simple types for flavor definitions - much simpler than the verbose legacy version.

use serde::Deserialize;
use std::collections::HashMap;

/// Root flavor file structure (matches TOML)
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FlavorFile {
    pub flavor: FlavorMeta,
    pub commands: Vec<CommandDef>,
}

/// Flavor metadata
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct FlavorMeta {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Runtime flavor (optimized for lookups)
#[derive(Debug, Clone, PartialEq)]
pub struct Flavor {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub commands: HashMap<String, CommandDef>,
}

/// GCode command definition
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CommandDef {
    pub name: String,
    pub description_short: Option<String>,
    pub description_long: Option<String>,
    pub parameters: Option<Vec<ParameterDef>>,
}

/// Command parameter definition
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ParameterDef {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    #[serde(default)]
    pub required: bool,
    pub description: String,
    pub constraints: Option<ParameterConstraints>,
    pub aliases: Option<Vec<String>>,
}

/// Parameter data types
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Int,
    Float,
    String,
    Bool,
}

/// Parameter validation constraints
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ParameterConstraints {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub enum_values: Option<Vec<String>>,
}

impl From<FlavorFile> for Flavor {
    fn from(file: FlavorFile) -> Self {
        // Convert to HashMap for fast lookups
        let commands = file
            .commands
            .into_iter()
            .map(|cmd| (cmd.name.clone(), cmd))
            .collect();

        Self {
            name: file.flavor.name,
            version: file.flavor.version,
            description: file.flavor.description,
            commands,
        }
    }
}

impl CommandDef {
    /// Find parameter by name (including aliases)
    pub fn find_parameter(&self, name: &str) -> Option<&ParameterDef> {
        self.parameters
            .as_ref()?
            .iter()
            .find(|param| param.matches_name(name))
    }

    /// Get required parameters
    pub fn required_parameters(&self) -> Vec<&ParameterDef> {
        self.parameters
            .as_ref()
            .map(|params| params.iter().filter(|p| p.required).collect())
            .unwrap_or_default()
    }
}

impl ParameterDef {
    /// Check if parameter matches name (including aliases)
    pub fn matches_name(&self, name: &str) -> bool {
        if self.name.eq_ignore_ascii_case(name) {
            return true;
        }

        self.aliases
            .as_ref()
            .map(|aliases| aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name)))
            .unwrap_or(false)
    }

    /// Validate parameter value
    pub fn validate(&self, value: &str) -> Result<(), String> {
        // Type validation and constraint checking
        match self.param_type {
            ParameterType::Int => {
                let val: i64 = value.parse().map_err(|_| {
                    format!("Parameter '{}' expects integer, got '{}'", self.name, value)
                })?;

                // Check constraints (convert to float for comparison)
                if let Some(constraints) = &self.constraints {
                    let val_f = val as f64;
                    if let Some(min) = constraints.min_value {
                        if val_f < min {
                            return Err(format!(
                                "Parameter '{}' value {} below minimum {}",
                                self.name, val, min
                            ));
                        }
                    }
                    if let Some(max) = constraints.max_value {
                        if val_f > max {
                            return Err(format!(
                                "Parameter '{}' value {} exceeds maximum {}",
                                self.name, val, max
                            ));
                        }
                    }
                }
            }
            ParameterType::Float => {
                let val: f64 = value.parse().map_err(|_| {
                    format!("Parameter '{}' expects number, got '{}'", self.name, value)
                })?;

                // Check constraints
                if let Some(constraints) = &self.constraints {
                    if let Some(min) = constraints.min_value {
                        if val < min {
                            return Err(format!(
                                "Parameter '{}' value {} below minimum {}",
                                self.name, val, min
                            ));
                        }
                    }
                    if let Some(max) = constraints.max_value {
                        if val > max {
                            return Err(format!(
                                "Parameter '{}' value {} exceeds maximum {}",
                                self.name, val, max
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
                                "Parameter '{}' value '{}' not in allowed values: {}",
                                self.name,
                                value,
                                enum_values.join(", ")
                            ));
                        }
                    }
                }
            }
            ParameterType::Bool => {
                // Bool parameters often don't have values in GCode
                if !value.is_empty() {
                    return Err(format!(
                        "Parameter '{}' is boolean and should not have value '{}'",
                        self.name, value
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flavor_from_file() {
        let file = FlavorFile {
            flavor: FlavorMeta {
                name: "test".to_string(),
                version: Some("1.0".to_string()),
                description: None,
            },
            commands: vec![CommandDef {
                name: "G1".to_string(),
                description_short: Some("Linear move".to_string()),
                description_long: None,
                parameters: None,
            }],
        };

        let flavor = Flavor::from(file);
        assert_eq!(flavor.name, "test");
        assert_eq!(flavor.commands.len(), 1);
        assert!(flavor.commands.contains_key("G1"));
    }

    #[test]
    fn test_parameter_matches_name() {
        let param = ParameterDef {
            name: "X".to_string(),
            param_type: ParameterType::Float,
            required: false,
            description: "X coordinate".to_string(),
            constraints: None,
            aliases: Some(vec!["x".to_string()]),
        };

        assert!(param.matches_name("X"));
        assert!(param.matches_name("x"));
        assert!(param.matches_name("X"));
        assert!(!param.matches_name("Y"));
    }

    #[test]
    fn test_parameter_validation() {
        let param = ParameterDef {
            name: "S".to_string(),
            param_type: ParameterType::Int,
            required: false,
            description: "Speed".to_string(),
            constraints: Some(ParameterConstraints {
                min_value: Some(0.0),
                max_value: Some(255.0),
                enum_values: None,
            }),
            aliases: None,
        };

        assert!(param.validate("100").is_ok());
        assert!(param.validate("300").is_err()); // Above max
        assert!(param.validate("-10").is_err()); // Below min
        assert!(param.validate("abc").is_err()); // Not a number
    }

    #[test]
    fn test_command_find_parameter() {
        let cmd = CommandDef {
            name: "G1".to_string(),
            description_short: None,
            description_long: None,
            parameters: Some(vec![ParameterDef {
                name: "X".to_string(),
                param_type: ParameterType::Float,
                required: false,
                description: "X coordinate".to_string(),
                constraints: None,
                aliases: None,
            }]),
        };

        assert!(cmd.find_parameter("X").is_some());
        assert!(cmd.find_parameter("Y").is_none());
    }
}
