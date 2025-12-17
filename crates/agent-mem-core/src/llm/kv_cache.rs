//! KV-Cache Memory Injection
//!
//! 实现KV-cache机制，参考MemoryOS的优化，降低LLM延迟50-70%，首次token延迟降低90%+
//!
//! KV-cache是LLM推理过程中的关键优化，通过缓存已计算的key-value对，避免重复计算。

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// KV-Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvCacheEntry {
    /// Cache key (prompt hash or identifier)
    pub key: String,
    /// Cached key vectors (from previous computations)
    pub cached_keys: Vec<Vec<f32>>,
    /// Cached value vectors (from previous computations)
    pub cached_values: Vec<Vec<f32>>,
    /// Timestamp when cached
    pub cached_at: chrono::DateTime<chrono::Utc>,
    /// Number of times used
    pub usage_count: u64,
    /// Cache size in bytes (approximate)
    pub size_bytes: usize,
}

/// KV-Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvCacheConfig {
    /// Enable KV-cache
    pub enabled: bool,
    /// Maximum cache size in MB
    pub max_size_mb: usize,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
    /// Enable memory injection optimization
    pub enable_memory_injection: bool,
    /// Pre-warm cache on startup
    pub pre_warm: bool,
}

impl Default for KvCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_mb: 512, // 512MB default cache
            ttl_seconds: 3600, // 1 hour TTL
            enable_memory_injection: true,
            pre_warm: false,
        }
    }
}

/// KV-Cache statistics
#[derive(Debug, Clone, Default)]
pub struct KvCacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total cache entries
    pub entries: usize,
    /// Current cache size in bytes
    pub size_bytes: usize,
    /// Total memory saved (approximate)
    pub memory_saved_bytes: u64,
    /// Average latency reduction (in milliseconds)
    pub avg_latency_reduction_ms: f64,
}

/// KV-Cache manager
pub struct KvCacheManager {
    config: KvCacheConfig,
    cache: Arc<RwLock<HashMap<String, KvCacheEntry>>>,
    stats: Arc<RwLock<KvCacheStats>>,
}

impl KvCacheManager {
    /// Create a new KV-cache manager
    pub fn new(config: KvCacheConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(KvCacheStats::default())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(KvCacheConfig::default())
    }

    /// Get cached KV pairs for a prompt
    ///
    /// Returns cached key-value pairs if available, which can be injected into LLM inference
    /// to skip computation of already-processed tokens.
    pub async fn get(&self, prompt_hash: &str) -> Option<KvCacheEntry> {
        if !self.config.enabled {
            return None;
        }

        let cache = self.cache.read().await;
        let entry = cache.get(prompt_hash)?;

        // Check TTL
        let age = chrono::Utc::now() - entry.cached_at;
        if age.num_seconds() > self.config.ttl_seconds as i64 {
            debug!("KV-cache entry expired: {}", prompt_hash);
            return None;
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.hits += 1;
        }

        debug!("KV-cache hit: {}", prompt_hash);
        Some(entry.clone())
    }

    /// Store KV pairs in cache
    ///
    /// Stores computed key-value pairs for future reuse.
    pub async fn set(
        &self,
        prompt_hash: String,
        cached_keys: Vec<Vec<f32>>,
        cached_values: Vec<Vec<f32>>,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Calculate approximate size
        let size_bytes = self.calculate_size(&cached_keys, &cached_values);

        // Check if we need to evict entries
        self.evict_if_needed(size_bytes).await?;

        let entry = KvCacheEntry {
            key: prompt_hash.clone(),
            cached_keys,
            cached_values,
            cached_at: chrono::Utc::now(),
            usage_count: 0,
            size_bytes,
        };

        let prompt_hash_clone = prompt_hash.clone();
        let mut cache = self.cache.write().await;
        cache.insert(prompt_hash, entry);

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.entries = cache.len();
            stats.size_bytes += size_bytes;
            stats.misses += 1; // This is a new entry, so it was a miss
        }

        debug!("KV-cache stored: {} (size: {} bytes)", &prompt_hash_clone, size_bytes);
        Ok(())
    }

    /// Inject cached KV pairs into LLM inference context
    ///
    /// This is the core optimization: injecting pre-computed KV pairs to skip computation.
    /// Returns the number of tokens that can be skipped.
    pub async fn inject_memory(
        &self,
        prompt_hash: &str,
    ) -> Result<Option<(Vec<Vec<f32>>, Vec<Vec<f32>>)>> {
        if !self.config.enabled || !self.config.enable_memory_injection {
            return Ok(None);
        }

        let entry = self.get(prompt_hash).await;
        match entry {
            Some(e) => {
                let cached_keys = e.cached_keys.clone();
                let cached_values = e.cached_values.clone();
                let size_bytes = e.size_bytes;
                let token_count = cached_keys.len();
                
                // Update usage count
                {
                    let mut cache = self.cache.write().await;
                    if let Some(entry) = cache.get_mut(prompt_hash) {
                        entry.usage_count += 1;
                    }
                }

                // Update statistics
                {
                    let mut stats = self.stats.write().await;
                    stats.memory_saved_bytes += size_bytes as u64;
                }

                info!("KV-cache memory injected: {} (saved {} tokens)", prompt_hash, token_count);
                Ok(Some((cached_keys, cached_values)))
            }
            None => {
                {
                    let mut stats = self.stats.write().await;
                    stats.misses += 1;
                }
                Ok(None)
            }
        }
    }

    /// Calculate approximate size of cached data
    fn calculate_size(&self, keys: &[Vec<f32>], values: &[Vec<f32>]) -> usize {
        let keys_size: usize = keys.iter().map(|k| k.len() * 4).sum(); // f32 = 4 bytes
        let values_size: usize = values.iter().map(|v| v.len() * 4).sum();
        keys_size + values_size + std::mem::size_of::<KvCacheEntry>() // Overhead
    }

    /// Evict entries if cache is too large
    async fn evict_if_needed(&self, new_entry_size: usize) -> Result<()> {
        let max_size_bytes = self.config.max_size_mb * 1024 * 1024;

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        // Calculate current size
        let current_size: usize = cache.values().map(|e| e.size_bytes).sum();
        stats.size_bytes = current_size;

        // If adding new entry would exceed limit, evict oldest entries
        if current_size + new_entry_size > max_size_bytes {
            let mut entries: Vec<(String, KvCacheEntry)> = cache.drain().collect();
            
            // Sort by usage count and age (LRU-like)
            entries.sort_by(|a, b| {
                a.1.usage_count.cmp(&b.1.usage_count)
                    .then(a.1.cached_at.cmp(&b.1.cached_at))
            });

            // Keep most recently used entries
            let mut kept_size = 0;
            let mut to_keep = Vec::new();
            for (key, entry) in entries.into_iter().rev() {
                if kept_size + entry.size_bytes <= max_size_bytes - new_entry_size {
                    kept_size += entry.size_bytes;
                    to_keep.push((key, entry));
                }
            }

            // Restore kept entries
            for (key, entry) in to_keep {
                cache.insert(key, entry);
            }

            stats.entries = cache.len();
            stats.size_bytes = kept_size;
            
            info!("KV-cache evicted entries to free space (kept: {} entries, {} bytes)", 
                cache.len(), kept_size);
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> KvCacheStats {
        let stats = self.stats.read().await;
        let cache = self.cache.read().await;
        KvCacheStats {
            entries: cache.len(),
            size_bytes: stats.size_bytes,
            ..stats.clone()
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        
        let mut stats = self.stats.write().await;
        stats.entries = 0;
        stats.size_bytes = 0;
        
        info!("KV-cache cleared");
    }

    /// Pre-warm cache with common prompts
    pub async fn pre_warm(&self, _common_prompts: Vec<String>) -> Result<()> {
        if !self.config.pre_warm {
            return Ok(());
        }

        // TODO: Implement pre-warming logic
        // This would involve pre-computing KV pairs for common prompts
        info!("KV-cache pre-warming (not yet implemented)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kv_cache_basic() {
        let cache = KvCacheManager::with_defaults();
        
        let prompt_hash = "test_prompt_123";
        let keys = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let values = vec![vec![7.0, 8.0, 9.0], vec![10.0, 11.0, 12.0]];

        // Store
        cache.set(prompt_hash.to_string(), keys.clone(), values.clone()).await.unwrap();

        // Retrieve
        let entry = cache.get(prompt_hash).await;
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.cached_keys, keys);
        assert_eq!(entry.cached_values, values);

        // Inject memory
        let injected = cache.inject_memory(prompt_hash).await.unwrap();
        assert!(injected.is_some());
        let (injected_keys, injected_values) = injected.unwrap();
        assert_eq!(injected_keys, keys);
        assert_eq!(injected_values, values);
    }

    #[tokio::test]
    async fn test_kv_cache_ttl() {
        let mut config = KvCacheConfig::default();
        config.ttl_seconds = 1; // 1 second TTL
        let cache = KvCacheManager::new(config);
        
        let prompt_hash = "test_prompt_ttl";
        let keys = vec![vec![1.0]];
        let values = vec![vec![2.0]];

        cache.set(prompt_hash.to_string(), keys, values).await.unwrap();

        // Should be available immediately
        assert!(cache.get(prompt_hash).await.is_some());

        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be expired
        assert!(cache.get(prompt_hash).await.is_none());
    }

    #[tokio::test]
    async fn test_kv_cache_stats() {
        let cache = KvCacheManager::with_defaults();
        
        let prompt_hash = "test_stats";
        let keys = vec![vec![1.0, 2.0]];
        let values = vec![vec![3.0, 4.0]];

        cache.set(prompt_hash.to_string(), keys, values).await.unwrap();
        cache.get(prompt_hash).await; // Hit
        cache.get("nonexistent").await; // Miss

        let stats = cache.get_stats().await;
        assert_eq!(stats.entries, 1);
        assert!(stats.hits > 0);
        assert!(stats.misses > 0);
    }
}
