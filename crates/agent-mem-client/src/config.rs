//! Client configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Base URL of the AgentMem server
    pub base_url: String,

    /// API key for authentication (optional)
    pub api_key: Option<String>,

    /// JWT token for authentication (optional)
    pub jwt_token: Option<String>,

    /// Request timeout
    pub timeout: Duration,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Retry backoff base delay
    pub retry_base_delay: Duration,

    /// Maximum retry delay
    pub retry_max_delay: Duration,

    /// User agent string
    pub user_agent: String,

    /// Enable request/response logging
    pub enable_logging: bool,

    /// Connection pool settings
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout: Duration,
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            jwt_token: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_base_delay: Duration::from_millis(100),
            retry_max_delay: Duration::from_secs(10),
            user_agent: format!("agent-mem-client/{}", env!("CARGO_PKG_VERSION")),
            enable_logging: false,
            pool_max_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(90),
        }
    }

    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set JWT token for authentication
    pub fn with_jwt_token(mut self, token: impl Into<String>) -> Self {
        self.jwt_token = Some(token.into());
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Enable request/response logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate base URL
        if let Err(e) = Url::parse(&self.base_url) {
            return Err(format!("Invalid base URL: {e}"));
        }

        // Validate timeouts
        if self.timeout.is_zero() {
            return Err("Timeout must be greater than zero".to_string());
        }

        if self.connect_timeout.is_zero() {
            return Err("Connect timeout must be greater than zero".to_string());
        }

        // Validate retry settings
        if self.retry_base_delay.is_zero() {
            return Err("Retry base delay must be greater than zero".to_string());
        }

        if self.retry_max_delay < self.retry_base_delay {
            return Err("Retry max delay must be greater than or equal to base delay".to_string());
        }

        Ok(())
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::new("http://localhost:8080")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ClientConfig::new("http://localhost:8080");
        assert_eq!(config.base_url, "http://localhost:8080");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_with_auth() {
        let config = ClientConfig::new("http://localhost:8080")
            .with_api_key("test-key")
            .with_jwt_token("test-token");

        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.jwt_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_config_validation() {
        let mut config = ClientConfig::new("invalid-url");
        assert!(config.validate().is_err());

        config.base_url = "http://localhost:8080".to_string();
        assert!(config.validate().is_ok());

        config.timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());
    }
}
