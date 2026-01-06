//! Storage trait definitions

use crate::{
    Entity, Filters, HistoryEntry, Metadata, Relation, Result, SearchResult, Session, Vector,
    VectorData, VectorSearchResult,
};
use async_trait::async_trait;

/// Legacy vector store trait (kept for compatibility)
#[async_trait]
pub trait LegacyVectorStore: Send + Sync {
    /// Insert vectors with metadata
    async fn insert(&self, vectors: &[Vector], metadata: &[Metadata]) -> Result<Vec<String>>;

    /// Search for similar vectors
    async fn search(
        &self,
        query: &Vector,
        limit: usize,
        filters: &Filters,
    ) -> Result<Vec<SearchResult>>;

    /// Update a vector and its metadata
    async fn update(&self, id: &str, vector: &Vector, metadata: &Metadata) -> Result<()>;

    /// Delete a vector
    async fn delete(&self, id: &str) -> Result<()>;

    /// Reset the vector store (for testing)
    async fn reset(&self) -> Result<()>;

    /// Get store statistics
    async fn stats(&self) -> Result<VectorStoreStats>;
}

/// Modern vector store trait
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Add vectors to the store
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>>;

    /// Search for similar vectors
    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>>;

    /// Search with advanced filters and threshold
    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &std::collections::HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>>;

    /// Delete vectors by IDs
    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()>;

    /// Update existing vectors
    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()>;

    /// Get a specific vector by ID
    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>>;

    /// Count total vectors in the store
    async fn count_vectors(&self) -> Result<usize>;

    /// Clear all vectors from the store
    async fn clear(&self) -> Result<()>;

    /// Health check for the vector store
    async fn health_check(&self) -> Result<crate::HealthStatus>;

    /// Get vector store statistics
    async fn get_stats(&self) -> Result<VectorStoreStats>;

    /// Batch operations for vectors
    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>>;

    /// Delete vectors by batch
    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>>;
}

/// Embedding-focused vector store trait
#[async_trait]
pub trait EmbeddingVectorStore: Send + Sync {
    /// Store an embedding with metadata
    async fn store_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &std::collections::HashMap<String, String>,
    ) -> Result<()>;

    /// Search for similar embeddings
    async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<SearchResult>>;

    /// Delete an embedding
    async fn delete_embedding(&self, memory_id: &str) -> Result<()>;

    /// Update an embedding
    async fn update_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &std::collections::HashMap<String, String>,
    ) -> Result<()>;

    /// Get an embedding by ID
    async fn get_embedding(&self, memory_id: &str) -> Result<Option<Vec<f32>>>;

    /// List all embedding IDs
    async fn list_embeddings(&self, prefix: Option<&str>) -> Result<Vec<String>>;
}

/// Graph store trait
#[async_trait]
pub trait GraphStore: Send + Sync {
    /// Add entities to the graph
    async fn add_entities(&self, entities: &[Entity], session: &Session) -> Result<()>;

    /// Add relations to the graph
    async fn add_relations(&self, relations: &[Relation], session: &Session) -> Result<()>;

    /// Search the graph
    async fn search_graph(&self, query: &str, session: &Session) -> Result<Vec<GraphResult>>;

    /// Get neighbors of an entity
    async fn get_neighbors(&self, entity_id: &str, depth: usize) -> Result<Vec<Entity>>;

    /// Reset the graph store (for testing)
    async fn reset(&self) -> Result<()>;
}

/// Key-value store trait
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    /// Set a key-value pair
    async fn set(&self, key: &str, value: &str) -> Result<()>;

    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<String>>;

    /// Delete a key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Set with expiration
    async fn set_with_expiry(&self, key: &str, value: &str, seconds: u64) -> Result<()>;
}

/// History store trait
#[async_trait]
pub trait HistoryStore: Send + Sync {
    /// Add a history entry
    async fn add_history_entry(&self, memory_id: &str, entry: &HistoryEntry) -> Result<()>;

    /// Get history for a memory
    async fn get_history(&self, memory_id: &str) -> Result<Vec<HistoryEntry>>;

    /// Delete history for a memory
    async fn delete_history(&self, memory_id: &str) -> Result<()>;
}

/// Vector store statistics
#[derive(Debug, Clone)]
pub struct VectorStoreStats {
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_size: usize,
}

/// Graph search result
#[derive(Debug, Clone)]
pub struct GraphResult {
    pub entity: Entity,
    pub relations: Vec<Relation>,
    pub score: f32,
}
