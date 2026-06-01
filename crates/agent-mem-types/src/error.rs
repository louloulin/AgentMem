//! Error types for AgentMem
//! 
//! Unified error handling across the AgentMem ecosystem.

use thiserror::Error;

/// Error severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error context for debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub details: Vec<(String, String)>,
    pub timestamp: Option<i64>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            details: Vec::new(),
            timestamp: Some(chrono::Utc::now().timestamp()),
        }
    }

    /// Add detail
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.push((key.into(), value.into()));
        self
    }

    /// Format as string
    pub fn format(&self) -> String {
        let mut parts = vec![format!("operation: {}", self.operation)];
        for (k, v) in &self.details {
            parts.push(format!("{k}: {v}"));
        }
        if let Some(ts) = self.timestamp {
            parts.push(format!("timestamp: {ts}"));
        }
        parts.join(", ")
    }
}

/// Main error type
#[derive(Error, Debug)]
pub enum AgentMemError {
    #[error("Memory operation failed: {0}")]
    MemoryError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, AgentMemError>;

impl AgentMemError {
    /// Create memory error
    pub fn memory_error(msg: impl Into<String>) -> Self {
        Self::MemoryError(msg.into())
    }

    /// Create storage error
    pub fn storage_error(msg: impl Into<String>) -> Self {
        Self::StorageError(msg.into())
    }

    /// Create embedding error
    pub fn embedding_error(msg: impl Into<String>) -> Self {
        Self::EmbeddingError(msg.into())
    }

    /// Create validation error
    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Get severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound(_) | Self::InvalidInput(_) => ErrorSeverity::Medium,
            Self::ValidationError(_) | Self::UnsupportedOperation(_) => ErrorSeverity::Medium,
            _ => ErrorSeverity::High,
        }
    }

    /// Check if retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::StorageError(_) | Self::IoError(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AgentMemError::memory_error("test");
        assert!(matches!(err, AgentMemError::MemoryError(_)));
    }

    #[test]
    fn test_error_context() {
        let ctx = ErrorContext::new("test_op")
            .with_detail("key", "value");
        assert!(ctx.format().contains("test_op"));
    }
}
