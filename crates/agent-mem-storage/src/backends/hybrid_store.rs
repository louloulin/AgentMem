//! Hybrid Vector Store - Combines vector search with full-text search
//!
//! Uses Reciprocal Rank Fusion (RRF) to combine results from multiple search methods.

use agent_mem_traits::{
    AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Search method type
#[derive(Debug, Clone, Copy)]
pub enum SearchMethod {
    /// Vector similarity search
    Vector,
    /// Full-text search
    Fts,
    /// Both methods
    Both,
}

/// Hybrid vector search result with ranking
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub id: String,
    pub score: f32,
    pub vector_score: Option<f32>,
    pub fts_score: Option<f32>,
}

/// Reciprocal Rank Fusion combiner
pub struct RrfCombiner {
    /// RRF k parameter (default 60)
    k: u32,
}

impl RrfCombiner {
    pub fn new(k: u32) -> Self {
        Self { k }
    }

    /// Apply RRF to combine ranked lists
    pub fn combine(&self, results: Vec<Vec<(String, f32)>>) -> Vec<(String, f32)> {
        use std::collections::HashMap;
        
        let mut scores: HashMap<String, f32> = HashMap::new();
        
        for result_list in results {
            for (rank, (id, score)) in result_list.into_iter().enumerate() {
                let rrf_score = 1.0 / (self.k as usize + rank + 1) as f32;
                *scores.entry(id.clone()).or_insert(0.0) += rrf_score;
            }
        }
        
        // Sort by combined RRF score
        let mut sorted: Vec<_> = scores.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        sorted
    }
}

impl Default for RrfCombiner {
    fn default() -> Self {
        Self { k: 60 }
    }
}

/// Hybrid vector store combining vector and FTS search
pub struct HybridVectorStore {
    vector_store: Arc<dyn VectorStore>,
    fts_store: Option<Arc<dyn FtsSearch>>,
    rrf: RrfCombiner,
    method: SearchMethod,
}

/// Trait for full-text search
#[async_trait]
pub trait FtsSearch: Send + Sync {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>>;
}

/// FTS search result
#[derive(Debug, Clone)]
pub struct FtsSearchResult {
    pub id: String,
    pub score: f32,
}

use std::sync::Arc;

impl HybridVectorStore {
    pub fn new(vector_store: Arc<dyn VectorStore>) -> Self {
        Self {
            vector_store,
            fts_store: None,
            rrf: RrfCombiner::default(),
            method: SearchMethod::Vector,
        }
    }

    pub fn with_fts(vector_store: Arc<dyn VectorStore>, fts_store: Arc<dyn FtsSearch>) -> Self {
        Self {
            vector_store,
            fts_store: Some(fts_store),
            rrf: RrfCombiner::default(),
            method: SearchMethod::Both,
        }
    }

    pub fn with_method(mut self, method: SearchMethod) -> Self {
        self.method = method;
        self
    }

    /// Hybrid search combining vector and FTS
    pub async fn hybrid_search(
        &self,
        query: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>> {
        let mut vector_list: Vec<(String, f32)> = Vec::new();
        let mut fts_list: Vec<(String, f32)> = Vec::new();

        match self.method {
            SearchMethod::Vector | SearchMethod::Both => {
                let vector_results = self.vector_store
                    .search_vectors(query_vector.clone(), limit, None)
                    .await?;
                
                vector_list = vector_results
                    .into_iter()
                    .map(|r| (r.id.clone(), r.similarity))
                    .collect();
            }
            SearchMethod::Fts => {}
        }

        if let Some(fts_store) = &self.fts_store {
            if matches!(self.method, SearchMethod::Fts | SearchMethod::Both) {
                let fts_results = fts_store.search(query, limit).await?;
                fts_list = fts_results
                    .into_iter()
                    .map(|r| (r.id.clone(), r.score))
                    .collect();
            }
        }

        // Combine results based on available sources
        if vector_list.is_empty() && fts_list.is_empty() {
            return Ok(Vec::new());
        }
        
        if fts_list.is_empty() {
            // Vector only
            return Ok(vector_list
                .into_iter()
                .map(|(id, score)| HybridSearchResult {
                    id,
                    score,
                    vector_score: Some(score),
                    fts_score: None,
                })
                .collect());
        }
        
        if vector_list.is_empty() {
            // FTS only
            return Ok(fts_list
                .into_iter()
                .map(|(id, score)| HybridSearchResult {
                    id,
                    score,
                    vector_score: None,
                    fts_score: Some(score),
                })
                .collect());
        }

        // Both sources - apply RRF
        let fused = self.rrf.combine(vec![vector_list, fts_list]);
        Ok(fused
            .into_iter()
            .map(|(id, score)| HybridSearchResult {
                id,
                score,
                vector_score: None,
                fts_score: None,
            })
            .take(limit)
            .collect())
    }
}

#[async_trait]
impl VectorStore for HybridVectorStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        self.vector_store.add_vectors(vectors).await
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        self.vector_store.search_vectors(query_vector, limit, threshold).await
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        self.vector_store.search_with_filters(query_vector, limit, filters, threshold).await
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        self.vector_store.delete_vectors(ids).await
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        self.vector_store.update_vectors(vectors).await
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        self.vector_store.get_vector(id).await
    }

    async fn count_vectors(&self) -> Result<usize> {
        self.vector_store.count_vectors().await
    }

    async fn clear(&self) -> Result<()> {
        self.vector_store.clear().await
    }

    async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
        self.vector_store.health_check().await
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        self.vector_store.get_stats().await
    }

    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        self.vector_store.add_vectors_batch(batches).await
    }

    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        self.vector_store.delete_vectors_batch(id_batches).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_combiner() {
        let rrf = RrfCombiner::default();
        let results = vec![
            vec![("a".to_string(), 1.0), ("b".to_string(), 0.9)],
            vec![("b".to_string(), 0.95), ("c".to_string(), 0.85)],
        ];
        
        let fused = rrf.combine(results);
        
        // "b" should be ranked first (appears in both lists)
        assert_eq!(fused.len(), 3);
    }

    #[tokio::test]
    async fn test_hybrid_search_creation() {
        use agent_mem_storage::backends::memory::MemoryVectorStore;
        
        let store = MemoryVectorStore::new();
        let hybrid = HybridVectorStore::new(Arc::new(store));
        
        assert!(hybrid.fts_store.is_none());
        assert!(matches!(hybrid.method, SearchMethod::Vector));
    }
}
