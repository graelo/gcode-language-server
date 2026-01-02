//! Validation Engine
//!
//! Clean separation of validation logic from parsing and LSP concerns.

pub mod engine;

pub use engine::{validate_document, validate_line, Diagnostic, Severity};

// Re-export common types
pub use engine::ValidationResult;
