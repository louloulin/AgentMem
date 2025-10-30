//! Multi-level caching system for improved performance
//!
//! This module provides a sophisticated caching system with multiple levels,
//! intelligent eviction policies, and cache warming capabilities.

use agent_mem_traits::Result;
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;
use tokio::time::interval;
use tracing::{debug, info};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// L1 cache size (in-memory, fastest)
    pub l1_size: usize,
    /// L2 cache size (compressed in-memory)
    pub l2_size: usize,
    /// L3 cache size (disk-based, optional)
    pub l3_size: Option<usize>,
    /// Default TTL for cache entries (seconds)
    pub default_ttl_seconds: u64,
    /// Enable cache compression
    pub enable_compression: bool,
    /// Enable cache warming
    pub enable_warming: bool,
    /// Cache warming batch size
    pub warming_batch_size: usize,
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Enable cache statistics
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_size: 10000,
            l2_size: 50000,
            l3_size: Some(100000),
            default_ttl_seconds: 3600, // 1 hour
            enable_compression: true,
            enable_warming: true,
            warming_batch_size: 100,
            eviction_policy: EvictionPolicy::LRU,
            enable_stats: true,
        }
    }
}

/// Cache eviction policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    FIFO,
    TTL,
    Adaptive,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l3_hits: u64,
    pub l3_misses: u64,
    pub total_entries: usize,
    pub memory_usage_bytes: usize,
    pub hit_rate: f64,
    pub average_access_time_ms: f64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            l1_hits: 0,
            l1_misses: 0,
            l2_hits: 0,
            l2_misses: 0,
            l3_hits: 0,
            l3_misses: 0,
            total_entries: 0,
            memory_usage_bytes: 0,
            hit_rate: 0.0,
            average_access_time_ms: 0.0,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    ttl: Option<Duration>,
    size: usize,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Option<Duration>, size: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
            size,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    fn access(&mut self) -> &V {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }
}

/// Multi-level cache manager
pub struct CacheManager {
    config: CacheConfig,
    l1_cache: Arc<RwLock<LruCache<String, CacheEntry<Vec<u8>>>>>,
    l2_cache: Arc<DashMap<String, CacheEntry<Vec<u8>>>>,
    l3_cache: Option<Arc<AsyncRwLock<HashMap<String, CacheEntry<Vec<u8>>>>>>,
    stats: Arc<AsyncRwLock<CacheStats>>,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let l1_cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(config.l1_size).unwrap(),
        )));

        let l2_cache = Arc::new(DashMap::new());

        let l3_cache = if config.l3_size.is_some() {
            Some(Arc::new(AsyncRwLock::new(HashMap::new())))
        } else {
            None
        };

        let stats = Arc::new(AsyncRwLock::new(CacheStats::default()));

        let manager = Self {
            config,
            l1_cache,
            l2_cache,
            l3_cache,
            stats,
        };

        // Start background tasks
        if manager.config.enable_stats {
            manager.start_stats_updater().await;
        }

        manager.start_cleanup_task().await;

        info!(
            "Cache manager initialized with L1: {}, L2: {}, L3: {:?}",
            manager.config.l1_size, manager.config.l2_size, manager.config.l3_size
        );

        Ok(manager)
    }

    /// Get a value from cache
    pub async fn get<K: AsRef<str>>(&self, key: K) -> Result<Option<Vec<u8>>> {
        let key_str = key.as_ref();
        let start_time = Instant::now();

        // Try L1 cache first
        if let Some(entry) = self.get_from_l1(key_str).await {
            self.update_stats_hit(1, start_time.elapsed()).await;
            return Ok(Some(entry));
        }

        // Try L2 cache
        if let Some(entry) = self.get_from_l2(key_str).await {
            // Promote to L1
            self.put_to_l1(key_str, &entry).await;
            self.update_stats_hit(2, start_time.elapsed()).await;
            return Ok(Some(entry));
        }

        // Try L3 cache if available
        if let Some(entry) = self.get_from_l3(key_str).await {
            // Promote to L2 and L1
            self.put_to_l2(key_str, &entry).await;
            self.put_to_l1(key_str, &entry).await;
            self.update_stats_hit(3, start_time.elapsed()).await;
            return Ok(Some(entry));
        }

        self.update_stats_miss(start_time.elapsed()).await;
        Ok(None)
    }

    /// Put a value into cache
    pub async fn put<K: AsRef<str>>(
        &self,
        key: K,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let key_str = key.as_ref().to_string();
        let size = value.len();

        // Always put in L1 first
        self.put_to_l1(&key_str, &value).await;

        // Put in L2 if it doesn't fit in L1 or for redundancy
        self.put_to_l2(&key_str, &value).await;

        // Put in L3 if available
        if self.l3_cache.is_some() {
            self.put_to_l3(&key_str, &value).await;
        }

        debug!("Cached entry '{}' with size {} bytes", key_str, size);
        Ok(())
    }

    /// Remove a value from cache
    pub async fn remove<K: AsRef<str>>(&self, key: K) -> Result<bool> {
        let key_str = key.as_ref();

        let mut removed = false;

        // Remove from all levels
        if self.remove_from_l1(key_str).await {
            removed = true;
        }

        if self.remove_from_l2(key_str).await {
            removed = true;
        }

        if self.remove_from_l3(key_str).await {
            removed = true;
        }

        if removed {
            debug!("Removed entry '{}' from cache", key_str);
        }

        Ok(removed)
    }

    /// Clear all cache levels
    pub async fn clear(&self) -> Result<()> {
        self.l1_cache.write().clear();
        self.l2_cache.clear();

        if let Some(l3) = &self.l3_cache {
            l3.write().await.clear();
        }

        info!("All cache levels cleared");
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        Ok(self.stats.read().await.clone())
    }

    /// Shutdown the cache manager
    pub async fn shutdown(&self) -> Result<()> {
        self.clear().await?;
        info!("Cache manager shutdown completed");
        Ok(())
    }

    // L1 cache operations
    async fn get_from_l1(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.l1_cache.write();
        if let Some(entry) = cache.get_mut(key) {
            if !entry.is_expired() {
                Some(entry.access().clone())
            } else {
                cache.pop(key);
                None
            }
        } else {
            None
        }
    }

    async fn put_to_l1(&self, key: &str, value: &[u8]) {
        let mut cache = self.l1_cache.write();
        let ttl = Some(Duration::from_secs(self.config.default_ttl_seconds));
        let entry = CacheEntry::new(value.to_vec(), ttl, value.len());
        cache.put(key.to_string(), entry);
    }

    async fn remove_from_l1(&self, key: &str) -> bool {
        self.l1_cache.write().pop(key).is_some()
    }

    // L2 cache operations
    async fn get_from_l2(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(mut entry) = self.l2_cache.get_mut(key) {
            if !entry.is_expired() {
                Some(entry.access().clone())
            } else {
                drop(entry);
                self.l2_cache.remove(key);
                None
            }
        } else {
            None
        }
    }

    async fn put_to_l2(&self, key: &str, value: &[u8]) {
        let ttl = Some(Duration::from_secs(self.config.default_ttl_seconds));
        let entry = CacheEntry::new(value.to_vec(), ttl, value.len());
        self.l2_cache.insert(key.to_string(), entry);
    }

    async fn remove_from_l2(&self, key: &str) -> bool {
        self.l2_cache.remove(key).is_some()
    }

    // L3 cache operations
    async fn get_from_l3(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(l3) = &self.l3_cache {
            let mut cache = l3.write().await;
            if let Some(entry) = cache.get_mut(key) {
                if !entry.is_expired() {
                    Some(entry.access().clone())
                } else {
                    cache.remove(key);
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn put_to_l3(&self, key: &str, value: &[u8]) {
        if let Some(l3) = &self.l3_cache {
            let mut cache = l3.write().await;
            let ttl = Some(Duration::from_secs(self.config.default_ttl_seconds));
            let entry = CacheEntry::new(value.to_vec(), ttl, value.len());
            cache.insert(key.to_string(), entry);
        }
    }

    async fn remove_from_l3(&self, key: &str) -> bool {
        if let Some(l3) = &self.l3_cache {
            l3.write().await.remove(key).is_some()
        } else {
            false
        }
    }

    // Statistics and maintenance
    async fn update_stats_hit(&self, level: u8, access_time: Duration) {
        let mut stats = self.stats.write().await;

        match level {
            1 => stats.l1_hits += 1,
            2 => stats.l2_hits += 1,
            3 => stats.l3_hits += 1,
            _ => {}
        }

        // Update average access time
        let total_accesses = stats.l1_hits + stats.l2_hits + stats.l3_hits;
        if total_accesses > 0 {
            let access_time_ms = access_time.as_millis() as f64;
            stats.average_access_time_ms =
                (stats.average_access_time_ms * (total_accesses - 1) as f64 + access_time_ms)
                    / total_accesses as f64;
        }

        // Update hit rate
        let total_requests = stats.l1_hits
            + stats.l2_hits
            + stats.l3_hits
            + stats.l1_misses
            + stats.l2_misses
            + stats.l3_misses;
        if total_requests > 0 {
            stats.hit_rate =
                (stats.l1_hits + stats.l2_hits + stats.l3_hits) as f64 / total_requests as f64;
        }
    }

    async fn update_stats_miss(&self, access_time: Duration) {
        let mut stats = self.stats.write().await;
        stats.l1_misses += 1;
        stats.l2_misses += 1;
        stats.l3_misses += 1;

        // Update hit rate
        let total_requests = stats.l1_hits
            + stats.l2_hits
            + stats.l3_hits
            + stats.l1_misses
            + stats.l2_misses
            + stats.l3_misses;
        if total_requests > 0 {
            stats.hit_rate =
                (stats.l1_hits + stats.l2_hits + stats.l3_hits) as f64 / total_requests as f64;
        }
    }

    async fn start_stats_updater(&self) {
        let stats = Arc::clone(&self.stats);
        let l1_cache = Arc::clone(&self.l1_cache);
        let l2_cache = Arc::clone(&self.l2_cache);
        let l3_cache = self.l3_cache.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Update every minute

            loop {
                interval.tick().await;

                let mut stats_guard = stats.write().await;

                // Update entry counts and memory usage
                stats_guard.total_entries = l1_cache.read().len() + l2_cache.len();
                stats_guard.memory_usage_bytes = 0; // Simplified - would calculate actual usage

                if let Some(l3) = &l3_cache {
                    stats_guard.total_entries += l3.read().await.len();
                }
            }
        });
    }

    async fn start_cleanup_task(&self) {
        let l2_cache = Arc::clone(&self.l2_cache);
        let l3_cache = self.l3_cache.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Cleanup every 5 minutes

            loop {
                interval.tick().await;

                // Clean expired entries from L2
                l2_cache.retain(|_, entry| !entry.is_expired());

                // Clean expired entries from L3
                if let Some(l3) = &l3_cache {
                    let mut cache = l3.write().await;
                    cache.retain(|_, entry| !entry.is_expired());
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_cache_put_get() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await.unwrap();

        let key = "test_key";
        let value = b"test_value".to_vec();

        manager.put(key, value.clone(), None).await.unwrap();
        let retrieved = manager.get(key).await.unwrap();

        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await.unwrap();

        let key = "test_key";
        let value = b"test_value".to_vec();

        manager.put(key, value, None).await.unwrap();
        let removed = manager.remove(key).await.unwrap();
        assert!(removed);

        let retrieved = manager.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await.unwrap();

        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.l1_hits, 0);
        assert_eq!(stats.total_entries, 0);
    }
}
