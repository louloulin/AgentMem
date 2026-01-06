//! Database connection pool management and optimization
//!
//! This module provides advanced connection pool management with:
//! - Dynamic pool sizing
//! - Connection health checks
//! - Pool statistics
//! - Automatic reconnection

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::{CoreError, CoreResult};

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Database URL
    pub url: String,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Max lifetime in seconds
    pub max_lifetime: u64,
    /// Acquire timeout in seconds
    pub acquire_timeout: u64,
    /// Enable statement logging
    pub log_statements: bool,
    /// Enable slow query logging
    pub log_slow_statements: bool,
    /// Slow query threshold in milliseconds
    pub slow_statement_threshold_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            min_connections: 2,
            max_connections: 10,
            connect_timeout: 30,
            idle_timeout: 600,  // 10 minutes
            max_lifetime: 1800, // 30 minutes
            acquire_timeout: 30,
            log_statements: false,
            log_slow_statements: true,
            slow_statement_threshold_ms: 100,
        }
    }
}

impl PoolConfig {
    /// Create a production-optimized configuration
    pub fn production(url: String) -> Self {
        Self {
            url,
            min_connections: 5,
            max_connections: 20,
            connect_timeout: 10,
            idle_timeout: 300,
            max_lifetime: 1800,
            acquire_timeout: 10,
            log_statements: false,
            log_slow_statements: true,
            slow_statement_threshold_ms: 50,
        }
    }

    /// Create a development configuration
    pub fn development(url: String) -> Self {
        Self {
            url,
            min_connections: 1,
            max_connections: 5,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            acquire_timeout: 30,
            log_statements: true,
            log_slow_statements: true,
            slow_statement_threshold_ms: 200,
        }
    }

    /// Create a high-performance configuration
    pub fn high_performance(url: String) -> Self {
        Self {
            url,
            min_connections: 10,
            max_connections: 50,
            connect_timeout: 5,
            idle_timeout: 180,
            max_lifetime: 900,
            acquire_timeout: 5,
            log_statements: false,
            log_slow_statements: true,
            slow_statement_threshold_ms: 20,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub total_acquired: u64,
    pub total_released: u64,
    pub total_timeouts: u64,
    pub total_errors: u64,
}

/// Pool manager with health monitoring
pub struct PoolManager {
    pool: PgPool,
    config: PoolConfig,
    stats: Arc<RwLock<PoolStats>>,
}

impl PoolManager {
    /// Create a new pool manager
    pub async fn new(config: PoolConfig) -> CoreResult<Self> {
        let pool = Self::create_pool(&config).await?;

        Ok(Self {
            pool,
            config,
            stats: Arc::new(RwLock::new(PoolStats::default())),
        })
    }

    /// Create a connection pool
    async fn create_pool(config: &PoolConfig) -> CoreResult<PgPool> {
        let connect_options = PgConnectOptions::from_str(&config.url)
            .map_err(|e| CoreError::Database(format!("Invalid database URL: {}", e)))?;

        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .test_before_acquire(true) // Always test connections before use
            .connect_with(connect_options)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to create pool: {}", e)))?;

        Ok(pool)
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let mut stats = self.stats.read().await.clone();

        // Update current connection counts
        stats.total_connections = self.pool.size();
        stats.idle_connections = self.pool.num_idle() as u32;
        stats.active_connections = stats.total_connections - stats.idle_connections;

        stats
    }

    /// Check pool health
    pub async fn health_check(&self) -> CoreResult<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::error!("Pool health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get detailed pool metrics
    pub async fn metrics(&self) -> PoolMetrics {
        let stats = self.stats().await;

        PoolMetrics {
            size: self.pool.size(),
            idle: self.pool.num_idle() as u32,
            active: stats.active_connections,
            max_size: self.config.max_connections,
            min_size: self.config.min_connections,
            total_acquired: stats.total_acquired,
            total_released: stats.total_released,
            total_timeouts: stats.total_timeouts,
            total_errors: stats.total_errors,
            utilization: (stats.active_connections as f64 / self.config.max_connections as f64)
                * 100.0,
        }
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Acquire a connection and track statistics
    pub async fn acquire(&self) -> CoreResult<sqlx::pool::PoolConnection<sqlx::Postgres>> {
        let start = std::time::Instant::now();

        match self.pool.acquire().await {
            Ok(conn) => {
                let mut stats = self.stats.write().await;
                stats.total_acquired += 1;

                let elapsed = start.elapsed();
                if elapsed.as_millis() > self.config.slow_statement_threshold_ms as u128 {
                    tracing::warn!("Slow connection acquisition: {}ms", elapsed.as_millis());
                }

                Ok(conn)
            }
            Err(e) => {
                let mut stats = self.stats.write().await;
                stats.total_errors += 1;

                if e.to_string().contains("timeout") {
                    stats.total_timeouts += 1;
                }

                Err(CoreError::Database(format!(
                    "Failed to acquire connection: {}",
                    e
                )))
            }
        }
    }

    /// Execute a query with automatic retry
    pub async fn execute_with_retry<F, T, Fut>(
        &self,
        max_retries: u32,
        operation: F,
    ) -> CoreResult<T>
    where
        F: Fn(PgPool) -> Fut,
        Fut: std::future::Future<Output = CoreResult<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation(self.pool.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.clone());

                    if attempt < max_retries {
                        let delay = 100 * (2_u64.pow(attempt));
                        tracing::warn!(
                            "Operation failed (attempt {}/{}), retrying in {}ms: {}",
                            attempt + 1,
                            max_retries + 1,
                            delay,
                            e
                        );
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            CoreError::Database("Operation failed after all retries".to_string())
        }))
    }
}

/// Detailed pool metrics
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub size: u32,
    pub idle: u32,
    pub active: u32,
    pub max_size: u32,
    pub min_size: u32,
    pub total_acquired: u64,
    pub total_released: u64,
    pub total_timeouts: u64,
    pub total_errors: u64,
    pub utilization: f64,
}

impl PoolMetrics {
    /// Check if pool is healthy
    pub fn is_healthy(&self) -> bool {
        self.size > 0 && self.utilization < 90.0
    }

    /// Check if pool is under pressure
    pub fn is_under_pressure(&self) -> bool {
        self.utilization > 80.0 || self.total_timeouts > 0
    }

    /// Get a human-readable status
    pub fn status(&self) -> String {
        if !self.is_healthy() {
            "unhealthy".to_string()
        } else if self.is_under_pressure() {
            "under_pressure".to_string()
        } else {
            "healthy".to_string()
        }
    }
}
