# ARCHITECTURE: gcode-language-server

Status: Current (Updated Aug 2025)

## High-level overview

The `gcode-language-server` is a Rust process that implements the Language Server Protocol (LSP) over JSON-RPC on stdio. Editors (Neovim, VS Code, etc.) spawn the `gcodels` binary and communicate using the standard LSP messages. The server parses G-code files, consults a flavor database (built-in + user-provided TOML files), and provides LSP features: diagnostics, hover, completions, document symbols, and semantic tokens.

## Implementation Status

### ✅ Completed Components

- **CLI bootstrap** - Command-line argument parsing with `clap`
- **LSP layer** - Full implementation based on `tower-lsp` with JSON-RPC message routing  
- **Parser & tokenizer** - Streaming tokenizer for large files (tested up to 20MB+)
- **Flavor manager** - Complete system with built-in flavors, user/workspace loading, file watching, modeline support, and configuration-based selection
- **Configuration system** - CLI flags, project config files (.gcode.toml), hierarchical search
- **Basic hover provider** - Command descriptions from active flavor

### 🚧 In Progress / Future Work

- **Diagnostics engine** - Flavor-aware validation (Issue 0004)
- **Completion provider** - Command and parameter suggestions (Issue 0005)  
- **Document symbols** - AST-based symbol extraction
- **Semantic tokens** - Syntax highlighting support
- **LSP workspace configuration** - Runtime config changes

## Components

### CLI Bootstrap ✅
- Parses startup flags: `--flavor`, `--flavor-dir`, `--log-level`
- Uses `clap` with derive macros for argument parsing
- Integrates with configuration system for flavor selection

### LSP Layer ✅
- Based on `tower-lsp` to handle JSON-RPC and LSP message routing
- Implements core LSP methods: `initialize`, `didOpen`, `didChange`, `hover`
- Document state management with flavor-aware command lookup
- Async architecture with `tokio`

### Parser & Tokenizer ✅
- Streaming tokenizer (line-oriented) handles large files efficiently (20MB+ tested)
- Produces tokens and lightweight AST nodes for diagnostics and document symbols
- Incremental parsing capabilities with position-based token lookup
- Performance benchmarks included (`cargo bench`)

### Flavor Manager ✅
- **Multi-source loading**: Built-in flavors embedded + user flavors from directories
- **Priority system**: workspace > user-global > built-in
- **File watching**: Live reload with `notify` crate when flavor files change  
- **Configuration integration**: CLI flags, project config, and modeline detection
- **Selection priority**: modeline → CLI flag → project config → built-in default
- **Error handling**: Graceful fallbacks and LSP error reporting

### Configuration System ✅ 
- **Command-line**: `--flavor`, `--flavor-dir`, `--log-level` flags
- **Project config**: `.gcode.toml` files with hierarchical directory search
- **Per-file overrides**: Modeline detection (`; gcode_flavor=flavor_name`)
- **Validation**: Comprehensive error handling and user feedback

### Hover Provider ✅ (Basic)
- Provides hover contents based on flavor metadata (command descriptions)
- Token detection under cursor with command lookup
- Integrated with flavor selection system

### Future Components 🚧
- **Diagnostics engine** - Flavor-aware validation rules
- **Completion provider** - Context-aware command and parameter suggestions  
- **Document symbols** - AST-based symbol extraction
- **Semantic tokens** - Editor syntax highlighting support

## Data flow

1. Editor opens a G-code file and sends `textDocument/didOpen`.
2. LSP layer forwards content to parser.
3. Parser tokenizes the file (incrementally when possible) and produces diagnostics using the active flavor.
4. Diagnostics are sent back via `textDocument/publishDiagnostics`.
5. Hover/completion requests consult the flavor manager for command metadata and return formatted results.

## Flavor file schema (summary)

- flavor.name: string
- flavor.version: string
- commands: array of command tables
  - name: string (e.g., "M250")
  - pattern: optional regex to match command line form
  - description_short: optional string
  - description_long: optional string
  - parameters: optional array
    - name
    - type (int/float/string/enum)
    - required: bool
    - validation: optional pattern or range

The TOML schema will be validated at load time and errors reported to the editor.

## Performance considerations

- Use line-oriented streaming parsing to avoid allocating the entire file when possible.
- Use incremental parsing on didChange (apply edits when feasible). For large single edits, fall back to re-parse of affected ranges.
- Cache flavor metadata in memory; reload only when flavor files change.

## Flavor Selection System ✅

The server implements a sophisticated priority-based flavor selection:

### Priority Order (Highest → Lowest)
1. **Per-file modeline**: `; gcode_flavor=flavor_name` in file header/footer
2. **Command-line flag**: `--flavor=flavor_name` 
3. **Project configuration**: `default_flavor` in `.gcode.toml` file
4. **Built-in default**: Falls back to embedded "prusa" flavor

### Configuration Sources
- **CLI arguments**: Parsed with `clap` derive macros
- **Project config**: `.gcode.toml` files found via hierarchical directory search
- **Modeline detection**: Regex-based parsing in first/last 5 lines of files (or entire file if ≤10 lines)

### Example Usage
```bash
# Explicit flavor selection
gcodels --flavor=marlin

# Custom flavor directory
gcodels --flavor-dir=~/.my-flavors/

# Project configuration (.gcode.toml)
[project]
default_flavor = "prusa"

# Per-file override
; gcode_flavor=custom_printer   (can be at top or bottom of file)
```

## CLI & Configuration ✅

### Current Implementation
```bash
# Executable name: gcodels (like luals for lua-language-server)
gcodels [OPTIONS]

# Available flags:
--flavor <FLAVOR>          # G-code flavor to use (e.g., 'prusa', 'marlin')  
--flavor-dir <FLAVOR_DIR>  # Directory containing flavor TOML files
--log-level <LOG_LEVEL>    # Log level: trace, debug, info, warn, error (default: info)
-h, --help                 # Print help
-V, --version             # Print version
```

### Project Configuration (.gcode.toml)
```toml
[project]
default_flavor = "prusa"

[project.settings]
enable_diagnostics = true
completion_style = "detailed"
```

### Directory Structure
```
~/.config/gcode-ls/flavors/    # User-global flavors
./.gcode-ls/flavors/           # Workspace-specific flavors  
.gcode.toml                    # Project configuration (searched hierarchically)
```

### Future: LSP Workspace Configuration
- Runtime configuration changes via LSP `workspace/didChangeConfiguration`
- Per-workspace flavor settings

## Error handling & fallback

- If a flavor is missing or invalid, the server falls back to a conservative built-in grammar (recognize standard G/M/X/Y commands generically) and reports flavor errors as warnings.

## Testing & Quality Assurance ✅

### Comprehensive Test Suite (25+ tests)
- **Unit tests**: Parser, tokenizer, flavor loading, configuration
- **Integration tests**: End-to-end LSP flows, flavor selection, file watching
- **Performance benchmarks**: Large file handling (tested up to 20MB)
- **Real-world examples**: Sample G-code files with modeline detection

### Test Categories
```bash
cargo test                              # All tests
cargo test --test config_integration_tests      # Configuration system
cargo test --test flavor_manager_tests          # Flavor management  
cargo test --test flavor_file_watching_tests    # File watching
cargo test --test modeline_integration_tests    # Modeline detection
cargo bench                             # Performance benchmarks
```

## Current Status & Next Steps

### ✅ Production Ready Features
- Flavor management with live reload
- Configuration-based flavor selection  
- Basic LSP server with hover support
- Comprehensive testing and documentation

### 🎯 Next Implementation Priority
1. **Issue 0004**: Diagnostics engine with flavor-aware validation
2. **Issue 0005**: Completion provider for commands and parameters
3. **Issue 0006**: Incremental parsing performance improvements
4. **Issue 0007**: Packaging and release automation

The core architecture is complete and battle-tested with a solid foundation for adding the remaining LSP features.
