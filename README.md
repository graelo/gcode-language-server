# gcode-language-server (prototype)

Minimal scaffold for the gcode-language-server project. This prototype embeds a sample Prusa flavor TOML and starts a tower-lsp server over stdio.

Build

```bash
cargo build --release
```

Run (editor will spawn the binary);

```bash
./target/release/gcode-language-server
```

The current scaffold is minimal and does not yet implement full hover/completion behavior. It's a starting point for iterating on parser, flavor manager, and LSP features.
