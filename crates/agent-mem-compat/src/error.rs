//! Error types for Mem0 compatibility layer

use thiserror::Error;

/// Result type for Mem0 compatibility operations
pub type Result<T> = std::result::Result<T, Mem0Error>;

/// Errors that can occur in the Mem0 compatibility layer
#[derive(Error, Debug)]
pub enum Mem0Error {
    /// Memory not found
    #[error("Memory not found: {id}")]
    MemoryNotFound { id: String },

    /// Invalid user ID
    #[error("Invalid user ID: {user_id}")]
    InvalidUserId { user_id: String },

    /// Invalid memory content
    #[error("Invalid memory content: {reason}")]
    InvalidContent { reason: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Storage error
    #[error("Storage error: {message}")]
    StorageError { message: String },

    /// Embedding error
    #[error("Embedding error: {message}")]
    EmbeddingError { message: String },

    /// LLM error
    #[error("LLM error: {message}")]
    LlmError { message: String },

    /// Network error
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Internal error
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },

    /// Authentication error
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Security error
    #[error("Security error: {0}")]
    SecurityError(String),

    /// Feature not enabled
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

impl From<agent_mem_traits::AgentMemError> for Mem0Error {
    fn from(error: agent_mem_traits::AgentMemError) -> Self {
        match error {
            agent_mem_traits::AgentMemError::NotFound(msg) => Mem0Error::MemoryNotFound { id: msg },
            agent_mem_traits::AgentMemError::StorageError(msg) => {
                Mem0Error::StorageError { message: msg }
            }
            agent_mem_traits::AgentMemError::EmbeddingError(msg) => {
                Mem0Error::EmbeddingError { message: msg }
            }
            agent_mem_traits::AgentMemError::LLMError(msg) => Mem0Error::LlmError { message: msg },
            agent_mem_traits::AgentMemError::NetworkError(msg) => {
                Mem0Error::NetworkError { message: msg }
            }
            agent_mem_traits::AgentMemError::SerializationError(err) => {
                Mem0Error::SerializationError {
                    message: err.to_string(),
                }
            }
            agent_mem_traits::AgentMemError::ConfigError(msg) => {
                Mem0Error::ConfigError { message: msg }
            }
            agent_mem_traits::AgentMemError::AuthError(msg) => {
                Mem0Error::AuthenticationError { message: msg }
            }
            agent_mem_traits::AgentMemError::RateLimitError(msg) => {
                Mem0Error::RateLimitExceeded { message: msg }
            }
            agent_mem_traits::AgentMemError::ValidationError(msg) => {
                Mem0Error::InvalidContent { reason: msg }
            }
            _ => Mem0Error::InternalError {
                message: error.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for Mem0Error {
    fn from(error: serde_json::Error) -> Self {
        Mem0Error::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for Mem0Error {
    fn from(error: reqwest::Error) -> Self {
        Mem0Error::NetworkError {
            message: error.to_string(),
        }
    }
}
