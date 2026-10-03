# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Add a Makefile as the canonical local verification workflow, including
  formatting, linting, tests, audits, documentation, manpage, security, and
  coverage targets
- Add a `gcode-ls(1)` manpage and manpage linting instructions

### Changed

- Replace the unmaintained `tower-lsp` 0.20 with the community-maintained
  `tower-lsp-server` 0.23 fork. The LSP types are now re-exported from
  `ls-types` instead of `lsp-types` (notably `Uri` replaces `url::Url`), and
  the `LanguageServer` trait uses native async-in-trait instead of the
  `#[async_trait]` macro. This is a breaking change for users of the library
  crate's `lsp` module; the `gcode-ls` binary behavior is unchanged. It also
  resolves a Clippy 1.99 false positive
  (`double_must_use` in the `#[async_trait]` macro expansion) that broke CI
  on stable toolchains
- Document the Makefile-based workflow in the README and contributor guide
- Use the repository README as the package documentation instead of duplicating
  crate-level documentation in `src/lib.rs`, and add complete crates.io/docs.rs
  package metadata
- Align CI with github-actions-playbook v1.9: install Rust with
  actions-rust-lang/setup-rust-toolchain pinned to the v2.0.0 tag instead of
  the untagged dtolnay/rust-toolchain, rename targets to the singular target
  input, keep the action cache off, deny warnings in cargo builds, and drop
  the now-dead zizmor superfluous-actions config
- Remove the unused `env_logger` and `notify` dependencies, flagged by the
  cargo unused-dependencies lint on Rust 1.101 beta and denied by the
  build-warnings default of the new setup-rust-toolchain action

## [0.0.2] - 2026-06-02

### Changed

- CI workflows aligned with playbook conventions (matrix jobs, tooling install)
- CI release workflow: publish prerelease tags as GitHub pre-releases, switch
  homebrew bump to brew CLI, consistent binary/archive vocabulary
- CI audit: guard cargo-pants install with command -v fallback
- Pin github-actions-playbook to v1.1
- Scope GitHub App token permissions explicitly in renovate workflow
- Dependency updates (minor and patch)

## [0.0.1] - 2025-09-25

### Added

- LSP server with JSON-RPC over stdin/stdout
- G-code parser with streaming tokenization (240-360 MiB/s on 20MB files)
- Flavor system with Prusa, Marlin, Klipper support
- Multi-source flavor configuration:
  - Built-in flavors (embedded)
  - User-global flavors (`~/.config/gcode-ls/flavors/`)
  - Workspace flavors (`./.gcode-ls/flavors/`)
  - Project config (`.gcode.toml`)
  - CLI flags (`--flavor`, `--flavor-dir`)
  - Per-file modeline (`; gcode_flavor=name`)
- Hover provider with command descriptions from active flavor
- Diagnostics for unknown commands and invalid parameters
- Command and parameter completions with G-code format
- File watching with live flavor reload
- Performance benchmark suite (criterion)
- Comprehensive test coverage

### Architecture

- Clean module separation: parser, flavor, validation, lsp, core
- Synchronous core with async only at LSP boundary
- Zero-copy tokenization
- Unidirectional dependencies

[Unreleased]: https://github.com/graelo/gcode-language-server/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/graelo/gcode-language-server/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/graelo/gcode-language-server/releases/tag/v0.0.1
