//! Server configuration

use serde::{Deserialize, Serialize};
use std::env;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,
    /// Server host
    pub host: String,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable authentication
    pub enable_auth: bool,
    /// JWT secret key
    pub jwt_secret: String,
    /// Enable metrics
    pub enable_metrics: bool,
    /// Enable OpenAPI documentation
    pub enable_docs: bool,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Max request body size in bytes
    pub max_body_size: usize,
    /// Enable request logging
    pub enable_logging: bool,
    /// Log level
    pub log_level: String,
    /// Multi-tenant mode
    pub multi_tenant: bool,
    /// Rate limiting
    pub rate_limit_requests_per_minute: u32,
    /// Database URL
    pub database_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: env::var("AGENT_MEM_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            host: env::var("AGENT_MEM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            enable_cors: env::var("AGENT_MEM_ENABLE_CORS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_auth: env::var("AGENT_MEM_ENABLE_AUTH")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            jwt_secret: env::var("AGENT_MEM_JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-change-in-production".to_string()),
            enable_metrics: env::var("AGENT_MEM_ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_docs: env::var("AGENT_MEM_ENABLE_DOCS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            request_timeout: env::var("AGENT_MEM_REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_body_size: env::var("AGENT_MEM_MAX_BODY_SIZE")
                .unwrap_or_else(|_| "1048576".to_string()) // 1MB
                .parse()
                .unwrap_or(1048576),
            enable_logging: env::var("AGENT_MEM_ENABLE_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            log_level: env::var("AGENT_MEM_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            multi_tenant: env::var("AGENT_MEM_MULTI_TENANT")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            rate_limit_requests_per_minute: env::var("AGENT_MEM_RATE_LIMIT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://agentmem:password@localhost:5432/agentmem".to_string()
            }),
        }
    }
}

impl ServerConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Load configuration from TOML file
    ///
    /// Phase 10 (MVP P0-1): 配置文件加载系统
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }

    /// Load configuration with precedence: File < Env < CLI
    ///
    /// 1. Load from file (if specified)
    /// 2. Override with environment variables
    /// 3. Override with CLI arguments (done in main.rs)
    pub fn load(config_file: Option<impl AsRef<std::path::Path>>) -> Result<Self, String> {
        let mut config = if let Some(path) = config_file {
            // Load from file
            Self::from_file(path)?
        } else {
            // Use default with env vars
            Self::default()
        };
        
        // Override with environment variables (explicit, higher priority than file)
        config = config.override_from_env();
        
        Ok(config)
    }

    /// Override configuration from environment variables
    ///
    /// This is called after loading from file to allow env vars to override file values
    fn override_from_env(mut self) -> Self {
        if let Ok(port) = env::var("AGENT_MEM_PORT") {
            if let Ok(p) = port.parse() {
                self.port = p;
            }
        }
        if let Ok(host) = env::var("AGENT_MEM_HOST") {
            self.host = host;
        }
        if let Ok(cors) = env::var("AGENT_MEM_ENABLE_CORS") {
            if let Ok(c) = cors.parse() {
                self.enable_cors = c;
            }
        }
        if let Ok(auth) = env::var("AGENT_MEM_ENABLE_AUTH") {
            if let Ok(a) = auth.parse() {
                self.enable_auth = a;
            }
        }
        if let Ok(secret) = env::var("AGENT_MEM_JWT_SECRET") {
            self.jwt_secret = secret;
        }
        if let Ok(level) = env::var("AGENT_MEM_LOG_LEVEL") {
            self.log_level = level;
        }
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }

        if self.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters".to_string());
        }

        if self.request_timeout == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }

        if self.max_body_size == 0 {
            return Err("Max body size must be greater than 0".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.enable_cors);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ServerConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());
    }
}
