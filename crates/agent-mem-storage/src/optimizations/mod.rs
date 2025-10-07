// ! Database optimization utilities
//!
//! This module provides database optimization features including:
//! - Connection pool configuration
//! - Query optimization helpers
//! - Performance monitoring

pub mod pool;
pub mod query_cache;
pub mod query_optimizer;

pub use pool::{PoolConfig, create_optimized_pool};
pub use query_cache::{QueryCache, QueryCacheConfig};
pub use query_optimizer::{QueryOptimizer, QueryPlan};

