# 0003 — Flavor manager (load, watch, validate)

Status: open

Goal
- Provide a flavor manager that loads built-in flavors and user flavors from `~/.config/gcode-ls/flavors/` and supports live reload and per-file modeline overrides.

Acceptance criteria
- Flavor files (TOML) load and validate
- Flavor directory is watched; changes trigger reload
- Per-file modeline `gcode_flavor=<name>` is detected and applied

Tasks
- [ ] Implement flavor TOML schema and deserialization
- [ ] Implement loading order: built-in < user-global < workspace
- [ ] Add file watcher for flavor-dir
- [ ] Implement modeline detection in parser or LSP document open
- [ ] Error reporting for invalid flavor files
