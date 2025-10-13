//! Error types for AgentMem
//!
//! This module provides comprehensive error handling for AgentMem operations.
//! All errors include context information for better debugging and error recovery.

use thiserror::Error;

/// Error severity level for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - informational, no action needed
    Low,
    /// Medium severity - warning, may need attention
    Medium,
    /// High severity - error, needs attention
    High,
    /// Critical severity - critical error, immediate action required
    Critical,
}

/// Error context information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation that failed
    pub operation: String,
    /// Additional context details
    pub details: Vec<(String, String)>,
    /// Timestamp when error occurred
    pub timestamp: Option<i64>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            details: Vec::new(),
            timestamp: Some(chrono::Utc::now().timestamp()),
        }
    }

    /// Add a detail to the context
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.push((key.into(), value.into()));
        self
    }

    /// Format context as a string
    pub fn format(&self) -> String {
        let mut parts = vec![format!("operation: {}", self.operation)];
        for (key, value) in &self.details {
            parts.push(format!("{}: {}", key, value));
        }
        if let Some(ts) = self.timestamp {
            parts.push(format!("timestamp: {}", ts));
        }
        parts.join(", ")
    }
}

/// Main error type for AgentMem operations
#[derive(Error, Debug)]
pub enum AgentMemError {
    #[error("Memory operation failed: {0}")]
    MemoryError(String),

    #[error("LLM provider error: {0}")]
    LLMError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("UUID error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("System not running")]
    SystemNotRunning,

    #[error("Health checker already running")]
    HealthCheckerAlreadyRunning,

    #[error("Performance monitor already running")]
    PerformanceMonitorAlreadyRunning,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Config file not found: {0}")]
    ConfigFileNotFound(String),

    #[error("Config load error: {0}")]
    ConfigLoadError(String),

    #[error("Config parse error: {0}")]
    ConfigParseError(String),

    #[error("Config serialize error: {0}")]
    ConfigSerializeError(String),

    #[error("Config save error: {0}")]
    ConfigSaveError(String),

    #[error("Config validation error: {0}")]
    ConfigValidationError(String),

    #[error("Generic error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias for AgentMem operations
pub type Result<T> = std::result::Result<T, AgentMemError>;

// Implement From<sqlx::Error> for AgentMemError
#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for AgentMemError {
    fn from(err: sqlx::Error) -> Self {
        Self::StorageError(err.to_string())
    }
}

impl AgentMemError {
    /// Create a memory error
    pub fn memory_error(msg: impl Into<String>) -> Self {
        Self::MemoryError(msg.into())
    }

    /// Create an LLM error
    pub fn llm_error(msg: impl Into<String>) -> Self {
        Self::LLMError(msg.into())
    }

    /// Create a storage error
    pub fn storage_error(msg: impl Into<String>) -> Self {
        Self::StorageError(msg.into())
    }

    /// Create an embedding error
    pub fn embedding_error(msg: impl Into<String>) -> Self {
        Self::EmbeddingError(msg.into())
    }

    /// Create a session error
    pub fn session_error(msg: impl Into<String>) -> Self {
        Self::SessionError(msg.into())
    }

    /// Create a configuration error
    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create an unsupported provider error
    pub fn unsupported_provider(provider: impl Into<String>) -> Self {
        Self::UnsupportedProvider(provider.into())
    }

    /// Create an invalid config error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Create a network error
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }

    /// Create an authentication error
    pub fn auth_error(msg: impl Into<String>) -> Self {
        Self::AuthError(msg.into())
    }

    /// Create a rate limit error
    pub fn rate_limit_error(msg: impl Into<String>) -> Self {
        Self::RateLimitError(msg.into())
    }

    /// Create a timeout error
    pub fn timeout_error(msg: impl Into<String>) -> Self {
        Self::TimeoutError(msg.into())
    }

    /// Create a template error
    pub fn template_error(msg: impl Into<String>) -> Self {
        Self::TemplateError(msg.into())
    }

    /// Create a parsing error
    pub fn parsing_error(msg: impl Into<String>) -> Self {
        Self::ParsingError(msg.into())
    }

    /// Create a processing error
    pub fn processing_error(msg: impl Into<String>) -> Self {
        Self::ProcessingError(msg.into())
    }

    /// Create an unsupported operation error
    pub fn unsupported_operation(msg: impl Into<String>) -> Self {
        Self::UnsupportedOperation(msg.into())
    }

    /// Create a validation error
    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an internal error
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::ProcessingError(msg.into())
    }

    /// Get the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical errors - system cannot function
            Self::SystemNotRunning => ErrorSeverity::Critical,
            Self::StorageError(_) => ErrorSeverity::Critical,
            Self::ConfigError(_) => ErrorSeverity::Critical,
            Self::InvalidConfig(_) => ErrorSeverity::Critical,

            // High severity - operation failed but system can continue
            Self::MemoryError(_) => ErrorSeverity::High,
            Self::LLMError(_) => ErrorSeverity::High,
            Self::EmbeddingError(_) => ErrorSeverity::High,
            Self::NetworkError(_) => ErrorSeverity::High,
            Self::AuthError(_) => ErrorSeverity::High,
            Self::TimeoutError(_) => ErrorSeverity::High,

            // Medium severity - recoverable errors
            Self::RateLimitError(_) => ErrorSeverity::Medium,
            Self::ValidationError(_) => ErrorSeverity::Medium,
            Self::InvalidInput(_) => ErrorSeverity::Medium,
            Self::NotFound(_) => ErrorSeverity::Medium,
            Self::UnsupportedProvider(_) => ErrorSeverity::Medium,
            Self::UnsupportedOperation(_) => ErrorSeverity::Medium,

            // Low severity - informational
            Self::HealthCheckerAlreadyRunning => ErrorSeverity::Low,
            Self::PerformanceMonitorAlreadyRunning => ErrorSeverity::Low,

            // Default to medium for others
            _ => ErrorSeverity::Medium,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_)
                | Self::TimeoutError(_)
                | Self::RateLimitError(_)
                | Self::StorageError(_)
        )
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::MemoryError(msg) => format!("记忆操作失败: {}", msg),
            Self::LLMError(msg) => format!("LLM 服务错误: {}", msg),
            Self::StorageError(msg) => format!("存储错误: {}", msg),
            Self::EmbeddingError(msg) => format!("嵌入模型错误: {}", msg),
            Self::NetworkError(msg) => format!("网络错误: {}", msg),
            Self::AuthError(msg) => format!("认证失败: {}", msg),
            Self::RateLimitError(msg) => format!("请求频率超限: {}", msg),
            Self::TimeoutError(msg) => format!("操作超时: {}", msg),
            Self::ValidationError(msg) => format!("验证失败: {}", msg),
            Self::NotFound(msg) => format!("未找到: {}", msg),
            Self::InvalidInput(msg) => format!("无效输入: {}", msg),
            Self::ConfigError(msg) => format!("配置错误: {}", msg),
            _ => self.to_string(),
        }
    }

    /// Get recovery suggestion for this error
    pub fn recovery_suggestion(&self) -> Option<String> {
        match self {
            Self::NetworkError(_) => Some("请检查网络连接并重试".to_string()),
            Self::TimeoutError(_) => Some("操作超时，请稍后重试".to_string()),
            Self::RateLimitError(_) => Some("请求过于频繁，请稍后再试".to_string()),
            Self::AuthError(_) => Some("请检查认证凭据是否正确".to_string()),
            Self::ConfigError(_) => Some("请检查配置文件是否正确".to_string()),
            Self::InvalidConfig(_) => Some("请修正配置参数".to_string()),
            Self::StorageError(_) => Some("请检查数据库连接和权限".to_string()),
            Self::NotFound(_) => Some("请确认资源是否存在".to_string()),
            Self::ValidationError(_) => Some("请检查输入参数是否符合要求".to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        // Critical errors
        assert_eq!(AgentMemError::SystemNotRunning.severity(), ErrorSeverity::Critical);
        assert_eq!(AgentMemError::storage_error("test").severity(), ErrorSeverity::Critical);

        // High severity errors
        assert_eq!(AgentMemError::memory_error("test").severity(), ErrorSeverity::High);
        assert_eq!(AgentMemError::llm_error("test").severity(), ErrorSeverity::High);

        // Medium severity errors
        assert_eq!(AgentMemError::rate_limit_error("test").severity(), ErrorSeverity::Medium);
        assert_eq!(AgentMemError::validation_error("test").severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_retryability() {
        // Retryable errors
        assert!(AgentMemError::network_error("test").is_retryable());
        assert!(AgentMemError::timeout_error("test").is_retryable());
        assert!(AgentMemError::rate_limit_error("test").is_retryable());
        assert!(AgentMemError::storage_error("test").is_retryable());

        // Non-retryable errors
        assert!(!AgentMemError::validation_error("test").is_retryable());
        assert!(!AgentMemError::not_found("test").is_retryable());
        assert!(!AgentMemError::config_error("test").is_retryable());
    }

    #[test]
    fn test_user_message() {
        let error = AgentMemError::memory_error("无法保存");
        assert!(error.user_message().contains("记忆操作失败"));

        let error = AgentMemError::network_error("连接失败");
        assert!(error.user_message().contains("网络错误"));
    }

    #[test]
    fn test_recovery_suggestion() {
        let error = AgentMemError::network_error("test");
        assert!(error.recovery_suggestion().is_some());
        assert!(error.recovery_suggestion().unwrap().contains("网络连接"));

        let error = AgentMemError::memory_error("test");
        assert!(error.recovery_suggestion().is_none());
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .with_detail("user_id", "user123")
            .with_detail("agent_id", "agent456");

        let formatted = context.format();
        assert!(formatted.contains("test_operation"));
        assert!(formatted.contains("user_id: user123"));
        assert!(formatted.contains("agent_id: agent456"));
    }
}
