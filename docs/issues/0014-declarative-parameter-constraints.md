# Issue 0014: Declarative Parameter Constraints

**Priority**: Medium  
**Status**: Open  
**Created**: 2025-09-23  
**Assignee**: TBD  

## Summary

Replace hardcoded validation logic for movement commands with declarative parameter constraints in TOML flavor definitions. This will eliminate architectural mismatch between schema and validation logic while enabling extensible constraint types for complex parameter relationships.

## Background

### Current Problem

Movement commands (G0, G1) require special validation logic that's hardcoded in the validation engine:

```rust
// src/validation/engine.rs - hardcoded special case
if cmd.name == "G0" || cmd.name == "G1" {
    let has_coordinate = cmd.parameters.iter().any(|p| {
        let param_name = p.letter.to_string().to_uppercase();
        param_name == "X" || param_name == "Y" || param_name == "Z"
    });
    if !has_coordinate {
        result.add_error(/* requires at least one coordinate */);
    }
}
```

While in the TOML flavor definitions, all parameters are correctly marked as individually optional:

```toml
[[commands.parameters]]
name = "X"
type = "float"
required = false  # Correct - X is individually optional
description = "X coordinate"
```

### Architectural Issues

1. **Schema-Logic Mismatch**: TOML says parameters are optional, but code enforces "at least one required"
2. **Maintainability**: Each new constraint type requires code changes
3. **Scalability**: Hardcoded command names don't scale to more firmwares
4. **Testability**: Business logic scattered between schema and validation code
5. **Extensibility**: Cannot express complex parameter relationships declaratively

## Proposed Solution

### Declarative Constraints in TOML

Add a `constraints` section to command definitions that can express complex parameter relationships:

```toml
[[commands]]
name = "G0"
description_short = "Rapid positioning"
description_long = "Move to position at rapid rate without extrusion"

# Individual parameters remain optional (semantically correct)
[[commands.parameters]]
name = "X"
type = "float"
required = false
description = "X coordinate"

[[commands.parameters]]
name = "Y"
type = "float"
required = false
description = "Y coordinate"

[[commands.parameters]]
name = "Z"
type = "float"
required = false
description = "Z coordinate"

[[commands.parameters]]
name = "F"
type = "float"
required = false
description = "Feed rate"

# Express constraints declaratively
[[commands.constraints]]
type = "require_any_of"
parameters = ["X", "Y", "Z"]
message = "Movement command requires at least one coordinate parameter"
```

### Schema Extensions

**Enhanced CommandDef:**
```rust
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CommandDef {
    pub name: String,
    pub description_short: Option<String>,
    pub description_long: Option<String>,
    pub parameters: Option<Vec<ParameterDef>>,
    pub constraints: Option<Vec<ParameterConstraint>>,  // New field
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ParameterConstraint {
    #[serde(rename = "type")]
    pub constraint_type: ConstraintType,
    pub parameters: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ConstraintType {
    #[serde(rename = "require_any_of")]
    RequireAnyOf,           // At least one parameter from list required
    #[serde(rename = "require_all_of")]
    RequireAllOf,           // All parameters from list required
    #[serde(rename = "mutually_exclusive")]
    MutuallyExclusive,      // Only one parameter from list allowed
    #[serde(rename = "conditional_require")]
    ConditionalRequire,     // If condition met, require other parameters
}
```

### Generic Validation Logic

Replace hardcoded special cases with generic constraint processing:

```rust
// Generic constraint validation - no more hardcoded command names!
fn validate_parameter_constraints(
    cmd: &Command,
    command_def: &CommandDef,
    result: &mut ValidationResult,
    line_num: usize,
) {
    if let Some(constraints) = &command_def.constraints {
        for constraint in constraints {
            match constraint.constraint_type {
                ConstraintType::RequireAnyOf => {
                    let has_any = constraint.parameters.iter().any(|param_name| {
                        cmd.parameters.iter()
                            .any(|p| p.letter.to_string().to_uppercase() == *param_name)
                    });
                    
                    if !has_any {
                        let message = constraint.message.as_deref()
                            .unwrap_or("Command requires at least one of the specified parameters");
                        result.add_error(line_num, message.to_string());
                    }
                }
                ConstraintType::MutuallyExclusive => {
                    let present_params: Vec<_> = constraint.parameters.iter()
                        .filter(|param_name| {
                            cmd.parameters.iter()
                                .any(|p| p.letter.to_string().to_uppercase() == **param_name)
                        })
                        .collect();
                    
                    if present_params.len() > 1 {
                        let message = constraint.message.as_deref()
                            .unwrap_or("Parameters are mutually exclusive");
                        result.add_error(line_num, format!("{}: {}", message, present_params.join(", ")));
                    }
                }
                // Handle other constraint types...
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. **Schema Updates**
   - Add `ParameterConstraint` and `ConstraintType` to schema
   - Update `CommandDef` to include optional `constraints` field
   - Ensure backward compatibility with existing TOML files

2. **Validation Engine**
   - Implement generic constraint validation logic
   - Add constraint processing to main validation loop
   - Maintain existing behavior during transition

3. **Unit Tests**
   - Test constraint parsing from TOML
   - Test each constraint type independently
   - Test error message generation

### Phase 2: Migration
1. **Update Flavor Files**
   - Add constraints to G0/G1 commands in Prusa flavor
   - Add constraints to G0/G1 commands in Marlin flavor
   - Validate that new constraints produce identical behavior

2. **Remove Hardcoded Logic**
   - Remove special case handling for G0/G1 from validation engine
   - Update validation tests to expect constraint-driven errors
   - Verify no regression in validation behavior

3. **Integration Tests**
   - Test complete validation pipeline with constraints
   - Verify error messages are clear and helpful
   - Test performance impact (should be minimal)

### Phase 3: Advanced Constraints
1. **Extended Constraint Types**
   - Implement `ConditionalRequire` for complex relationships
   - Add numeric constraints (value ranges, relationships)
   - Support parameter dependency chains

2. **Enhanced Error Messages**
   - Context-aware error messages
   - Suggestions for fixing constraint violations
   - Multi-language support preparation

## Constraint Types and Use Cases

### RequireAnyOf
```toml
# Movement commands need at least one coordinate
[[commands.constraints]]
type = "require_any_of"
parameters = ["X", "Y", "Z"]
message = "Movement command requires at least one coordinate parameter"
```

### MutuallyExclusive
```toml
# Cannot specify both absolute and relative positioning
[[commands.constraints]]
type = "mutually_exclusive"
parameters = ["G90", "G91"]
message = "Cannot specify both absolute and relative positioning modes"
```

### RequireAllOf
```toml
# Arc commands need both I and J parameters
[[commands.constraints]]
type = "require_all_of"
parameters = ["I", "J"]
message = "Arc commands require both I and J parameters"
```

### ConditionalRequire (Future)
```toml
# If E is specified, need movement in X or Y
[[commands.constraints]]
type = "conditional_require"
if_parameter = "E"
then_require_any_of = ["X", "Y"]
message = "Extrusion requires movement in X or Y axis"
```

## Benefits

### Architectural Improvements
- **Unified Logic**: All validation rules expressed in schema
- **Maintainability**: No hardcoded command names in validation code
- **Extensibility**: Easy to add new constraint types
- **Consistency**: Same constraint system across all flavors

### Development Experience
- **Declarative**: Complex relationships expressed in TOML
- **Testable**: Constraints can be unit tested independently  
- **Debuggable**: Clear mapping from TOML to validation behavior
- **Scalable**: Works for any number of commands and firmwares

### User Experience
- **Better Errors**: Context-aware constraint violation messages
- **Consistency**: Same error format for all constraint types
- **Clarity**: Explicit requirements in flavor definitions

## Risks and Mitigations

### Performance Impact
- **Risk**: Additional constraint processing overhead
- **Mitigation**: Benchmark validation performance, optimize hot paths

### Complexity Increase
- **Risk**: More complex schema and validation logic
- **Mitigation**: Comprehensive documentation and examples

### Migration Effort
- **Risk**: Need to update all existing flavor files
- **Mitigation**: Backward compatibility, gradual migration

## Testing Strategy

1. **Unit Tests**
   - Each constraint type independently
   - TOML parsing and deserialization
   - Error message generation

2. **Integration Tests**
   - Complete validation pipeline with constraints
   - Migration from hardcoded to constraint-based validation
   - Performance benchmarks

3. **Regression Tests**
   - Existing G0/G1 validation behavior preserved
   - All current test cases continue to pass
   - Error message consistency

## Success Criteria

- [ ] All hardcoded validation logic replaced with declarative constraints
- [ ] G0/G1 validation behavior identical to current implementation
- [ ] Performance impact negligible (< 5% validation overhead)
- [ ] Clear error messages for all constraint violations
- [ ] Comprehensive test coverage for all constraint types
- [ ] Documentation and examples for flavor authors
- [ ] Backward compatibility with existing TOML files

## Dependencies

- No external dependencies
- Requires schema updates in `src/flavor/schema.rs`
- Requires validation engine updates in `src/validation/engine.rs`  
- Requires flavor file updates in `resources/flavors/`

## Timeline

- **Week 1**: Schema design and core constraint infrastructure
- **Week 2**: Validation engine integration and unit tests
- **Week 3**: Flavor file migration and integration tests
- **Week 4**: Documentation, examples, and performance optimization

---

**Related Issues**: #0013 (Modular Flavor Architecture), #0011 (Comprehensive Flavor Coverage)  
**Epic**: Flavor System Enhancement