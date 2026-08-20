# Contributing to gcode-language-server

## Build, test, and check

The `Makefile` is the canonical definition of every local task; run
`make help` to list them. The day-to-day targets are:

```sh
cargo build                # debug build
make release               # release build with native CPU opts
make test                  # full test suite
make check                 # fmt + lint + test — run before `git push`
make check-all             # add audits, docs, and security checks before a PR
make fix                   # auto-format and apply Clippy fixes
```

For a focused test, use Nextest directly:

```sh
cargo nextest run test_name
cargo nextest run module::tests
```

## Code style

- Format with `cargo fmt --all`
- Lint with `cargo clippy --locked --all-targets -- -D warnings`
- Follow idiomatic Rust patterns
- Add rustdoc comments for public APIs

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — new features
- `fix:` — bug fixes
- `docs:` — documentation changes
- `refactor:` — code refactoring
- `test:` — test additions or changes
- `chore:` — maintenance tasks
- `perf:` — performance improvements

Examples:

```text
feat: add document symbols support
fix: correct parameter validation for G1 commands
docs: update flavor manager documentation
refactor: simplify token lookup logic
test: add integration tests for modeline detection
```

## Adding a new flavor

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

2. Add tests in `tests/`.
3. Update the user-facing documentation.
4. Run `make check`.

## Benchmarks

```sh
cargo bench
cargo bench --bench parsing_benchmarks
```

## Code coverage

```sh
make coverage
```

The HTML report is written to `target/llvm-cov/html/index.html`.

## Manpage

The `gcode-ls` manpage lives in `man/gcode-ls.1` as roff source.

Preview it with:

```sh
mandoc man/gcode-ls.1 | less
```

Lint it with:

```sh
make man
```

Update the manpage when adding, removing, or renaming a CLI flag, changing a
default or flavor, or changing server behavior. Update the version and date in
the `.TH` header on each release.

## Submitting changes

1. Create a feature branch: `git checkout -b feat/my-feature`.
2. Make changes with tests and documentation.
3. Run `make check`; run `make check-all` before opening a pull request.
4. Submit a pull request with a clear description.

## Reporting issues

Use GitHub Issues with:

- A clear description
- Steps to reproduce
- Expected and actual behavior
- A G-code sample, if applicable
- The flavor being used
