//! Error types for extraction pipeline

use thiserror::Error;

/// Extraction pipeline error types
#[derive(Error, Debug)]
pub enum ExtractionError {
    /// Resource not found or inaccessible
    #[error("Resource error: {0}")]
    ResourceNotFound(String),

    /// Invalid resource URI or format
    #[error("Invalid URI: {0}")]
    InvalidURI(String),

    /// Media type not supported
    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    /// Stage execution failed
    #[error("Stage '{name}' failed: {message}")]
    StageFailed {
        name: String,
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Pipeline configuration error
    #[error("Pipeline configuration error: {0}")]
    ConfigurationError(String),

    /// Timeout during extraction
    #[error("Extraction timeout after {0}s")]
    Timeout(u64),

    /// Duplicate detection failed
    #[error("Duplicate detection failed: {0}")]
    DuplicateDetectionError(String),

    /// Categorization failed
    #[error("Categorization failed: {0}")]
    CategorizationError(String),

    /// Index persistence failed
    #[error("Index persistence failed: {0}")]
    PersistenceError(String),

    /// LLM API error
    #[error("LLM API error: {0}")]
    LLMError(String),

    /// IO error
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

/// Result type for extraction operations
pub type Result<T> = std::result::Result<T, ExtractionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ExtractionError::ResourceNotFound("resource-123".to_string());
        assert_eq!(err.to_string(), "Resource error: resource-123");

        let err = ExtractionError::InvalidURI("invalid-uri".to_string());
        assert_eq!(err.to_string(), "Invalid URI: invalid-uri");

        let err = ExtractionError::UnsupportedMediaType("video/xyz".to_string());
        assert_eq!(err.to_string(), "Unsupported media type: video/xyz");
    }

    #[test]
    fn test_stage_failed_error() {
        let err = ExtractionError::StageFailed {
            name: "ItemExtractor".to_string(),
            message: "Failed to parse content".to_string(),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            )),
        };
        assert!(err.to_string().contains("ItemExtractor"));
        assert!(err.to_string().contains("Failed to parse content"));
    }
}
