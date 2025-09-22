# 0010 — Performance Benchmarks & Optimization

**Status:** open  
**Priority:** medium  
**Created:** 2025-09-23  

## Problem

Currently, the gcode-language-server lacks comprehensive performance benchmarks to:
- Measure parsing performance on large G-code files
- Validate LSP responsiveness under load
- Ensure validation and completion performance scales appropriately
- Identify performance regressions during development

## Goal

Implement a comprehensive benchmarking suite to measure and monitor performance characteristics of the language server.

## Acceptance Criteria

- [ ] **Parsing Benchmarks**: Measure tokenization and parsing performance on files of various sizes (1KB, 100KB, 1MB, 10MB+)
- [ ] **LSP Operation Benchmarks**: Measure hover, completion, and diagnostic performance
- [ ] **Memory Usage Profiling**: Track memory consumption for large documents and concurrent operations
- [ ] **Validation Performance**: Benchmark validation engine with different flavor configurations
- [ ] **Automated Performance Testing**: Integration with CI to detect performance regressions
- [ ] **Performance Baselines**: Establish acceptable performance thresholds for different operations

## Implementation Plan

### Phase 1: Core Parsing Benchmarks
- Extend existing `benches/large_file_benchmark.rs` with comprehensive test cases
- Add benchmarks for different G-code patterns (movement-heavy, parameter-heavy, comment-heavy)
- Measure parsing throughput (lines/second, MB/second)

### Phase 2: LSP Operation Benchmarks
- Benchmark hover request latency with various document sizes
- Measure completion response time for different cursor positions
- Test diagnostic generation performance on complex validation scenarios

### Phase 3: Memory and Scalability
- Profile memory usage with multiple open documents
- Test concurrent LSP operations (multiple clients, rapid requests)
- Measure flavor registry performance with large flavor definitions

### Phase 4: Integration and Monitoring
- Add performance tests to CI pipeline
- Create performance regression detection
- Document performance characteristics and optimization guidelines

## Success Metrics

- **Parsing Performance**: >1MB/s parsing throughput for typical G-code files
- **LSP Responsiveness**: <100ms response time for hover/completion on files <1MB
- **Memory Efficiency**: <50MB memory usage for typical workloads
- **Scalability**: Handle 10+ concurrent documents without degradation

## Files to Modify

- `benches/` - Comprehensive benchmark suite
- `.github/workflows/` - CI integration for performance testing
- `Cargo.toml` - Benchmark configuration and dependencies
- `docs/` - Performance characteristics documentation

## Dependencies

- Complete basic LSP functionality (hover, completion, diagnostics) ✅
- Stable validation engine ✅ 
- Mature flavor system ✅

## Testing Strategy

- Benchmark against realistic G-code files from various 3D printers
- Test performance across different hardware configurations
- Validate performance characteristics match documented expectations
- Ensure benchmarks are deterministic and reliable

## Notes

This benchmarking foundation is essential before optimizing performance or adding resource-intensive features. It provides objective metrics for development decisions and ensures the language server remains responsive as complexity increases.

Consider using:
- `criterion` for statistical benchmarking
- `heaptrack` or `valgrind` for memory profiling  
- `flamegraph` for performance profiling
- Realistic G-code samples from Prusa, Marlin, and Klipper ecosystems