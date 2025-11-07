//! Query result caching
//!
//! Provides in-memory caching for frequently accessed query results
//! to reduce database load and improve response times.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Query cache configuration
#[derive(Debug, Clone)]
pub struct QueryCacheConfig {
    /// Maximum number of cached entries
    pub max_entries: usize,

    /// Default TTL for cached entries
    pub default_ttl: Duration,

    /// Enable cache statistics
    pub enable_stats: bool,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(5 * 60), // 5 minutes
            enable_stats: true,
        }
    }
}

impl QueryCacheConfig {
    /// Create a production configuration
    pub fn production() -> Self {
        Self {
            max_entries: 10000,
            default_ttl: Duration::from_secs(10 * 60), // 10 minutes
            enable_stats: true,
        }
    }

    /// Create a development configuration
    pub fn development() -> Self {
        Self {
            max_entries: 500,
            default_ttl: Duration::from_secs(2 * 60), // 2 minutes
            enable_stats: true,
        }
    }
}

/// Cache key for query results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Query identifier (e.g., "get_memories_by_agent")
    pub query_id: String,

    /// Query parameters (serialized)
    pub params: String,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(query_id: impl Into<String>, params: impl Serialize) -> Self {
        let params_str = serde_json::to_string(&params).unwrap_or_default();
        Self {
            query_id: query_id.into(),
            params: params_str,
        }
    }

    /// Create a cache key from raw strings
    pub fn from_strings(query_id: String, params: String) -> Self {
        Self { query_id, params }
    }
}

/// Cached entry
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// Cached value
    value: T,

    /// When the entry was created
    created_at: Instant,

    /// Time-to-live
    ttl: Duration,

    /// Access count
    access_count: u64,

    /// Last accessed time
    last_accessed: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            ttl,
            access_count: 0,
            last_accessed: now,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,

    /// Total number of cache misses
    pub misses: u64,

    /// Total number of evictions
    pub evictions: u64,

    /// Current number of entries
    pub entries: usize,
}

impl CacheStats {
    /// Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Query result cache
pub struct QueryCache<T: Clone> {
    /// Cache entries
    entries: Arc<RwLock<HashMap<CacheKey, CacheEntry<T>>>>,

    /// Configuration
    config: QueryCacheConfig,

    /// Statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl<T: Clone> QueryCache<T> {
    /// Create a new query cache
    pub fn new(config: QueryCacheConfig) -> Self {
        info!(
            "Creating query cache with max_entries: {}",
            config.max_entries
        );
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &CacheKey) -> Option<T> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                // Remove expired entry
                entries.remove(key);
                self.record_miss().await;
                debug!("Cache miss (expired): {:?}", key);
                None
            } else {
                // Update access stats
                entry.access();
                self.record_hit().await;
                debug!("Cache hit: {:?}", key);
                Some(entry.value.clone())
            }
        } else {
            self.record_miss().await;
            debug!("Cache miss (not found): {:?}", key);
            None
        }
    }

    /// Put a value into the cache
    pub async fn put(&self, key: CacheKey, value: T) {
        self.put_with_ttl(key, value, self.config.default_ttl).await;
    }

    /// Put a value into the cache with custom TTL
    pub async fn put_with_ttl(&self, key: CacheKey, value: T, ttl: Duration) {
        let mut entries = self.entries.write().await;

        // Check if we need to evict entries
        if entries.len() >= self.config.max_entries {
            self.evict_lru(&mut entries).await;
        }

        entries.insert(key.clone(), CacheEntry::new(value, ttl));
        debug!("Cache put: {:?}", key);

        // Update stats
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.entries = entries.len();
        }
    }

    /// Invalidate a cache entry
    pub async fn invalidate(&self, key: &CacheKey) {
        let mut entries = self.entries.write().await;
        if entries.remove(key).is_some() {
            debug!("Cache invalidated: {:?}", key);

            // Update stats
            if self.config.enable_stats {
                let mut stats = self.stats.write().await;
                stats.entries = entries.len();
            }
        }
    }

    /// Invalidate all entries matching a query_id prefix
    pub async fn invalidate_prefix(&self, query_id_prefix: &str) {
        let mut entries = self.entries.write().await;
        let keys_to_remove: Vec<_> = entries
            .keys()
            .filter(|k| k.query_id.starts_with(query_id_prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            entries.remove(&key);
            debug!("Cache invalidated (prefix): {:?}", key);
        }

        // Update stats
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.entries = entries.len();
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        info!("Cache cleared");

        // Update stats
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.entries = 0;
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Evict least recently used entry
    async fn evict_lru(&self, entries: &mut HashMap<CacheKey, CacheEntry<T>>) {
        if let Some((key, _)) = entries.iter().min_by_key(|(_, entry)| entry.last_accessed) {
            let key = key.clone();
            entries.remove(&key);
            debug!("Cache evicted (LRU): {:?}", key);

            // Update stats
            if self.config.enable_stats {
                let mut stats = self.stats.write().await;
                stats.evictions += 1;
            }
        }
    }

    async fn record_hit(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.hits += 1;
        }
    }

    async fn record_miss(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
        }
    }
}

impl<T: Clone> Clone for QueryCache<T> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = QueryCache::<String>::new(QueryCacheConfig::default());
        let key = CacheKey::new("test_query", vec!["param1", "param2"]);

        cache.put(key.clone(), "test_value".to_string()).await;
        let value = cache.get(&key).await;

        assert_eq!(value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = QueryCache::<String>::new(QueryCacheConfig::default());
        let key = CacheKey::new("test_query", vec!["param1"]);

        let value = cache.get(&key).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = QueryCache::<String>::new(QueryCacheConfig::default());
        let key = CacheKey::new("test_query", vec!["param1"]);

        cache.put(key.clone(), "test_value".to_string()).await;
        cache.invalidate(&key).await;

        let value = cache.get(&key).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = QueryCache::<String>::new(QueryCacheConfig::default());
        let key = CacheKey::new("test_query", vec!["param1"]);

        cache.put(key.clone(), "test_value".to_string()).await;
        cache.get(&key).await; // hit
        cache.get(&CacheKey::new("other", vec!["param"])).await; // miss

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 50.0);
    }
}
