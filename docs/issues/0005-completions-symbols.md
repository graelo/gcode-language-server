# 0005 — Document Symbols (Outline/Navigation)

Status: open  
Updated: 2025-09-23

## Completed ✅
- ✅ Command completions implemented and working
- ✅ Parameter completions implemented with correct G-code format (X0.0, Y10, etc.)
- ✅ Integration tests passing

## Remaining Goal
Implement document symbols (outline) for easy navigation through G-code files.

## Acceptance Criteria
- [ ] Document symbols show command structure and hierarchy
- [ ] Symbols include line numbers and command types (movement, temperature, etc.)
- [ ] Navigation works in editors (jump to symbol, outline view)
- [ ] Symbols are categorized by command type for better organization

## Tasks
- [ ] Implement document symbol extraction from parsed tokens
- [ ] Categorize symbols by command type (Movement, Temperature, Bed Leveling, etc.)
- [ ] Add tests for symbol extraction
- [ ] Verify editor integration (Neovim, VS Code)
