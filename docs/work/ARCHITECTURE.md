# ARCHITECTURE: gcode-language-server

Status: Draft

## High-level overview

The `gcode-language-server` is a Rust process that implements the Language Server Protocol (LSP) over JSON-RPC on stdio. Editors (Neovim, VS Code, etc.) spawn the binary and communicate using the standard LSP messages. The server parses G-code files, consults a flavor database (built-in + user-provided TOML files), and provides LSP features: diagnostics, hover, completions, document symbols, and semantic tokens.

## Components

- CLI bootstrap
  - Parses startup flags (e.g., `--description=short|long`, `--flavor-dir`)
  - Sets up logging and config
- LSP layer
  - Based on `tower-lsp` to handle JSON-RPC and LSP message routing
  - Implements LSP methods (initialize, didOpen, didChange, hover, completion, documentSymbol, etc.)
- Parser & tokenizer
  - A streaming tokenizer (line-oriented) to handle large files (target: ~20 MB)
  - Produces tokens and lightweight AST nodes sufficient for diagnostics and document symbols
- Flavor manager
  - Loads built-in flavors embedded in the binary
  - Loads user flavors from `--flavor-dir` (default `~/.config/gcode-ls/flavors/`) and workspace overrides
  - Watches flavor directory for changes and supports per-file flavor overrides (modeline)
  - Validates flavor TOML using `serde` + schema checks
- Diagnostics engine
  - Flavor-aware checks for unknown commands, malformed parameters, and parameter validation rules
- Hover & completion provider
  - Provides hover contents based on flavor metadata (short or long descriptions per startup flag)
  - Completion suggestions for commands and parameters (context-aware)
- Tests & CI
  - Unit tests for parser and flavor loader
  - Integration tests for LSP flows using a lightweight test client

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

## Per-file flavor selection

Support a modeline-like mechanism. Example (Vim modeline):

; vim: gcode_flavor=prusa

The server will inspect the first/last few lines for a modeline and if present will override the flavor selection for that document.

## CLI & configuration

- Flags:
  - `--description=short|long` (default: short)
  - `--flavor-dir <path>` (default: `~/.config/gcode-ls/flavors/`)
  - `--log-level <level>`

Configuration is also available via LSP workspace/didChangeConfiguration.

## Error handling & fallback

- If a flavor is missing or invalid, the server falls back to a conservative built-in grammar (recognize standard G/M/X/Y commands generically) and reports flavor errors as warnings.

## Next implementation steps

1. Create project scaffold and CI (Cargo.toml, src/, tests/)
2. Implement minimal LSP server that responds to initialize and can return a hover for a hard-coded command
3. Add parser/tokenizer and small set of unit tests
4. Add flavor manager with sample Prusa TOML and live reload
