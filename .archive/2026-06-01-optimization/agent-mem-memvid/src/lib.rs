//! AgentMem 2.0 + MemVid Integration
//!
//! This crate provides the MemVid storage backend for AgentMem 2.0,
//! replacing the complex multi-database architecture with a single-file
//! portable memory layer.
//!
//! # Features
//!
//! - **Single File Storage**: All data in one `.mv2` file
//! - **<5ms Search**: Full-text and vector search
//! - **Time Travel**: Query historical versions
//! - **Zero Config**: No database setup required
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_memvid::{MemvidStore, MemvidConfig};
//! use agent_mem_traits::{Memory, Content, AttributeSet, MetadataV4};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create or open a MemVid store
//! let config = MemvidConfig::new("memory.mv2");
//! let store = MemvidStore::create(config).await?;
//!
//! // Add a memory
//! let memory = Memory {
//!     id: Default::default(),
//!     content: Content::text("Hello, world!"),
//!     attributes: AttributeSet::new(),
//!     relations: Default::default(),
//!     metadata: MetadataV4::default(),
//! };
//! store.add(&memory).await?;
//!
//! // Search
//! let results = store.search("hello", 10).await?;
//! # Ok(())
//! # }
//! ```

pub mod advanced_search;
pub mod conversion;
pub mod embedding;
pub mod error;
pub mod search;
pub mod store;
pub mod store_trait;
pub mod timeline;
pub mod vector_search;

#[cfg(test)]
pub mod benchmarks;

#[cfg(test)]
mod integration_tests;

// Real MemVid API integration
pub mod memvid_store;

// Re-exports
pub use advanced_search::{AdvancedSearch, SearchOptions, SearchResult as AdvancedSearchResult};
pub use error::{MemvidError, Result};

// Vector search exports
pub use embedding::{cosine_similarity, euclidean_distance, SimilarityResult, SimilarityType};
pub use embedding::{EmbeddingVector, LocalEmbedding, OpenAIEmbedding};
pub use vector_search::{
    EmbeddingGenerator, HybridSearchResult, HybridSearcher, VectorIndex, VectorSearchConfig,
    VectorSearchResult,
};

// Real MemVid exports (main API)
pub use memvid_store::{
    Memvid,
    MemvidStore,     // Public facade (implements MemoryProvider trait)
    MemvidStoreImpl, // Internal implementation
    OpenReadOptions,
    PutOptions,
    SearchRequest,
    TimelineQuery,
    VersionInfo,
};


/// Type alias for MemvidStoreImpl (used in benchmarks and tests)
pub type RealMemvidStore = MemvidStoreImpl;

/// MemVid store configuration
#[derive(Debug, Clone)]
pub struct MemvidConfig {
    /// Path to the `.mv2` file
    pub path: String,

    /// Create file if it doesn't exist
    pub create_if_missing: bool,

    /// Enable full-text search
    pub enable_lex: bool,

    /// Enable vector search
    pub enable_vec: bool,

    /// Cache size (number of memories)
    pub cache_size: usize,

    /// Auto-commit interval in seconds
    pub auto_commit_interval_secs: u64,
}

impl Default for MemvidConfig {
    fn default() -> Self {
        Self {
            path: "agent_memory.mv2".to_string(),
            create_if_missing: true,
            enable_lex: true,
            enable_vec: true,
            cache_size: 1000,
            auto_commit_interval_secs: 60,
        }
    }
}

impl MemvidConfig {
    /// Create a new configuration
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Set the cache size
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }

    /// Disable auto-commit
    pub fn without_auto_commit(mut self) -> Self {
        self.auto_commit_interval_secs = 0;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = MemvidConfig::default();
        assert_eq!(config.path, "agent_memory.mv2");
        assert_eq!(config.cache_size, 1000);
        assert!(config.create_if_missing);
    }

    #[test]
    fn test_config_builder() {
        let config = MemvidConfig::new("test.mv2")
            .with_cache_size(500)
            .without_auto_commit();

        assert_eq!(config.path, "test.mv2");
        assert_eq!(config.cache_size, 500);
        assert_eq!(config.auto_commit_interval_secs, 0);
    }
}
