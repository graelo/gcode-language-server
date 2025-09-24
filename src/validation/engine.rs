//! Validation Engine
//!
//! Core validation logic separated from parsing and LSP concerns.

use crate::flavor::FlavorRegistry;
use crate::parser::{Command, ParsedLine};

/// Severity of a diagnostic message
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A diagnostic message for a validation issue
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
    pub severity: Severity,
}

/// Result of validating a document or line
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn add_error(&mut self, line: usize, message: String) {
        self.diagnostics.push(Diagnostic {
            line,
            message,
            severity: Severity::Error,
        });
    }

    pub fn add_warning(&mut self, line: usize, message: String) {
        self.diagnostics.push(Diagnostic {
            line,
            message,
            severity: Severity::Warning,
        });
    }

    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

/// Validate a single line of GCode
pub fn validate_line(
    line_num: usize,
    parsed: &ParsedLine,
    flavor: &FlavorRegistry,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    match parsed {
        ParsedLine::Command(cmd) => {
            validate_command(line_num, cmd, flavor, &mut result);
        }
        ParsedLine::Comment(_) | ParsedLine::Empty => {
            // Comments and empty lines are always valid
        }
    }

    result
}

/// Validate an entire document
pub fn validate_document(content: &str, flavor: &FlavorRegistry) -> ValidationResult {
    let mut result = ValidationResult::new();

    for (line_num, line) in content.lines().enumerate() {
        let parsed = crate::parser::parse_line(line);
        let line_result = validate_line(line_num + 1, &parsed, flavor);
        result.diagnostics.extend(line_result.diagnostics);
    }

    result
}

/// Validate a command using the flavor registry
fn validate_command(
    line_num: usize,
    cmd: &Command,
    flavor: &FlavorRegistry,
    result: &mut ValidationResult,
) {
    // Check if command exists in the active flavor
    if let Some(command_def) = flavor.get_command(&cmd.name) {
        // Command exists, validate parameters and constraints
        
        // Validate parameter constraints (independent of parameter definitions)
        let cmd_param_names: Vec<String> = cmd
            .parameters
            .iter()
            .map(|p| p.letter.to_string().to_uppercase())
            .collect();
        
        let constraint_errors = command_def.validate_constraints(&cmd_param_names);
        
        for error in constraint_errors {
            result.add_error(line_num, error);
        }
        
        // Validate individual parameters if they're defined
        if let Some(expected_params) = &command_def.parameters {
            // Check for required parameters
            for expected_param in expected_params {
                if expected_param.required {
                    let found = cmd
                        .parameters
                        .iter()
                        .any(|p| p.letter.to_string().to_uppercase() == expected_param.name);
                    if !found {
                        result.add_error(
                            line_num,
                            format!(
                                "Missing required parameter '{}' for command '{}'",
                                expected_param.name, cmd.name
                            ),
                        );
                    }
                }
            }

            // Check for unknown parameters
            for actual_param in &cmd.parameters {
                let param_name = actual_param.letter.to_string().to_uppercase();
                let found = expected_params.iter().any(|p| p.name == param_name);
                if !found {
                    result.add_warning(
                        line_num,
                        format!(
                            "Unknown parameter '{}' for command '{}'",
                            param_name, cmd.name
                        ),
                    );
                }
            }
        }
    } else {
        // Unknown command
        result.add_warning(line_num, format!("Unknown command '{}'", cmd.name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::parser::{Command, Parameter, Comment};

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());

        result.add_warning(1, "Test warning".to_string());
        assert!(result.is_valid()); // Warnings don't make it invalid

        result.add_error(2, "Test error".to_string());
        assert!(!result.is_valid()); // Errors make it invalid
    }

    #[test]
    fn test_validate_empty_line() {
        let registry = FlavorRegistry::new(); // Will implement this
        let result = validate_line(1, &ParsedLine::Empty, &registry);
        assert!(result.is_valid());
    }

    #[test]
    fn test_constraint_validation() {
        use crate::flavor::schema::{CommandDef, ParameterConstraint, ConstraintType};
        use crate::parser::{Command, Parameter};

        // Create a mock flavor registry with constraint-enabled G0 command
        let mut registry = FlavorRegistry::new();
        let mut commands = std::collections::HashMap::new();
        
        let g0_cmd = CommandDef {
            name: "G0".to_string(),
            description_short: Some("Rapid positioning".to_string()),
            description_long: None,
            parameters: None,
            constraints: Some(vec![ParameterConstraint {
                constraint_type: ConstraintType::RequireAnyOf,
                parameters: vec!["X".to_string(), "Y".to_string(), "Z".to_string()],
                message: Some("Movement command requires at least one coordinate parameter (X, Y, or Z)".to_string()),
            }]),
        };
        
        commands.insert("G0".to_string(), g0_cmd);
        
        let flavor = crate::flavor::schema::Flavor {
            name: "test".to_string(),
            version: None,
            description: None,
            commands,
        };
        
        registry.add_flavor(flavor);
        registry.set_active_flavor("test");

        // Test 1: Valid G0 command with X parameter
        let valid_cmd = Command {
            name: "G0".to_string(),
            parameters: vec![Parameter {
                letter: 'X',
                value: "10.0".to_string(),
            }],
            comment: None,
        };
        
        let mut result = ValidationResult::new();
        validate_command(1, &valid_cmd, &registry, &mut result);
        assert!(result.is_valid(), "G0 with X parameter should be valid");

        // Test 2: Invalid G0 command with no coordinates (only F parameter)
        let invalid_cmd = Command {
            name: "G0".to_string(),
            parameters: vec![Parameter {
                letter: 'F',
                value: "1000.0".to_string(),
            }],
            comment: None,
        };
        
        let mut result = ValidationResult::new();
        validate_command(1, &invalid_cmd, &registry, &mut result);
        
        assert!(!result.is_valid(), "G0 without coordinates should be invalid");
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("requires at least one coordinate"));

        // Test 3: Valid G0 command with multiple coordinates
        let valid_multi_cmd = Command {
            name: "G0".to_string(),
            parameters: vec![
                Parameter {
                    letter: 'X',
                    value: "10.0".to_string(),
                },
                Parameter {
                    letter: 'Y',
                    value: "20.0".to_string(),
                },
            ],
            comment: None,
        };
        
        let mut result = ValidationResult::new();
        validate_command(1, &valid_multi_cmd, &registry, &mut result);
        assert!(result.is_valid(), "G0 with multiple coordinates should be valid");
    }
}
