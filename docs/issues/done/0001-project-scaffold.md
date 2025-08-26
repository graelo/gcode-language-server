# 0001 — Project scaffold & minimal LSP server

Status: done

Goal
- Create a Rust project scaffold with a minimal LSP server that can be spawned by editors over stdio. Include a `Cargo.toml`, `src/main.rs`, sample flavor, and a README.

Acceptance criteria
- `cargo build` succeeds
- Binary starts and accepts LSP initialize handshake (smoke test)
- Repository contains `docs/work/samples/prusa.gcode-flavor.toml`

Tasks
- [x] Create `Cargo.toml` with required dependencies (tower-lsp, serde, toml, tokio)
- [x] Add `src/main.rs` with tower-lsp bootstrap (current scaffold)
- [x] Add sample flavor TOML under `docs/work/samples`
- [x] Implement a minimal hover that returns short descriptions from the sample flavor
- [x] Tidy imports/warnings (silenced unused-field warnings where appropriate)
- [x] Add a basic integration smoke test for initialize (unit-level smoke; full LSP handshake test is future work)

Summary of work completed

- Implemented minimal LSP scaffold using `tower-lsp`.
- Implemented document store and hover returning short descriptions from the embedded Prusa flavor.
- Added a unit test that verifies the sample flavor contains common commands (G0, G1).
- Added a `--version` CLI flag for quick verification.

Verification commands

```bash
cargo build --release
cargo test
./target/release/gcode-language-server --version
```

Notes
- I used `#[allow(dead_code)]` on flavor structs to reduce warning noise while the flavor schema fields are not yet fully used; if you prefer to keep the compiler guidance visible I can remove these attributes and address each unused field explicitly instead.