//! Multi-level caching system for AgentMem
//!
//! Provides a comprehensive caching strategy with:
//! - L1: In-memory cache (fastest, limited capacity)
//! - L2: Redis cache (fast, larger capacity, distributed)
//! - Cache warming (preload frequently accessed data)
//! - Cache invalidation strategies (TTL, LRU, manual)

pub mod learning_warmer;
pub mod memory_cache;
pub mod monitor;
pub mod multi_layer;
pub mod multi_level;
pub mod warming;

pub use learning_warmer::{LearningBasedCacheWarmer, LearningWarmingConfig};
pub use memory_cache::{MemoryCache, MemoryCacheConfig, MemoryCacheStats};
pub use monitor::{CacheMonitor, MonitorConfig, PerformanceReport, PerformanceSnapshot};
pub use multi_layer::MultiLayerCache;
pub use multi_level::{CacheLevel, MultiLevelCache, MultiLevelCacheConfig};
pub use warming::{CacheWarmer, CacheWarmingConfig, DataLoader, WarmingStats, WarmingStrategy};

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache key type
pub type CacheKey = String;

/// Cache value trait
pub trait CacheValue: Clone + Send + Sync + 'static {}

impl<T: Clone + Send + Sync + 'static> CacheValue for T {}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// When the entry was created
    pub created_at: u64,

    /// Time-to-live in seconds
    pub ttl_seconds: u64,

    /// Access count
    pub access_count: u64,

    /// Last accessed timestamp
    pub last_accessed: u64,

    /// Entry size in bytes (approximate)
    pub size_bytes: usize,

    /// Cache level where this entry resides
    pub level: CacheLevel,
}

impl CacheMetadata {
    /// Create new metadata
    pub fn new(ttl_seconds: u64, size_bytes: usize, level: CacheLevel) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time should be after UNIX_EPOCH (this should never fail)")
            .as_secs();

        Self {
            created_at: now,
            ttl_seconds,
            access_count: 0,
            last_accessed: now,
            size_bytes,
            level,
        }
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time should be after UNIX_EPOCH (this should never fail)")
            .as_secs();

        now - self.created_at > self.ttl_seconds
    }

    /// Record an access
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of get operations
    pub total_gets: u64,

    /// Number of cache hits
    pub hits: u64,

    /// Number of cache misses
    pub misses: u64,

    /// Number of set operations
    pub total_sets: u64,

    /// Number of evictions
    pub evictions: u64,

    /// Number of invalidations
    pub invalidations: u64,

    /// Total size in bytes
    pub total_size_bytes: usize,

    /// Number of entries
    pub entry_count: usize,
}

impl CacheStats {
    /// Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_gets == 0 {
            0.0
        } else {
            (self.hits as f64 / self.total_gets as f64) * 100.0
        }
    }

    /// Calculate miss rate
    pub fn miss_rate(&self) -> f64 {
        100.0 - self.hit_rate()
    }

    /// Merge stats from another source
    pub fn merge(&mut self, other: &CacheStats) {
        self.total_gets += other.total_gets;
        self.hits += other.hits;
        self.misses += other.misses;
        self.total_sets += other.total_sets;
        self.evictions += other.evictions;
        self.invalidations += other.invalidations;
        self.total_size_bytes += other.total_size_bytes;
        self.entry_count += other.entry_count;
    }
}

/// Cache invalidation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationStrategy {
    /// Time-to-live based invalidation
    TTL(Duration),

    /// Least Recently Used eviction
    LRU,

    /// Least Frequently Used eviction
    LFU,

    /// Manual invalidation only
    Manual,

    /// Combination of strategies
    Hybrid {
        ttl: Duration,
        eviction: EvictionPolicy,
    },
}

/// Eviction policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,

    /// Least Frequently Used
    LFU,

    /// First In First Out
    FIFO,

    /// Random eviction
    Random,
}

/// Cache trait for different cache implementations
#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>>;

    /// Set a value in the cache
    async fn set(&self, key: CacheKey, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;

    /// Delete a value from the cache
    async fn delete(&self, key: &CacheKey) -> Result<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &CacheKey) -> Result<bool>;

    /// Clear all entries
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;

    /// Get cache level
    fn level(&self) -> CacheLevel;
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable L1 (memory) cache
    pub enable_l1: bool,

    /// Enable L2 (Redis) cache
    pub enable_l2: bool,

    /// L1 cache max entries
    pub l1_max_entries: usize,

    /// L1 cache max size in bytes
    pub l1_max_size_bytes: usize,

    /// L1 default TTL
    pub l1_default_ttl: Duration,

    /// L2 Redis URL
    pub l2_redis_url: Option<String>,

    /// L2 default TTL
    pub l2_default_ttl: Duration,

    /// Invalidation strategy
    pub invalidation_strategy: InvalidationStrategy,

    /// Enable cache warming
    pub enable_warming: bool,

    /// Enable statistics
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_l1: true,
            enable_l2: false,
            l1_max_entries: 10000,
            l1_max_size_bytes: 100 * 1024 * 1024, // 100 MB
            l1_default_ttl: Duration::from_secs(5 * 60), // 5 minutes
            l2_redis_url: None,
            l2_default_ttl: Duration::from_secs(30 * 60), // 30 minutes
            invalidation_strategy: InvalidationStrategy::Hybrid {
                ttl: Duration::from_secs(5 * 60),
                eviction: EvictionPolicy::LRU,
            },
            enable_warming: false,
            enable_stats: true,
        }
    }
}

impl CacheConfig {
    /// Create production configuration
    pub fn production() -> Self {
        Self {
            enable_l1: true,
            enable_l2: true,
            l1_max_entries: 50000,
            l1_max_size_bytes: 500 * 1024 * 1024, // 500 MB
            l1_default_ttl: Duration::from_secs(10 * 60), // 10 minutes
            l2_redis_url: Some("redis://localhost:6379".to_string()),
            l2_default_ttl: Duration::from_secs(60 * 60), // 1 hour
            invalidation_strategy: InvalidationStrategy::Hybrid {
                ttl: Duration::from_secs(10 * 60),
                eviction: EvictionPolicy::LRU,
            },
            enable_warming: true,
            enable_stats: true,
        }
    }

    /// Create development configuration
    pub fn development() -> Self {
        Self {
            enable_l1: true,
            enable_l2: false,
            l1_max_entries: 1000,
            l1_max_size_bytes: 10 * 1024 * 1024,         // 10 MB
            l1_default_ttl: Duration::from_secs(2 * 60), // 2 minutes
            l2_redis_url: None,
            l2_default_ttl: Duration::from_secs(10 * 60), // 10 minutes
            invalidation_strategy: InvalidationStrategy::TTL(Duration::from_secs(2 * 60)),
            enable_warming: false,
            enable_stats: true,
        }
    }
}
