# Constraint Analysis Template for G-code Language Server

## Overview
This document provides a systematic template for analyzing and implementing parameter constraints for G-code commands across different 3D printer firmware flavors. This is designed for delegation to less sophisticated agents who can follow structured analysis patterns.

## What Has Been Completed ✅

### Architectural Foundation
- **Constraint System Architecture**: Complete ParameterConstraint schema in `src/flavor/schema.rs`
  - `ConstraintType` enum with `RequireAnyOf`, `RequireAllOf`, `MutuallyExclusive`
  - Generic validation engine in `validation/engine.rs`
  - TOML deserialization with proper snake_case formatting

### Implemented Constraint Patterns
1. **Movement Constraints (G0/G1)**: 
   - Pattern: `require_any_of` with `["X", "Y", "Z", "E"]`
   - Message: "G1 requires at least one axis parameter (X, Y, Z, or E)"
   - Applied to: Marlin, Prusa, Klipper flavors

2. **Arc Constraints (G2/G3)**:
   - Pattern: `require_any_of` with `["I", "J", "K"]` 
   - Message: "G2/G3 arc move requires arc center offset (I, J, or K)"
   - Applied to: Marlin, Prusa, Klipper flavors

### Flavor Implementations Status
- **Klipper**: ✅ Complete (80+ commands with proper constraints)
- **Marlin**: ✅ Complete with constraint patterns
- **Prusa**: ✅ Complete with constraint patterns

## What's Left (The Grunt Work) 📋

### Command Families Needing Constraint Analysis

#### 1. Temperature Commands (M104-M190 family)
**Analysis Pattern:**
```toml
# For heating commands requiring temperature
[[commands.constraints]]
type = "require_any_of"
parameters = ["S"]
message = "Temperature command requires target temperature (S parameter)"

# For commands with tool selection
[[commands.constraints]]
type = "mutually_exclusive"
parameters = ["T0", "T1", "T2"] # if tool selection is exclusive
message = "Cannot specify multiple tools simultaneously"
```

**Commands to analyze:**
- M104 (Set Extruder Temperature)
- M109 (Set Extruder Temperature and Wait)
- M140 (Set Bed Temperature)
- M190 (Set Bed Temperature and Wait)
- M141 (Set Chamber Temperature)
- M191 (Set Chamber Temperature and Wait)

#### 2. Fan Control Commands (M106-M107 family)
**Analysis Pattern:**
```toml
# Fan speed validation
[[commands.constraints]]
type = "require_any_of" 
parameters = ["S", "P"] # S for speed, P for fan index
message = "Fan command requires speed or fan selection parameter"
```

**Commands to analyze:**
- M106 (Set Fan Speed)
- M107 (Turn Fan Off)
- M710 (Controller Fan settings)

#### 3. Motion Control Commands (G-code movement)
**Analysis Pattern:**
```toml
# Feed rate validation for motion commands
[[commands.constraints]]
type = "require_any_of"
parameters = ["F", "existing_feedrate"] # F or use current feedrate
message = "Motion command requires feedrate (F parameter) or existing feedrate"
```

**Commands to analyze:**
- G0/G1 (already done - template for others)
- G2/G3 (already done - template for others)  
- G5 (Cubic B-spline)
- G38.x (Probe moves)

#### 4. Coordinate System Commands (G54-G59 family)
**Analysis Pattern:**
```toml
# Coordinate system selection - mutually exclusive
[[commands.constraints]]
type = "mutually_exclusive"
parameters = ["G54", "G55", "G56", "G57", "G58", "G59"]
message = "Only one coordinate system can be active"
```

**Commands to analyze:**
- G54-G59 (Work coordinate systems)
- G92 (Set Position - already done in Klipper)

#### 5. Tool Control Commands (T0-T9, M6 family)
**Analysis Pattern:**
```toml
# Tool selection constraint
[[commands.constraints]] 
type = "require_any_of"
parameters = ["tool_number", "T"]
message = "Tool command requires tool number specification"
```

**Commands to analyze:**
- T0-T9 (Tool selection)
- M6 (Tool change)
- M104/M109 with T parameter (tool-specific heating)

#### 6. Spindle/Laser Commands (M3-M5 family)
**Analysis Pattern:**
```toml
# Spindle speed requirement
[[commands.constraints]]
type = "require_any_of"
parameters = ["S"]
message = "Spindle command requires speed parameter (S)"
```

**Commands to analyze:**
- M3 (Spindle On Clockwise)
- M4 (Spindle On Counter-clockwise)  
- M5 (Spindle Off)

#### 7. Probing Commands (G38.x family)
**Analysis Pattern:**
```toml
# Probe target requirement
[[commands.constraints]]
type = "require_any_of"
parameters = ["X", "Y", "Z"]
message = "Probe command requires target position"
```

**Commands to analyze:**
- G38.2 (Probe toward workpiece, stop on contact, signal error if failure)
- G38.3 (Probe toward workpiece, stop on contact)
- G38.4 (Probe away from workpiece, stop on loss of contact, signal error if failure)
- G38.5 (Probe away from workpiece, stop on loss of contact)

## Systematic Analysis Workflow 🔄

### Step 1: Command Identification
For each command family:
1. List all related commands in the flavor
2. Identify common parameter patterns
3. Note firmware-specific variations

### Step 2: Constraint Pattern Recognition
Common patterns to look for:
- **Required Parameters**: Commands that need at least one specific parameter
- **Mutually Exclusive**: Commands where only one option can be used
- **Conditional Requirements**: Parameters required only when others are present
- **Value Range Validation**: Parameters with min/max bounds (future enhancement)

### Step 3: TOML Implementation Template
```toml
[[commands.constraints]]
type = "require_any_of"  # or "require_all_of" or "mutually_exclusive"
parameters = ["PARAM1", "PARAM2", "PARAM3"]
message = "Clear error message explaining the constraint"
```

### Step 4: Testing Pattern
For each implemented constraint:
1. Add test case to validation binary
2. Verify error message appears for invalid usage
3. Verify valid usage passes without errors

## Delegation Instructions 🤖

### For Less Sophisticated Agents:
1. **Pick one command family** from the "What's Left" section
2. **Research the commands** in that family using firmware documentation
3. **Apply the analysis pattern** provided for that family type
4. **Implement constraints** using the TOML template
5. **Test the implementation** using the validation binary
6. **Document your findings** and move to the next family

### Quality Checklist:
- [ ] All commands in the family have been analyzed
- [ ] Constraint types match the command requirements
- [ ] Error messages are clear and helpful
- [ ] TOML syntax follows the established pattern
- [ ] Testing confirms constraints work as expected
- [ ] Documentation is updated with findings

## Future Enhancements (Beyond Current Scope)

### Advanced Constraint Types (Not Yet Implemented)
- **Value Range Constraints**: Min/max validation for numeric parameters
- **Enum Constraints**: Parameter values from predefined sets
- **Conditional Constraints**: Requirements based on other parameter values
- **Cross-Command Constraints**: Validation across multiple G-code lines

### Multi-Flavor Analysis
- Compare constraint differences between Marlin, Klipper, RepRap, etc.
- Identify firmware-specific constraint patterns
- Implement flavor-specific constraint extensions

## Success Metrics 📊

### Current Status:
- ✅ 3/3 major flavors have basic constraint framework
- ✅ 2/8 command families fully analyzed (Movement, Arc)
- ✅ Core constraint engine operational
- ✅ Validation testing infrastructure ready

### Target Completion:
- 🎯 8/8 command families analyzed
- 🎯 All major G-code commands have appropriate constraints
- 🎯 Comprehensive test coverage for constraint validation
- 🎯 Clear error messages for all common G-code mistakes

## Next Steps After Template Usage 🚀

Once constraint analysis is complete:
1. **Document Symbols Implementation**: LSP symbols for navigation
2. **Autocompletion Enhancement**: Use constraints to improve suggestions
3. **Real-time Validation**: Live error detection in editors
4. **Flavor-specific Optimization**: Performance tuning for large files

---

**Note**: This template enables systematic, methodical analysis of G-code constraints without requiring deep architectural understanding. Focus on one command family at a time, follow the patterns, and validate thoroughly.