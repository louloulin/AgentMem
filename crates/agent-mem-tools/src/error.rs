//! Error types for the tool execution framework

use thiserror::Error;

/// Result type for tool operations
pub type ToolResult<T> = Result<T, ToolError>;

/// Errors that can occur during tool execution
#[derive(Error, Debug, Clone)]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Timeout
    #[error("Execution timeout")]
    Timeout,

    /// Schema validation failed
    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Tool already registered
    #[error("Tool already registered: {0}")]
    AlreadyRegistered(String),

    /// Dependency error
    #[error("Dependency error: {0}")]
    DependencyError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        ToolError::SerializationError(err.to_string())
    }
}

/// Errors that can occur in the sandbox
#[derive(Error, Debug, Clone)]
pub enum SandboxError {
    /// Execution timeout
    #[error("Sandbox execution timeout")]
    Timeout,

    /// Memory limit exceeded
    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,

    /// CPU limit exceeded
    #[error("CPU limit exceeded")]
    CpuLimitExceeded,

    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<SandboxError> for ToolError {
    fn from(err: SandboxError) -> Self {
        match err {
            SandboxError::Timeout => ToolError::Timeout,
            SandboxError::MemoryLimitExceeded => {
                ToolError::ResourceLimitExceeded("Memory limit exceeded".to_string())
            }
            SandboxError::CpuLimitExceeded => {
                ToolError::ResourceLimitExceeded("CPU limit exceeded".to_string())
            }
            SandboxError::ExecutionFailed(msg) => ToolError::ExecutionFailed(msg),
            SandboxError::Internal(msg) => ToolError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ToolError::NotFound("test_tool".to_string());
        assert_eq!(err.to_string(), "Tool not found: test_tool");

        let err = ToolError::PermissionDenied("user123".to_string());
        assert_eq!(err.to_string(), "Permission denied: user123");

        let err = ToolError::Timeout;
        assert_eq!(err.to_string(), "Execution timeout");
    }

    #[test]
    fn test_sandbox_error_conversion() {
        let sandbox_err = SandboxError::Timeout;
        let tool_err: ToolError = sandbox_err.into();
        assert!(matches!(tool_err, ToolError::Timeout));

        let sandbox_err = SandboxError::MemoryLimitExceeded;
        let tool_err: ToolError = sandbox_err.into();
        assert!(matches!(tool_err, ToolError::ResourceLimitExceeded(_)));
    }
}
