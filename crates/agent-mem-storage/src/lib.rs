//! # Agent Memory Storage
//!
//! 存储后端模块，为AgentMem记忆平台提供多种存储解决方案。
//!
//! 本模块提供：
//! - 统一的存储接口抽象
//! - 多种向量存储后端支持
//! - 本地和云端存储选项
//! - 存储工厂模式
//! - 特性门控支持

pub mod backends;
pub mod cache;
pub mod factory;  // Memory store factory (factory/mod.rs)
pub mod vector_factory;  // Vector store factory (vector_factory.rs)
pub mod graph;
pub mod optimizations;
pub mod performance;
pub mod utils;
pub mod vector;

// Re-export memory store factory
pub use factory::StorageFactory as MemoryStoreFactory;
pub use factory::{StorageConfig, StorageBackend, AllStores, create_factory};

// Re-export vector store factory
pub use vector_factory::StorageFactory as VectorStoreFactory;

pub use graph::GraphStoreFactory;
pub use optimizations::{QueryCache, QueryCacheConfig};
#[cfg(feature = "optimizations")]
pub use optimizations::{PoolConfig, QueryOptimizer, create_optimized_pool};

// 重新导出常用类型
pub use agent_mem_traits::{AgentMemError, Result, VectorStore, VectorStoreConfig};
