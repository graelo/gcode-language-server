# 0003 — Flavor manager (load, watch, validate)

Status: done

Goal
- Provide a flavor manager that loads built-in flavors and user flavors with flexible flavor selection via command-line flags or project configuration files.

Acceptance criteria
- Flavor files (TOML) load and validate
- Flavor directory is watched; changes trigger reload
- Flavor selection via `--flavor=<name>` command-line flag
- If no flag provided, search file hierarchy for `.gcode.toml` configuration file
- Per-file modeline `gcode_flavor=<name>` is detected and applied (highest priority)
- Loading priority: built-in < user-global < workspace < project config < command-line < modeline

Tasks
- [x] Implement flavor TOML schema and deserialization
- [x] Implement loading order: built-in < user-global < workspace
- [x] Add file watcher for flavor-dir
- [x] Implement modeline detection in parser or LSP document open
- [x] Error reporting for invalid flavor files
- [x] Add command-line flag `--flavor=<name>` for explicit flavor selection
- [x] Add hierarchical search for `.gcode.toml` project configuration
- [x] Update flavor selection priority to include command-line and project config
