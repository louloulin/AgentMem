//! AgentMem Plugin SDK
//! 
//! This SDK provides the core types and utilities for developing AgentMem plugins.

pub mod types;
pub mod plugin;
pub mod host;
pub mod macros;

// Re-export commonly used types
pub use types::*;
pub use plugin::*;
pub use host::*;

// Re-export extism_pdk for plugin development
pub use extism_pdk;

