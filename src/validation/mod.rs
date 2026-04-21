//! Validation Engine
//!
//! Clean separation of validation logic from parsing and LSP concerns.

pub mod engine;

pub use engine::{Diagnostic, Severity, validate_document, validate_line};

// Re-export common types
pub use engine::ValidationResult;
