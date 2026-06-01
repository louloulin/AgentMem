//! Error types for MemVid integration

use agent_mem_traits::AgentMemError;
use thiserror::Error;

/// MemVid-specific error type
#[derive(Error, Debug)]
pub enum MemvidError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// MemVid core error
    #[error("MemVid error: {0}")]
    Memvid(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Memory not found
    #[error("Memory not found: {0}")]
    MemoryNotFound(String),

    /// Invalid memory data
    #[error("Invalid memory data: {0}")]
    InvalidMemory(String),

    /// Conversion error
    #[error("Conversion error: {0}")]
    Conversion(String),

    /// Search error
    #[error("Search error: {0}")]
    Search(String),

    /// Version not found
    #[error("Version not found: {0}")]
    VersionNotFound(String),

    /// Store is closed
    #[error("Store is closed")]
    StoreClosed,

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl From<serde_json::Error> for MemvidError {
    fn from(err: serde_json::Error) -> Self {
        MemvidError::Serialization(err.to_string())
    }
}

/// Result type for MemVid operations
pub type Result<T> = std::result::Result<T, MemvidError>;

impl From<MemvidError> for AgentMemError {
    fn from(err: MemvidError) -> Self {
        match err {
            MemvidError::MemoryNotFound(id) => {
                AgentMemError::memory_error(format!("Memory not found: {}", id))
            }
            MemvidError::Io(e) => AgentMemError::storage_error(format!("I/O: {}", e)),
            _ => AgentMemError::storage_error(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let err = MemvidError::MemoryNotFound("test-id".to_string());
        let agent_err: AgentMemError = err.into();
        matches!(agent_err, AgentMemError::MemoryError(_));
    }
}
