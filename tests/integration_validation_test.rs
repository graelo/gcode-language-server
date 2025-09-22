use std::collections::HashMap;
use gcode_language_server::flavor::{CommandDef, ParameterDef, ParameterType, ParameterConstraints};
use gcode_language_server::gcode::{validate_text, ValidationError};

#[test]
fn test_integration_with_prusa_flavor_style() {
    let mut commands = HashMap::new();
    
    // Create commands similar to our enhanced Prusa flavor
    
    // G1 command with X, Y, Z, E, F parameters
    let g1_params = vec![
        ParameterDef {
            name: "X".to_string(),
            param_type: ParameterType::Float,
            required: false,
            description: "X-axis destination".to_string(),
            constraints: Some(ParameterConstraints {
                min_value: Some(-999.0),
                max_value: Some(999.0),
                enum_values: None,
                pattern: None,
            }),
            default_value: None,
            aliases: None,
        },
        ParameterDef {
            name: "Y".to_string(),
            param_type: ParameterType::Float,
            required: false,
            description: "Y-axis destination".to_string(),
            constraints: Some(ParameterConstraints {
                min_value: Some(-999.0),
                max_value: Some(999.0),
                enum_values: None,
                pattern: None,
            }),
            default_value: None,
            aliases: None,
        },
        ParameterDef {
            name: "F".to_string(),
            param_type: ParameterType::Float,
            required: false,
            description: "Feedrate".to_string(),
            constraints: Some(ParameterConstraints {
                min_value: Some(1.0),
                max_value: Some(10000.0),
                enum_values: None,
                pattern: None,
            }),
            default_value: None,
            aliases: None,
        },
    ];
    
    commands.insert("G1".to_string(), CommandDef {
        name: "G1".to_string(),
        pattern: Some("^G1( .*)?$".to_string()),
        description_short: Some("Controlled linear move".to_string()),
        description_long: Some("G1 moves the toolhead linearly".to_string()),
        parameters: Some(g1_params),
    });
    
    // G28 command with boolean parameters
    let g28_params = vec![
        ParameterDef {
            name: "X".to_string(),
            param_type: ParameterType::Bool,
            required: false,
            description: "Home X axis".to_string(),
            constraints: None,
            default_value: Some("false".to_string()),
            aliases: None,
        },
    ];
    
    commands.insert("G28".to_string(), CommandDef {
        name: "G28".to_string(),
        pattern: Some("^G28( .*)?$".to_string()),
        description_short: Some("Home axes".to_string()),
        description_long: Some("G28 homes printer axes".to_string()),
        parameters: Some(g28_params),
    });
    
    // Test a realistic G-code file
    let gcode_content = r#"G28 X ; Home X axis
G1 X100 Y50 F1800 ; Move to position
G1 X-2000 ; This should trigger constraint violation
G999 Z10 ; Unknown command
G1 Q20 ; Unknown parameter for G1"#;
    
    let validated_tokens = validate_text(gcode_content, &commands);
    let errors: Vec<_> = validated_tokens.iter()
        .filter_map(|vt| vt.validation.as_ref())
        .flat_map(|v| &v.errors)
        .collect();
    
    println!("Found {} validation errors:", errors.len());
    for error in &errors {
        match error {
            ValidationError::UnknownCommand { command, .. } => {
                println!("  - Unknown command: {}", command);
            }
            ValidationError::UnknownParameter { param, command, .. } => {
                println!("  - Unknown parameter '{}' for command '{}'", param, command);
            }
            ValidationError::ConstraintViolation { param, value, constraint, .. } => {
                println!("  - Constraint violation: {} = {} ({})", param, value, constraint);
            }
            _ => {
                println!("  - Other error: {:?}", error);
            }
        }
    }
    
    // Verify we found the expected errors
    assert!(errors.len() >= 3, "Should find at least 3 errors in the test G-code");
    
    // Check for specific expected errors
    let has_constraint_violation = errors.iter().any(|e| {
        matches!(e, ValidationError::ConstraintViolation { param, .. } if param == "X")
    });
    let has_unknown_command = errors.iter().any(|e| {
        matches!(e, ValidationError::UnknownCommand { command, .. } if command == "G999")
    });
    let has_unknown_parameter = errors.iter().any(|e| {
        matches!(e, ValidationError::UnknownParameter { param, command, .. } if param == "Q" && command == "G1")
    });
    
    assert!(has_constraint_violation, "Should detect X parameter constraint violation");
    assert!(has_unknown_command, "Should detect unknown command G999");
    assert!(has_unknown_parameter, "Should detect unknown parameter Q for G1");
}