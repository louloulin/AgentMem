//! Validation middleware for AgentMem API
//!
//! This module provides middleware for validating incoming requests before they reach handlers.
//! It uses the validator crate to ensure data integrity and security.
//!
//! 🎯 P1 Task: Input validation middleware
//! 📅 Created: 2025-01-07
//! 🏗️ Architecture: Security validation at API boundary

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::{error, warn};

use crate::routes::memory::validators::{
    AddMemoryRequest, UpdateMemoryRequest, SearchMemoryRequest,
    DeleteMemoryRequest, BatchAddMemoriesRequest,
};

/// Validation error response
#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref field) = self.field {
            write!(f, "Validation error for field '{}': {}", field, self.message)
        } else {
            write!(f, "Validation error: {}", self.message)
        }
    }
}

impl std::error::Error for ValidationError {}

/// Convert ValidationError to HTTP response
pub fn validation_error_response(error: String) -> Response {
    warn!("Request validation failed: {}", error);
    
    let body = Json(json!({
        "success": false,
        "error": {
            "code": "VALIDATION_ERROR",
            "message": error,
            "details": "Request validation failed. Please check your input and try again."
        }
    }));

    // Convert Json to response body
    Json(body).into_response()
}

/// Validate add memory request
pub fn validate_add_memory_request(
    content: String,
    metadata: Option<std::collections::HashMap<String, String>>,
    tags: Option<Vec<String>>,
    importance: Option<f64>,
    agent_id: Option<String>,
    session_id: Option<String>,
) -> Result<(), String> {
    let request = AddMemoryRequest {
        content,
        metadata,
        tags,
        importance,
        agent_id,
        session_id,
    };

    request.validate_payload()
}

/// Validate update memory request
pub fn validate_update_memory_request(
    id: String,
    content: String,
    metadata: Option<std::collections::HashMap<String, String>>,
    tags: Option<Vec<String>>,
    importance: Option<f64>,
) -> Result<(), String> {
    let request = UpdateMemoryRequest {
        id,
        content,
        metadata,
        tags,
        importance,
    };

    request.validate_payload()
}

/// Validate search memory request
pub fn validate_search_request(
    query: String,
    limit: usize,
    agent_id: Option<String>,
    tags: Option<Vec<String>>,
    min_importance: Option<f64>,
) -> Result<(), String> {
    let request = SearchMemoryRequest {
        query,
        limit,
        agent_id,
        tags,
        min_importance,
    };

    request.validate_payload()
}

/// Validate delete memory request
pub fn validate_delete_request(id: String) -> Result<(), String> {
    let request = DeleteMemoryRequest { id };
    request.validate_payload()
}

/// Validate batch add memories request
pub fn validate_batch_add_request(
    memories: Vec<AddMemoryRequest>,
) -> Result<(), String> {
    let request = BatchAddMemoriesRequest { memories };
    request.validate_payload()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validate_add_memory_valid() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            Some(0.5),
            Some("agent-123".to_string()),
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_add_memory_html_content() {
        let result = validate_add_memory_request(
            "<script>alert('xss')</script>".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("content_contains_html_or_script"));
    }

    #[test]
    fn test_validate_add_memory_too_long() {
        let result = validate_add_memory_request(
            "a".repeat(50_001),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_add_memory_invalid_importance() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            Some(1.5), // Invalid: > 1.0
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_search_valid() {
        let result = validate_search_request(
            "rust programming".to_string(),
            10,
            Some("agent-123".to_string()),
            Some(vec!["rust".to_string()]),
            Some(0.3),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_search_empty_query() {
        let result = validate_search_request(
            "".to_string(),
            10,
            None,
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_search_invalid_limit() {
        let result = validate_search_request(
            "test query".to_string(),
            200, // Invalid: > 100
            None,
            None,
            None,
        );

        assert!(result.is_err());
    }
}
