# AGENTS.md

This file contains instructions for coding agents working in this repository.

- Repository: <https://github.com/graelo/gcode-language-server>
- Prefer `gh` for GitHub operations.
- Do not mention an agent or assistant in issues, pull requests, comments, or
  commit messages.
- Do not expose private local information, including machine-specific paths.

## Project

`gcode-language-server` is a Rust Language Server Protocol implementation for
G-code files. The package contains the reusable library implementation and the
`gcode-ls` stdio LSP executable.

The server provides parsing, validation, completion, hover, and document symbol
features, with built-in Prusa, Marlin, and Klipper flavor support.

Rust 1.95 or later is required. The crate uses edition 2024.

## Architecture

1. `gcode-ls` parses command-line configuration and starts the LSP server.
2. The LSP layer owns the asynchronous JSON-RPC boundary.
3. The synchronous core parses documents, loads flavors, and validates G-code.
4. Flavor definitions describe commands, parameters, and validation constraints.

Key modules:

- `src/parser/`: G-code tokenization and AST construction.
- `src/flavor/`: flavor schemas, registries, and embedded definitions.
- `src/validation/`: command and parameter diagnostics.
- `src/lsp/`: LSP backend, handlers, and stdio server.
- `src/core/`: shared document and diagnostic types.
- `src/config.rs`: command-line and flavor-directory configuration.
- `src/bin/gcode-ls.rs`: executable entry point.
- `resources/flavors/`: built-in flavor TOML files.

## Verification

The `Makefile` is the canonical definition of local verification tasks. **Read
it before choosing or running verification commands**; do not duplicate its
command implementations here. `make help` lists every target.

The primary targets are:

- `make check`: pre-push gate (formatting, linting, and tests).
- `make check-all`: pre-PR gate (adds dependency, commit-message, Markdown,
  manpage, and GitHub Actions security checks).
- `make fix`: formats code and applies Clippy fixes.
- `make md`: lints Markdown against `rumdl.toml`.
- `make man`: lints the `gcode-ls` roff manpage.
- `make ci-security`: runs the Poutine and Zizmor GitHub Actions scans.

The check targets mirror the GitHub workflows and use locked dependency
resolution where applicable. They assume external tools such as
`cargo-nextest`, `cargo-deny`, `cargo-pants`, `convco`, `poutine`, `zizmor`,
`rumdl`, `mandoc`, and `cargo-llvm-cov` are installed locally.

For focused Rust tests, use `cargo nextest run <test_name>` or
`cargo nextest run <module::tests::name>`. The complete CI test sequence is
implemented in `ci/test_full.sh`; its Nextest CI profile is configured in
`.config/nextest.toml`.

## Documentation and releases

Keep user-facing documentation in sync with behavior:

- Update `README.md` and `CONTRIBUTING.md` when the user workflow changes.
- Update `man/gcode-ls.1` when changing a CLI flag, default, flavor, or server
  behavior. Preview it with `mandoc man/gcode-ls.1 | less` and lint it with
  `make man`.
- For a release version bump, update `Cargo.toml`, `Cargo.lock`, the versioned
  section and comparison links in `CHANGELOG.md`, and the manpage `.TH` header.
  Create a `vX.Y.Z` tag; the release workflow derives artifact and GitHub
  Release versions from it.
- Commit messages must follow `.convco` Conventional Commit rules. Use
  `make commits` to check them.

`Cargo.toml`, `Cargo.lock`, `deny.toml`, and the GitHub workflows define the
release and supply-chain constraints. Preserve `--locked` behavior in Cargo
commands that resolve dependencies.
