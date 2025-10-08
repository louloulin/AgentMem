//! LibSQL storage backend implementation
//!
//! Provides embedded database support using LibSQL (SQLite fork)

pub mod connection;
pub mod migrations;
pub mod user_repository;

// Re-export commonly used types
pub use connection::{create_libsql_pool, DatabaseStats, LibSqlConnectionManager};
pub use migrations::run_migrations;
pub use user_repository::LibSqlUserRepository;

