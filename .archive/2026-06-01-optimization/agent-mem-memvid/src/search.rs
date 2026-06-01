//! Search functionality for MemVid store

use crate::error::Result;
use crate::store::MemvidStore;
use agent_mem_traits::{Filters, Memory, MemoryId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Search result from MemVid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Memory ID
    pub id: MemoryId,

    /// Score/relevance
    pub score: f32,

    /// Snippet of matched content
    pub snippet: Option<String>,

    /// Highlighted positions
    pub highlights: Vec<Highlight>,

    /// Full memory (lazy loaded)
    pub memory: Option<Memory>,
}

/// Text highlight position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    /// Start position
    pub start: usize,

    /// End position
    pub end: usize,

    /// Highlight text
    pub text: String,
}

/// Search request builder
pub struct SearchBuilder {
    query: String,
    top_k: usize,
    threshold: Option<f32>,
    filters: Filters,
    hybrid_alpha: Option<f32>,
}

impl SearchBuilder {
    /// Create a new search builder
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            top_k: 10,
            threshold: None,
            filters: Filters::default(),
            hybrid_alpha: None,
        }
    }

    /// Set top-k results
    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }

    /// Set similarity threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Set filters
    pub fn with_filters(mut self, filters: Filters) -> Self {
        self.filters = filters;
        self
    }

    /// Set hybrid search alpha (0.0 = full text, 1.0 = vector)
    pub fn with_hybrid_alpha(mut self, alpha: f32) -> Self {
        self.hybrid_alpha = Some(alpha);
        self
    }

    /// Execute the search
    pub async fn execute(self, store: &MemvidStore) -> Result<Vec<SearchResult>> {
        if let Some(alpha) = self.hybrid_alpha {
            store.search_hybrid(&self.query, self.top_k, alpha).await
        } else {
            store.search(&self.query, self.top_k).await
        }
    }
}

/// Search trait for MemVid store
#[async_trait]
pub trait MemvidSearch: Send + Sync {
    /// Full-text search
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>;

    /// Vector similarity search
    async fn search_vector(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>;

    /// Hybrid search (text + vector)
    async fn search_hybrid(
        &self,
        query: &str,
        top_k: usize,
        alpha: f32,
    ) -> Result<Vec<SearchResult>>;
}

#[async_trait]
impl MemvidSearch for MemvidStore {
    /// Full-text search using Tantivy
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        tracing::debug!("Full-text search: query='{}', top_k={}", query, top_k);

        // TODO: Integrate with memvid-core search API
        // For now, use simple linear search
        let filters = Filters::default();
        let memories = self.list(&filters).await?;

        let mut results = Vec::new();

        for memory in memories {
            let score = Self::text_similarity(query, &memory);
            if score > 0.0 {
                results.push(SearchResult {
                    id: memory.id.clone(),
                    score,
                    snippet: Self::extract_snippet(&memory, query),
                    highlights: vec![],
                    memory: Some(memory),
                });
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Take top-k
        results.truncate(top_k);

        Ok(results)
    }

    /// Vector similarity search
    async fn search_vector(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        tracing::debug!("Vector search: query='{}', top_k={}", query, top_k);

        // TODO: Integrate with memvid-core vector search API
        // For now, return empty results
        Ok(Vec::new())
    }

    /// Hybrid search combining text and vector
    async fn search_hybrid(
        &self,
        query: &str,
        top_k: usize,
        alpha: f32,
    ) -> Result<Vec<SearchResult>> {
        tracing::debug!(
            "Hybrid search: query='{}', top_k={}, alpha={}",
            query,
            top_k,
            alpha
        );

        // Execute both searches in parallel
        let (text_results, vector_results) = tokio::try_join!(
            self.search(query, top_k * 2),
            self.search_vector(query, top_k * 2)
        )?;

        // Merge results with weighted scores
        let mut merged = std::collections::HashMap::new();

        for result in text_results {
            let entry = merged
                .entry(result.id.clone())
                .or_insert_with(|| result.clone());
            entry.score = (1.0 - alpha) * entry.score;
        }

        for result in vector_results {
            let entry = merged
                .entry(result.id.clone())
                .or_insert_with(|| result.clone());
            entry.score += alpha * result.score;
        }

        // Convert to vec and sort
        let mut results: Vec<_> = merged.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(top_k);

        Ok(results)
    }
}

impl MemvidStore {
    /// Calculate text similarity score
    fn text_similarity(query: &str, memory: &Memory) -> f32 {
        let query_lower = query.to_lowercase();
        let text = memory.content.to_string().to_lowercase();

        // Simple word overlap score
        let query_words: std::collections::HashSet<&str> = query_lower.split_whitespace().collect();
        let text_words: std::collections::HashSet<&str> = text.split_whitespace().collect();

        if query_words.is_empty() {
            return 0.0;
        }

        let intersection = query_words.intersection(&text_words).count();
        let union = query_words.union(&text_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Extract snippet from memory
    fn extract_snippet(memory: &Memory, query: &str) -> Option<String> {
        let text = memory.content.to_string();
        let query_lower = query.to_lowercase();

        if let Some(pos) = text.to_lowercase().find(&query_lower) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(text.len());
            let snippet = &text[start..end];

            let prefix = if start > 0 { "..." } else { "" };
            let suffix = if end < text.len() { "..." } else { "" };

            Some(format!("{}{}{}", prefix, snippet, suffix))
        } else {
            None
        }
    }

    /// Public search method
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        <Self as MemvidSearch>::search(self, query, top_k).await
    }

    /// Public vector search method
    pub async fn search_vector(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        <Self as MemvidSearch>::search_vector(self, query, top_k).await
    }

    /// Public hybrid search method
    pub async fn search_hybrid(
        &self,
        query: &str,
        top_k: usize,
        alpha: f32,
    ) -> Result<Vec<SearchResult>> {
        <Self as MemvidSearch>::search_hybrid(self, query, top_k, alpha).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemvidConfig, MemvidStore};
    use agent_mem_traits::{AttributeSet, Content, MetadataV4};

    #[tokio::test]
    async fn test_search() {
        let config = MemvidConfig::new("test_search.mv2");
        let store = MemvidStore::create(config).await.unwrap();

        // Add test memories
        let memory1 = Memory {
            id: MemoryId::from_string("test-1".to_string()),
            content: Content::text("Hello world test"),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        };

        let memory2 = Memory {
            id: MemoryId::from_string("test-2".to_string()),
            content: Content::text("Another memory with different content"),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        };

        store.add(&memory1).await.unwrap();
        store.add(&memory2).await.unwrap();

        // Search for "hello" - should match memory1 due to case-insensitive comparison
        let results = store.search("hello", 10).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id.as_str(), "test-1");

        // Cleanup
        let _ = tokio::fs::remove_file("test_search.mv2").await;
    }
}
