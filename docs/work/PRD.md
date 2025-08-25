# PRD: G-code Language Server (Rust)

Status: Draft

## Purpose

Provide a Language Server Protocol (LSP) implementation for G-code, written in Rust, that integrates with editors such as Neovim and is extensible to support multiple G-code "flavors" (starting with Prusa). The server will offer diagnostics, hover, completions, and other editor-friendly features to improve developer and user productivity when authoring G-code.

## Requirements (from user)

- LSP server usable with Neovim (standard transport recommended).
- Implemented in Rust.
- Support multiple G-code flavors; start with Prusa-specific commands.
- Allow developers to define custom flavors (configurable, pluggable).
- Store plans and artifacts in `docs/work` and issues in `docs/issues/`.

## Goals (MVP)

- Startable as a standard LSP using stdio (for Neovim). JSON-RPC over stdio.
- Parse G-code files and provide:
  - Syntax recognition & tokenization
  - Diagnostics for unknown/invalid commands (flavor-aware)
  - Hover information for commands (signature + description)
  - Basic completions for commands and parameters
  - Outline/document symbols for easy navigation
- Read built-in Prusa flavor specification and user-defined flavor files from workspace or global config.

## Non-goals (initial)

- Live firmware communication or simulation.
- Full G-code execution/visualization.

## Success Criteria

- Neovim (and other editors) can connect via LSP stdio and receive diagnostics and hovers.
- Prusa-specific commands are recognized and documented on hover.
- Developers can add a flavor file (TOML/JSON) in the workspace and the server uses it without recompilation.

## Constraints & assumptions

- Use stable Rust toolchain and prefer minimal native dependencies.
- Editor integrations will use standard LSP (JSON-RPC) over stdio — this is the de-facto standard for Neovim and coc.nvim.
- Flavor definitions should be serializable (TOML/JSON/YAML). We'll propose TOML for readability and simplicity.

## Transport recommendation

Use LSP (Language Server Protocol) over stdio by default (JSON-RPC over stdio). Rationale:
- Stdin/stdout JSON-RPC is standard for editor-to-language-server communication and works well with Neovim and its LSP clients.
- Alternative transports (TCP, stdio+socket) can be added later if needed.

The user confirmed JSON-RPC over stdio as the preferred transport.

## Implementation outline

- Rust crates and libraries (proposal):
  - `tower-lsp` for LSP server scaffolding
  - `tokio` for async runtime (if needed)
  - `serde` + `serde_json`/`toml` for flavor serialization
  - `nom` or a small hand-written parser for G-code tokenization/parsing
  - `regex` for pattern matching command formats


- Flavor model (example schema):

```toml
[flavor]
name = "prusa"
version = "0.1"

[[commands]]
name = "M250"
pattern = "^M250( .*)?$"
description = "Prusa Buddy command example"

[[commands.parameters]]
name = "S"
type = "int"
required = false
description = "example parameter"
```

Flavors are loaded from user-global flavor folders (default: `~/.config/gcode-ls/flavors/`) and merged with built-in flavors. Workspace-level flavor files will also be supported, but by user preference flavor files will primarily live in the user-global location.

The user confirmed TOML for flavors and requested that implementing additional commands should not require code changes — only extension of the TOML flavor files.


## Extensibility for user-defined flavors

- Flavor files parsed at startup and on change. Live reload via file-watch will be supported; users can also force a flavor change using an in-file modeline (editor-specific, e.g., a Vim modeline) to select a different flavor per-file.
- Schema-driven validation for flavor files with clear error messages.
- Public API shape (internal): flavor -> command definitions -> parameter schemas -> help text.

The user requested flavor changes may be done live using something like a Vim modeline; the server will support per-file flavor selection via modeline and live reload.

## LSP features (prioritized)

1. Diagnostics (unknown command, malformed parameters)
2. Hover (signature + docs)
3. Completion (commands and parameters)
4. Document symbols / Outline
5. Formatting (optional, low priority)
6. Semantic tokens (colorization) — medium priority


## Testing & validation

- Unit tests for parser and flavor loading.
- Integration tests simulating LSP requests (using `tower-lsp` tests or a small client harness).
- Performance testing for large files (target: comfortably edit files up to ~20 MB). The server will use streaming and incremental parsing where possible to avoid large allocations and repeated full-file parses on every change.

The user indicated they expect edits to files up to ~20 MB.

## Security & privacy

- No network access by default. Flavor files are read from the workspace or via LSP config only.


## Configuration & startup flags

- `--description=short|long` — choose between short and long hover descriptions at startup (user requested a startup flag to pick between short and long descriptions).
- `--flavor-dir <path>` — override default user-global flavor folder (default: `~/.config/gcode-ls/flavors/`).

## Milestones / Roadmap (suggested)

1. PRD (this doc) — done
2. Architecture doc (next)
3. Project skeleton + CI + basic LSP server (hello world) using `tower-lsp`
4. Parser + tokenizer + tests
5. Built-in Prusa flavor data + hover + diagnostics
6. Flavor file loader + workspace configuration + live reload
7. Completions + document symbols + semantic tokens

8. Packaging, release, and Neovim integration notes

## Project metadata

- License: MIT (per user choice)
- Project name: `gcode-language-server` (confirmed by user)


## Open questions (see docs/issues/0001-prd-questions.md)

See linked questions file for clarifying items required before architecture and implementation.

## Next steps

- Review these goals and answer the open questions.
- I'll prepare an architecture document that maps components, data flow, and storage of flavor files.
