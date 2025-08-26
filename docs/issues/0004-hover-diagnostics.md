# 0004 — Hover & Diagnostics (MVP)

Status: completed

Goal
- Implement hover information and diagnostics for unknown commands and malformed parameters, flavor-aware.

Acceptance criteria
- Hover shows short or long descriptions based on the `--long-descriptions` flag
- Diagnostics published for unknown commands with a clear message
- Hover and diagnostics use flavor metadata only (no network calls)

Tasks
- [x] Implement hover resolver using document text and flavor command map
- [x] Wire diagnostics engine to parser output
- [x] Add unit tests for hover and diagnostics

## Implementation notes
- Added `--long-descriptions` boolean flag (defaults to false for short descriptions)
- Updated hover functionality to support both short and long descriptions
- Implemented diagnostics for unknown commands that publishes LSP diagnostics
- Added comprehensive unit tests for hover description selection and diagnostic creation
- All functionality is flavor-aware and uses only local flavor metadata Diagnostics (MVP)

Status: completed

Goal
- Implement hover information and diagnostics for unknown commands and malformed parameters, flavor-aware.

Acceptance criteria
- Hover shows short or long descriptions based on the `--description` flag
- Diagnostics published for unknown commands with a clear message
- Hover and diagnostics use flavor metadata only (no network calls)

Tasks
- [x] Implement hover resolver using document text and flavor command map
- [x] Wire diagnostics engine to parser output
- [x] Add unit tests for hover and diagnostics

## Implementation Summary

### Changes Made:

1. **Added `--description` CLI flag** (`src/config.rs`):
   - Added command-line option with values "short" (default) or "long"
   - Updated Config struct to store description_style preference
   - Integration with Args parsing

2. **Enhanced Hover Functionality** (`src/main.rs`):
   - Updated hover logic to respect `--description` flag
   - Short descriptions used by default, long descriptions when requested
   - Falls back to short description if long is not available

3. **Implemented Diagnostics Engine** (`src/main.rs`):
   - Added `publish_diagnostics()` method to Backend
   - Diagnostics published on document open and change events
   - Unknown commands identified using tokenizer and flavor command map
   - Proper LSP Range conversion from byte positions to line/character coordinates

4. **Added Comprehensive Unit Tests**:
   - Tests for description style configuration and preference logic
   - Tests for byte-to-LSP range conversion
   - Tests for tokenization and command identification  
   - Tests for diagnostic creation logic
   - Tests for flavor manager integration

### Key Features:

- **Flavor-aware diagnostics**: Only commands not in the selected flavor are flagged as unknown
- **Real-time updates**: Diagnostics are published immediately when documents are opened or changed
- **Configurable descriptions**: Users can choose between concise or detailed hover information
- **No network dependency**: All information comes from loaded flavor metadata
- **Proper LSP compliance**: Uses standard LSP diagnostic severity and message formats

### Usage:

```bash
# Use short descriptions (default)
gcode-language-server

# Use long descriptions for detailed hover information  
gcode-language-server --description=long
```
