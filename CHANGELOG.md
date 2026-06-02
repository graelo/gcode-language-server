# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
