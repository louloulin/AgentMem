//! Cached Vector Store Wrapper
//!
//! Wraps a vector store with caching layer for improved performance.

use agent_mem_traits::{
    AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// Cache entry with expiration
struct CachedEntry<T> {
    data: T,
    expires_at: Option<Instant>,
}

impl<T> CachedEntry<T> {
    fn new(data: T, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|d| Instant::now() + d);
        Self { data, expires_at }
    }
    
    fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Instant::now() > e).unwrap_or(false)
    }
}

/// Simple vector cache for search results
pub struct SearchResultCache {
    cache: RwLock<HashMap<String, (Vec<VectorSearchResult>, Instant)>>,
    ttl: Duration,
    max_entries: usize,
}

impl Default for SearchResultCache {
    fn default() -> Self {
        Self::new(Some(Duration::from_secs(300)), 1000)
    }
}

impl SearchResultCache {
    pub fn new(ttl: Option<Duration>, max_entries: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl: ttl.unwrap_or(Duration::from_secs(300)),
            max_entries,
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<Vec<VectorSearchResult>> {
        let cache = self.cache.read().await;
        if let Some((results, expires_at)) = cache.get(key) {
            if Instant::now() < *expires_at {
                return Some(results.clone());
            }
        }
        None
    }
    
    pub async fn put(&self, key: String, results: Vec<VectorSearchResult>) {
        let mut cache = self.cache.write().await;
        
        // Evict if full
        if cache.len() >= self.max_entries {
            let keys: Vec<_> = cache.keys().take(100).cloned().collect();
            for k in keys {
                cache.remove(&k);
            }
        }
        
        cache.insert(key, (results, Instant::now() + self.ttl));
    }
    
    pub async fn invalidate(&self, key: &str) {
        self.cache.write().await.remove(key);
    }
    
    pub async fn clear(&self) {
        self.cache.write().await.clear();
    }
}

/// Cached vector store wrapper
pub struct CachedVectorStore {
    inner: Arc<dyn VectorStore>,
    search_cache: Arc<SearchResultCache>,
    enable_cache: bool,
}

impl CachedVectorStore {
    pub fn new(inner: Arc<dyn VectorStore>) -> Self {
        Self {
            inner,
            search_cache: Arc::new(SearchResultCache::default()),
            enable_cache: true,
        }
    }
    
    pub fn with_cache_config(mut self, cache: Arc<SearchResultCache>) -> Self {
        self.search_cache = cache;
        self
    }
    
    pub fn with_cache_enabled(mut self, enabled: bool) -> Self {
        self.enable_cache = enabled;
        self
    }
    
    /// Generate cache key for search query
    fn search_cache_key(query: &[f32], limit: usize, threshold: Option<f32>) -> String {
        let query_slice: Vec<_> = query.iter().take(10).copied().collect();
        format!(
            "{:?}_{}_{:?}",
            query_slice, limit, threshold
        )
    }
    
    /// Invalidate cache when vectors change
    pub async fn invalidate_cache(&self) {
        self.search_cache.clear().await;
    }
}

#[async_trait]
impl VectorStore for CachedVectorStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let ids = self.inner.add_vectors(vectors).await?;
        // Invalidate search cache on mutation
        self.invalidate_cache().await;
        Ok(ids)
    }
    
    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        if !self.enable_cache {
            return self.inner.search_vectors(query_vector, limit, threshold).await;
        }
        
        let cache_key = Self::search_cache_key(&query_vector, limit, threshold);
        
        // Try cache first
        if let Some(cached) = self.search_cache.get(&cache_key).await {
            return Ok(cached);
        }
        
        // Cache miss - search underlying store
        let results = self.inner.search_vectors(query_vector, limit, threshold).await?;
        
        // Store in cache
        self.search_cache.put(cache_key, results.clone()).await;
        
        Ok(results)
    }
    
    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        // Don't cache filtered searches
        self.inner.search_with_filters(query_vector, limit, filters, threshold).await
    }
    
    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        let result = self.inner.delete_vectors(ids).await;
        self.invalidate_cache().await;
        result
    }
    
    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        let result = self.inner.update_vectors(vectors).await;
        self.invalidate_cache().await;
        result
    }
    
    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        self.inner.get_vector(id).await
    }
    
    async fn count_vectors(&self) -> Result<usize> {
        self.inner.count_vectors().await
    }
    
    async fn clear(&self) -> Result<()> {
        let result = self.inner.clear().await;
        self.invalidate_cache().await;
        result
    }
    
    async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
        self.inner.health_check().await
    }
    
    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        self.inner.get_stats().await
    }
    
    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        let results = self.inner.add_vectors_batch(batches).await;
        self.invalidate_cache().await;
        results
    }
    
    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        let results = self.inner.delete_vectors_batch(id_batches).await;
        self.invalidate_cache().await;
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_storage::backends::memory::MemoryVectorStore;
    
    #[tokio::test]
    async fn test_cached_store() {
        let inner = Arc::new(MemoryVectorStore::new());
        let cached = CachedVectorStore::new(inner);
        
        // Add vectors
        let vectors = vec![
            VectorData {
                id: "v1".to_string(),
                vector: vec![0.1, 0.2, 0.3],
                metadata: HashMap::new(),
            },
            VectorData {
                id: "v2".to_string(),
                vector: vec![0.4, 0.5, 0.6],
                metadata: HashMap::new(),
            },
        ];
        
        cached.add_vectors(vectors).await.unwrap();
        
        // Search should work
        let results = cached.search_vectors(vec![0.1, 0.2, 0.3], 10, None).await.unwrap();
        assert!(!results.is_empty());
        
        // Second search should use cache
        let cached_results = cached.search_vectors(vec![0.1, 0.2, 0.3], 10, None).await.unwrap();
        assert_eq!(results.len(), cached_results.len());
    }
}
