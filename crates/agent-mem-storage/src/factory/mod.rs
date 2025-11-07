//! Storage Factory Module
//!
//! Provides factory patterns for creating storage backends.
//! Supports multiple backends (PostgreSQL, LibSQL, etc.) with unified interface.

use agent_mem_traits::{
    CoreMemoryStore, EpisodicMemoryStore, ProceduralMemoryStore, Result, SemanticMemoryStore,
    WorkingMemoryStore,
};
use async_trait::async_trait;
use std::sync::Arc;

pub mod libsql;
#[cfg(feature = "postgres")]
pub mod postgres;

/// Storage backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    /// PostgreSQL backend
    PostgreSQL,
    /// LibSQL backend
    LibSQL,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Backend type
    pub backend: StorageBackend,
    /// Connection string or path
    pub connection: String,
    /// Optional organization ID
    pub organization_id: Option<String>,
}

impl StorageConfig {
    /// Create a new storage configuration
    pub fn new(backend: StorageBackend, connection: String) -> Self {
        Self {
            backend,
            connection,
            organization_id: None,
        }
    }

    /// Set organization ID
    pub fn with_organization(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }
}

/// Storage factory trait
///
/// Provides methods to create all types of memory stores.
#[async_trait]
pub trait StorageFactory: Send + Sync {
    /// Create episodic memory store
    async fn create_episodic_store(&self) -> Result<Arc<dyn EpisodicMemoryStore>>;

    /// Create semantic memory store
    async fn create_semantic_store(&self) -> Result<Arc<dyn SemanticMemoryStore>>;

    /// Create procedural memory store
    async fn create_procedural_store(&self) -> Result<Arc<dyn ProceduralMemoryStore>>;

    /// Create core memory store
    async fn create_core_store(&self) -> Result<Arc<dyn CoreMemoryStore>>;

    /// Create working memory store
    async fn create_working_store(&self) -> Result<Arc<dyn WorkingMemoryStore>>;

    /// Create all stores at once
    async fn create_all_stores(&self) -> Result<AllStores> {
        Ok(AllStores {
            episodic: self.create_episodic_store().await?,
            semantic: self.create_semantic_store().await?,
            procedural: self.create_procedural_store().await?,
            core: self.create_core_store().await?,
            working: self.create_working_store().await?,
        })
    }
}

/// Container for all memory stores
pub struct AllStores {
    pub episodic: Arc<dyn EpisodicMemoryStore>,
    pub semantic: Arc<dyn SemanticMemoryStore>,
    pub procedural: Arc<dyn ProceduralMemoryStore>,
    pub core: Arc<dyn CoreMemoryStore>,
    pub working: Arc<dyn WorkingMemoryStore>,
}

/// Create a storage factory from configuration
pub async fn create_factory(config: StorageConfig) -> Result<Box<dyn StorageFactory>> {
    match config.backend {
        #[cfg(feature = "postgres")]
        StorageBackend::PostgreSQL => {
            let factory = postgres::PostgresStorageFactory::new(&config.connection).await?;
            Ok(Box::new(factory))
        }
        #[cfg(not(feature = "postgres"))]
        StorageBackend::PostgreSQL => Err(agent_mem_traits::AgentMemError::storage_error(
            "PostgreSQL backend not enabled. Enable 'postgres' feature.",
        )),
        StorageBackend::LibSQL => {
            let factory = libsql::LibSqlStorageFactory::new(&config.connection).await?;
            Ok(Box::new(factory))
        }
    }
}
