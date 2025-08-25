# 0001 — PRD Clarifying Questions

These questions surface assumptions and choices needed before writing the architecture document and starting implementation.

1. Transport & Editor compatibility
   - Confirm: prefer LSP over stdio (JSON-RPC) as default? (I recommend yes — it works with Neovim builtin LSP and is standard.)
   - Answer: yes, JSON-RPC over stdio

2. Flavor file format
   - Do you prefer TOML for flavor files (recommended) or JSON/YAML?
   - Answer: TOML

3. Scope of commands
   - Should we implement every documented Prusa Buddy command upfront, or implement a representative subset for the MVP and iterate?
   - Answer: implement a representative subset for MVP. Additional commands must be addable via TOML only (no code change required).
4. Hover/docs source
   - Do you expect the server to bundle detailed documentation for commands, or is a short summary per command sufficient for MVP?
   - Answer: add a startup flag to choose short or long descriptions (user will choose at startup).
5. Configuration & storage
   - Where should user flavor files live by default? Options:
     - workspace root `.gcode-flavor.toml`
     - `.config/gcode-ls/flavors/` (user-global)
     - LSP configuration only (sent via client)
   - Answer: user-global (e.g., `~/.config/gcode-ls/flavors/`). Workspace-level files still allowed but user-global is preferred.
6. Performance expectations

   - Answer: edit 20MB files easily if possible; prefer incremental parsing and streaming.

7. Edition and reload
   - Answer: live reload; also support per-file modeline to change flavor (e.g., Vim modeline).
8. CLI usage
   - Do you want a small CLI wrapper to run the server manually (e.g., `gcode-lsp`), or rely on editor to spawn the binary directly?
   - Answer: in production the editor will spawn the binary directly. We'll still provide a small CLI for manual runs.
9. License preference
   - Any license you want for the project? (MIT, Apache-2.0, etc.)
   - Answer: MIT
10. Branding / project name
   - Any preferred project name beyond `gcode-language-server`?
   - Answer: `gcode-language-server` is fine.
Please answer the above or point to decisions you'd like me to make; I'll record the choices and proceed to draft the architecture document.
