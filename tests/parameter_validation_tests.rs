use std::collections::HashMap;
use gcode_language_server::flavor::{CommandDef, ParameterDef, ParameterType, ParameterConstraints};
use gcode_language_server::gcode::{validate_line, validate_text, ValidationError, ValidationWarning};

/// Helper to create a simple command definition for testing
fn create_test_command(name: &str, params: Vec<ParameterDef>) -> CommandDef {
    CommandDef {
        name: name.to_string(),
        pattern: Some(format!("^{}( .*)?$", name)),
        description_short: Some(format!("Test command {}", name)),
        description_long: Some(format!("Test command {} for validation testing", name)),
        parameters: if params.is_empty() { None } else { Some(params) },
    }
}

/// Helper to create a parameter definition for testing
fn create_test_parameter(
    name: &str, 
    param_type: ParameterType, 
    required: bool,
    constraints: Option<ParameterConstraints>
) -> ParameterDef {
    ParameterDef {
        name: name.to_string(),
        param_type,
        required,
        description: format!("Test parameter {}", name),
        constraints,
        default_value: None,
        aliases: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_type_validation() {
        let mut commands = HashMap::new();
        
        // Create G1 command with float parameters
        let g1_params = vec![
            create_test_parameter("X", ParameterType::Float, false, None),
            create_test_parameter("Y", ParameterType::Float, false, None),
        ];
        commands.insert("G1".to_string(), create_test_command("G1", g1_params));
        
        // Test valid float parameters
        let validated_tokens = validate_line("G1 X10.5 Y20.3", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 0, "Valid float parameters should not generate errors");
        
        // Test invalid parameter type - letters in number
        let validated_tokens = validate_line("G1 X10.5a Y20.3", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 1, "Invalid parameter type should generate one error");
    }

    #[test]
    fn test_parameter_constraints() {
        let mut commands = HashMap::new();
        
        // Create G1 command with constrained parameters
        let constraints = ParameterConstraints {
            min_value: Some(0.0),
            max_value: Some(100.0),
            enum_values: None,
            pattern: None,
        };
        let g1_params = vec![
            create_test_parameter("X", ParameterType::Float, false, Some(constraints)),
        ];
        commands.insert("G1".to_string(), create_test_command("G1", g1_params));
        
        // Test value within constraints
        let validated_tokens = validate_line("G1 X50.0", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 0, "Value within constraints should not generate errors");
        
        // Test value below minimum
        let validated_tokens = validate_line("G1 X-10.0", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 1, "Value below minimum should generate constraint violation error");
        
        // Test value above maximum
        let validated_tokens = validate_line("G1 X150.0", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 1, "Value above maximum should generate constraint violation error");
    }

    #[test]
    fn test_required_parameters() {
        let mut commands = HashMap::new();
        
        // Create command with required parameter
        let m104_params = vec![
            create_test_parameter("S", ParameterType::Float, true, None), // Required temperature
        ];
        commands.insert("M104".to_string(), create_test_command("M104", m104_params));
        
        // Test with required parameter present
        let validated_tokens = validate_line("M104 S200", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 0, "Command with required parameter should not generate errors");
        
        // Test with required parameter missing
        let validated_tokens = validate_line("M104", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 1, "Command without required parameter should generate error");
        
        if let Some(error) = errors.first() {
            match error {
                ValidationError::MissingRequiredParameter { param, command, .. } => {
                    assert_eq!(param, "S");
                    assert_eq!(command, "M104");
                }
                _ => panic!("Expected MissingRequiredParameter error"),
            }
        }
    }

    #[test]
    fn test_unknown_command() {
        let commands = HashMap::new(); // Empty command definitions
        
        let validated_tokens = validate_line("G999 X10", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        
        assert_eq!(errors.len(), 2, "Unknown command should generate errors for command and parameter");
        
        // Check that we get an unknown command error and unknown parameter error
        let has_unknown_command = errors.iter().any(|e| matches!(e, ValidationError::UnknownCommand { .. }));
        let has_unknown_parameter = errors.iter().any(|e| matches!(e, ValidationError::UnknownParameter { .. }));
        
        assert!(has_unknown_command, "Should have unknown command error");
        assert!(has_unknown_parameter, "Should have unknown parameter error");
    }

    #[test]
    fn test_unknown_parameter() {
        let mut commands = HashMap::new();
        
        // Create G1 command with only X parameter
        let g1_params = vec![
            create_test_parameter("X", ParameterType::Float, false, None),
        ];
        commands.insert("G1".to_string(), create_test_command("G1", g1_params));
        
        // Test with known parameter
        let validated_tokens = validate_line("G1 X10", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 0, "Known parameter should not generate errors");
        
        // Test with unknown parameter
        let validated_tokens = validate_line("G1 Z10", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 1, "Unknown parameter should generate error");
        
        if let Some(error) = errors.first() {
            match error {
                ValidationError::UnknownParameter { param, command, .. } => {
                    assert_eq!(param, "Z");
                    assert_eq!(command, "G1");
                }
                _ => panic!("Expected UnknownParameter error"),
            }
        }
    }

    #[test]
    fn test_boolean_parameters() {
        let mut commands = HashMap::new();
        
        // Create G28 command with boolean parameters
        let g28_params = vec![
            create_test_parameter("X", ParameterType::Bool, false, None),
            create_test_parameter("Y", ParameterType::Bool, false, None),
        ];
        commands.insert("G28".to_string(), create_test_command("G28", g28_params));
        
        // Test with boolean-style parameters (no explicit value)
        let validated_tokens = validate_line("G28 X Y", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 0, "Boolean parameters without values should not generate errors");
        
        // Test with explicit boolean values
        let validated_tokens = validate_line("G28 X1 Y0", 0, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        assert_eq!(errors.len(), 0, "Explicit boolean values should not generate errors");
    }

    #[test]
    fn test_multi_line_validation() {
        let mut commands = HashMap::new();
        
        // Create some test commands
        let g1_params = vec![
            create_test_parameter("X", ParameterType::Float, false, None),
            create_test_parameter("Y", ParameterType::Float, false, None),
        ];
        commands.insert("G1".to_string(), create_test_command("G1", g1_params));
        commands.insert("G28".to_string(), create_test_command("G28", vec![]));
        
        let gcode_text = "G28\nG1 X10 Y20\nG999 Z30"; // Valid, valid, invalid command with invalid param
        
        let validated_tokens = validate_text(gcode_text, &commands);
        let errors: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.errors)
            .collect();
        
        assert_eq!(errors.len(), 2, "Should have 2 errors: unknown command and unknown parameter");
        
        let has_unknown_command = errors.iter().any(|e| {
            matches!(e, ValidationError::UnknownCommand { command, .. } if command == "G999")
        });
        let has_unknown_parameter = errors.iter().any(|e| {
            matches!(e, ValidationError::UnknownParameter { param, .. } if param == "Z")
        });
        
        assert!(has_unknown_command, "Should detect unknown command G999");
        assert!(has_unknown_parameter, "Should detect unknown parameter Z");
    }

    #[test]
    fn test_parameter_matching() {
        let param_def = create_test_parameter("X", ParameterType::Float, false, None);
        
        // Test exact match
        assert!(param_def.matches_name("X"));
        assert!(param_def.matches_name("x")); // Case insensitive
        
        // Test non-match
        assert!(!param_def.matches_name("Y"));
        assert!(!param_def.matches_name("Z"));
    }

    #[test]
    fn test_parameter_value_validation() {
        // Test float parameter validation
        let float_param = create_test_parameter("X", ParameterType::Float, false, None);
        assert!(float_param.validate_value("10.5").is_ok());
        assert!(float_param.validate_value("-5.3").is_ok());
        assert!(float_param.validate_value("abc").is_err());
        
        // Test int parameter validation
        let int_param = create_test_parameter("S", ParameterType::Int, false, None);
        assert!(int_param.validate_value("10").is_ok());
        assert!(int_param.validate_value("-5").is_ok());
        assert!(int_param.validate_value("10.5").is_err());
        assert!(int_param.validate_value("abc").is_err());
        
        // Test bool parameter validation
        let bool_param = create_test_parameter("B", ParameterType::Bool, false, None);
        assert!(bool_param.validate_value("true").is_ok());
        assert!(bool_param.validate_value("false").is_ok());
        assert!(bool_param.validate_value("1").is_ok());
        assert!(bool_param.validate_value("0").is_ok());
        assert!(bool_param.validate_value("on").is_ok());
        assert!(bool_param.validate_value("off").is_ok());
        assert!(bool_param.validate_value("maybe").is_err());
    }

    #[test]
    fn test_move_commands_without_coordinates() {
        let mut commands = HashMap::new();
        
        // Create G0 and G1 commands with coordinate parameters
        let move_params = vec![
            create_test_parameter("X", ParameterType::Float, false, None),
            create_test_parameter("Y", ParameterType::Float, false, None),
            create_test_parameter("F", ParameterType::Float, false, None), // Feedrate is not a coordinate
        ];
        commands.insert("G0".to_string(), create_test_command("G0", move_params.clone()));
        commands.insert("G1".to_string(), create_test_command("G1", move_params));
        
        // Test G0 with coordinates - should not generate warning
        let validated_tokens = validate_line("G0 X10 Y20", 0, &commands);
        let warnings: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.warnings)
            .collect();
        assert_eq!(warnings.len(), 0, "G0 with coordinates should not generate warnings");
        
        // Test G0 without coordinates - should generate warning
        let validated_tokens = validate_line("G0", 0, &commands);
        let warnings: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.warnings)
            .collect();
        assert_eq!(warnings.len(), 1, "G0 without coordinates should generate warning");
        
        // Test G0 with only feedrate (no coordinates) - should generate warning
        let validated_tokens = validate_line("G0 F1500", 0, &commands);
        let warnings: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.warnings)
            .collect();
        assert_eq!(warnings.len(), 1, "G0 with only feedrate should generate warning");
        
        // Test G1 without coordinates - should generate warning
        let validated_tokens = validate_line("G1", 0, &commands);
        let warnings: Vec<_> = validated_tokens.iter()
            .filter_map(|vt| vt.validation.as_ref())
            .flat_map(|v| &v.warnings)
            .collect();
        assert_eq!(warnings.len(), 1, "G1 without coordinates should generate warning");
        
        // Verify warning type
        if let Some(warning) = warnings.first() {
            match warning {
                ValidationWarning::MoveCommandWithoutCoordinates { command, .. } => {
                    assert!(matches!(command.as_str(), "G0" | "G1"));
                }
                _ => panic!("Expected MoveCommandWithoutCoordinates warning"),
            }
        }
    }
}