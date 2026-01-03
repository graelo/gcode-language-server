# gcode-language-server

[![release](https://img.shields.io/github/v/release/graelo/gcode-language-server)](https://github.com/graelo/gcode-language-server/releases/latest)
[![build status](https://github.com/graelo/gcode-language-server/actions/workflows/ci.yml/badge.svg)](https://github.com/graelo/gcode-language-server/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/graelo/gcode-language-server/branch/main/graph/badge.svg)](https://codecov.io/gh/graelo/gcode-language-server)
[![rust 2021 edition](https://img.shields.io/badge/edition-2021-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![license](https://img.shields.io/github/license/graelo/gcode-language-server)](LICENSE)

A Language Server Protocol (LSP) implementation for G-code, written in Rust. Provides intelligent editing features for G-code files used in 3D printing and CNC machines.

## Features

- **Hover**: Command descriptions from active flavor
- **Diagnostics**: Unknown commands, invalid parameters
- **Completions**: Commands and parameters with G-code format
- **Document Symbols**: Navigation outline (in progress)
- **Multi-flavor support**: Prusa, Marlin, Klipper
- **Live reload**: Flavor files watched for changes
- **High performance**: 240-360 MiB/s tokenization on 20MB files

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/gcode-ls --help
```

## Editor Setup

### Neovim

#### 1. Filetype Detection

Add to `~/.config/nvim/filetype.lua`:

```lua
vim.filetype.add({
  extension = {
    gcode = 'gcode',
    nc = 'gcode',
    ngc = 'gcode',
  },
})
```

#### 2. LSP Configuration

Add to your LSP configuration (e.g., `~/.config/nvim/lua/plugins/lsp.lua`):

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Register gcode-ls as a custom LSP server
if not configs.gcode_ls then
  configs.gcode_ls = {
    default_config = {
      cmd = { 'gcode-ls' },
      filetypes = { 'gcode' },
      root_dir = lspconfig.util.find_git_ancestor,
      single_file_support = true,
    },
  }
end

-- Setup with options
lspconfig.gcode_ls.setup({
  cmd = { 'gcode-ls', '--flavor=prusa' },  -- or marlin, klipper
  on_attach = function(client, bufnr)
    -- Keymaps
    local opts = { buffer = bufnr, silent = true }
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('i', '<C-k>', vim.lsp.buf.signature_help, opts)
  end,
})
```

#### 3. Minimal Setup (init.vim)

For users preferring Vimscript:

```vim
" ~/.config/nvim/init.vim
autocmd BufRead,BufNewFile *.gcode,*.nc,*.ngc setfiletype gcode

lua << EOF
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

configs.gcode_ls = {
  default_config = {
    cmd = { 'gcode-ls' },
    filetypes = { 'gcode' },
    root_dir = lspconfig.util.find_git_ancestor,
    single_file_support = true,
  },
}

lspconfig.gcode_ls.setup{}
EOF
```

### VS Code

Create `.vscode/settings.json` in your workspace:

```json
{
  "gcode-ls.flavor": "prusa",
  "gcode-ls.logLevel": "info"
}
```

> Note: A VS Code extension is planned but not yet available.

## CLI Options

```bash
gcode-ls [OPTIONS]

Options:
  --flavor <FLAVOR>          G-code flavor (prusa, marlin, klipper)
  --flavor-dir <DIR>         Custom flavor directory
  --log-level <LEVEL>        Log level: trace, debug, info, warn, error
  -h, --help                 Print help
  -V, --version              Print version
```

## Flavor Selection

Priority (highest to lowest):

1. **Per-file modeline**: `; gcode_flavor=prusa`
2. **CLI flag**: `--flavor=marlin`
3. **Project config**: `.gcode.toml` with `default_flavor`
4. **Built-in default**: `prusa`

### Project Configuration

Create `.gcode.toml` in your project root:

```toml
[project]
default_flavor = "marlin"
```

### Per-file Override

Add a modeline comment to your G-code file:

```gcode
; gcode_flavor=klipper
G28  ; Home all axes
```

## Custom Flavors

Create a TOML file in `~/.config/gcode-ls/flavors/`:

```toml
[flavor]
name = "my_printer"
version = "1.0"
description = "Custom flavor for my printer"

[[commands]]
name = "G28"
description_short = "Home axes"
description_long = "Home printer axes to endstop positions"

[[commands.parameters]]
name = "X"
type = "bool"
required = false
description = "Home X axis only"
```

## Development

```bash
# Build
cargo build

# Test
cargo test

# Benchmarks
cargo bench

# Lint
cargo clippy
```

## License

MIT
