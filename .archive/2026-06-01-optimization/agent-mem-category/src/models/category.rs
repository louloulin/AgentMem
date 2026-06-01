//! Category model representing a folder-like entity in the hierarchy

use super::{CategoryScope, CategoryStatus};
use crate::error::{CategoryError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(String);

impl CategoryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CategoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CategoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Category metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetadata {
    /// Custom key-value pairs
    pub tags: Vec<String>,
    /// Additional user-defined metadata
    pub extra: serde_json::Value,
}

impl Default for CategoryMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            extra: serde_json::json!({}),
        }
    }
}

/// Category representing a folder-like entity in the hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: CategoryId,
    /// Hierarchical path (e.g., "/preferences/communication/style")
    pub path: String,
    /// Display name (e.g., "style")
    pub name: String,
    /// Parent category ID (None for root categories)
    pub parent_id: Option<CategoryId>,
    /// Child category IDs
    pub children_ids: Vec<CategoryId>,
    /// LLM-generated summary
    pub summary: Option<String>,
    /// Category embedding for semantic search
    pub embedding: Option<Vec<f32>>,
    /// Number of memory items in this category
    pub item_count: u64,
    /// Category status
    pub status: CategoryStatus,
    /// Category metadata
    pub metadata: CategoryMetadata,
    /// Scope (user_id and optional agent_id)
    pub scope: CategoryScope,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Category {
    /// Create a new category
    pub fn new(path: String, name: String, scope: CategoryScope) -> Self {
        let now = Utc::now();
        Self {
            id: CategoryId::new(),
            path,
            name,
            parent_id: None,
            children_ids: Vec::new(),
            summary: None,
            embedding: None,
            item_count: 0,
            status: CategoryStatus::Active,
            metadata: CategoryMetadata::default(),
            scope,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a child category
    pub fn add_child(&mut self, child_id: CategoryId) -> Result<()> {
        if self.children_ids.contains(&child_id) {
            return Err(CategoryError::CategoryAlreadyExists(
                "Child already exists".to_string(),
            ));
        }
        self.children_ids.push(child_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove a child category
    pub fn remove_child(&mut self, child_id: &CategoryId) {
        self.children_ids.retain(|id| id != child_id);
        self.updated_at = Utc::now();
    }

    /// Update the category summary
    pub fn update_summary(&mut self, summary: String) {
        self.summary = Some(summary);
        self.updated_at = Utc::now();
    }

    /// Update the category embedding
    pub fn update_embedding(&mut self, embedding: Vec<f32>) -> Result<()> {
        if embedding.is_empty() {
            return Err(CategoryError::InvalidEmbedding(
                "Embedding cannot be empty".to_string(),
            ));
        }
        self.embedding = Some(embedding);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Increment the item count
    pub fn increment_item_count(&mut self) {
        self.item_count += 1;
        self.updated_at = Utc::now();
    }

    /// Decrement the item count
    pub fn decrement_item_count(&mut self) {
        if self.item_count > 0 {
            self.item_count -= 1;
            self.updated_at = Utc::now();
        }
    }

    /// Check if this is a root category (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Check if this is a leaf category (no children)
    pub fn is_leaf(&self) -> bool {
        self.children_ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_id() {
        let id = CategoryId::new();
        assert!(!id.as_str().is_empty());

        let id2 = CategoryId::from_string("custom-id".to_string());
        assert_eq!(id2.as_str(), "custom-id");
    }

    #[test]
    fn test_category_creation() {
        let scope = CategoryScope::new("user-123".to_string());
        let category = Category::new(
            "/preferences/communication".to_string(),
            "communication".to_string(),
            scope,
        );

        assert_eq!(category.path, "/preferences/communication");
        assert_eq!(category.name, "communication");
        assert!(category.parent_id.is_none());
        assert!(category.children_ids.is_empty());
        assert!(category.summary.is_none());
        assert_eq!(category.item_count, 0);
        assert!(category.is_root());
        assert!(category.is_leaf());
    }

    #[test]
    fn test_add_child() {
        let scope = CategoryScope::new("user-123".to_string());
        let mut parent = Category::new(
            "/preferences".to_string(),
            "preferences".to_string(),
            scope.clone(),
        );
        let child_id = CategoryId::new();

        parent.add_child(child_id.clone()).unwrap();
        assert_eq!(parent.children_ids.len(), 1);
        assert_eq!(parent.children_ids[0], child_id);
        assert!(!parent.is_leaf());

        // Test duplicate child
        let result = parent.add_child(child_id.clone());
        assert!(matches!(
            result,
            Err(CategoryError::CategoryAlreadyExists(_))
        ));
    }

    #[test]
    fn test_remove_child() {
        let scope = CategoryScope::new("user-123".to_string());
        let mut parent =
            Category::new("/preferences".to_string(), "preferences".to_string(), scope);
        let child_id = CategoryId::new();

        parent.add_child(child_id.clone()).unwrap();
        assert_eq!(parent.children_ids.len(), 1);

        parent.remove_child(&child_id);
        assert_eq!(parent.children_ids.len(), 0);
        assert!(parent.is_leaf());
    }

    #[test]
    fn test_update_summary() {
        let scope = CategoryScope::new("user-123".to_string());
        let mut category =
            Category::new("/preferences".to_string(), "preferences".to_string(), scope);

        assert!(category.summary.is_none());
        category.update_summary("User preferences and settings".to_string());
        assert_eq!(
            category.summary,
            Some("User preferences and settings".to_string())
        );
    }

    #[test]
    fn test_update_embedding() {
        let scope = CategoryScope::new("user-123".to_string());
        let mut category =
            Category::new("/preferences".to_string(), "preferences".to_string(), scope);

        let embedding = vec![0.1, 0.2, 0.3];
        category.update_embedding(embedding.clone()).unwrap();
        assert_eq!(category.embedding, Some(embedding));

        // Test empty embedding
        let result = category.update_embedding(vec![]);
        assert!(matches!(result, Err(CategoryError::InvalidEmbedding(_))));
    }

    #[test]
    fn test_item_count() {
        let scope = CategoryScope::new("user-123".to_string());
        let mut category =
            Category::new("/preferences".to_string(), "preferences".to_string(), scope);

        assert_eq!(category.item_count, 0);
        category.increment_item_count();
        assert_eq!(category.item_count, 1);
        category.increment_item_count();
        assert_eq!(category.item_count, 2);
        category.decrement_item_count();
        assert_eq!(category.item_count, 1);
        category.decrement_item_count();
        assert_eq!(category.item_count, 0);
        category.decrement_item_count(); // Should not go negative
        assert_eq!(category.item_count, 0);
    }
}
