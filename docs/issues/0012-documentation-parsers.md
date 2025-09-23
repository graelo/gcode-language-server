# Issue 0012: Documentation Parsers for Command Extraction

**Status**: Open
**Priority**: Medium
**Category**: Tooling
**Created**: 2025-09-23

## Problem Statement

We need automated tools to extract G-code command names from official firmware documentation sources to:
1. Validate completeness of our flavor files
2. Detect missing commands when documentation is updated
3. Ensure comprehensive coverage of all supported commands
4. Automate maintenance of flavor definitions

Currently, we manually create flavor files by reading documentation, which is:
- Time-consuming and error-prone
- Difficult to keep synchronized with upstream changes
- Hard to verify completeness

## Proposed Solution

Create parsers for the three major firmware documentation sources:

### 1. Prusa Documentation Parser
- **Source**: https://help.prusa3d.com/article/buddy-firmware-specific-g-code-commands_633112
- **Target**: Extract all G-codes (G0, G1, G26, etc.) and M-codes (M104, M109, M862.3, etc.)
- **Format**: HTML parsing of structured content
- **Special handling**: Multi-part commands (M862.1, M862.2, etc.), T-codes, P-codes

### 2. Marlin Documentation Parser  
- **Source**: https://marlinfw.org/docs/gcode/ (G-code reference)
- **Target**: Extract comprehensive Marlin 2.x command set
- **Format**: Website scraping of command documentation
- **Special handling**: Version-specific commands, configuration-dependent features

### 3. Klipper Documentation Parser
- **Source**: https://www.klipper3d.org/G-Codes.html
- **Target**: Extract Klipper-specific commands and macro system
- **Format**: Markdown/HTML parsing
- **Special handling**: Configuration commands, macro definitions, extended commands

## Technical Requirements

### Parser Implementation Options
1. **Rust** (preferred for consistency with main project)
   - Use `reqwest` for HTTP requests
   - Use `scraper` or `select` for HTML parsing
   - Use `serde` for structured output

2. **Python** (alternative for rapid development)
   - Use `requests` + `BeautifulSoup` for web scraping
   - Use `argparse` for CLI interface
   - Output JSON/TOML for consumption

### Output Format
```json
{
  "source": "prusa|marlin|klipper",
  "url": "documentation_url",
  "extracted_date": "2025-09-23",
  "commands": [
    {
      "name": "G28",
      "category": "movement",
      "description": "Move to Origin (Home)",
      "parameters": ["X", "Y", "Z", "C", "P", "I"]
    },
    {
      "name": "M862.3", 
      "category": "checking",
      "description": "Check model name",
      "parameters": ["P", "Q"]
    }
  ]
}
```

### Parser Features
- **Command extraction**: Parse command names, descriptions, parameters
- **Categorization**: Group commands by function (movement, temperature, etc.)
- **Parameter detection**: Extract parameter names and types where available
- **Validation**: Compare against existing flavor files
- **Reporting**: Generate missing command reports
- **Caching**: Store results to avoid repeated requests

## Implementation Plan

### Phase 1: Prusa Parser
- [x] Manual extraction completed for current Prusa flavor
- [ ] Create automated parser for https://help.prusa3d.com/article/buddy-firmware-specific-g-code-commands_633112
- [ ] Extract G-codes, M-codes, T-codes sections
- [ ] Handle special command formats (M862.x series)
- [ ] Generate comparison report with current prusa.gcode-flavor.toml

### Phase 2: Marlin Parser
- [x] Manual extraction completed for current Marlin flavor  
- [ ] Create parser for Marlin documentation site
- [ ] Handle multiple documentation pages/sections
- [ ] Extract version-specific command information
- [ ] Generate comparison report with current marlin.gcode-flavor.toml

### Phase 3: Klipper Parser
- [ ] Research Klipper documentation structure
- [ ] Create parser for Klipper G-code reference
- [ ] Handle macro system and configuration commands
- [ ] Prepare for Klipper flavor implementation

### Phase 4: Integration
- [ ] Create unified CLI tool for all parsers
- [ ] Add CI/CD integration for regular checks
- [ ] Create documentation update workflows
- [ ] Generate flavor file templates from parsed data

## Usage Examples

```bash
# Extract commands from Prusa documentation
cargo run --bin doc-parser -- --source prusa --output prusa-commands.json

# Compare with existing flavor file
cargo run --bin doc-parser -- --source prusa --compare resources/flavors/prusa.gcode-flavor.toml

# Generate missing commands report
cargo run --bin doc-parser -- --source marlin --missing-report marlin-missing.txt

# Update flavor file template
cargo run --bin doc-parser -- --source klipper --generate-template klipper-template.toml
```

## Benefits

1. **Automated Validation**: Verify flavor file completeness automatically
2. **Maintenance Efficiency**: Quickly identify documentation updates
3. **Quality Assurance**: Ensure no commands are missed during manual creation
4. **CI/CD Integration**: Automated checks in continuous integration
5. **Future-Proofing**: Easy updates when firmware documentation changes

## Dependencies

- Web scraping libraries (reqwest, scraper for Rust or requests, BeautifulSoup for Python)
- JSON/TOML parsing libraries
- Command-line argument parsing
- Optional: HTTP caching for development efficiency

## Success Criteria

- [ ] Successfully extract all commands from Prusa documentation
- [ ] Generate accurate comparison with existing prusa.gcode-flavor.toml
- [ ] Create working parser for Marlin documentation
- [ ] Implement Klipper documentation parser
- [ ] Validate that automated extraction matches manual flavor files
- [ ] Create comprehensive missing command reports
- [ ] Integrate parsers into development workflow

## Notes

- Parsers should be robust to minor documentation format changes
- Consider rate limiting and respectful scraping practices
- Store parser results for offline development and testing
- Document parser usage and maintenance procedures
- Plan for handling breaking changes in documentation sites