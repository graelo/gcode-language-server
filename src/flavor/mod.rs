//! Clean Flavor System
//!
//! Simplified flavor management without the over-engineering.

pub mod registry;
pub mod schema;

pub use registry::FlavorRegistry;
pub use schema::{CommandDef, Flavor, ParameterDef};
