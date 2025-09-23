# Performance Benchmarking Guide

This document describes the comprehensive performance benchmarking system for the gcode-language-server.

## Overview

The benchmarking suite provides comprehensive performance measurement and monitoring for:

- **Parsing Performance**: Tokenization and AST construction speed
- **LSP Operations**: Hover, completion, and diagnostic response times  
- **Validation Engine**: Error detection and parameter validation performance
- **Memory Usage**: Resource consumption patterns and scalability
- **Throughput**: Processing speed for different file sizes and patterns

## Benchmark Suites

### 1. Parsing Benchmarks (`parsing_benchmarks.rs`)

Tests the core parsing engine performance:

- **Single Line Parsing**: Individual command parsing speed
- **File Parsing**: Multi-line document processing
- **Throughput Testing**: Bytes/second processing rates
- **Real File Testing**: Performance on actual G-code files
- **Tokenization**: Raw lexer performance

**Key Metrics:**
- Lines per second parsing rate
- Bytes per second throughput  
- Memory allocation patterns
- Performance across different G-code patterns

### 2. LSP Benchmarks (`lsp_benchmarks.rs`)

Tests Language Server Protocol operation performance:

- **Validation Performance**: Document validation speed
- **Full Pipeline**: Parse + validate workflow timing
- **Error Scenarios**: Performance with different error densities
- **Flavor Operations**: Command lookup and registry performance
- **Position Operations**: Cursor-based operations (hover simulation)
- **Concurrent Operations**: Multiple request handling simulation

**Key Metrics:**
- Response time for different document sizes
- Validation speed across error scenarios
- Flavor registry lookup performance
- Memory usage patterns

### 3. Validation Benchmarks (`validation_benchmarks.rs`)

Tests the validation engine specifically:

- **Error Density Testing**: Performance across different error rates
- **Scalability Testing**: Performance with increasing file sizes
- **Command Type Testing**: Validation speed by command category
- **Parameter Validation**: Parameter parsing and constraint checking
- **Memory Patterns**: Resource usage under different scenarios

**Key Metrics:**
- Validation time per line
- Error detection accuracy vs speed
- Memory consumption patterns
- Scaling characteristics

## Running Benchmarks

### Local Development

```bash
# Run all benchmarks
./benches/run_benchmarks.sh

# Run specific benchmark suite
cargo bench --bench parsing_benchmarks
cargo bench --bench lsp_benchmarks  
cargo bench --bench validation_benchmarks

# Quick benchmarks for development
cargo bench --bench parsing_benchmarks -- --quick

# Generate test files
python3 benches/generate_test_files.py
```

### Automated CI/CD

Benchmarks run automatically:

- **On every push** to main/develop branches
- **On pull requests** with performance impact analysis
- **Weekly** for performance trend monitoring
- **On-demand** via GitHub Actions workflow dispatch

### Benchmark Script Options

```bash
# Run specific benchmark type
./benches/run_benchmarks.sh parsing
./benches/run_benchmarks.sh lsp
./benches/run_benchmarks.sh validation

# Generate summary report only
./benches/run_benchmarks.sh summary

# Clean benchmark results
./benches/run_benchmarks.sh clean
```

## Performance Targets

### Parsing Performance
- **Target**: >1MB/s parsing throughput for typical G-code files
- **Acceptable**: >500KB/s for complex files with heavy parameter usage
- **Single Line**: <1µs per line for simple commands
- **Complex Lines**: <5µs per line for parameter-heavy commands

### LSP Responsiveness  
- **Hover/Completion**: <100ms response time for files <1MB
- **Validation**: <200ms for complete document validation <1MB
- **Large Files**: <500ms for files up to 10MB
- **Memory Usage**: <50MB for typical workloads

### Scalability
- **Concurrent Documents**: Handle 10+ open documents without degradation  
- **Large Files**: Process 50MB+ files without memory issues
- **Error Handling**: Maintain performance with high error rates (>50% invalid lines)

## Test Data

### Generated Test Files

The benchmark suite uses both real and generated G-code files:

- **Small files**: 100-1,000 lines for quick iteration testing
- **Medium files**: 10,000-50,000 lines for realistic scenarios  
- **Large files**: 100,000+ lines for stress testing
- **Pattern variations**: Movement-heavy, parameter-heavy, comment-heavy, mixed

### Real Test Files

- `tests/fixtures/sample_prusa.gcode`: Real Prusa output
- `tests/fixtures/sample_workspace.gcode`: Complex workspace setup
- `tests/fixtures/sample_bottom_modeline.gcode`: Bottom modeline testing

### Generated Test Files

- `benches/test_files/large_print.gcode`: ~20,000 line typical print
- `benches/test_files/very_large_print.gcode`: ~50,000 line stress test
- `benches/test_files/complex_cnc.gcode`: CNC operations with advanced features
- `benches/test_files/error_heavy.gcode`: High error rate for validation testing

## Interpreting Results

### Criterion Output

Benchmarks use the Criterion library for statistical analysis:

```
parsing_throughput/throughput/10000
                        time:   [2.0986 ms 2.1193 ms 2.1245 ms]
                        thrpt:  [68.210 MiB/s 68.377 MiB/s 69.051 MiB/s]
```

- **time**: Lower bound / estimate / upper bound of execution time
- **thrpt**: Throughput measurement (higher is better)
- **Statistical confidence**: 95% confidence intervals

### HTML Reports

Detailed HTML reports are generated at `target/criterion/reports/index.html`:

- Performance trends over time
- Statistical analysis and confidence intervals
- Performance comparisons between runs
- Detailed timing histograms

### Performance Analysis

Key indicators to monitor:

1. **Regression Detection**: >10% performance decrease
2. **Memory Growth**: Unexpected memory usage increases  
3. **Scaling Issues**: Non-linear performance degradation with file size
4. **Error Handling Impact**: Significant slowdown with error-heavy files

## Continuous Monitoring

### GitHub Actions Integration

- **Automated Execution**: Runs on every significant code change
- **Performance Comments**: Automatic PR comments with benchmark results
- **Artifact Storage**: Benchmark results stored for historical analysis
- **Regression Detection**: Compares performance against baseline

### Performance Trends

Weekly automated runs provide:

- Long-term performance trend analysis
- Early detection of performance regressions
- Baseline establishment for new features
- Performance impact assessment of dependencies

## Troubleshooting

### Common Issues

1. **Benchmark Compilation Failures**
   ```bash
   # Ensure dev dependencies are available
   cargo tree | grep criterion
   ```

2. **Missing Test Files**
   ```bash
   # Regenerate test files
   python3 benches/generate_test_files.py
   ```

3. **Inconsistent Results**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

4. **CI Benchmark Failures**
   - Check system resources and timeout limits
   - Verify test file generation in CI environment
   - Review performance thresholds for CI hardware differences

### Performance Debugging

For performance issues:

1. **Profile with detailed benchmarks**: Use `--profile-time` flag
2. **Memory analysis**: Use `heaptrack` or `valgrind` locally
3. **Flame graphs**: Generate performance flame graphs for hotspot analysis
4. **Incremental testing**: Isolate specific components with focused benchmarks

## Contributing

### Adding New Benchmarks

1. **Choose appropriate suite**: parsing, lsp, or validation
2. **Follow naming conventions**: Clear, descriptive benchmark names
3. **Include multiple scenarios**: Various file sizes and error conditions
4. **Document expected performance**: Set clear performance expectations
5. **Test locally**: Verify benchmarks work across different hardware

### Performance Standards

- All benchmarks must be deterministic and repeatable
- Use realistic test data representative of actual usage
- Include both positive (fast) and negative (slow) test cases
- Document performance expectations and acceptable ranges
- Consider memory usage alongside execution time

## Future Enhancements

### Planned Features

- **Comparative Analysis**: Automatic comparison with previous versions
- **Performance Budgets**: Configurable performance thresholds  
- **Memory Profiling**: Detailed memory usage analysis
- **Concurrency Testing**: Multi-threaded performance scenarios
- **Real-world Simulation**: Long-running LSP session simulation

### Integration Opportunities

- **IDE Integration**: Performance testing within development environments
- **Release Gating**: Prevent releases with significant performance regressions
- **Performance Dashboards**: Web-based performance trend visualization
- **Automated Optimization**: Performance-guided code optimization suggestions