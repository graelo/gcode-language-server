//! GCode Language Server
//!
//! A clean, fast Language Server Protocol implementation for GCode files.
//!
//! This library provides:
//! - GCode parsing and validation
//! - LSP protocol implementation  
//! - Flavor-based command definitions
//! - Configuration management

// New clean modules
pub mod config;
pub mod core;
pub mod flavor;
pub mod lsp;
pub mod parser;
pub mod validation;

// Re-exports for clean public API
pub use config::Config;
pub use flavor::{Flavor, FlavorRegistry};
pub use parser::{parse_line, ParsedLine};
pub use validation::{validate_document, Diagnostic};
