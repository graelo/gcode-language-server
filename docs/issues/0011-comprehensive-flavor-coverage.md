# 0011 — Comprehensive G-code Flavor Coverage

**Status:** open  
**Priority:** high  
**Created:** 2025-09-23  

## Problem

Currently, the gcode-language-server only provides comprehensive support for Prusa flavor. The 3D printing ecosystem uses diverse firmware with different G-code command sets and parameter requirements:

- **Marlin**: Most popular 3D printer firmware, used by many manufacturers
- **Klipper**: High-performance firmware with unique macro system and commands  
- **RepRapFirmware**: Duet controller firmware with distinct command set
- **Smoothieware**: CNC and 3D printer firmware with specific extensions

Limited flavor support restricts the language server's usefulness across the broader 3D printing community.

## Goal

Implement comprehensive G-code flavor definitions for major 3D printer firmwares, providing full LSP support (hover, completion, validation) for each ecosystem.

## Acceptance Criteria

### Core Flavors
- [ ] **Marlin Flavor**: Complete command set for Marlin 2.x firmware
- [ ] **Klipper Flavor**: Klipper-specific commands, macros, and configuration syntax
- [ ] **RepRapFirmware Flavor**: Duet controller command set and parameters
- [ ] **Enhanced Prusa Flavor**: Complete Prusa-specific extensions and parameters

### Flavor Features
- [ ] **Complete Parameter Definitions**: All parameters with proper types, constraints, and validation
- [ ] **Command Documentation**: Rich hover information with firmware-specific details
- [ ] **Parameter Validation**: Firmware-specific parameter requirements and constraints
- [ ] **Intelligent Completion**: Context-aware parameter suggestions
- [ ] **Cross-compatibility Notes**: Document command differences between flavors

### Quality Standards
- [ ] **Comprehensive Coverage**: 90%+ command coverage for each flavor
- [ ] **Accurate Validation**: Proper parameter validation matching firmware behavior
- [ ] **Rich Documentation**: Detailed descriptions, examples, and usage notes
- [ ] **Tested Compatibility**: Validation against real G-code files from each ecosystem

## Implementation Plan

### Phase 1: Research & Analysis
- Analyze Marlin 2.x source code for complete command reference
- Study Klipper documentation and configuration examples
- Research RepRapFirmware command reference and parameter specifications
- Collect representative G-code samples from each ecosystem

### Phase 2: Marlin Flavor Implementation
- Create `resources/flavors/marlin.gcode-flavor.toml`
- Implement complete Marlin command set (G0-G34, M0-M999, etc.)
- Add Marlin-specific parameters and validation rules
- Include advanced features (linear advance, input shaping, etc.)

### Phase 3: Klipper Flavor Implementation  
- Create `resources/flavors/klipper.gcode-flavor.toml`
- Implement Klipper's extended G-code command set
- Add macro system support and configuration commands
- Include Klipper-specific features (pressure advance, resonance testing, etc.)

### Phase 4: RepRapFirmware & Quality
- Create `resources/flavors/reprapfirmware.gcode-flavor.toml`
- Implement Duet-specific command extensions
- Enhance Prusa flavor with missing commands and parameters
- Comprehensive testing and validation

### Phase 5: Integration & Documentation
- Update CLI to support all flavors (`--flavor marlin|klipper|reprap|prusa`)
- Create flavor comparison documentation
- Add flavor detection heuristics based on common patterns
- Integration testing with real-world G-code files

## Success Metrics

- **Coverage**: 90%+ of commonly used commands for each flavor
- **Accuracy**: Parameter validation matches firmware behavior
- **Usability**: Developers can switch between flavors seamlessly
- **Community Adoption**: Positive feedback from Marlin/Klipper/RepRap communities

## Technical Considerations

### Flavor File Structure
```toml
[flavor]
name = "marlin"
version = "2.1"
description = "Marlin 2.x firmware G-code command set"
extends = ["core"]  # Optional inheritance

[[commands]]
name = "M851"
description_short = "Set Z probe offset"
description_long = "Configure Z probe offset for bed leveling systems"
firmware_versions = ["2.0+"]
aliases = ["M851.1", "M851.2"]  # Marlin sub-commands

[[commands.parameters]]
name = "Z"
type = "float"
required = false
description = "Z offset in mm (negative values lower nozzle)"
constraints = { min = -5.0, max = 5.0 }
```

### Command Categories
- **Movement**: G0, G1, G2, G3, G28, G29
- **Temperature**: M104, M109, M140, M190, M106, M107  
- **Bed Leveling**: G29, M420, M421, M851
- **Linear/Pressure Advance**: M900, M572 (Klipper)
- **Firmware Features**: M503, M500, M502

## Files to Create/Modify

- `resources/flavors/marlin.gcode-flavor.toml`
- `resources/flavors/klipper.gcode-flavor.toml` 
- `resources/flavors/reprapfirmware.gcode-flavor.toml`
- `resources/flavors/prusa.gcode-flavor.toml` (enhancements)
- `docs/FLAVOR_COMPARISON.md`
- `tests/flavor_integration_tests.rs`

## Dependencies

- Enhanced parameter system with constraints ✅
- Robust validation engine ✅
- Flavor registry with embedded support ✅
- LSP integration for all features ✅

## Testing Strategy

- **Real G-code Validation**: Test with actual printer output files
- **Parameter Coverage**: Verify all parameters are properly defined
- **Cross-flavor Compatibility**: Ensure flavor switching works seamlessly
- **Community Validation**: Engage with firmware communities for accuracy review

## Community Impact

This comprehensive flavor support will:
- Make gcode-ls useful for the entire 3D printing community
- Reduce G-code debugging time across different firmwares
- Improve G-code quality through better validation and completion
- Establish gcode-ls as the standard G-code development tool

## Research Resources

- [Marlin G-code Reference](https://marlinfw.org/docs/gcode/)
- [Klipper G-code Commands](https://www.klipper3d.org/G-Codes.html)
- [RepRapFirmware G-code Dictionary](https://docs.duet3d.com/User_manual/Reference/Gcodes)
- [RepRap G-code Wiki](https://reprap.org/wiki/G-code)