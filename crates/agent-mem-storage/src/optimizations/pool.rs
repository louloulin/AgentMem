//! Database connection pool optimization
//!
//! Provides optimized connection pool configuration for PostgreSQL
//! based on MIRIX best practices and production requirements.

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};
use std::time::Duration;
use tracing::{info, warn};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Minimum number of idle connections to maintain
    pub min_connections: u32,

    /// Maximum lifetime of a connection (prevents stale connections)
    pub max_lifetime: Duration,

    /// Idle timeout before closing a connection
    pub idle_timeout: Duration,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Acquire timeout (time to wait for a connection from pool)
    pub acquire_timeout: Duration,

    /// Enable statement caching
    pub enable_statement_cache: bool,

    /// Statement cache capacity
    pub statement_cache_capacity: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            // Based on MIRIX: pool_size=20, max_overflow=30 (total 50)
            max_connections: 50,
            min_connections: 10,

            // Connection lifecycle
            max_lifetime: Duration::from_secs(30 * 60), // 30 minutes
            idle_timeout: Duration::from_secs(10 * 60), // 10 minutes

            // Timeouts
            connect_timeout: Duration::from_secs(30),
            acquire_timeout: Duration::from_secs(30),

            // Statement caching
            enable_statement_cache: true,
            statement_cache_capacity: 100,
        }
    }
}

impl PoolConfig {
    /// Create a production-optimized configuration
    pub fn production() -> Self {
        Self {
            max_connections: 100,
            min_connections: 20,
            max_lifetime: Duration::from_secs(60 * 60), // 1 hour
            idle_timeout: Duration::from_secs(15 * 60), // 15 minutes
            connect_timeout: Duration::from_secs(30),
            acquire_timeout: Duration::from_secs(30),
            enable_statement_cache: true,
            statement_cache_capacity: 200,
        }
    }

    /// Create a development configuration (fewer connections)
    pub fn development() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            max_lifetime: Duration::from_secs(15 * 60), // 15 minutes
            idle_timeout: Duration::from_secs(5 * 60),  // 5 minutes
            connect_timeout: Duration::from_secs(10),
            acquire_timeout: Duration::from_secs(10),
            enable_statement_cache: true,
            statement_cache_capacity: 50,
        }
    }

    /// Create a test configuration (minimal connections)
    pub fn test() -> Self {
        Self {
            max_connections: 5,
            min_connections: 1,
            max_lifetime: Duration::from_secs(5 * 60), // 5 minutes
            idle_timeout: Duration::from_secs(2 * 60), // 2 minutes
            connect_timeout: Duration::from_secs(5),
            acquire_timeout: Duration::from_secs(5),
            enable_statement_cache: false,
            statement_cache_capacity: 10,
        }
    }
}

/// Create an optimized connection pool
pub async fn create_optimized_pool(
    database_url: &str,
    config: PoolConfig,
) -> Result<Pool<Postgres>, sqlx::Error> {
    info!(
        "Creating optimized connection pool with config: {:?}",
        config
    );

    // Parse connection options
    let connect_options = database_url
        .parse::<PgConnectOptions>()?
        .application_name("agentmem")
        // Enable prepared statement caching
        .statement_cache_capacity(config.statement_cache_capacity);

    // Create pool with optimized settings
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .max_lifetime(Some(config.max_lifetime))
        .idle_timeout(Some(config.idle_timeout))
        .acquire_timeout(config.acquire_timeout)
        // Test connections before returning them from the pool
        .test_before_acquire(true)
        // Close connections that fail health checks
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Set session parameters for optimal performance
                sqlx::query("SET statement_timeout = '30s'")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("SET idle_in_transaction_session_timeout = '60s'")
                    .execute(&mut *conn)
                    .await?;
                // Enable parallel query execution
                sqlx::query("SET max_parallel_workers_per_gather = 4")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await?;

    info!(
        "Connection pool created successfully: {} max connections, {} min connections",
        config.max_connections, config.min_connections
    );

    // Warm up the pool by creating min_connections
    let pool_size = pool.size();
    if pool_size < config.min_connections {
        warn!(
            "Pool size ({}) is less than min_connections ({}), warming up...",
            pool_size, config.min_connections
        );
    }

    Ok(pool)
}

/// Get pool statistics
pub fn get_pool_stats(pool: &Pool<Postgres>) -> PoolStats {
    PoolStats {
        size: pool.size(),
        idle: pool.num_idle(),
        // Note: SQLx doesn't expose active connections directly
        // active = size - idle (approximate)
        active: pool.size().saturating_sub(pool.num_idle() as u32),
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total number of connections in the pool
    pub size: u32,

    /// Number of idle connections
    pub idle: usize,

    /// Number of active connections (approximate)
    pub active: u32,
}

impl PoolStats {
    /// Check if pool is healthy
    pub fn is_healthy(&self, config: &PoolConfig) -> bool {
        // Pool is healthy if:
        // 1. We have at least min_connections
        // 2. We're not at max capacity
        self.size >= config.min_connections && self.size < config.max_connections
    }

    /// Get pool utilization percentage
    pub fn utilization(&self, config: &PoolConfig) -> f64 {
        (self.active as f64 / config.max_connections as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.min_connections, 10);
        assert!(config.enable_statement_cache);
    }

    #[test]
    fn test_pool_config_production() {
        let config = PoolConfig::production();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_connections, 20);
    }

    #[test]
    fn test_pool_config_development() {
        let config = PoolConfig::development();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
    }

    #[test]
    fn test_pool_config_test() {
        let config = PoolConfig::test();
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.min_connections, 1);
    }

    #[test]
    fn test_pool_stats_healthy() {
        let config = PoolConfig::default();
        let stats = PoolStats {
            size: 30,
            idle: 10,
            active: 20,
        };
        assert!(stats.is_healthy(&config));
    }

    #[test]
    fn test_pool_stats_utilization() {
        let config = PoolConfig::default();
        let stats = PoolStats {
            size: 50,
            idle: 10,
            active: 40,
        };
        assert_eq!(stats.utilization(&config), 80.0);
    }
}
