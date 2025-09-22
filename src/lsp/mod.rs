//! LSP Protocol Implementation
//!
//! Clean LSP backend focused only on protocol handling.

pub mod backend;
pub mod document;
pub mod handlers;
pub mod server;

pub use backend::Backend;
