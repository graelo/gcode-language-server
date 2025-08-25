# 0001 — Project scaffold & minimal LSP server

Status: in-progress

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
- [ ] Implement a minimal hover that returns short descriptions from the sample flavor
- [ ] Tidy imports/warnings and ensure `cargo build` completes without warnings where practical
- [ ] Add a basic integration smoke test for initialize

Notes
- Current status: scaffold created and partially compiled; remaining work: implement hover and reduce warnings.
