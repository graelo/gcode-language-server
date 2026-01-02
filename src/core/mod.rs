//! Core Business Logic
//!
//! Document management and core LSP functionality.

pub mod diagnostics;
pub mod document;

pub use diagnostics::DiagnosticProvider;
pub use document::DocumentManager;
