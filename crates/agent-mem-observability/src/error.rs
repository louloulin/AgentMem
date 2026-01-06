//! Error types for observability

use thiserror::Error;

/// Result type for observability operations
pub type ObservabilityResult<T> = Result<T, ObservabilityError>;

/// Errors that can occur in observability operations
#[derive(Error, Debug)]
pub enum ObservabilityError {
    /// Tracing initialization failed
    #[error("Tracing initialization failed: {0}")]
    TracingInitFailed(String),

    /// Metrics initialization failed
    #[error("Metrics initialization failed: {0}")]
    MetricsInitFailed(String),

    /// Logging initialization failed
    #[error("Logging initialization failed: {0}")]
    LoggingInitFailed(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    /// Export failed
    #[error("Export failed: {0}")]
    ExportFailed(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ObservabilityError::TracingInitFailed("test".to_string());
        assert_eq!(err.to_string(), "Tracing initialization failed: test");

        let err = ObservabilityError::MetricsInitFailed("test".to_string());
        assert_eq!(err.to_string(), "Metrics initialization failed: test");
    }
}
