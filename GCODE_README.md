# G-code Tokenizer and Parser

A streaming, line-oriented G-code tokenizer and lightweight parser designed for large files (tested up to 20MB).

## Features

- **Streaming tokenization**: Process files line-by-line without loading entire file into memory
- **Token types**: Command (G1, M104), Parameter (X10.0, S200), Comment (;..., (...))
- **Position lookup**: Find token at any byte position for hover/diagnostics
- **Lightweight AST**: Parse tokens into Command structures for analysis
- **Performance**: Handles 20MB files at ~300 MiB/s

## Usage

### Basic Tokenization

```rust
use gcode_language_server::gcode::*;

// Tokenize a single line
let tokens = tokenize_line("G1 X10.0 Y5 ; move", 0);

// Tokenize entire text
let text = "G28 ; home\nM104 S200\nG1 X0 Y0";
let tokens = tokenize_text(text);

// Find token at position
let token = token_at(&tokens, 15).expect("token found");
```

### Streaming Iterator (for large files)

```rust
use std::io::BufReader;
use std::fs::File;

let file = File::open("large_file.gcode")?;
let reader = BufReader::new(file);
let iterator = TokenIterator::new(reader);

for token in iterator {
    println!("{:?}: {}", token.kind, token.text);
}
```

### AST Parsing

```rust
let line = "G1 X10.0 Y5.2 F1500 ; extrude";
let tokens = tokenize_line(line, 0);
let command = parse_command_from_tokens(&tokens).expect("command");

println!("Code: {}", command.code);
for param in command.params {
    println!("  {}: {}", param.letter, param.value);
}
```

## Benchmarks

Run performance benchmarks:

```bash
cargo bench --bench large_file_benchmark
```

Results on test hardware:
- 20MB file tokenization: **~195 MiB/s** (nom-optimized)  
- Token lookup: ~47ns per operation
- Streaming iterator: **~108 MiB/s** (includes I/O overhead)

## Performance Notes

The parser uses nom for structure but optimizes hot paths with manual parsing:
- Whitespace handling via nom's `multispace0` 
- Token recognition via fast character iteration
- Zero-copy string slicing for all tokens
- Maintains excellent performance while keeping code readable

## Testing

```bash
cargo test
```

Includes edge case tests:
- Missing newline at EOF
- Multiple commands per line
- Long comment blocks (10k+ characters)
