//! API request and response models

use agent_mem_core::MemoryType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use validator::Validate;

/// Request to add a new memory
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct MemoryRequest {
    /// Agent ID (optional, defaults to "default-agent" or "default-agent-{user_id}")
    #[validate(length(min = 1, max = 255))]
    pub agent_id: Option<String>,

    /// User ID (optional)
    #[validate(length(max = 255))]
    pub user_id: Option<String>,

    /// Memory content
    #[validate(length(min = 1, max = 10000))]
    pub content: String,

    /// Memory type
    pub memory_type: Option<MemoryType>,

    /// Importance score (0.0 to 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub importance: Option<f32>,

    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to update a memory
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateMemoryRequest {
    /// New content (optional)
    #[validate(length(max = 10000))]
    pub content: Option<String>,

    /// New importance score (optional)
    #[validate(range(min = 0.0, max = 1.0))]
    pub importance: Option<f32>,
}

/// Response for memory operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryResponse {
    /// Memory ID
    pub id: String,

    /// Response message
    pub message: String,
}

/// Request to search memories
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct SearchRequest {
    /// Search query
    #[validate(length(min = 1, max = 1000))]
    pub query: String,

    /// Agent ID (optional)
    #[validate(length(max = 255))]
    pub agent_id: Option<String>,

    /// User ID (optional)
    #[validate(length(max = 255))]
    pub user_id: Option<String>,

    /// Memory type filter (optional)
    pub memory_type: Option<MemoryType>,

    /// Maximum number of results
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Similarity threshold
    #[validate(range(min = 0.0, max = 1.0))]
    pub threshold: Option<f32>,
}

/// Response for search operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<serde_json::Value>,

    /// Total number of results
    pub total: usize,
}

/// Request for batch operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct BatchRequest {
    /// List of memory requests
    #[validate(length(min = 1, max = 100))]
    pub memories: Vec<MemoryRequest>,
}

/// Response for batch operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComponentStatus {
    /// Component status (healthy, degraded, unhealthy)
    pub status: String,

    /// Optional status message
    pub message: Option<String>,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Overall health status
    pub status: String,

    /// Timestamp of the health check
    pub timestamp: DateTime<Utc>,

    /// Service version
    pub version: String,

    /// Individual component health checks
    pub checks: HashMap<String, ComponentStatus>,
}

/// Metrics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MetricsResponse {
    /// Timestamp of metrics collection
    pub timestamp: DateTime<Utc>,

    /// Collected metrics
    pub metrics: HashMap<String, f64>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,

    /// Success status
    #[serde(default = "default_true")]
    pub success: bool,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

fn default_true() -> bool {
    true
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            data,
            success: true,
            message: None,
        }
    }

    /// Create a successful response with a message
    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            data,
            success: true,
            message: Some(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_request_validation() {
        let request = MemoryRequest {
            agent_id: Some("test_agent".to_string()),
            user_id: Some("test_user".to_string()),
            content: "Test memory content".to_string(),
            memory_type: Some(MemoryType::Episodic),
            importance: Some(0.8),
            metadata: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_memory_request_validation_fails() {
        let request = MemoryRequest {
            agent_id: Some("".to_string()), // Empty agent_id should fail
            user_id: Some("test_user".to_string()),
            content: "Test memory content".to_string(),
            memory_type: Some(MemoryType::Episodic),
            importance: Some(0.8),
            metadata: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_search_request_validation() {
        let request = SearchRequest {
            query: "test query".to_string(),
            agent_id: Some("test_agent".to_string()),
            user_id: Some("test_user".to_string()),
            memory_type: Some(MemoryType::Semantic),
            limit: Some(10),
            threshold: Some(0.3),  // 🔧 降低阈值以支持商品ID等精确查询
        };

        assert!(request.validate().is_ok());
    }
}
