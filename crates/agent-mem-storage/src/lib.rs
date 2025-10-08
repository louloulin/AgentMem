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
pub mod factory;
pub mod graph;
pub mod optimizations;
pub mod performance;
pub mod utils;
pub mod vector;

pub use factory::StorageFactory;
pub use graph::GraphStoreFactory;
pub use optimizations::{QueryCache, QueryCacheConfig};
#[cfg(feature = "optimizations")]
pub use optimizations::{PoolConfig, QueryOptimizer, create_optimized_pool};

// 重新导出常用类型
pub use agent_mem_traits::{AgentMemError, Result, VectorStore, VectorStoreConfig};
