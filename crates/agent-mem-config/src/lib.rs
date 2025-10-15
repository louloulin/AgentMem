//! # Agent Memory Configuration
//!
//! Configuration management for the AgentMem memory platform.

pub mod database;
pub mod factory;
pub mod memory;
pub mod storage;
pub mod validation;

pub use database::{DatabaseBackend, DatabaseConfig};
pub use factory::ConfigFactory;
pub use memory::MemoryConfig;
pub use storage::{
    DeploymentMode, EmbeddedModeConfig, PoolConfig, ServerModeConfig, VectorServiceType,
    VectorStoreConfig,
};
pub use validation::*;
