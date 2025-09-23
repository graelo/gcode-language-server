# Issue 0013: Modular Flavor Architecture

**Priority**: Medium  
**Status**: Open  
**Created**: 2025-09-23  
**Assignee**: TBD  

## Summary

Implement a modular flavor architecture using systemd-style directory splits with numeric prefixes to replace the current monolithic TOML files. This will improve maintainability, enable collaborative development, and support user customizations.

## Background

Current flavor files are becoming unwieldy:
- `prusa.gcode-flavor.toml`: 1,218 lines, 93 commands
- `marlin.gcode-flavor.toml`: 50+ commands (growing)
- `klipper.gcode-flavor.toml`: Expected to be even larger

Large monolithic files create several issues:
- Difficult to navigate and maintain
- Merge conflicts in collaborative development
- Poor IDE performance with large files
- Hard to organize commands by logical categories
- No mechanism for user customization without modifying base files

## Proposed Solution

### Directory Structure

Replace monolithic files with systemd-style drop-in directories:

```
resources/flavors/
├── prusa.gcode-flavor.d/
│   ├── 10-metadata.toml       # flavor name, version, description
│   ├── 20-movement.toml       # G0-G3, G26-G30, G64-G65
│   ├── 30-temperature.toml    # M104-M190
│   ├── 40-extrusion.toml      # M83, M221, extrusion control
│   ├── 50-mmu.toml           # M701-M709, T0-T4 MMU commands
│   ├── 60-calibration.toml    # M862.x, bed mesh, probing
│   ├── 70-input-shaper.toml   # M593 and advanced motion control
│   ├── 80-visual.toml         # M150-M151 LED control
│   ├── 95-site-custom.toml    # organizational additions
│   └── 99-user-local.toml     # user overrides
├── marlin.gcode-flavor.d/
│   ├── 10-metadata.toml
│   ├── 20-movement.toml
│   ├── 30-temperature.toml
│   └── ...
└── klipper.gcode-flavor.d/
    ├── 10-metadata.toml
    ├── 20-movement.toml
    ├── 30-macros.toml
    └── ...
```

### Loading Algorithm

1. **Directory Detection**: Check for `{flavor}.gcode-flavor.d/` directory first
2. **File Enumeration**: Collect all `.toml` files in directory
3. **Sorting**: Sort files lexicographically (numeric prefixes work naturally)
4. **Sequential Loading**: Load and merge files in order
5. **Conflict Resolution**: Later definitions override earlier ones
6. **Validation**: Ensure required metadata exists after merging

### File Organization Conventions

**Base Files (10-89)**:
- `10-metadata.toml` - Flavor definition and metadata
- `20-movement.toml` - Basic movement commands (G0-G3)
- `30-temperature.toml` - Temperature control (M104-M190)
- `40-extrusion.toml` - Extrusion and flow control
- `50-mmu.toml` - Multi-material unit commands
- `60-calibration.toml` - Calibration and probing
- `70-input-shaper.toml` - Advanced motion control
- `80-visual.toml` - LED control and visual feedback

**Extension Files (90-99)**:
- `95-site-custom.toml` - Organizational customizations
- `98-experimental.toml` - Beta/testing commands
- `99-user-local.toml` - Personal user overrides

### Backward Compatibility

- Support both monolithic and directory formats
- If both `flavor.gcode-flavor.toml` and `flavor.gcode-flavor.d/` exist, prefer directory
- Provide migration tooling to convert existing monolithic files

## Implementation Plan

### Phase 1: Core Infrastructure
1. **Enhanced Schema Types**
   - `FlavorFragment` for partial flavor definitions
   - `FlavorBuilder` for merging fragments
   - Conflict detection and resolution logic

2. **Directory Loader**
   - Scan and sort directory contents
   - Sequential loading with override support
   - Enhanced error reporting (which file caused issues)

3. **Validation Framework**
   - Ensure essential metadata after merging
   - Warn about command overrides
   - Validate file numbering conventions

### Phase 2: Migration and Tooling
1. **Migration Tools**
   - Automatic splitting of existing monolithic files
   - Command categorization logic
   - Preservation of existing parameter definitions

2. **Development Tools**
   - Flavor validation command
   - Override visualization
   - Documentation generation from merged results

### Phase 3: Enhanced Features
1. **User Customization**
   - Support for user-specific directories
   - Site-wide customization patterns
   - Override conflict detection

2. **Documentation**
   - Updated flavor authoring guide
   - Migration documentation
   - Best practices for file organization

## Technical Specifications

### File Format

Each fragment file follows the same TOML structure:

```toml
# 10-metadata.toml (only in this file)
[flavor]
name = "prusa"
version = "2.0"
description = "Comprehensive Prusa Buddy firmware G-code flavor"

# Any numbered file can contain commands
[[commands]]
name = "G0"
description_short = "Rapid positioning"
# ... parameters
```

### Loading Logic Pseudocode

```rust
fn load_flavor_directory(dir_path: &Path) -> Result<Flavor> {
    let mut entries: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("toml"))
        .collect();
    
    entries.sort_by_key(|e| e.file_name());
    
    let mut flavor_builder = FlavorBuilder::new();
    
    for entry in entries {
        let content = fs::read_to_string(entry.path())?;
        let fragment: FlavorFragment = toml::from_str(&content)?;
        flavor_builder.merge(fragment, entry.file_name())?;
    }
    
    flavor_builder.build()
}
```

### Error Handling

- **File Load Errors**: Clear indication of which file failed to parse
- **Merge Conflicts**: Warnings when commands are overridden
- **Missing Metadata**: Error if essential flavor info is missing after merging
- **Numbering Issues**: Warnings for unconventional file numbering

## Benefits

### Maintainability
- **Logical Organization**: Commands grouped by functional categories
- **Smaller Files**: Each file focuses on specific command groups
- **Clear Boundaries**: Easy to understand file responsibilities

### Collaboration
- **Focused PRs**: Changes target specific command categories
- **Reduced Conflicts**: Multiple developers can work on different categories
- **Clear Ownership**: Specific files can have designated maintainers

### Extensibility
- **User Customization**: Override mechanism without modifying base files
- **Organizational Flexibility**: Site-specific additions via numbered files
- **Plugin Architecture**: Third-party extensions can add command categories

### DevX Improvements
- **IDE Performance**: Better handling of smaller files
- **Navigation**: Jump to specific command categories quickly
- **Debugging**: Trace commands back to defining files
- **Documentation**: Generate category-specific documentation

## Risks and Mitigations

### Complexity Risk
- **Risk**: More complex loading logic
- **Mitigation**: Comprehensive testing and clear error messages

### Override Confusion
- **Risk**: Users may not understand override behavior
- **Mitigation**: Clear documentation and warning messages

### File Proliferation
- **Risk**: Too many small files become hard to manage
- **Mitigation**: Sensible categorization and naming conventions

## Testing Strategy

1. **Unit Tests**: FlavorBuilder merge logic and conflict resolution
2. **Integration Tests**: Directory loading with various file combinations
3. **Migration Tests**: Verify monolithic → directory conversion accuracy
4. **Performance Tests**: Ensure directory loading doesn't degrade performance
5. **User Scenario Tests**: Common override and customization patterns

## Success Criteria

- [ ] All existing flavor files successfully migrated to directory structure
- [ ] Loading performance equivalent to or better than monolithic files
- [ ] Clear error messages for all failure scenarios
- [ ] User override mechanism working correctly
- [ ] Comprehensive documentation and migration guide
- [ ] Backward compatibility maintained for existing integrations

## Dependencies

- No external dependencies
- Requires updates to flavor loading logic in `src/flavor/registry.rs`
- Schema updates in `src/flavor/schema.rs`
- Documentation updates for flavor authoring

## Timeline

- **Week 1**: Core infrastructure and loading logic
- **Week 2**: Migration tooling and testing
- **Week 3**: User override support and documentation
- **Week 4**: Polish, performance optimization, and release prep

---

**Related Issues**: #0011 (Comprehensive Flavor Coverage), #0012 (Documentation Parsers)  
**Epic**: Flavor System Enhancement