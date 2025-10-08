//! LibSQL storage backend implementation
//!
//! Provides embedded database support using LibSQL (SQLite fork)

pub mod connection;
pub mod migrations;

// Re-export commonly used types
pub use connection::{create_libsql_pool, DatabaseStats, LibSqlConnectionManager};
pub use migrations::run_migrations;

