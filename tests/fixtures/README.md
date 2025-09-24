# Test Fixtures

This directory contains G-code test files used by various tests and benchmarks.

## File Organization

### Integration Test Files
- `sample_bottom_modeline.gcode` - Tests modeline parsing at bottom of files
- `sample_prusa.gcode` - Real Prusa printer output for integration testing and benchmarking
- `sample_workspace.gcode` - Complex workspace setup scenarios

## Usage in Tests

These files are referenced by:
- Integration tests in `tests/`
- Benchmarking suites in `benches/`
- Parser validation tests

## Adding New Fixtures

When adding new test fixtures:
1. Use descriptive filenames that indicate their purpose
2. Include a comment header explaining what the file tests
3. Keep files focused on specific testing scenarios
4. Update this README when adding new categories