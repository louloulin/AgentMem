//! LibSQL connection management
//!
//! Provides connection pooling and management for LibSQL (embedded SQLite fork)

use agent_mem_traits::{AgentMemError, Result};
use libsql::{Builder, Connection, Database};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL connection manager
pub struct LibSqlConnectionManager {
    db: Database,
}

impl LibSqlConnectionManager {
    /// Create a new connection manager
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

    /// Get a connection from the pool
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

/// Create a LibSQL connection pool (simplified version)
///
/// This is a convenience function that creates a connection manager
/// and returns a single connection wrapped in Arc<Mutex<>>
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

        let manager = LibSqlConnectionManager::new(db_path_str).await.unwrap();
        let conn = manager.get_connection().await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        let manager = LibSqlConnectionManager::new(db_path_str).await.unwrap();
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

        let manager = LibSqlConnectionManager::new(db_path_str).await.unwrap();
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

        let manager = LibSqlConnectionManager::new(db_path_str).await.unwrap();

        // Get multiple connections
        let conn1 = manager.get_connection().await;
        let conn2 = manager.get_connection().await;

        assert!(conn1.is_ok());
        assert!(conn2.is_ok());
    }
}
