//! In-memory cache implementation (L1 cache)
//!
//! Provides fast, thread-safe in-memory caching with:
//! - LRU eviction policy
//! - TTL support
//! - Size-based eviction
//! - Statistics tracking

use super::{Cache, CacheKey, CacheLevel, CacheMetadata, CacheStats};
use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Memory cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,

    /// Maximum total size in bytes
    pub max_size_bytes: usize,

    /// Default TTL for entries
    pub default_ttl: Duration,

    /// Enable statistics
    pub enable_stats: bool,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            max_size_bytes: 100 * 1024 * 1024,        // 100 MB
            default_ttl: Duration::from_secs(5 * 60), // 5 minutes
            enable_stats: true,
        }
    }
}

/// Cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Entry value
    value: Vec<u8>,

    /// Entry metadata
    metadata: CacheMetadata,
}

impl CacheEntry {
    fn new(value: Vec<u8>, ttl_seconds: u64) -> Self {
        let size_bytes = value.len();
        let metadata = CacheMetadata::new(ttl_seconds, size_bytes, CacheLevel::L1);

        Self { value, metadata }
    }

    fn is_expired(&self) -> bool {
        self.metadata.is_expired()
    }

    fn record_access(&mut self) {
        self.metadata.record_access();
    }
}

/// In-memory cache statistics
pub type MemoryCacheStats = CacheStats;

/// In-memory cache implementation
pub struct MemoryCache {
    /// Cache entries
    entries: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,

    /// Configuration
    config: MemoryCacheConfig,

    /// Statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl MemoryCache {
    /// Create a new memory cache
    pub fn new(config: MemoryCacheConfig) -> Self {
        info!(
            "Creating memory cache with max_entries: {}, max_size: {} MB",
            config.max_entries,
            config.max_size_bytes / (1024 * 1024)
        );

        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get current size in bytes
    async fn current_size(&self) -> usize {
        let entries = self.entries.read().await;
        entries.values().map(|e| e.metadata.size_bytes).sum()
    }

    /// Evict expired entries
    async fn evict_expired(&self) {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        let before_count = entries.len();
        entries.retain(|_, entry| !entry.is_expired());
        let after_count = entries.len();

        let evicted = before_count - after_count;
        if evicted > 0 {
            stats.evictions += evicted as u64;
            debug!("Evicted {} expired entries", evicted);
        }
    }

    /// Evict LRU entry
    async fn evict_lru(&self) {
        let mut entries = self.entries.write().await;

        if let Some((key, _)) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.metadata.last_accessed)
        {
            let key = key.clone();
            entries.remove(&key);

            if self.config.enable_stats {
                let mut stats = self.stats.write().await;
                stats.evictions += 1;
            }

            debug!("Evicted LRU entry: {}", key);
        }
    }

    /// Ensure capacity
    async fn ensure_capacity(&self, new_entry_size: usize) {
        // Evict expired entries first
        self.evict_expired().await;

        let entries = self.entries.read().await;
        let current_count = entries.len();
        let current_size = entries
            .values()
            .map(|e| e.metadata.size_bytes)
            .sum::<usize>();
        drop(entries);

        // Check entry count limit
        if current_count >= self.config.max_entries {
            self.evict_lru().await;
        }

        // Check size limit
        while current_size + new_entry_size > self.config.max_size_bytes {
            self.evict_lru().await;

            let entries = self.entries.read().await;
            let new_size = entries
                .values()
                .map(|e| e.metadata.size_bytes)
                .sum::<usize>();
            drop(entries);

            if new_size == current_size {
                // No more entries to evict
                break;
            }
        }
    }

    /// Get memory cache statistics
    pub async fn memory_stats(&self) -> MemoryCacheStats {
        let stats = self.stats.read().await;
        let mut result = stats.clone();

        let entries = self.entries.read().await;
        result.entry_count = entries.len();
        result.total_size_bytes = entries.values().map(|e| e.metadata.size_bytes).sum();

        result
    }
}

#[async_trait::async_trait]
impl Cache for MemoryCache {
    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>> {
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.total_gets += 1;
        }

        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                // Remove expired entry
                entries.remove(key);

                if self.config.enable_stats {
                    let mut stats = self.stats.write().await;
                    stats.misses += 1;
                    stats.evictions += 1;
                }

                debug!("Cache miss (expired): {}", key);
                Ok(None)
            } else {
                // Update access metadata
                entry.record_access();

                if self.config.enable_stats {
                    let mut stats = self.stats.write().await;
                    stats.hits += 1;
                }

                debug!("Cache hit: {}", key);
                Ok(Some(entry.value.clone()))
            }
        } else {
            if self.config.enable_stats {
                let mut stats = self.stats.write().await;
                stats.misses += 1;
            }

            debug!("Cache miss (not found): {}", key);
            Ok(None)
        }
    }

    async fn set(&self, key: CacheKey, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        let ttl_seconds = ttl.unwrap_or(self.config.default_ttl).as_secs();

        let entry_size = value.len();

        // Ensure we have capacity
        self.ensure_capacity(entry_size).await;

        let entry = CacheEntry::new(value, ttl_seconds);

        let mut entries = self.entries.write().await;
        entries.insert(key.clone(), entry);

        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.total_sets += 1;
        }

        debug!("Cache set: {}", key);
        Ok(())
    }

    async fn delete(&self, key: &CacheKey) -> Result<bool> {
        let mut entries = self.entries.write().await;
        let removed = entries.remove(key).is_some();

        if removed && self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.invalidations += 1;
        }

        debug!("Cache delete: {} (removed: {})", key, removed);
        Ok(removed)
    }

    async fn exists(&self, key: &CacheKey) -> Result<bool> {
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(key) {
            Ok(!entry.is_expired())
        } else {
            Ok(false)
        }
    }

    async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.clear();

        info!("Cache cleared");
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        Ok(self.memory_stats().await)
    }

    fn level(&self) -> CacheLevel {
        CacheLevel::L1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_set_get() {
        let cache = MemoryCache::new(MemoryCacheConfig::default());

        cache
            .set("key1".to_string(), b"value1".to_vec(), None)
            .await
            .unwrap();
        let value = cache.get(&"key1".to_string()).await?;

        assert_eq!(value, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_memory_cache_miss() {
        let cache = MemoryCache::new(MemoryCacheConfig::default());

        let value = cache.get(&"nonexistent".to_string()).await?;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_memory_cache_delete() {
        let cache = MemoryCache::new(MemoryCacheConfig::default());

        cache
            .set("key1".to_string(), b"value1".to_vec(), None)
            .await
            .unwrap();
        let removed = cache.delete(&"key1".to_string()).await?;
        assert!(removed);

        let value = cache.get(&"key1".to_string()).await?;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_memory_cache_stats() {
        let cache = MemoryCache::new(MemoryCacheConfig::default());

        cache
            .set("key1".to_string(), b"value1".to_vec(), None)
            .await
            .unwrap();
        cache.get(&"key1".to_string()).await?; // hit
        cache.get(&"key2".to_string()).await?; // miss

        let stats = cache.stats().await?;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_sets, 1);
    }
}
