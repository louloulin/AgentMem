//! Search configuration
//! 
//! Configuration structures for search engines.

use serde::{Deserialize, Serialize};

/// Search engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Maximum results to return
    pub max_results: usize,
    /// Minimum relevance score (0.0-1.0)
    pub min_score: f32,
    /// Enable vector search
    pub use_vector: bool,
    /// Enable BM25 text search
    pub use_bm25: bool,
    /// Enable hybrid search
    pub use_hybrid: bool,
    /// Hybrid search weight for vector (0.0-1.0)
    pub vector_weight: f32,
    /// Hybrid search weight for BM25 (0.0-1.0)
    pub bm25_weight: f32,
    /// Enable result caching
    pub enable_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            min_score: 0.5,
            use_vector: true,
            use_bm25: true,
            use_hybrid: false,
            vector_weight: 0.5,
            bm25_weight: 0.5,
            enable_cache: true,
            cache_ttl_seconds: 300,
        }
    }
}

/// Hybrid search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchConfig {
    /// Vector weight
    pub vector_weight: f32,
    /// BM25 weight
    pub bm25_weight: f32,
    /// RRF k parameter for Reciprocal Rank Fusion
    pub rrf_k: u32,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.5,
            bm25_weight: 0.5,
            rrf_k: 60,
        }
    }
}

/// BM25 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Config {
    /// BM25 k1 parameter
    pub k1: f32,
    /// BM25 b parameter
    pub b: f32,
    /// Average document length (optional)
    pub avg_doc_len: Option<f32>,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            k1: 1.5,
            b: 0.75,
            avg_doc_len: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();
        assert_eq!(config.max_results, 10);
        assert!(config.use_vector);
        assert!(config.use_bm25);
    }

    #[test]
    fn test_hybrid_config_default() {
        let config = HybridSearchConfig::default();
        assert_eq!(config.vector_weight, 0.5);
        assert_eq!(config.bm25_weight, 0.5);
        assert_eq!(config.rrf_k, 60);
    }

    #[test]
    fn test_bm25_config_default() {
        let config = BM25Config::default();
        assert_eq!(config.k1, 1.5);
        assert_eq!(config.b, 0.75);
    }
}
