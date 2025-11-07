//! Database configuration for AgentMem
//!
//! Supports multiple database backends with simple configuration switching

use serde::{Deserialize, Serialize};
use std::env;

/// Database backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    /// LibSQL (embedded SQLite fork) - default
    #[serde(rename = "libsql")]
    LibSql,

    /// PostgreSQL (enterprise-grade)
    #[serde(rename = "postgres")]
    Postgres,
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        Self::LibSql
    }
}

impl std::fmt::Display for DatabaseBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibSql => write!(f, "libsql"),
            Self::Postgres => write!(f, "postgres"),
        }
    }
}

impl std::str::FromStr for DatabaseBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "libsql" | "sqlite" => Ok(Self::LibSql),
            "postgres" | "postgresql" | "pg" => Ok(Self::Postgres),
            _ => Err(format!("Unknown database backend: {s}")),
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of connections in the pool
    pub min_connections: u32,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Connection acquire timeout in seconds
    pub acquire_timeout_seconds: u64,

    /// Idle connection timeout in seconds
    pub idle_timeout_seconds: u64,

    /// Maximum connection lifetime in seconds
    pub max_lifetime_seconds: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 1800,
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database backend type
    pub backend: DatabaseBackend,

    /// Connection URL
    /// - For LibSQL: file path (e.g., "./data/agentmem.db")
    /// - For PostgreSQL: connection string (e.g., "postgresql://localhost/agentmem")
    pub url: String,

    /// Connection pool configuration
    pub pool: PoolConfig,

    /// Whether to automatically run migrations on startup
    pub auto_migrate: bool,

    /// Enable query logging
    pub log_queries: bool,

    /// Slow query threshold in milliseconds
    pub slow_query_threshold_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: DatabaseBackend::LibSql,
            url: "./data/agentmem.db".to_string(),
            pool: PoolConfig::default(),
            auto_migrate: true,
            log_queries: false,
            slow_query_threshold_ms: 1000,
        }
    }
}

impl DatabaseConfig {
    /// Create a new database configuration
    pub fn new(backend: DatabaseBackend, url: String) -> Self {
        Self {
            backend,
            url,
            ..Default::default()
        }
    }

    /// Create LibSQL configuration
    pub fn libsql(path: &str) -> Self {
        Self {
            backend: DatabaseBackend::LibSql,
            url: path.to_string(),
            ..Default::default()
        }
    }

    /// Create PostgreSQL configuration
    pub fn postgres(url: &str) -> Self {
        Self {
            backend: DatabaseBackend::Postgres,
            url: url.to_string(),
            ..Default::default()
        }
    }

    /// Load configuration from environment variables
    ///
    /// Environment variables:
    /// - `DATABASE_BACKEND`: "libsql" or "postgres" (default: "libsql")
    /// - `DATABASE_URL`: connection URL (default: "./data/agentmem.db" for LibSQL)
    /// - `DATABASE_AUTO_MIGRATE`: "true" or "false" (default: "true")
    /// - `DATABASE_LOG_QUERIES`: "true" or "false" (default: "false")
    /// - `DATABASE_MAX_CONNECTIONS`: maximum pool size (default: 10)
    pub fn from_env() -> Self {
        let backend_str = env::var("DATABASE_BACKEND").unwrap_or_else(|_| "libsql".to_string());

        let backend = backend_str
            .parse::<DatabaseBackend>()
            .unwrap_or(DatabaseBackend::LibSql);

        let default_url = match backend {
            DatabaseBackend::LibSql => "./data/agentmem.db".to_string(),
            DatabaseBackend::Postgres => "postgresql://localhost/agentmem".to_string(),
        };

        let url = env::var("DATABASE_URL").unwrap_or(default_url);

        let auto_migrate = env::var("DATABASE_AUTO_MIGRATE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let log_queries = env::var("DATABASE_LOG_QUERIES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let slow_query_threshold_ms = env::var("DATABASE_SLOW_QUERY_THRESHOLD_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        Self {
            backend,
            url,
            pool: PoolConfig {
                max_connections,
                ..Default::default()
            },
            auto_migrate,
            log_queries,
            slow_query_threshold_ms,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate URL
        if self.url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        // Validate pool config
        if self.pool.min_connections > self.pool.max_connections {
            return Err("min_connections cannot be greater than max_connections".to_string());
        }

        if self.pool.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }

        // Backend-specific validation
        match self.backend {
            DatabaseBackend::LibSql => {
                // LibSQL uses file paths
                if self.url.starts_with("postgresql://") || self.url.starts_with("postgres://") {
                    return Err(
                        "LibSQL backend requires a file path, not a PostgreSQL URL".to_string()
                    );
                }
            }
            DatabaseBackend::Postgres => {
                // PostgreSQL uses connection strings
                if !self.url.starts_with("postgresql://") && !self.url.starts_with("postgres://") {
                    return Err("PostgreSQL backend requires a connection URL starting with postgresql:// or postgres://".to_string());
                }
            }
        }

        Ok(())
    }

    /// Get a display-safe version of the URL (hides passwords)
    pub fn safe_url(&self) -> String {
        if self.url.contains('@') {
            // PostgreSQL URL with credentials
            if let Some(at_pos) = self.url.rfind('@') {
                if let Some(scheme_end) = self.url.find("://") {
                    let scheme = &self.url[..scheme_end + 3];
                    let host_part = &self.url[at_pos..];
                    return format!("{scheme}***{host_part}");
                }
            }
        }
        self.url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_backend_default() {
        assert_eq!(DatabaseBackend::default(), DatabaseBackend::LibSql);
    }

    #[test]
    fn test_database_backend_from_str() {
        assert_eq!(
            "libsql".parse::<DatabaseBackend>().unwrap(),
            DatabaseBackend::LibSql
        );
        assert_eq!(
            "sqlite".parse::<DatabaseBackend>().unwrap(),
            DatabaseBackend::LibSql
        );
        assert_eq!(
            "postgres".parse::<DatabaseBackend>().unwrap(),
            DatabaseBackend::Postgres
        );
        assert_eq!(
            "postgresql".parse::<DatabaseBackend>().unwrap(),
            DatabaseBackend::Postgres
        );
        assert_eq!(
            "pg".parse::<DatabaseBackend>().unwrap(),
            DatabaseBackend::Postgres
        );
        assert!("unknown".parse::<DatabaseBackend>().is_err());
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.backend, DatabaseBackend::LibSql);
        assert_eq!(config.url, "./data/agentmem.db");
        assert!(config.auto_migrate);
    }

    #[test]
    fn test_database_config_libsql() {
        let config = DatabaseConfig::libsql("./test.db");
        assert_eq!(config.backend, DatabaseBackend::LibSql);
        assert_eq!(config.url, "./test.db");
    }

    #[test]
    fn test_database_config_postgres() {
        let config = DatabaseConfig::postgres("postgresql://localhost/test");
        assert_eq!(config.backend, DatabaseBackend::Postgres);
        assert_eq!(config.url, "postgresql://localhost/test");
    }

    #[test]
    fn test_database_config_validate() {
        let config = DatabaseConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = DatabaseConfig::default();
        invalid_config.url = String::new();
        assert!(invalid_config.validate().is_err());

        let mut invalid_pool = DatabaseConfig::default();
        invalid_pool.pool.min_connections = 20;
        invalid_pool.pool.max_connections = 10;
        assert!(invalid_pool.validate().is_err());
    }

    #[test]
    fn test_safe_url() {
        let config = DatabaseConfig::postgres("postgresql://user:password@localhost/db");
        assert_eq!(config.safe_url(), "postgresql://***@localhost/db");

        let config = DatabaseConfig::libsql("./data/test.db");
        assert_eq!(config.safe_url(), "./data/test.db");
    }
}
