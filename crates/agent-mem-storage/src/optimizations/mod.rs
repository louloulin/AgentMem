// ! Database optimization utilities
//!
//! This module provides database optimization features including:
//! - Connection pool configuration
//! - Query optimization helpers
//! - Performance monitoring

#[cfg(feature = "optimizations")]
pub mod pool;
pub mod query_cache;
#[cfg(feature = "optimizations")]
pub mod query_optimizer;

#[cfg(feature = "optimizations")]
pub use pool::{PoolConfig, create_optimized_pool};
pub use query_cache::{QueryCache, QueryCacheConfig};
#[cfg(feature = "optimizations")]
pub use query_optimizer::{QueryOptimizer, QueryPlan};

