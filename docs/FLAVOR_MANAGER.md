# G-code Flavor Manager

The G-code Language Server includes a comprehensive flavor management system that supports multiple G-code dialects with flexible selection methods, live reloading, and per-file flavor overrides.

## Features

### ✅ Flavor Loading System
- **Built-in flavors**: Embedded Prusa flavor loaded by default
- **User-global flavors**: Load from `~/.config/gcode-ls/flavors/*.toml`
- **Workspace flavors**: Load from `./.gcode-ls/flavors/*.toml`
- **Priority system**: workspace > user-global > built-in

### ✅ Flavor Selection Methods

The language server supports multiple ways to select the active flavor, in order of priority:

1. **Per-file modeline** (highest priority): Detected in G-code file headers
2. **Command-line flag**: `--flavor=<name>` when starting the server
3. **Project configuration**: `.gcode.toml` file found by searching up the directory hierarchy
4. **Default fallback**: Built-in "prusa" flavor

#### Command-Line Selection
Start the language server with a specific flavor:
```bash
gcodels --flavor=marlin
```

#### Project Configuration
Create a `.gcode.toml` file in your project root or any parent directory:
```toml
[project]
default_flavor = "prusa"
```

The server will search from the current working directory upward until it finds a `.gcode.toml` file.

### ✅ File Watching & Live Reload
- Automatically monitors flavor directories for changes
- Reloads flavors when `.toml` files are added, modified, or removed
- No need to restart the language server

### ✅ Modeline Support
Detect per-file flavor overrides using modelines in G-code files:
```gcode
; vim: gcode_flavor=prusa
; gcode_flavor=marlin  
// gcode_flavor=custom_flavor
```

**Modeline Detection Behavior:**
- **Short files (≤10 lines)**: Searches entire file
- **Long files (>10 lines)**: Searches first 5 and last 5 lines
- **Supports multiple comment styles**: `;`, `//`
- **Flexible format**: `gcode_flavor=name` with optional whitespace

### ✅ Error Handling
- Graceful handling of invalid TOML files
- Reports errors through LSP messages
- Falls back to default behavior for missing flavors

## Flavor File Format

Flavors are defined in TOML files with the following schema:

```toml
[flavor]
name = "my_flavor"
version = "1.0"
description = "Custom G-code flavor"

[[commands]]
name = "G28"
pattern = "^G28( .*)?$"
description_short = "Home axes"
description_long = "Home printer axes to their endstop positions"

[[commands.parameters]]
name = "X"
type = "bool"
required = false
description = "Home X axis only"

[[commands]]
name = "M104"
description_short = "Set hotend temperature"
description_long = "Set target temperature for hotend"
```

## Directory Structure

```
~/.config/gcode-ls/flavors/    # User-global flavors
├── marlin.toml
├── klipper.toml
└── custom.toml

./.gcode-ls/flavors/           # Workspace-specific flavors
├── project_specific.toml
└── overrides.toml

.gcode.toml                    # Project configuration (searched hierarchically)
```

## Project Configuration Format

The `.gcode.toml` file uses a simple TOML format:

```toml
[project]
default_flavor = "prusa"

# Optional: additional project settings
[project.settings]
enable_diagnostics = true
completion_style = "detailed"
```

## Integration with LSP

The flavor system integrates seamlessly with LSP features:

1. **Document Open**: Detects modeline and loads appropriate flavor
2. **Hover**: Provides command descriptions from active flavor
3. **Completion**: Suggests commands from active flavor
4. **Diagnostics**: Validates commands against active flavor rules

## Example Usage

### 1. Using Command-line Flavor Selection
```bash
# Start language server with specific flavor
gcodels --flavor=marlin

# Use custom flavor directory
gcodels --flavor-dir=./my-flavors/

# Combine options
gcodels --flavor=prusa --log-level=debug
```

### 2. Using Project Configuration
Create a `.gcode.toml` file in your project root:
```toml
[project]
default_flavor = "marlin"

[project.settings]
enable_diagnostics = true
completion_style = "detailed"
```

The language server will automatically find and use this configuration when started without a `--flavor` flag.

### 3. Using Modeline Override
In your G-code file, add a flavor modeline (highest priority):

**At the top of the file:**
```gcode
; vim: gcode_flavor=prusa
; This file uses Prusa-specific commands
G28        ; Home all axes
M300 S1000 P500  ; Beep (if supported by flavor)
```

**Or at the bottom of the file:**
```gcode
G28        ; Home all axes  
G1 X10 Y10 ; Move to position
M300 S1000 P500  ; Beep

; End of file - modeline detected here too
; gcode_flavor=prusa
```

### 4. Creating Workspace Flavors
```bash
mkdir -p .gcode-ls/flavors
cat > .gcode-ls/flavors/my_printer.toml << 'EOF'
[flavor]
name = "my_printer"
version = "1.0"

[[commands]]
name = "M300"
description_short = "Beep"
description_long = "Play a beep sound with specified frequency and duration"
EOF
```

2. Use in G-code file:
```gcode
; gcode_flavor=my_printer
G28        ; Home all axes
M300 S1000 P500  ; Beep at 1000Hz for 500ms
```

### 5. Flavor Selection Priority
The language server uses the following priority order (highest to lowest):

1. **Modeline in G-code file**: `; gcode_flavor=flavor_name`
2. **Command-line flag**: `--flavor=flavor_name`  
3. **Project configuration**: `default_flavor` in `.gcode.toml`
4. **Built-in default**: Falls back to "prusa" flavor

The language server will:
- Detect the modeline or use configured flavor
- Load the specified flavor definition
- Provide hover help for flavor-specific commands
- Show completion suggestions for command parameters

## Testing

The flavor system includes comprehensive tests covering all functionality:

### Test Coverage
- **25 total tests** across 6 test suites
- **Configuration system**: CLI parsing and `.gcode.toml` hierarchical search
- **Flavor manager core**: Loading, selection, and priority handling
- **File watching**: Live reload when flavor files change
- **Modeline detection**: Per-file flavor overrides
- **Integration tests**: End-to-end document state management
- **Smoke tests**: LSP initialization and basic functionality

### Running Tests
```bash
# Test flavor loading and management
cargo test --test flavor_manager_tests

# Test file watching functionality  
cargo test --test flavor_file_watching_tests

# Test modeline detection
cargo test --test modeline_integration_tests

# Test configuration system (CLI args and .gcode.toml)
cargo test --test config_integration_tests

# Run all tests
cargo test
```

## Implementation Status

✅ **Complete Implementation** - All features are implemented and tested:

- **Command-line interface**: `--flavor`, `--flavor-dir`, `--log-level` flags
- **Project configuration**: Hierarchical `.gcode.toml` search with TOML parsing
- **Priority system**: modeline → CLI → project config → built-in default
- **File watching**: Live reload of flavor definitions without restart
- **Error handling**: Comprehensive validation and graceful fallbacks
- **LSP integration**: Seamless flavor-aware hover, completion, and diagnostics
- **Backward compatibility**: Existing code works unchanged

The flavor selection system is production-ready with comprehensive testing and documentation.
