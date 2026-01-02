# Contributing to gcode-language-server

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/graelo/gcode-language-server.git
   cd gcode-language-server
   ```

2. Install Rust (stable toolchain):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Build and test:
   ```bash
   cargo build
   cargo test
   ```

## Code Style

- Format with `cargo fmt`
- Lint with `cargo clippy`
- Follow idiomatic Rust patterns
- Doc comments for public APIs

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions/changes
- `chore:` - Maintenance tasks
- `perf:` - Performance improvements

Examples:
```
feat: add document symbols support
fix: correct parameter validation for G1 commands
docs: update flavor manager documentation
refactor: simplify token lookup logic
test: add integration tests for modeline detection
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make changes with tests
4. Ensure CI passes:
   ```bash
   cargo fmt --check
   cargo clippy
   cargo test
   ```
5. Submit PR with clear description

## Adding a New Flavor

1. Create `resources/flavors/yourflavor.gcode-flavor.toml`:
   ```toml
   [flavor]
   name = "yourflavor"
   version = "1.0"
   description = "Your flavor description"

   [[commands]]
   name = "G28"
   description_short = "Home axes"
   description_long = "Home printer axes to their endstop positions"

   [[commands.parameters]]
   name = "X"
   type = "bool"
   required = false
   description = "Home X axis only"
   ```

2. Test your flavor:
   ```bash
   cargo run --bin gcode-ls -- --flavor=yourflavor
   ```

3. Add tests in `tests/`
4. Update documentation

## Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific suite
cargo bench --bench parsing_benchmarks
```

## Code Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Reporting Issues

Use GitHub Issues with:
- Clear description
- Steps to reproduce
- Expected vs actual behavior
- G-code sample if applicable
- Flavor being used

## Questions?

Open a GitHub issue or discussion.
