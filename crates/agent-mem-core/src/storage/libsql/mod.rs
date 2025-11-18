//! LibSQL storage backend implementation
//!
//! Provides embedded database support using LibSQL (SQLite fork)

pub mod agent_repository;
pub mod api_key_repository;
pub mod association_repository;
pub mod block_repository;
pub mod connection;
pub mod learning_repository;
pub mod memory_repository;
pub mod message_repository;
pub mod migrations;
pub mod operations_adapter;
pub mod organization_repository;
pub mod tool_repository;
pub mod user_repository;

// Re-export commonly used types
pub use agent_repository::LibSqlAgentRepository;
pub use api_key_repository::LibSqlApiKeyRepository;
pub use association_repository::LibSqlAssociationRepository;
pub use block_repository::LibSqlBlockRepository;
pub use connection::{create_libsql_pool, DatabaseStats, LibSqlConnectionManager};
pub use learning_repository::{LearningRepositoryTrait, LibSqlLearningRepository};
pub use memory_repository::LibSqlMemoryRepository;
pub use message_repository::LibSqlMessageRepository;
pub use migrations::run_migrations;
pub use operations_adapter::LibSqlMemoryOperations;
pub use organization_repository::LibSqlOrganizationRepository;
pub use tool_repository::LibSqlToolRepository;
pub use user_repository::LibSqlUserRepository;
