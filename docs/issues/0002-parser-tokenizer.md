# 0002 — Parser & tokenizer

Status: open

Goal
- Implement a streaming, line-oriented tokenizer and simple parser for G-code that can handle large files and produce tokens and a lightweight AST for diagnostics and symbols.

Acceptance criteria
- Tokenizer can parse a sample G-code file into tokens (commands, parameters, comments)
- Unit tests cover common cases and edge cases (empty lines, comments, invalid tokens)
- Parser supports retrieval of token at a position for hover/diagnostics use

Tasks
- [ ] Design token format and small AST shape
- [ ] Implement line-oriented tokenizer (no full-file allocation)
- [ ] Implement parser functions used by diagnostics and document symbols
- [ ] Add unit tests and benchmarks for large file handling (target: 20 MB)

Edge cases
- Missing newline at EOF
- Lines with multiple commands
- Long comment blocks
