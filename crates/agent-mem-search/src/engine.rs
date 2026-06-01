//! Search engine core
//! 
//! Core search engine implementation for AgentMem.

use agent_mem_types::{MemorySearchResult, MemoryType, Query, QueryContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Search result with score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Memory ID
    pub id: String,
    /// Content preview
    pub content: String,
    /// Relevance score (0.0-1.0)
    pub score: f32,
    /// Memory type
    pub memory_type: MemoryType,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SearchResult {
    /// Create new search result
    pub fn new(id: String, content: String, score: f32, memory_type: MemoryType) -> Self {
        Self {
            id,
            content,
            score,
            memory_type,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Search engine trait
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Search by query
    async fn search(&self, query: &Query) -> Result<Vec<SearchResult>, SearchError>;
    
    /// Search with context
    async fn search_with_context(&self, query: &Query, ctx: &QueryContext) -> Result<Vec<SearchResult>, SearchError>;
    
    /// Get engine name
    fn name(&self) -> &str;
}

/// Search error types
#[derive(Debug, Clone)]
pub enum SearchError {
    /// Query parsing error
    QueryParseError(String),
    /// Index not found
    IndexNotFound(String),
    /// Storage error
    StorageError(String),
    /// Embedding error
    EmbeddingError(String),
    /// General error
    General(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::QueryParseError(msg) => write!(f, "Query parse error: {msg}"),
            SearchError::IndexNotFound(id) => write!(f, "Index not found: {id}"),
            SearchError::StorageError(msg) => write!(f, "Storage error: {msg}"),
            SearchError::EmbeddingError(msg) => write!(f, "Embedding error: {msg}"),
            SearchError::General(msg) => write!(f, "Search error: {msg}"),
        }
    }
}

impl std::error::Error for SearchError {}

/// Basic search engine implementation
pub struct BasicSearchEngine {
    name: String,
    config: super::SearchConfig,
}

impl BasicSearchEngine {
    /// Create new basic search engine
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: super::SearchConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(name: &str, config: super::SearchConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
        }
    }

    /// Get config
    pub fn config(&self) -> &super::SearchConfig {
        &self.config
    }
}

#[async_trait]
impl SearchEngine for BasicSearchEngine {
    async fn search(&self, query: &Query) -> Result<Vec<SearchResult>, SearchError> {
        // Placeholder implementation
        Ok(Vec::new())
    }

    async fn search_with_context(&self, query: &Query, _ctx: &QueryContext) -> Result<Vec<SearchResult>, SearchError> {
        self.search(query).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_types::Query;

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult::new(
            "test-id".to_string(),
            "Test content".to_string(),
            0.95,
            MemoryType::Episodic,
        );
        assert_eq!(result.id, "test-id");
        assert_eq!(result.score, 0.95);
    }

    #[test]
    fn test_search_result_with_metadata() {
        let result = SearchResult::new(
            "test-id".to_string(),
            "Test content".to_string(),
            0.95,
            MemoryType::Semantic,
        ).with_metadata("source", "test");
        
        assert_eq!(result.metadata.get("source"), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_basic_engine_search() {
        let engine = BasicSearchEngine::new("test-engine");
        let query = Query::new("test query");
        let results = engine.search(&query).await;
        assert!(results.is_ok());
    }
}
