# Implementation Plan: Command Parameter Definitions & Validation
**Issue**: #0008  
**Created**: 2025-09-22  
**Priority**: High (blocks #0005 completions)

## Overview

This document outlines the implementation plan for adding comprehensive parameter definitions to G-code commands and implementing validation in the parser. This is a foundational improvement that enables advanced LSP features like intelligent completions, parameter validation, and enhanced diagnostics.

## Current State Analysis

### What We Have
- Basic `ParameterDef` struct in `src/flavor.rs`
- Sample parameter definition for M250 S parameter
- Parser that can extract parameters (letter + value) but no validation
- Command definitions with textual parameter descriptions

### What's Missing
- Comprehensive parameter definitions for all commands
- Parameter type system and constraints
- Parser integration with flavor definitions
- Validation logic for parameters against command schemas
- Rich diagnostic messages for parameter errors

## Implementation Phases

### Phase 1: Enhanced Schema Design
**Goal**: Create robust parameter definition system
**Files**: `src/flavor.rs`

#### Tasks:
1. **Enhanced ParameterDef struct**:
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
   ```

2. **Parameter type system**:
   ```rust
   #[derive(Clone, Debug, Deserialize)]
   pub enum ParameterType {
       Int,
       Float,
       String, 
       Bool,
   }
   ```

3. **Constraint system**:
   ```rust
   #[derive(Clone, Debug, Deserialize)]
   pub struct ParameterConstraints {
       pub min_value: Option<f64>,
       pub max_value: Option<f64>,
       pub enum_values: Option<Vec<String>>,
       pub pattern: Option<String>,
   }
   ```

**Deliverables**:
- Enhanced ParameterDef with full type system
- TOML deserialization support
- Backward compatibility with existing flavor files

### Phase 2: Comprehensive Parameter Definitions
**Goal**: Define parameters for all G-code commands
**Files**: `docs/work/samples/prusa.gcode-flavor.toml`

#### Commands to Define:
1. **Movement Commands**:
   - G0 (rapid move): X, Y, Z, F parameters
   - G1 (linear move): X, Y, Z, E, F parameters
   - G28 (home): X, Y, Z, W optional parameters

2. **Coordinate Commands**:
   - G91 (relative mode): no parameters
   - G92 (set position): X, Y, Z, E parameters

3. **Machine Commands**:
   - M250: S parameter (already exists, enhance)

**Parameter Categories**:
- **Coordinates**: X, Y, Z (float, optional, with machine limits)
- **Extruder**: E (float, optional, unlimited)  
- **Feedrate**: F (float, optional, 1-10000 range)
- **Settings**: S, R, P (various types, command-specific)

**Deliverables**:
- Complete parameter definitions for all sample commands
- Realistic constraints based on 3D printer capabilities
- Clear parameter descriptions
- Example of parameter aliases where applicable

### Phase 3: Parser Integration
**Goal**: Validate parameters during parsing
**Files**: `src/gcode.rs`

#### Enhancement Areas:
1. **Parser Context**:
   - Accept flavor definitions during parsing
   - Link parsed parameters to command definitions
   - Generate validation context for each token

2. **Validation Logic**:
   ```rust
   pub struct ValidationResult {
       pub is_valid: bool,
       pub errors: Vec<ValidationError>,
       pub warnings: Vec<ValidationWarning>,
   }
   
   pub enum ValidationError {
       UnknownParameter { param: String, command: String },
       MissingRequiredParameter { param: String, command: String },
       InvalidParameterType { param: String, expected: ParameterType, actual: String },
       ConstraintViolation { param: String, constraint: String, value: String },
   }
   ```

3. **Enhanced Token Structure**:
   - Add parameter validation context to tokens
   - Include parameter definition references
   - Provide rich error information

**Deliverables**:
- Parameter validation function
- Enhanced token structure with validation context
- Comprehensive error types and messages
- Integration points for LSP diagnostics

### Phase 4: LSP Integration
**Goal**: Use parameter validation in language server
**Files**: `src/main.rs`

#### Integration Points:
1. **Diagnostic Generation**:
   - Convert validation errors to LSP diagnostics
   - Provide actionable error messages
   - Include parameter help in diagnostics

2. **Hover Enhancement**:
   - Show parameter information in hover
   - Display parameter constraints and types
   - Provide usage examples

3. **Preparation for Completions**:
   - Export parameter definitions for completion provider
   - Structure parameter context for intelligent suggestions

**Deliverables**:
- Enhanced diagnostic messages
- Parameter-aware hover information
- Foundation for completion system

### Phase 5: Testing & Validation
**Goal**: Ensure robust parameter validation
**Files**: `tests/parameter_validation_tests.rs`, existing test files

#### Test Categories:
1. **Unit Tests**:
   - Parameter type validation
   - Constraint checking (ranges, enums)
   - Required parameter detection
   - Parameter alias resolution

2. **Integration Tests**:
   - End-to-end parameter validation
   - LSP diagnostic generation
   - Flavor file parsing with parameters
   - Error message quality

3. **Edge Cases**:
   - Invalid parameter types
   - Missing required parameters
   - Constraint violations
   - Malformed parameter definitions

**Deliverables**:
- Comprehensive test suite
- Performance benchmarks for validation
- Documentation updates

## Dependencies & Sequencing

### Critical Path:
1. Phase 1 (Schema) → Phase 2 (Definitions) → Phase 3 (Parser) → Phase 4 (LSP)
2. Phase 5 (Testing) runs parallel to implementation phases

### External Dependencies:
- No breaking changes to existing LSP interface
- Backward compatibility with current flavor files
- Integration with existing diagnostic system

### Blocked Issues:
- Issue #0005 (completions) depends on Phase 1-3 completion
- Future parameter-aware features depend on this foundation

## Success Criteria

### MVP (Minimum Viable Product):
- [ ] Enhanced ParameterDef with type system
- [ ] Complete parameter definitions for sample commands
- [ ] Basic parameter validation in parser
- [ ] LSP diagnostic integration

### Full Implementation:
- [ ] All parameter types and constraints supported
- [ ] Comprehensive error messages
- [ ] Parameter-aware hover information
- [ ] Foundation for intelligent completions
- [ ] Full test coverage

## Risk Mitigation

### Potential Issues:
1. **Performance**: Parameter validation on every parse
   - *Mitigation*: Optimize validation logic, cache results
2. **Complexity**: Parameter constraint system too complex
   - *Mitigation*: Start simple, iterate based on usage
3. **Compatibility**: Breaking existing flavor files
   - *Mitigation*: Maintain backward compatibility, gradual migration

### Testing Strategy:
- Incremental testing after each phase
- Real G-code file validation
- Performance testing with large files

## Timeline Estimate

- **Phase 1**: 1-2 days (schema design and implementation)
- **Phase 2**: 1 day (parameter definitions) 
- **Phase 3**: 2-3 days (parser integration)
- **Phase 4**: 1-2 days (LSP integration)
- **Phase 5**: 1-2 days (testing and polish)

**Total**: 6-10 days for complete implementation

## Future Enhancements

Post-implementation opportunities:
- Context-sensitive parameter suggestions
- Parameter interdependency validation
- Firmware-specific parameter variations
- Parameter value auto-completion based on context
- Machine-specific parameter constraints