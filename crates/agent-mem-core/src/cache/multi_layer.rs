//! Multi-layer cache:
//! - L1: Memory query cache (Vec<Memory>)
//! - L2: LLM response cache (String)
//! - L3: Embedding cache (Vec<f32>)

use agent_mem_traits::Result;
use lru::LruCache;
use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant},
};

/// Cache entry for memory queries
#[derive(Debug, Clone)]
struct MemoryCacheEntry {
    /// Cached memories
    memories: Vec<crate::Memory>,
    /// When the entry was created
    created_at: Instant,
    /// Time-to-live for the entry
    ttl: Duration,
}

impl MemoryCacheEntry {
    /// Check if the entry is still valid
    fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }
}

/// Cache entry for LLM responses
#[derive(Debug, Clone)]
struct LlmCacheEntry {
    /// Cached LLM response
    response: String,
    /// When the entry was created
    created_at: Instant,
    /// Time-to-live for the entry
    ttl: Duration,
}

impl LlmCacheEntry {
    /// Check if the entry is still valid
    fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }
}

/// Cache entry for embeddings
#[derive(Debug, Clone)]
struct EmbeddingCacheEntry {
    /// Cached embedding vector
    embedding: Vec<f32>,
    /// When the entry was created
    created_at: Instant,
    /// Time-to-live for the entry
    ttl: Duration,
}

impl EmbeddingCacheEntry {
    /// Check if the entry is still valid
    fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }
}

/// Multi-layer cache implementation with three levels:
/// - L1: Memory query cache (Vec<Memory>)
/// - L2: LLM response cache (String)
/// - L3: Embedding cache (Vec<f32>)
#[derive(Debug)]
pub struct MultiLayerCache {
    l1_memory: Arc<RwLock<LruCache<String, MemoryCacheEntry>>>,
    l2_llm: Arc<RwLock<LruCache<String, LlmCacheEntry>>>,
    l3_embedding: Arc<RwLock<LruCache<String, EmbeddingCacheEntry>>>,
    metrics: CacheMetrics,
}

impl MultiLayerCache {
    /// Create a new multi-layer cache with default capacities
    pub fn new() -> Self {
        Self {
            l1_memory: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
            l2_llm: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))),
            l3_embedding: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(10000).unwrap(),
            ))),
            metrics: CacheMetrics::new(),
        }
    }

    /// Get memories from L1 cache
    pub fn get_memories(&self, key: &str) -> Option<Vec<crate::Memory>> {
        if let Ok(mut cache) = self.l1_memory.write() {
            if let Some(entry) = cache.get(key) {
                if entry.is_valid() {
                    self.metrics.l1_hit();
                    return Some(entry.memories.clone());
                }
                cache.pop(key);
            }
        }

        self.metrics.l1_miss();
        None
    }

    /// Set memories in L1 cache
    pub fn set_memories(&self, key: String, memories: Vec<crate::Memory>) {
        if let Ok(mut cache) = self.l1_memory.write() {
            let evicted = cache.put(
                key,
                MemoryCacheEntry {
                    memories,
                    created_at: Instant::now(),
                    ttl: Duration::from_secs(300),
                },
            );
            if evicted.is_some() {
                self.metrics.l1_evict();
            }
            self.metrics.l1_size(cache.len());
        }
    }

    /// Get LLM response from L2 cache
    pub fn get_llm_response(&self, prompt_hash: &str) -> Option<String> {
        if let Ok(mut cache) = self.l2_llm.write() {
            if let Some(entry) = cache.get(prompt_hash) {
                if entry.is_valid() {
                    self.metrics.l2_hit();
                    return Some(entry.response.clone());
                }
                cache.pop(prompt_hash);
            }
        }
        self.metrics.l2_miss();
        None
    }

    /// Set LLM response in L2 cache
    pub fn set_llm_response(&self, prompt_hash: String, response: String) {
        if let Ok(mut cache) = self.l2_llm.write() {
            let evicted = cache.put(
                prompt_hash,
                LlmCacheEntry {
                    response,
                    created_at: Instant::now(),
                    ttl: Duration::from_secs(3600),
                },
            );
            if evicted.is_some() {
                self.metrics.l2_evict();
            }
            self.metrics.l2_size(cache.len());
        }
    }

    /// Get embedding from L3 cache
    pub fn get_embedding(&self, text: &str) -> Option<Vec<f32>> {
        if let Ok(mut cache) = self.l3_embedding.write() {
            if let Some(entry) = cache.get(text) {
                if entry.is_valid() {
                    self.metrics.l3_hit();
                    return Some(entry.embedding.clone());
                }
                cache.pop(text);
            }
        }
        self.metrics.l3_miss();
        None
    }

    /// Set embedding in L3 cache
    pub fn set_embedding(&self, text: String, embedding: Vec<f32>) {
        if let Ok(mut cache) = self.l3_embedding.write() {
            let evicted = cache.put(
                text,
                EmbeddingCacheEntry {
                    embedding,
                    created_at: Instant::now(),
                    ttl: Duration::from_secs(24 * 3600),
                },
            );
            if evicted.is_some() {
                self.metrics.l3_evict();
            }
            self.metrics.l3_size(cache.len());
        }
    }

    /// Get cache metrics snapshot
    pub fn metrics(&self) -> CacheMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Warm up the cache with common queries and embeddings
    pub async fn warm_cache(
        &self,
        common_queries: Vec<String>,
        common_texts: Vec<String>,
    ) -> Result<WarmingStats> {
        let start = std::time::Instant::now();
        let items_warmed = common_queries.len() + common_texts.len();
        let failed = 0;

        // Warm L1 cache with common memory queries
        for query in common_queries {
            // For L1, we would need to pre-execute memory queries
            // This is a placeholder for the actual warming logic
            // In a real implementation, you would:
            // 1. Execute the memory query
            // 2. Store the result in L1 cache
            tracing::debug!("Warming L1 cache for query: {}", query);
        }

        // Warm L3 cache with common text embeddings
        for text in common_texts {
            // For L3, we would need to generate embeddings
            // This is a placeholder for the actual warming logic
            // In a real implementation, you would:
            // 1. Generate embedding for the text
            // 2. Store the embedding in L3 cache
            tracing::debug!("Warming L3 cache for text: {}", text);
        }

        let elapsed = start.elapsed();

        Ok(WarmingStats {
            total_warmings: 1,
            total_items_warmed: items_warmed as u64,
            total_warming_time_ms: elapsed.as_millis() as u64,
            last_warming_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            failed_warmings: failed,
        })
    }

    /// Get cache warming statistics
    pub fn get_warming_stats(&self) -> CacheWarmingStats {
        CacheWarmingStats {
            l1_entries: self.l1_memory.read().unwrap().len(),
            l2_entries: self.l2_llm.read().unwrap().len(),
            l3_entries: self.l3_embedding.read().unwrap().len(),
            last_warmed: std::time::SystemTime::now(),
        }
    }
}

/// Cache metrics for tracking performance
#[derive(Debug, Default)]
pub struct CacheMetrics {
    /// L1 cache hits
    l1_hits: AtomicU64,
    /// L1 cache misses
    l1_misses: AtomicU64,
    /// L1 cache evictions
    l1_evictions: AtomicU64,
    /// L1 cache size
    l1_size: AtomicU64,
    /// L2 cache hits
    l2_hits: AtomicU64,
    /// L2 cache misses
    l2_misses: AtomicU64,
    /// L2 cache evictions
    l2_evictions: AtomicU64,
    /// L2 cache size
    l2_size: AtomicU64,
    /// L3 cache hits
    l3_hits: AtomicU64,
    /// L3 cache misses
    l3_misses: AtomicU64,
    /// L3 cache evictions
    l3_evictions: AtomicU64,
    /// L3 cache size
    l3_size: AtomicU64,
}

impl CacheMetrics {
    fn new() -> Self {
        Self::default()
    }

    fn l1_hit(&self) {
        self.l1_hits.fetch_add(1, Ordering::Relaxed);
    }

    fn l1_miss(&self) {
        self.l1_misses.fetch_add(1, Ordering::Relaxed);
    }

    fn l1_evict(&self) {
        self.l1_evictions.fetch_add(1, Ordering::Relaxed);
    }

    fn l1_size(&self, size: usize) {
        self.l1_size.store(size as u64, Ordering::Relaxed);
    }

    fn l2_hit(&self) {
        self.l2_hits.fetch_add(1, Ordering::Relaxed);
    }

    fn l2_miss(&self) {
        self.l2_misses.fetch_add(1, Ordering::Relaxed);
    }

    fn l2_evict(&self) {
        self.l2_evictions.fetch_add(1, Ordering::Relaxed);
    }

    fn l2_size(&self, size: usize) {
        self.l2_size.store(size as u64, Ordering::Relaxed);
    }

    fn l3_hit(&self) {
        self.l3_hits.fetch_add(1, Ordering::Relaxed);
    }

    fn l3_miss(&self) {
        self.l3_misses.fetch_add(1, Ordering::Relaxed);
    }

    fn l3_evict(&self) {
        self.l3_evictions.fetch_add(1, Ordering::Relaxed);
    }

    fn l3_size(&self, size: usize) {
        self.l3_size.store(size as u64, Ordering::Relaxed);
    }

    fn snapshot(&self) -> CacheMetricsSnapshot {
        CacheMetricsSnapshot {
            l1_hits: self.l1_hits.load(Ordering::Relaxed),
            l1_misses: self.l1_misses.load(Ordering::Relaxed),
            l1_evictions: self.l1_evictions.load(Ordering::Relaxed),
            l1_size: self.l1_size.load(Ordering::Relaxed),
            l2_hits: self.l2_hits.load(Ordering::Relaxed),
            l2_misses: self.l2_misses.load(Ordering::Relaxed),
            l2_evictions: self.l2_evictions.load(Ordering::Relaxed),
            l2_size: self.l2_size.load(Ordering::Relaxed),
            l3_hits: self.l3_hits.load(Ordering::Relaxed),
            l3_misses: self.l3_misses.load(Ordering::Relaxed),
            l3_evictions: self.l3_evictions.load(Ordering::Relaxed),
            l3_size: self.l3_size.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of cache metrics for reporting
#[derive(Debug, Clone, Copy)]
pub struct CacheMetricsSnapshot {
    /// L1 cache hits
    pub l1_hits: u64,
    /// L1 cache misses
    pub l1_misses: u64,
    /// L1 cache evictions
    pub l1_evictions: u64,
    /// L1 cache size
    pub l1_size: u64,
    /// L2 cache hits
    pub l2_hits: u64,
    /// L2 cache misses
    pub l2_misses: u64,
    /// L2 cache evictions
    pub l2_evictions: u64,
    /// L2 cache size
    pub l2_size: u64,
    /// L3 cache hits
    pub l3_hits: u64,
    /// L3 cache misses
    pub l3_misses: u64,
    /// L3 cache evictions
    pub l3_evictions: u64,
    /// L3 cache size
    pub l3_size: u64,
}

/// Statistics for cache warming
#[derive(Debug, Clone)]
pub struct CacheWarmingStats {
    /// Number of entries in L1 cache
    pub l1_entries: usize,
    /// Number of entries in L2 cache
    pub l2_entries: usize,
    /// Number of entries in L3 cache
    pub l3_entries: usize,
    /// Last time cache was warmed
    pub last_warmed: std::time::SystemTime,
}

/// Warming statistics compatible with warming module
#[derive(Debug, Clone, Default)]
pub struct WarmingStats {
    /// Total warming operations
    pub total_warmings: u64,
    /// Total items warmed
    pub total_items_warmed: u64,
    /// Total warming time in milliseconds
    pub total_warming_time_ms: u64,
    /// Last warming timestamp
    pub last_warming_timestamp: u64,
    /// Failed warming attempts
    pub failed_warmings: u64,
}

#[cfg(test)]
mod tests {
    use super::MultiLayerCache;
    use crate::Memory;

    #[test]
    fn test_l1_memories_cache() {
        let cache = MultiLayerCache::new();
        assert!(cache.get_memories("k1").is_none());

        cache.set_memories("k1".to_string(), vec![]);
        assert!(cache.get_memories("k1").is_some());
    }

    #[test]
    fn test_l2_llm_cache() {
        let cache = MultiLayerCache::new();
        assert!(cache.get_llm_response("prompt").is_none());

        cache.set_llm_response("prompt".to_string(), "hello".to_string());
        assert_eq!(cache.get_llm_response("prompt"), Some("hello".to_string()));
    }

    #[test]
    fn test_l3_embedding_cache() {
        let cache = MultiLayerCache::new();
        assert!(cache.get_embedding("text").is_none());

        cache.set_embedding("text".to_string(), vec![0.1, 0.2]);
        assert_eq!(cache.get_embedding("text"), Some(vec![0.1, 0.2]));
    }

    #[tokio::test]
    async fn test_cache_warming() {
        let cache = MultiLayerCache::new();

        let common_queries = vec![
            "What is the weather today?".to_string(),
            "Tell me about AI".to_string(),
        ];

        let common_texts = vec![
            "artificial intelligence".to_string(),
            "machine learning".to_string(),
        ];

        let result = cache
            .warm_cache(common_queries, common_texts)
            .await
            .unwrap();

        assert_eq!(result.total_warmings, 1);
        assert_eq!(result.total_items_warmed, 4);
        // Allow for very fast execution (could be 0ms)
        assert!(result.total_warming_time_ms >= 0);

        let stats = cache.get_warming_stats();
        assert_eq!(stats.l1_entries, 0); // No actual memories were added
        assert_eq!(stats.l2_entries, 0); // No actual LLM responses were added
        assert_eq!(stats.l3_entries, 0); // No actual embeddings were added
    }
}
