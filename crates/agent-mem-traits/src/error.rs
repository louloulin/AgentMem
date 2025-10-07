//! Error types for AgentMem

use thiserror::Error;

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
    pub fn memory_error(msg: impl Into<String>) -> Self {
        Self::MemoryError(msg.into())
    }

    pub fn llm_error(msg: impl Into<String>) -> Self {
        Self::LLMError(msg.into())
    }

    pub fn storage_error(msg: impl Into<String>) -> Self {
        Self::StorageError(msg.into())
    }

    pub fn embedding_error(msg: impl Into<String>) -> Self {
        Self::EmbeddingError(msg.into())
    }

    pub fn session_error(msg: impl Into<String>) -> Self {
        Self::SessionError(msg.into())
    }

    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    pub fn unsupported_provider(provider: impl Into<String>) -> Self {
        Self::UnsupportedProvider(provider.into())
    }

    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }

    pub fn auth_error(msg: impl Into<String>) -> Self {
        Self::AuthError(msg.into())
    }

    pub fn rate_limit_error(msg: impl Into<String>) -> Self {
        Self::RateLimitError(msg.into())
    }

    pub fn timeout_error(msg: impl Into<String>) -> Self {
        Self::TimeoutError(msg.into())
    }

    pub fn template_error(msg: impl Into<String>) -> Self {
        Self::TemplateError(msg.into())
    }

    pub fn parsing_error(msg: impl Into<String>) -> Self {
        Self::ParsingError(msg.into())
    }

    pub fn processing_error(msg: impl Into<String>) -> Self {
        Self::ProcessingError(msg.into())
    }

    pub fn unsupported_operation(msg: impl Into<String>) -> Self {
        Self::UnsupportedOperation(msg.into())
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::ProcessingError(msg.into())
    }
}
