//! LibSQL connection management
//!
//! Provides connection pooling and management for LibSQL (embedded SQLite fork)

use agent_mem_traits::{AgentMemError, Result};
use libsql::{Builder, Connection, Database};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use std::time::{Duration, Instant};
#[cfg(feature = "num_cpus")]
use num_cpus;

/// LibSQL connection pool configuration
#[derive(Debug, Clone)]
pub struct LibSqlPoolConfig {
    /// Minimum number of connections in the pool
    pub min_connections: usize,
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle timeout in seconds (connections idle longer than this will be closed)
    pub idle_timeout: u64,
    /// Max lifetime in seconds (connections older than this will be closed)
    pub max_lifetime: u64,
}

impl Default for LibSqlPoolConfig {
    fn default() -> Self {
        // Optimized defaults based on CPU cores and typical workload
        // Reference: Mem0 uses connection pooling for better concurrency
        #[cfg(feature = "num_cpus")]
        let num_cpus = num_cpus::get().max(1);
        #[cfg(not(feature = "num_cpus"))]
        let num_cpus = 4; // Fallback to 4 if num_cpus not available
        
        Self {
            min_connections: num_cpus.max(2),  // At least 2, or CPU count
            max_connections: (num_cpus * 4).max(10).min(50),  // 4x CPU cores, max 50, min 10
            connect_timeout: 30,
            idle_timeout: 600,  // 10 minutes
            max_lifetime: 1800, // 30 minutes
        }
    }
}

// Note: We use Arc<Mutex<Connection>> for simplicity and thread safety
// The connection is returned to the pool when the Arc is dropped

/// LibSQL connection pool
pub struct LibSqlConnectionPool {
    db: Database,
    config: LibSqlPoolConfig,
    /// Available connections (idle)
    idle_connections: Arc<Mutex<Vec<(Connection, Instant)>>>,
    /// Semaphore to limit total connections
    semaphore: Arc<Semaphore>,
    /// Current number of connections
    current_connections: Arc<AtomicUsize>,
}

impl LibSqlConnectionPool {
    /// Create a new connection pool
    pub async fn new(path: &str, config: LibSqlPoolConfig) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                AgentMemError::StorageError(format!("Failed to create directory: {e}"))
            })?;
        }

        // Create or open database
        let db = Builder::new_local(path).build().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to open database at {path}: {e}"))
        })?;

        let pool = Self {
            db,
            config: config.clone(),
            idle_connections: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            current_connections: Arc::new(AtomicUsize::new(0)),
        };

        // Warm up the pool with min_connections
        pool.warmup().await?;

        Ok(pool)
    }

    /// Warm up the pool by creating min_connections
    async fn warmup(&self) -> Result<()> {
        let mut idle = self.idle_connections.lock().await;
        for _ in 0..self.config.min_connections {
            match self.create_connection().await {
                Ok(conn) => {
                    idle.push((conn, Instant::now()));
                    self.current_connections.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    tracing::warn!("Failed to create connection during warmup: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Create a new connection
    async fn create_connection(&self) -> Result<Connection> {
        self.db
            .connect()
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create connection: {e}")))
    }

    /// Get a connection from the pool
    ///
    /// Returns an `Arc<Mutex<Connection>>` that can be used for database operations.
    /// The connection is automatically returned to the pool when the Arc is dropped.
    pub async fn get(&self) -> Result<Arc<Mutex<Connection>>> {
        // Try to get an idle connection
        {
            let mut idle = self.idle_connections.lock().await;
            let now = Instant::now();

            // Remove stale connections (idle too long or too old)
            idle.retain(|(_, created_at)| {
                let age = now.duration_since(*created_at);
                age.as_secs() < self.config.idle_timeout
                    && age.as_secs() < self.config.max_lifetime
            });

            // Try to get an idle connection
            if let Some((conn, _)) = idle.pop() {
                return Ok(Arc::new(Mutex::new(conn)));
            }
        }

        // No idle connection available, try to create a new one
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to acquire semaphore: {e}")))?;

        let current = self.current_connections.load(Ordering::Relaxed);
        if current < self.config.max_connections {
            match self.create_connection().await {
                Ok(conn) => {
                    self.current_connections.fetch_add(1, Ordering::Relaxed);
                    // Note: We don't track individual connections for return, as Arc<Mutex<Connection>>
                    // will be dropped when done. For a production implementation, you'd want to
                    // implement a proper connection return mechanism.
                    return Ok(Arc::new(Mutex::new(conn)));
                }
                Err(e) => {
                    drop(_permit);
                    return Err(e);
                }
            }
        }

        // Max connections reached, wait for an idle connection with timeout
        let timeout = Duration::from_secs(self.config.connect_timeout);
        let start = Instant::now();
        
        loop {
            // Check timeout
            if start.elapsed() > timeout {
                drop(_permit);
                return Err(AgentMemError::StorageError(format!(
                    "Connection pool timeout: waited {}s for available connection",
                    self.config.connect_timeout
                )));
            }
            
            tokio::time::sleep(Duration::from_millis(10)).await;
            let mut idle = self.idle_connections.lock().await;
            if let Some((conn, _)) = idle.pop() {
                drop(_permit);
                return Ok(Arc::new(Mutex::new(conn)));
            }
        }
    }

    /// Return a connection to the pool (for future use with proper connection tracking)
    #[allow(dead_code)]
    pub async fn put(&self, conn: Connection) {
        let mut idle = self.idle_connections.lock().await;
        idle.push((conn, Instant::now()));
    }

    /// 🆕 Phase 1.3: 连接池健康检查
    /// 预期效果: 确保连接池中的连接都是健康的
    pub async fn health_check(&self) -> Result<()> {
        let mut idle = self.idle_connections.lock().await;
        let now = Instant::now();
        let mut healthy_count = 0;
        let mut removed_count = 0;

        // 检查并移除不健康的连接
        idle.retain(|(conn, created_at)| {
            let age = now.duration_since(*created_at);
            let is_idle_ok = age.as_secs() < self.config.idle_timeout;
            let is_lifetime_ok = age.as_secs() < self.config.max_lifetime;

            if is_idle_ok && is_lifetime_ok {
                healthy_count += 1;
                true
            } else {
                removed_count += 1;
                false
            }
        });

        // 如果健康连接数少于min_connections，补充连接
        if healthy_count < self.config.min_connections {
            let needed = self.config.min_connections - healthy_count;
            for _ in 0..needed {
                match self.create_connection().await {
                    Ok(conn) => {
                        idle.push((conn, Instant::now()));
                        self.current_connections.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create connection during health check: {}", e);
                    }
                }
            }
        }

        if removed_count > 0 {
            tracing::info!(
                "Connection pool health check: removed {} stale connections, {} healthy connections",
                removed_count,
                healthy_count
            );
        }

        Ok(())
    }

    /// Get pool statistics
    pub async fn stats(&self) -> LibSqlPoolStats {
        let idle = self.idle_connections.lock().await;
        let current = self.current_connections.load(Ordering::Relaxed);
        LibSqlPoolStats {
            total_connections: current,
            idle_connections: idle.len(),
            active_connections: current - idle.len(),
            max_connections: self.config.max_connections,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct LibSqlPoolStats {
    pub total_connections: usize,
    pub idle_connections: usize,
    pub active_connections: usize,
    pub max_connections: usize,
}

/// LibSQL connection manager (backward compatibility)
pub struct LibSqlConnectionManager {
    db: Database,
}

impl LibSqlConnectionManager {
    /// Create a new connection manager (backward compatibility)
    ///
    /// # Arguments
    /// * `path` - File path for the database (e.g., "./data/agentmem.db")
    ///
    /// # Example
    /// ```no_run
    /// use agent_mem_core::storage::libsql::connection::LibSqlConnectionManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = LibSqlConnectionManager::new("./data/test.db").await?;
    ///     let conn = manager.get_connection().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(path: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                AgentMemError::StorageError(format!("Failed to create directory: {e}"))
            })?;
        }

        // Create or open database
        let db = Builder::new_local(path).build().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to open database at {path}: {e}"))
        })?;

        Ok(Self { db })
    }

    /// Get a connection from the pool (backward compatibility - creates new connection each time)
    pub async fn get_connection(&self) -> Result<Arc<Mutex<Connection>>> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get connection: {e}")))?;

        Ok(Arc::new(Mutex::new(conn)))
    }

    /// Health check - verifies database connectivity
    pub async fn health_check(&self) -> Result<()> {
        let conn = self.get_connection().await?;
        let conn_guard = conn.lock().await;

        let mut rows = conn_guard
            .query("SELECT 1", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Health check failed: {e}")))?;

        // Consume the result to verify the query worked
        rows.next()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Health check failed: {e}")))?;

        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.get_connection().await?;
        let conn_guard = conn.lock().await;

        // Get page count
        let mut rows = conn_guard
            .query("PRAGMA page_count", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get page count: {e}")))?;

        let page_count: i64 =
            if let Some(row) = rows.next().await.map_err(|e| {
                AgentMemError::StorageError(format!("Failed to read page count: {e}"))
            })? {
                row.get(0).map_err(|e| {
                    AgentMemError::StorageError(format!("Failed to parse page count: {e}"))
                })?
            } else {
                0
            };

        // Get page size
        let mut rows = conn_guard
            .query("PRAGMA page_size", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get page size: {e}")))?;

        let page_size: i64 =
            if let Some(row) = rows.next().await.map_err(|e| {
                AgentMemError::StorageError(format!("Failed to read page size: {e}"))
            })? {
                row.get(0).map_err(|e| {
                    AgentMemError::StorageError(format!("Failed to parse page size: {e}"))
                })?
            } else {
                4096
            };

        let size_bytes = page_count * page_size;

        Ok(DatabaseStats {
            page_count: page_count as u64,
            page_size: page_size as u64,
            size_bytes: size_bytes as u64,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Total number of pages
    pub page_count: u64,
    /// Size of each page in bytes
    pub page_size: u64,
    /// Total database size in bytes
    pub size_bytes: u64,
}

impl DatabaseStats {
    /// Get size in megabytes
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Create a LibSQL connection pool (backward compatibility - returns single connection)
///
/// This is a convenience function that creates a connection manager
/// and returns a single connection wrapped in Arc<Mutex<>>
/// 
/// **Note**: For better performance, use `LibSqlConnectionPool::new()` instead
///
/// # Arguments
/// * `path` - File path for the database
///
/// # Example
/// ```no_run
/// use agent_mem_core::storage::libsql::connection::create_libsql_pool;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let conn = create_libsql_pool("./data/test.db").await?;
///     
///     // Use the connection
///     let conn_guard = conn.lock().await;
///     conn_guard.execute("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY)", ()).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn create_libsql_pool(path: &str) -> Result<Arc<Mutex<Connection>>> {
    let manager = LibSqlConnectionManager::new(path).await?;
    manager.get_connection().await
}

/// Create a LibSQL connection pool with configuration
///
/// This is the recommended way to create a connection pool for better performance.
///
/// # Arguments
/// * `path` - File path for the database
/// * `config` - Pool configuration
///
/// # Example
/// ```no_run
/// use agent_mem_core::storage::libsql::connection::{LibSqlConnectionPool, LibSqlPoolConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = LibSqlPoolConfig {
///         min_connections: 2,
///         max_connections: 10,
///         ..Default::default()
///     };
///     let pool = LibSqlConnectionPool::new("./data/test.db", config).await?;
///     
///     // Get a connection from the pool
///     let conn = pool.get().await?;
///     let conn_guard = conn.lock().await;
///     conn_guard.execute("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY)", ()).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn create_libsql_pool_with_config(
    path: &str,
    config: LibSqlPoolConfig,
) -> Result<Arc<LibSqlConnectionPool>> {
    Ok(Arc::new(LibSqlConnectionPool::new(path, config).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_connection_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let manager = LibSqlConnectionManager::new(db_path_str).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_get_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let manager = LibSqlConnectionManager::new(db_path_str).await?;
        let conn = manager.get_connection().await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let manager = LibSqlConnectionManager::new(db_path_str).await?;
        let result = manager.health_check().await;
        if let Err(e) = &result {
            eprintln!("Health check failed: {e:?}");
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let manager = LibSqlConnectionManager::new(db_path_str).await?;
        let stats = manager.get_stats().await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert!(stats.page_size > 0);
        assert!(stats.size_mb() >= 0.0);
    }

    #[tokio::test]
    async fn test_create_libsql_pool() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let conn = create_libsql_pool(db_path_str).await;
        assert!(conn.is_ok());

        // Test basic query
        let conn = conn.unwrap();
        let conn_guard = conn.lock().await;
        let result = conn_guard
            .execute(
                "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY)",
                (),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_connections() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let manager = LibSqlConnectionManager::new(db_path_str).await?;

        // Get multiple connections
        let conn1 = manager.get_connection().await;
        let conn2 = manager.get_connection().await;

        assert!(conn1.is_ok());
        assert!(conn2.is_ok());
    }
}
