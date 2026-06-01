//! Error types for resource operations

use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

/// Result type alias for resource operations
pub type Result<T> = std::result::Result<T, ResourceError>;

/// Errors that can occur during resource operations
#[derive(Error, Debug)]
pub enum ResourceError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Invalid URI format
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// Unsupported URI scheme
    #[error("Unsupported URI scheme: {0}")]
    UnsupportedScheme(String),

    /// Failed to resolve resource
    #[error("Failed to resolve resource: {0}")]
    ResolutionFailed(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(ResourceId),

    /// Failed to detect media type
    #[error("Failed to detect media type: {0}")]
    MediaTypeDetectionFailed(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),
}

/// Resource identifier wrapper for better error messages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ResourceId {
    fn from(id: String) -> Self {
        ResourceId(id)
    }
}

impl From<&str> for ResourceId {
    fn from(id: &str) -> Self {
        ResourceId(id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_id_display() {
        let id = ResourceId("res-123".to_string());
        assert_eq!(format!("{}", id), "res-123");
    }

    #[test]
    fn test_resource_id_from_string() {
        let id: ResourceId = "res-456".into();
        assert_eq!(id.0, "res-456");
    }

    #[test]
    fn test_error_display() {
        let err = ResourceError::InvalidUri("test://bad".to_string());
        assert_eq!(format!("{}", err), "Invalid URI: test://bad");
    }
}
