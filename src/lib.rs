// New clean modules
pub mod config;
pub mod core;
pub mod flavor;
pub mod lsp;
pub mod parser;
pub mod validation;

// Re-exports for clean public API
pub use config::{Args, Config};
pub use flavor::{Flavor, FlavorRegistry};
pub use parser::{ParsedLine, parse_line};
pub use validation::{Diagnostic, validate_document};
