//! Clean Flavor System
//!
//! Simplified flavor management without the over-engineering.

pub mod schema;
pub mod registry;

pub use schema::{Flavor, CommandDef, ParameterDef};
pub use registry::FlavorRegistry;