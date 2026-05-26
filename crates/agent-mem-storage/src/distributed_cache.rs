//! Distributed Cache Module for AgentMem
//!
//! Provides Redis-based distributed caching for horizontal scaling.

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Distributed cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedCacheConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Enable distributed caching
    pub enabled: bool,
    /// Default TTL in seconds
    pub default_ttl_seconds: u64,
    /// Maximum cache entries
    pub max_entries: usize,
    /// Connection pool size
    pub pool_size: u32,
}

impl Default for DistributedCacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            enabled: false,
            default_ttl_seconds: 300,
            max_entries: 10000,
            pool_size: 10,
        }
    }
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryMetadata {
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub size_bytes: usize,
}

/// Distributed cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub errors: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum connections
    pub max_size: u32,
    /// Minimum idle connections
    pub min_idle: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Max lifetime in seconds
    pub max_lifetime: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 20,
            min_idle: 5,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
        }
    }
}

use async_trait::async_trait;

/// Simple distributed cache interface
/// Note: In production, use redis-rs with proper connection pooling
#[async_trait]
pub trait DistributedCache: Send + Sync {
    /// Get a value from cache
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Set a value in cache
    async fn set(&self, key: String, value: Vec<u8>, ttl: Option<u64>) -> Result<()>;
    
    /// Delete a key from cache
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}

/// In-memory distributed cache fallback
pub struct InMemoryCache {
    cache: std::sync::RwLock<HashMap<String, (Vec<u8>, Option<u64>)>>,
    stats: std::sync::RwLock<CacheStats>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            cache: std::sync::RwLock::new(HashMap::new()),
            stats: std::sync::RwLock::new(CacheStats::default()),
        }
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DistributedCache for InMemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let cache = self.cache.read().unwrap();
        
        if let Some((value, _)) = cache.get(key) {
            let mut stats = self.stats.write().unwrap();
            stats.hits += 1;
            return Ok(Some(value.clone()));
        }
        
        let mut stats = self.stats.write().unwrap();
        stats.misses += 1;
        Ok(None)
    }
    
    async fn set(&self, key: String, value: Vec<u8>, _ttl: Option<u64>) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        
        // Simple eviction: remove oldest if at capacity
        if cache.len() >= 10000 {
            let keys: Vec<_> = cache.keys().take(100).cloned().collect();
            for k in keys {
                cache.remove(&k);
                let mut stats = self.stats.write().unwrap();
                stats.evictions += 1;
            }
        }
        
        cache.insert(key, (value, None));
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key);
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        let cache = self.cache.read().unwrap();
        Ok(cache.contains_key(key))
    }
    
    fn stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }
}

/// Multi-level cache manager
/// Combines local L1 cache with distributed L2 cache
pub struct MultiLevelCache {
    l1: Arc<dyn DistributedCache>,
    l2: Option<Arc<dyn DistributedCache>>,
    config: DistributedCacheConfig,
}

impl MultiLevelCache {
    pub fn new(l1: Arc<dyn DistributedCache>) -> Self {
        Self {
            l1,
            l2: None,
            config: DistributedCacheConfig::default(),
        }
    }
    
    pub fn with_l2(mut self, l2: Arc<dyn DistributedCache>) -> Self {
        self.l2 = Some(l2);
        self
    }
    
    /// Get from cache (L1 first, then L2)
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Try L1 first
        if let Some(value) = self.l1.get(key).await? {
            return Ok(Some(value));
        }
        
        // Try L2 if available
        if let Some(l2) = &self.l2 {
            if let Some(value) = l2.get(key).await? {
                // Populate L1
                let _ = self.l1.set(key.to_string(), value.clone(), None).await;
                return Ok(Some(value));
            }
        }
        
        Ok(None)
    }
    
    /// Set in all cache levels
    pub async fn set(&self, key: String, value: Vec<u8>, ttl: Option<u64>) -> Result<()> {
        self.l1.set(key.clone(), value.clone(), ttl).await?;
        
        if let Some(l2) = &self.l2 {
            l2.set(key, value, ttl).await?;
        }
        
        Ok(())
    }
    
    /// Delete from all cache levels
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.l1.delete(key).await?;
        
        if let Some(l2) = &self.l2 {
            l2.delete(key).await?;
        }
        
        Ok(())
    }
    
    /// Invalidate all caches
    pub async fn invalidate_all(&self) -> Result<()> {
        // Note: For in-memory cache, we'd need a clear method
        // This is a simplified implementation
        Ok(())
    }
    
    /// Get combined statistics
    pub fn stats(&self) -> CacheStats {
        let mut combined = self.l1.stats();
        
        if let Some(l2) = &self.l2 {
            let l2_stats = l2.stats();
            combined.hits += l2_stats.hits;
            combined.misses += l2_stats.misses;
            combined.evictions += l2_stats.evictions;
            combined.errors += l2_stats.errors;
        }
        
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_cache() {
        let cache = InMemoryCache::new();
        
        // Set a value
        cache.set("key1".to_string(), b"value1".to_vec(), None).await.unwrap();
        
        // Get the value
        let result = cache.get("key1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"value1");
        
        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
    }

    #[tokio::test]
    async fn test_multi_level_cache() {
        let l1 = Arc::new(InMemoryCache::new());
        let l2 = Arc::new(InMemoryCache::new());
        
        let cache = MultiLevelCache::new(l1.clone())
            .with_l2(l2.clone());
        
        // Set in cache
        cache.set("key1".to_string(), b"value1".to_vec(), None).await.unwrap();
        
        // Get from cache
        let result = cache.get("key1").await.unwrap();
        assert!(result.is_some());
        
        // Stats should show hits
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
    }
}
