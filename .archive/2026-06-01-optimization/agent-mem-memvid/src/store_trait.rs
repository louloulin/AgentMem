//! Storage trait for MemVid backend

use crate::error::Result;
use agent_mem_traits::{Filters, Memory, MemoryId};
use async_trait::async_trait;

/// Core memory store trait for MemVid backend
///
/// This trait defines the basic CRUD operations for memory storage.
/// It's designed to work with the Memory V4 abstraction.
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Add a new memory to the store
    async fn add(&self, memory: &Memory) -> Result<()>;

    /// Get a memory by ID
    async fn get(&self, id: &MemoryId) -> Result<Option<Memory>>;

    /// Update an existing memory
    async fn update(&self, memory: &Memory) -> Result<()>;

    /// Delete a memory
    async fn delete(&self, id: &MemoryId) -> Result<()>;

    /// List memories with optional filters
    async fn list(&self, filters: &Filters) -> Result<Vec<Memory>>;

    /// Count total memories
    async fn count(&self) -> Result<usize>;

    /// Clear all memories
    async fn clear(&self) -> Result<()>;

    /// Check if store is healthy
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    /// Get store statistics
    async fn stats(&self) -> Result<StoreStats> {
        Ok(StoreStats {
            total_memories: self.count().await?,
            store_type: "MemVid".to_string(),
            path: String::new(),
        })
    }
}

/// Store statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoreStats {
    /// Total number of memories
    pub total_memories: usize,

    /// Store type
    pub store_type: String,

    /// Store path
    pub path: String,
}
