//! AgentMem Category Hierarchy System
//!
//! This crate provides a hierarchical category system for organizing memory items
//! in a file-system-like structure. Categories are organized in a tree with paths
//! like "/preferences/communication/style".
//!
//! # Features
//!
//! - Hierarchical category organization with parent-child relationships
//! - Path-based navigation and browsing
//! - LLM-driven category summaries
//! - Semantic search with embeddings
//! - Multi-tenancy support (user_id + optional agent_id)
//! - In-memory and persistent storage backends
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_category::{InMemoryCategoryManager, CategoryManager, CategoryScope};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut manager = InMemoryCategoryManager::new();
//! let scope = CategoryScope::new("user-123".to_string());
//!
//! // Create a category (automatically creates parents)
//! let category = manager.create_category("/preferences/communication/style", scope.clone()).await?;
//!
//! // Navigate to a category
//! let category = manager.navigate_path("/preferences/communication", &scope).await?;
//!
//! // Browse children
//! let children = manager.browse_path("/preferences/communication", &scope).await?;
//!
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod manager;
pub mod models;

// Re-exports for convenience
pub use error::{CategoryError, Result};
pub use manager::{CategoryManager, InMemoryCategoryManager};
pub use models::{
    Category, CategoryId, CategoryMetadata, CategoryPath, CategoryScope, CategoryStatus,
    CategoryTreeNode,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const LIB_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(LIB_NAME, "agent-mem-category");
    }
}
