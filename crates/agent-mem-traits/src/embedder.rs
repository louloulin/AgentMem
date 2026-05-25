//! Embedder trait definitions

use crate::Result;
use async_trait::async_trait;

/// Cache statistics for embedding operations
#[derive(Debug, Clone, Default)]
pub struct EmbedderCacheStats {
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Current number of cached embeddings
    pub cache_size: usize,
    /// Total number of embeddings processed
    pub total_embeddings: u64,
}

impl EmbedderCacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Core trait for embedding providers
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Generate embeddings for text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get the dimension of embeddings produced by this embedder
    fn dimension(&self) -> usize;

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Get the model name being used
    fn model_name(&self) -> &str;

    /// Check if the embedder is available/healthy
    async fn health_check(&self) -> Result<bool>;

    /// Get cache statistics for this embedder
    /// Returns cache hits, misses, and current cache size
    fn get_cache_stats(&self) -> EmbedderCacheStats {
        EmbedderCacheStats::default()
    }

    /// Clear the embedding cache
    async fn clear_cache(&self) -> Result<()> {
        Ok(())
    }

    /// Get cache hit rate as a percentage
    fn cache_hit_rate(&self) -> f64 {
        self.get_cache_stats().hit_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_default() {
        let stats = EmbedderCacheStats::default();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.total_embeddings, 0);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = EmbedderCacheStats::default();
        stats.cache_hits = 80;
        stats.cache_misses = 20;
        assert_eq!(stats.hit_rate(), 0.8);
    }

    #[test]
    fn test_cache_stats_hit_rate_zero() {
        let stats = EmbedderCacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }
}
