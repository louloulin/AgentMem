//! Client error types

use thiserror::Error;

/// Client error types
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("URL parsing failed: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Server error (status: {status}): {message}")]
    ServerError { status: u16, message: String },

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Retry exhausted after {attempts} attempts: {last_error}")]
    RetryExhausted { attempts: u32, last_error: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Internal client error: {0}")]
    InternalError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Client result type
pub type ClientResult<T> = Result<T, ClientError>;

impl ClientError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            ClientError::HttpError(e) => {
                // Retry on network errors, timeouts, and 5xx server errors
                if e.is_timeout() || e.is_connect() || e.is_request() {
                    return true;
                }

                if let Some(status) = e.status() {
                    return status.is_server_error() || status == 429; // Rate limited
                }

                false
            }
            ClientError::TimeoutError(_) => true,
            ClientError::NetworkError(_) => true,
            ClientError::ServerError { status, .. } => {
                *status >= 500 || *status == 429 // 5xx errors or rate limiting
            }
            _ => false,
        }
    }

    /// Get the HTTP status code if available
    pub fn status_code(&self) -> Option<u16> {
        match self {
            ClientError::HttpError(e) => e.status().map(|s| s.as_u16()),
            ClientError::ServerError { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl From<agent_mem_traits::AgentMemError> for ClientError {
    fn from(err: agent_mem_traits::AgentMemError) -> Self {
        ClientError::InternalError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        let timeout_error = ClientError::TimeoutError("Request timeout".to_string());
        assert!(timeout_error.is_retryable());

        let server_error = ClientError::ServerError {
            status: 500,
            message: "Internal server error".to_string(),
        };
        assert!(server_error.is_retryable());

        let client_error = ClientError::ServerError {
            status: 400,
            message: "Bad request".to_string(),
        };
        assert!(!client_error.is_retryable());

        let auth_error = ClientError::AuthError("Invalid token".to_string());
        assert!(!auth_error.is_retryable());
    }

    #[test]
    fn test_status_code() {
        let server_error = ClientError::ServerError {
            status: 404,
            message: "Not found".to_string(),
        };
        assert_eq!(server_error.status_code(), Some(404));

        let config_error = ClientError::ConfigError("Invalid config".to_string());
        assert_eq!(config_error.status_code(), None);
    }
}
