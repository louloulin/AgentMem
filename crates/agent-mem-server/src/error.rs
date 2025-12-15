//! Error handling for the server
//!
//! This module provides comprehensive error handling with:
//! - Unified error types
//! - Error context and stack traces
//! - Error conversion traits
//! - Friendly error messages
//! - Error monitoring support

use crate::models::ErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use std::backtrace::Backtrace;
use std::fmt;
use thiserror::Error;

/// Error context for additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Contextual information about where the error occurred
    pub context: String,
    /// Optional additional details
    pub details: Option<String>,
    /// Timestamp when the error occurred
    pub timestamp: chrono::DateTime<Utc>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            details: None,
            timestamp: Utc::now(),
        }
    }

    /// Add additional details to the context
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new("Unknown error context")
    }
}

/// Server error types with context support
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Memory operation failed: {message}")]
    MemoryError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: Option<ErrorContext>,
        backtrace: Option<Backtrace>,
    },

    #[error("Resource not found: {message}")]
    NotFound {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Invalid request: {message}")]
    BadRequest {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Authentication failed: {message}")]
    Unauthorized {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Access forbidden: {message}")]
    Forbidden {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Quota exceeded: {message}")]
    QuotaExceeded {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Validation failed: {message}")]
    ValidationError {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Server binding failed: {message}")]
    BindError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: Option<ErrorContext>,
    },

    #[error("Server error: {message}")]
    ServerError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: Option<ErrorContext>,
        backtrace: Option<Backtrace>,
    },

    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: Option<ErrorContext>,
    },

    #[error("Telemetry setup failed: {message}")]
    TelemetryError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: Option<ErrorContext>,
    },

    #[error("Internal server error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: Option<ErrorContext>,
        backtrace: Option<Backtrace>,
    },
}

/// Server result type
pub type ServerResult<T> = Result<T, ServerError>;

impl ServerError {
    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        ServerError::NotFound {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a bad request error
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ServerError::BadRequest {
            message: msg.into(),
            context: None,
        }
    }

    /// Create an unauthorized error
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        ServerError::Unauthorized {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a forbidden error
    pub fn forbidden(msg: impl Into<String>) -> Self {
        ServerError::Forbidden {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a quota exceeded error
    pub fn quota_exceeded(msg: impl Into<String>) -> Self {
        ServerError::QuotaExceeded {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a quota exceeded error
    pub fn quota_exceeded(msg: impl Into<String>) -> Self {
        ServerError::QuotaExceeded {
            message: msg.into(),
            context: None,
        }
    }

    /// Create an internal error
    pub fn internal_error(msg: impl Into<String>) -> Self {
        ServerError::Internal {
            message: msg.into(),
            source: None,
            context: None,
            backtrace: Backtrace::capture(),
        }
    }

    /// Create a memory error
    pub fn memory_error(msg: impl Into<String>) -> Self {
        ServerError::MemoryError {
            message: msg.into(),
            source: None,
            context: None,
            backtrace: Backtrace::capture(),
        }
    }

    /// Create a server error
    pub fn server_error(msg: impl Into<String>) -> Self {
        ServerError::ServerError {
            message: msg.into(),
            source: None,
            context: None,
            backtrace: Backtrace::capture(),
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ServerError::MemoryError { message, .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "MEMORY_ERROR", message)
            }
            ServerError::NotFound { message, .. } => (StatusCode::NOT_FOUND, "NOT_FOUND", message),
            ServerError::BadRequest { message, .. } => (StatusCode::BAD_REQUEST, "BAD_REQUEST", message),
            ServerError::Unauthorized { message, .. } => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message),
            ServerError::Forbidden { message, .. } => (StatusCode::FORBIDDEN, "FORBIDDEN", message),
            ServerError::QuotaExceeded { message, .. } => {
                (StatusCode::TOO_MANY_REQUESTS, "QUOTA_EXCEEDED", message)
            }
            ServerError::ValidationError { message, .. } => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message),
            ServerError::BindError { message, .. } => (StatusCode::INTERNAL_SERVER_ERROR, "BIND_ERROR", message),
            ServerError::ServerError { message, .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "SERVER_ERROR", message)
            }
            ServerError::ConfigError { message, .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", message)
            }
            ServerError::TelemetryError { message, .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "TELEMETRY_ERROR", message)
            }
            ServerError::Internal { message, .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
            }
        };

        let error_response = ErrorResponse {
            code: code.to_string(),
            message,
            details: None,
            timestamp: Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<agent_mem_traits::AgentMemError> for ServerError {
    fn from(err: agent_mem_traits::AgentMemError) -> Self {
        ServerError::MemoryError {
            message: err.to_string(),
            source: Some(Box::new(err)),
            context: None,
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::BadRequest(format!("JSON parsing error: {err}"))
    }
}

impl From<validator::ValidationErrors> for ServerError {
    fn from(err: validator::ValidationErrors) -> Self {
        ServerError::ValidationError(format!("Validation failed: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_error_conversion() {
        let error = ServerError::NotFound("Test not found".to_string());
        let response = error.into_response();

        // We can't easily test the exact response content here,
        // but we can verify the error type conversion works
        assert!(matches!(response.status(), StatusCode::NOT_FOUND));
    }

    #[test]
    fn test_memory_error_conversion() {
        let memory_error = agent_mem_traits::AgentMemError::memory_error("test");
        let server_error: ServerError = memory_error.into();

        assert!(matches!(server_error, ServerError::MemoryError(_)));
    }
}
