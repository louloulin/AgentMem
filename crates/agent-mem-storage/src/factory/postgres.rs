//! PostgreSQL Storage Factory
//!
//! Factory for creating PostgreSQL-backed memory stores.

use super::StorageFactory;
use crate::backends::{
    PostgresCoreStore, PostgresEpisodicStore, PostgresProceduralStore, PostgresSemanticStore,
    PostgresWorkingStore,
};
use agent_mem_traits::{
    AgentMemError, CoreMemoryStore, EpisodicMemoryStore, ProceduralMemoryStore, Result,
    SemanticMemoryStore, WorkingMemoryStore,
};
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

/// PostgreSQL storage factory
pub struct PostgresStorageFactory {
    pool: Arc<PgPool>,
}

impl PostgresStorageFactory {
    /// Create a new PostgreSQL storage factory
    ///
    /// # Arguments
    ///
    /// * `connection_string` - PostgreSQL connection string
    ///
    /// # Example
    ///
    /// ```no_run
    /// use agent_mem_storage::factory::postgres::PostgresStorageFactory;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let factory = PostgresStorageFactory::new(
    ///     "postgresql://user:pass@localhost/agentmem"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(connection_string)
            .await
            .map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to connect to PostgreSQL: {}", e))
            })?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Create with existing pool
    pub fn with_pool(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &Arc<PgPool> {
        &self.pool
    }
}

#[async_trait]
impl StorageFactory for PostgresStorageFactory {
    async fn create_episodic_store(&self) -> Result<Arc<dyn EpisodicMemoryStore>> {
        Ok(Arc::new(PostgresEpisodicStore::new(self.pool.clone())))
    }

    async fn create_semantic_store(&self) -> Result<Arc<dyn SemanticMemoryStore>> {
        Ok(Arc::new(PostgresSemanticStore::new(self.pool.clone())))
    }

    async fn create_procedural_store(&self) -> Result<Arc<dyn ProceduralMemoryStore>> {
        Ok(Arc::new(PostgresProceduralStore::new(self.pool.clone())))
    }

    async fn create_core_store(&self) -> Result<Arc<dyn CoreMemoryStore>> {
        Ok(Arc::new(PostgresCoreStore::new(self.pool.clone())))
    }

    async fn create_working_store(&self) -> Result<Arc<dyn WorkingMemoryStore>> {
        Ok(Arc::new(PostgresWorkingStore::new(self.pool.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL connection
    async fn test_postgres_factory_creation() {
        let connection_string = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agentmem_test".to_string());

        let factory = PostgresStorageFactory::new(&connection_string).await;
        assert!(factory.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL connection
    async fn test_create_all_stores() {
        let connection_string = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agentmem_test".to_string());

        let factory = PostgresStorageFactory::new(&connection_string)
            .await
            .unwrap();

        let stores = factory.create_all_stores().await;
        assert!(stores.is_ok());

        let stores = stores.unwrap();
        assert!(Arc::strong_count(&stores.episodic) >= 1);
        assert!(Arc::strong_count(&stores.semantic) >= 1);
        assert!(Arc::strong_count(&stores.procedural) >= 1);
        assert!(Arc::strong_count(&stores.core) >= 1);
        assert!(Arc::strong_count(&stores.working) >= 1);
    }
}

