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
        // Command exists, validate parameters
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

            // Special validation for movement commands (G0, G1) - require at least one coordinate
            if cmd.name == "G0" || cmd.name == "G1" {
                let has_coordinate = cmd.parameters.iter().any(|p| {
                    let param_name = p.letter.to_string().to_uppercase();
                    param_name == "X" || param_name == "Y" || param_name == "Z"
                });

                if !has_coordinate {
                    result.add_error(
                        line_num,
                        format!("Movement command '{}' requires at least one coordinate parameter (X, Y, or Z)", cmd.name),
                    );
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
}
