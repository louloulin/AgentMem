//! Error types for the category hierarchy system

/// Main error type for category operations
#[derive(Debug, thiserror::Error)]
pub enum CategoryError {
    /// Category not found
    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    /// Invalid category path
    #[error("Invalid category path: {0}")]
    InvalidPath(String),

    /// Category already exists
    #[error("Category already exists: {0}")]
    CategoryAlreadyExists(String),

    /// Parent category not found
    #[error("Parent category not found: {0}")]
    ParentNotFound(String),

    /// Circular reference detected
    #[error("Circular reference detected in category hierarchy")]
    CircularReference,

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// LLM error (for summary generation)
    #[error("LLM error: {0}")]
    LLMError(String),

    /// Invalid embedding
    #[error("Invalid embedding: {0}")]
    InvalidEmbedding(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Result type for category operations
pub type Result<T> = std::result::Result<T, CategoryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CategoryError::CategoryNotFound("/preferences/programming".to_string());
        assert!(err.to_string().contains("Category not found"));
        assert!(err.to_string().contains("/preferences/programming"));
    }

    #[test]
    fn test_error_chain() {
        let err = CategoryError::ParentNotFound("/preferences".to_string());
        assert!(matches!(err, CategoryError::ParentNotFound(_)));
    }
}
