//! Category data models for hierarchical organization

mod category;
mod path;
mod tree;

pub use category::{Category, CategoryId, CategoryMetadata};
pub use path::CategoryPath;
pub use tree::CategoryTreeNode;

use serde::{Deserialize, Serialize};

/// Category status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CategoryStatus {
    /// Category is active and can contain items
    Active,
    /// Category is archived (read-only)
    Archived,
    /// Category is deleted (soft delete)
    Deleted,
}

impl Default for CategoryStatus {
    fn default() -> Self {
        CategoryStatus::Active
    }
}

/// Scope for category operations (multi-tenancy support)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryScope {
    pub user_id: String,
    pub agent_id: Option<String>,
}

impl CategoryScope {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            agent_id: None,
        }
    }

    pub fn with_agent(user_id: String, agent_id: String) -> Self {
        Self {
            user_id,
            agent_id: Some(agent_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_status_default() {
        let status = CategoryStatus::default();
        assert_eq!(status, CategoryStatus::Active);
    }

    #[test]
    fn test_category_scope() {
        let scope = CategoryScope::new("user-123".to_string());
        assert_eq!(scope.user_id, "user-123");
        assert_eq!(scope.agent_id, None);

        let scope_with_agent =
            CategoryScope::with_agent("user-123".to_string(), "agent-456".to_string());
        assert_eq!(scope_with_agent.agent_id, Some("agent-456".to_string()));
    }
}
