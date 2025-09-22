//! Core Business Logic
//!
//! Document management and core LSP functionality.

pub mod document;
pub mod diagnostics;

pub use document::DocumentManager;
pub use diagnostics::DiagnosticProvider;