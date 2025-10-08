//! LibSQL storage backend implementation
//!
//! Provides embedded database support using LibSQL (SQLite fork)

pub mod agent_repository;
pub mod connection;
pub mod migrations;
pub mod organization_repository;
pub mod user_repository;

// Re-export commonly used types
pub use agent_repository::LibSqlAgentRepository;
pub use connection::{create_libsql_pool, DatabaseStats, LibSqlConnectionManager};
pub use migrations::run_migrations;
pub use organization_repository::LibSqlOrganizationRepository;
pub use user_repository::LibSqlUserRepository;

