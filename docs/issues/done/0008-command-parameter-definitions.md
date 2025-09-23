# 0008 — Command Parameter Definitions & Validation

Status: completed  
Completed: 2025-09-23  
Priority: high  
Blocks: #0005 (completions require parameter definitions)

## Problem

Currently, our G-code commands in flavor files lack comprehensive parameter definitions. While the `ParameterDef` structure exists, most commands either have no parameter definitions or only mention parameters in text descriptions. This severely limits the language server's capabilities:

- **Completions**: Cannot suggest valid parameters for commands
- **Diagnostics**: Cannot detect invalid/missing parameters  
- **Hover**: Cannot show parameter-specific documentation
- **Validation**: Cannot validate parameter types, ranges, or requirements

For example, `G1` mentions "Parameters: X, Y, Z, E, F" in description but has no structured parameter definitions.

## Goal

Implement comprehensive parameter definitions for all G-code commands and enhance the parser to validate parameters against these definitions.

## Acceptance Criteria

### Phase 1: Enhanced Parameter Definitions
- [ ] All commands in flavor files have complete parameter definitions
- [ ] Parameter definitions include:
  - [ ] Type (int, float, string, bool)
  - [ ] Required/optional status
  - [ ] Value constraints (min/max, enum values)
  - [ ] Clear descriptions
- [ ] Support for parameter groups/variants (e.g., G28 can home all or specific axes)

### Phase 2: Parser Integration  
- [ ] Parser validates parameters against command definitions
- [ ] Parser reports unknown parameters as diagnostics
- [ ] Parser reports missing required parameters
- [ ] Parser validates parameter types and constraints
- [ ] Parser provides context for parameter resolution

### Phase 3: Enhanced Flavor Schema
- [ ] Extend `ParameterDef` to support:
  - [ ] Value constraints (ranges, enums)
  - [ ] Parameter aliases/synonyms  
  - [ ] Conditional requirements (param A required if param B present)
  - [ ] Default values
- [ ] Update flavor file documentation
- [ ] Validate flavor files against enhanced schema

## Implementation Plan

### 1. Enhance Parameter Definition Schema

Extend `ParameterDef` in `src/flavor.rs`:
```rust
pub struct ParameterDef {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub description: String,
    pub constraints: Option<ParameterConstraints>,
    pub default_value: Option<String>,
    pub aliases: Option<Vec<String>>,
}

pub enum ParameterType {
    Int,
    Float, 
    String,
    Bool,
}

pub struct ParameterConstraints {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub enum_values: Option<Vec<String>>,
    pub pattern: Option<String>,
}
```

### 2. Update All Flavor Definitions

For each command, add comprehensive parameter definitions:
- G0/G1: X, Y, Z, E, F parameters with proper types and constraints
- G28: Optional axis parameters (X, Y, Z, W)
- M-codes: All their specific parameters
- Temperature commands: S (set), R (wait) parameters
- etc.

### 3. Enhance Parser Validation

Update `src/gcode.rs` to:
- Accept flavor definitions for validation
- Cross-reference parsed parameters with command definitions  
- Generate detailed diagnostic messages
- Provide parameter context for completions

### 4. Integration Points

- Flavor manager provides parameter definitions to parser
- Parser validates and enriches token stream with parameter context
- LSP handlers use parameter context for diagnostics, completions, hover

## Example: Enhanced G1 Definition

```toml
[[commands]]
name = "G1"
pattern = "^G1( .*)?$"
description_short = "Controlled linear move"
description_long = "G1 moves the toolhead linearly while feeding filament if applicable."

[[commands.parameters]]
name = "X"
type = "float"
required = false
description = "X-axis destination coordinate"
constraints = { min_value = -999.0, max_value = 999.0 }

[[commands.parameters]]
name = "Y" 
type = "float"
required = false
description = "Y-axis destination coordinate"
constraints = { min_value = -999.0, max_value = 999.0 }

[[commands.parameters]]
name = "Z"
type = "float" 
required = false
description = "Z-axis destination coordinate"
constraints = { min_value = 0.0, max_value = 300.0 }

[[commands.parameters]]
name = "E"
type = "float"
required = false
description = "Extruder position/amount"

[[commands.parameters]]
name = "F"
type = "float"
required = false
description = "Feedrate (movement speed)"
constraints = { min_value = 1.0, max_value = 10000.0 }
```

## Dependencies

- Must complete before issue #0005 (completions)
- Works with existing flavor system
- Enhances diagnostic capabilities from issue #0004

## Testing

- [ ] Unit tests for parameter validation logic
- [ ] Integration tests with various parameter combinations
- [ ] Test invalid parameter detection
- [ ] Test parameter type validation
- [ ] Test constraint validation (ranges, enums)
- [ ] Test error message quality

## Notes

This is a foundational issue that significantly improves the language server's core functionality. Without proper parameter definitions, many LSP features remain superficial.

The enhanced parameter system should be extensible to support future requirements like:
- Parameter interdependencies
- Context-sensitive parameters  
- Firmware-specific parameter variations
- Parameter value suggestions based on context