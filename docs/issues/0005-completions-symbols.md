# 0005 — Document Symbols (Outline/Navigation)

Status: **in-progress**  
Updated: 2025-09-25

## Completed ✅
- ✅ Command completions implemented and working
- ✅ Parameter completions implemented with correct G-code format (X0.0, Y10, etc.)
- ✅ Integration tests passing
- ✅ Document symbols approach analyzed and strategy decided

## Current Goal
Implement document symbols (outline) for easy navigation through G-code files using **Approach 2: Descriptive Names + Flat List**.

## Implementation Strategy
**Chosen Approach**: Descriptive symbol names with flat list structure
- Symbol names: `"G1 X10 Y20 (Linear Move)"`, `"M104 S200 (Set extruder temperature)"`
- Structure: Flat list with LSP SymbolKind categorization
- Benefits: Clear context without overwhelming hierarchy, no flavor file burden
- Performance: O(n) parsing, handles large files efficiently

## Acceptance Criteria
- [ ] Document symbols appear in editor outline panels (VS Code, Neovim)
- [ ] Symbol names include command + key parameters + description from flavor
- [ ] Symbols are categorized using appropriate LSP SymbolKind values
- [ ] Navigation works: click symbol → jump to G-code line
- [ ] Performance acceptable on large files (20MB+ test target)
- [ ] No changes required to existing flavor files

## Implementation Plan
**Estimated effort**: 2-3 hours focused work

### Phase 1: Basic LSP Integration (~30 min)
- [ ] Add `document_symbol_provider` to ServerCapabilities
- [ ] Implement `document_symbol` handler in backend
- [ ] Return minimal symbols ("G1", "M104") for LSP verification

### Phase 2: Descriptive Parameters (~45 min)
- [ ] Extract key parameters for symbol names ("G1 X10 Y20", "M104 S200")
- [ ] Focus on movement parameters (X,Y,Z,E) and common values (S for temperatures)

### Phase 3: Flavor Integration (~60 min)
- [ ] Add flavor descriptions to symbol names ("G1 X10 Y20 (Linear Move)")
- [ ] Map commands to appropriate LSP SymbolKind values
- [ ] Handle commands not in active flavor gracefully

### Phase 4: Testing & Polish (~30 min)
- [ ] Test with real G-code files (small and large)
- [ ] Verify VS Code and Neovim integration
- [ ] Add unit tests for symbol extraction logic

### Future Enhancement Ideas (not in scope)
- Smart filtering for large files ("G1 movements (lines 45-847)")
- Optional hierarchical grouping by command type
- Symbol search/filtering capabilities
