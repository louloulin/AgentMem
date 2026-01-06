//! Memory provider trait definitions

use crate::{HistoryEntry, MemoryItem, Message, Result, Session};
use async_trait::async_trait;

/// Core trait for memory providers
#[async_trait]
pub trait MemoryProvider: Send + Sync {
    /// Add new memories from messages
    async fn add(&self, messages: &[Message], session: &Session) -> Result<Vec<MemoryItem>>;

    /// Get a specific memory by ID
    async fn get(&self, id: &str) -> Result<Option<MemoryItem>>;

    /// Search memories by query
    async fn search(&self, query: &str, session: &Session, limit: usize)
        -> Result<Vec<MemoryItem>>;

    /// Update an existing memory
    async fn update(&self, id: &str, data: &str) -> Result<()>;

    /// Delete a memory
    async fn delete(&self, id: &str) -> Result<()>;

    /// Get history of changes for a memory
    async fn history(&self, id: &str) -> Result<Vec<HistoryEntry>>;

    /// Get all memories for a session
    async fn get_all(&self, session: &Session) -> Result<Vec<MemoryItem>>;

    /// Reset all memories (for testing)
    async fn reset(&self) -> Result<()>;
}
