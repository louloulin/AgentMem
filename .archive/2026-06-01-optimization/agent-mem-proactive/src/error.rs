//! Error types for ProactiveAgent

use thiserror::Error;

/// Result type alias for ProactiveAgent operations
pub type Result<T> = std::result::Result<T, ProactiveError>;

/// Error types for ProactiveAgent operations
#[derive(Error, Debug)]
pub enum ProactiveError {
    /// Failed to initialize the scheduler
    #[error("Scheduler initialization failed: {0}")]
    SchedulerInit(String),

    /// Failed to schedule a task
    #[error("Failed to schedule task: {0}")]
    ScheduleError(String),

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    TaskExecution(String),

    /// Task not found
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Task already exists
    #[error("Task already exists: {0}")]
    TaskAlreadyExists(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Category error (from agent-mem-category)
    #[error("Category error: {0}")]
    CategoryError(String),

    /// Resource error (from agent-mem-resource)
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// Agent error (from agent-mem)
    #[error("Agent error: {0}")]
    AgentError(String),

    /// Shutdown error
    #[error("Shutdown error: {0}")]
    ShutdownError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for ProactiveError {
    fn from(err: std::io::Error) -> Self {
        ProactiveError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for ProactiveError {
    fn from(err: serde_json::Error) -> Self {
        ProactiveError::StorageError(err.to_string())
    }
}
