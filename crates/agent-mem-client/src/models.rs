//! Client data models

use agent_mem_core::MemoryType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to add a new memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemoryRequest {
    /// Agent ID
    pub agent_id: String,

    /// User ID (optional)
    pub user_id: Option<String>,

    /// Memory content
    pub content: String,

    /// Memory type
    pub memory_type: Option<MemoryType>,

    /// Importance score (0.0 to 1.0)
    pub importance: Option<f32>,

    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to update a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryRequest {
    /// New content (optional)
    pub content: Option<String>,

    /// New importance score (optional)
    pub importance: Option<f32>,
}

/// Batch update request item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateItem {
    /// Memory ID to update
    pub memory_id: String,

    /// User ID
    pub user_id: String,

    /// Update request
    pub update_request: UpdateMemoryRequest,
}

/// Batch delete request item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteItem {
    /// Memory ID to delete
    pub memory_id: String,

    /// User ID
    pub user_id: String,
}

/// Response for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResponse {
    /// Memory ID
    pub id: String,

    /// Response message
    pub message: String,
}

/// Memory data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Memory ID
    pub id: String,

    /// Agent ID
    pub agent_id: String,

    /// User ID (optional)
    pub user_id: Option<String>,

    /// Memory content
    pub content: String,

    /// Memory type
    pub memory_type: Option<MemoryType>,

    /// Importance score
    pub importance: Option<f32>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to search memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoriesRequest {
    /// Search query
    pub query: String,

    /// Agent ID (optional)
    pub agent_id: Option<String>,

    /// User ID (optional)
    pub user_id: Option<String>,

    /// Memory type filter (optional)
    pub memory_type: Option<MemoryType>,

    /// Maximum number of results
    pub limit: Option<usize>,

    /// Similarity threshold
    pub threshold: Option<f32>,
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Memory data
    pub memory: Memory,

    /// Similarity score
    pub score: f32,
}

/// Response for search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoriesResponse {
    /// Search results
    pub results: Vec<SearchResult>,

    /// Total number of results
    pub total: usize,
}

/// Request for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAddMemoriesRequest {
    /// List of memory requests
    pub memories: Vec<AddMemoryRequest>,
}

/// Response for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Number of successful operations
    pub successful: usize,

    /// Number of failed operations
    pub failed: usize,

    /// Results from successful operations
    pub results: Vec<String>,

    /// Error messages from failed operations
    pub errors: Vec<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: String,

    /// Timestamp of the health check
    pub timestamp: DateTime<Utc>,

    /// Service version
    pub version: String,

    /// Individual component health checks
    pub checks: HashMap<String, String>,
}

/// Metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Timestamp of metrics collection
    pub timestamp: DateTime<Utc>,

    /// Collected metrics
    pub metrics: HashMap<String, f64>,
}

/// Error response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Additional error details
    pub details: Option<serde_json::Value>,

    /// Timestamp of the error
    pub timestamp: DateTime<Utc>,
}

impl AddMemoryRequest {
    /// Create a new memory request
    pub fn new(agent_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            user_id: None,
            content: content.into(),
            memory_type: None,
            importance: None,
            metadata: None,
        }
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set memory type
    pub fn with_memory_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    /// Set importance score
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = Some(importance);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl SearchMemoriesRequest {
    /// Create a new search request
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            agent_id: None,
            user_id: None,
            memory_type: None,
            limit: None,
            threshold: None,
        }
    }

    /// Set agent ID filter
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Set user ID filter
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set memory type filter
    pub fn with_memory_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set similarity threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_memory_request_builder() {
        let request = AddMemoryRequest::new("agent1", "test content")
            .with_user_id("user1")
            .with_memory_type(MemoryType::Episodic)
            .with_importance(0.8);

        assert_eq!(request.agent_id, "agent1");
        assert_eq!(request.content, "test content");
        assert_eq!(request.user_id, Some("user1".to_string()));
        assert_eq!(request.memory_type, Some(MemoryType::Episodic));
        assert_eq!(request.importance, Some(0.8));
    }

    #[test]
    fn test_search_request_builder() {
        let request = SearchMemoriesRequest::new("test query")
            .with_agent_id("agent1")
            .with_limit(10)
            .with_threshold(0.7);

        assert_eq!(request.query, "test query");
        assert_eq!(request.agent_id, Some("agent1".to_string()));
        assert_eq!(request.limit, Some(10));
        assert_eq!(request.threshold, Some(0.7));
    }
}
