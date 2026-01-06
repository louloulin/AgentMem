//! LibSQL Storage Factory
//!
//! Factory for creating LibSQL-backed memory stores.

use super::StorageFactory;
use crate::backends::{
    LibSqlCoreStore, LibSqlEpisodicStore, LibSqlProceduralStore, LibSqlSemanticStore,
    LibSqlWorkingStore,
};
use agent_mem_traits::{
    AgentMemError, CoreMemoryStore, EpisodicMemoryStore, ProceduralMemoryStore, Result,
    SemanticMemoryStore, WorkingMemoryStore,
};
use async_trait::async_trait;
use libsql::{Builder, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL storage factory
pub struct LibSqlStorageFactory {
    connection_string: String,
}

impl LibSqlStorageFactory {
    /// Create a new LibSQL storage factory
    ///
    /// # Arguments
    ///
    /// * `connection_string` - LibSQL connection string or file path
    ///
    /// # Example
    ///
    /// ```no_run
    /// use agent_mem_storage::factory::libsql::LibSqlStorageFactory;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Local file
    /// let factory = LibSqlStorageFactory::new("file:agentmem.db").await?;
    ///
    /// // Remote LibSQL server
    /// let factory = LibSqlStorageFactory::new("libsql://localhost:8080").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(connection_string: &str) -> Result<Self> {
        // Validate connection by creating a test connection
        let _conn = Self::create_connection(connection_string).await?;

        Ok(Self {
            connection_string: connection_string.to_string(),
        })
    }

    /// Create a new connection
    async fn create_connection(connection_string: &str) -> Result<Connection> {
        let db = if connection_string.starts_with("libsql://")
            || connection_string.starts_with("https://")
        {
            // Remote connection
            Builder::new_remote(connection_string.to_string(), String::new())
                .build()
                .await
        } else if connection_string.starts_with("file:") {
            // Local file
            let path = connection_string
                .strip_prefix("file:")
                .unwrap_or(connection_string);
            Builder::new_local(path).build().await
        } else {
            // Assume local file without prefix
            Builder::new_local(connection_string).build().await
        };

        let db = db.map_err(|e| {
            AgentMemError::storage_error(format!("Failed to connect to LibSQL: {e}"))
        })?;

        let conn = db.connect().map_err(|e| {
            AgentMemError::storage_error(format!("Failed to create LibSQL connection: {e}"))
        })?;

        Ok(conn)
    }
}

#[async_trait]
impl StorageFactory for LibSqlStorageFactory {
    async fn create_episodic_store(&self) -> Result<Arc<dyn EpisodicMemoryStore>> {
        let conn = Self::create_connection(&self.connection_string).await?;
        Ok(Arc::new(LibSqlEpisodicStore::new(Arc::new(Mutex::new(
            conn,
        )))))
    }

    async fn create_semantic_store(&self) -> Result<Arc<dyn SemanticMemoryStore>> {
        let conn = Self::create_connection(&self.connection_string).await?;
        Ok(Arc::new(LibSqlSemanticStore::new(Arc::new(Mutex::new(
            conn,
        )))))
    }

    async fn create_procedural_store(&self) -> Result<Arc<dyn ProceduralMemoryStore>> {
        let conn = Self::create_connection(&self.connection_string).await?;
        Ok(Arc::new(LibSqlProceduralStore::new(Arc::new(Mutex::new(
            conn,
        )))))
    }

    async fn create_core_store(&self) -> Result<Arc<dyn CoreMemoryStore>> {
        let conn = Self::create_connection(&self.connection_string).await?;
        Ok(Arc::new(LibSqlCoreStore::new(Arc::new(Mutex::new(conn)))))
    }

    async fn create_working_store(&self) -> Result<Arc<dyn WorkingMemoryStore>> {
        let conn = Self::create_connection(&self.connection_string).await?;
        Ok(Arc::new(LibSqlWorkingStore::new(Arc::new(Mutex::new(
            conn,
        )))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_libsql_factory_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let factory = LibSqlStorageFactory::new(path).await;
        assert!(factory.is_ok());
    }

    #[tokio::test]
    async fn test_create_all_stores() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let factory = LibSqlStorageFactory::new(path).await?;

        let stores = factory.create_all_stores().await;
        assert!(stores.is_ok());

        let stores = stores.unwrap();
        assert!(Arc::strong_count(&stores.episodic) >= 1);
        assert!(Arc::strong_count(&stores.semantic) >= 1);
        assert!(Arc::strong_count(&stores.procedural) >= 1);
        assert!(Arc::strong_count(&stores.core) >= 1);
        assert!(Arc::strong_count(&stores.working) >= 1);
    }

    #[tokio::test]
    async fn test_file_prefix() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = format!("file:{}", temp_file.path().to_str().unwrap());

        let factory = LibSqlStorageFactory::new(&path).await;
        assert!(factory.is_ok());
    }
}
