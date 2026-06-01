//! CategoryManager trait for hierarchical category management

use crate::error::{CategoryError, Result};
use crate::models::{Category, CategoryId, CategoryPath, CategoryScope, CategoryTreeNode};
use async_trait::async_trait;

/// Trait for managing hierarchical categories
#[async_trait]
pub trait CategoryManager: Send + Sync {
    /// Create a new category at the given path
    /// Automatically creates parent categories if they don't exist
    async fn create_category(&mut self, path: &str, scope: CategoryScope) -> Result<Category>;

    /// Get a category by ID
    async fn get_category(&self, id: &CategoryId) -> Result<Category>;

    /// Get a category by path
    async fn get_category_by_path(&self, path: &str, scope: &CategoryScope) -> Result<Category>;

    /// Update a category
    async fn update_category(&mut self, category: Category) -> Result<()>;

    /// Delete a category (soft delete by setting status to Deleted)
    async fn delete_category(&mut self, id: &CategoryId) -> Result<()>;

    /// List all categories for a scope
    async fn list_categories(&self, scope: &CategoryScope) -> Result<Vec<Category>>;

    /// Get children of a category
    async fn get_children(&self, parent_id: &CategoryId) -> Result<Vec<Category>>;

    /// Navigate to a category path
    async fn navigate_path(&self, path: &str, scope: &CategoryScope) -> Result<Category>;

    /// Browse children at a path
    async fn browse_path(&self, path: &str, scope: &CategoryScope) -> Result<Vec<Category>>;

    /// Search categories by name or summary
    async fn search_categories(
        &self,
        query: &str,
        scope: &CategoryScope,
        limit: usize,
    ) -> Result<Vec<Category>>;

    /// Get category tree rooted at a path
    async fn get_tree(
        &self,
        path: &str,
        scope: &CategoryScope,
        depth: usize,
    ) -> Result<CategoryTreeNode>;

    /// Update category summary (LLM-driven)
    async fn update_summary(&mut self, id: &CategoryId, summary: String) -> Result<()>;

    /// Move a category to a new parent
    async fn move_category(
        &mut self,
        id: &CategoryId,
        new_parent_path: &str,
        scope: &CategoryScope,
    ) -> Result<()>;

    /// Increment item count in a category
    async fn increment_item_count(&mut self, id: &CategoryId) -> Result<()>;

    /// Decrement item count in a category
    async fn decrement_item_count(&mut self, id: &CategoryId) -> Result<()>;
}

/// In-memory category manager for testing and simple use cases
pub struct InMemoryCategoryManager {
    categories:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<CategoryId, Category>>>,
}

impl InMemoryCategoryManager {
    /// Create a new in-memory category manager
    pub fn new() -> Self {
        Self {
            categories: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// Insert a category directly (for internal use)
    async fn insert_category(&self, category: Category) -> Result<()> {
        let mut categories = self.categories.write().await;
        if categories.contains_key(&category.id) {
            return Err(CategoryError::CategoryAlreadyExists(
                category.id.to_string(),
            ));
        }
        categories.insert(category.id.clone(), category);
        Ok(())
    }

    /// Update parent-child relationships
    async fn update_parent_child(
        &self,
        parent_id: &CategoryId,
        child_id: CategoryId,
        add: bool,
    ) -> Result<()> {
        let mut categories = self.categories.write().await;
        let parent = categories
            .get_mut(parent_id)
            .ok_or_else(|| CategoryError::ParentNotFound(parent_id.to_string()))?;

        if add {
            parent.add_child(child_id)?;
        } else {
            parent.remove_child(&child_id);
        }

        Ok(())
    }
}

impl Default for InMemoryCategoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CategoryManager for InMemoryCategoryManager {
    async fn create_category(&mut self, path: &str, scope: CategoryScope) -> Result<Category> {
        let category_path = CategoryPath::new(path)?;
        let segments = category_path.segments();

        if segments.is_empty() {
            return Err(CategoryError::InvalidPath(
                "Cannot create root category".to_string(),
            ));
        }

        // Create parent categories first
        let mut parent_id: Option<CategoryId> = None;
        for i in 0..segments.len() - 1 {
            let parent_path = format!("/{}", segments[0..=i].join("/"));
            match self.get_category_by_path(&parent_path, &scope).await {
                Ok(parent) => {
                    parent_id = Some(parent.id.clone());
                }
                Err(_) => {
                    // Parent doesn't exist, create it
                    let parent =
                        Category::new(parent_path.clone(), segments[i].clone(), scope.clone());
                    parent_id = Some(parent.id.clone());
                    self.insert_category(parent).await?;
                }
            }
        }

        // Create the final category
        let mut category = Category::new(path.to_string(), segments.last().unwrap().clone(), scope);
        category.parent_id = parent_id.clone();

        // Insert the category
        self.insert_category(category.clone()).await?;

        // Update parent's children list
        if let Some(pid) = &parent_id {
            self.update_parent_child(pid, category.id.clone(), true)
                .await?;
        }

        Ok(category)
    }

    async fn get_category(&self, id: &CategoryId) -> Result<Category> {
        let categories = self.categories.read().await;
        categories
            .get(id)
            .cloned()
            .ok_or_else(|| CategoryError::CategoryNotFound(id.to_string()))
    }

    async fn get_category_by_path(&self, path: &str, scope: &CategoryScope) -> Result<Category> {
        let categories = self.categories.read().await;
        for category in categories.values() {
            if category.path == path && &category.scope == scope {
                return Ok(category.clone());
            }
        }
        Err(CategoryError::CategoryNotFound(path.to_string()))
    }

    async fn update_category(&mut self, category: Category) -> Result<()> {
        let mut categories = self.categories.write().await;
        if !categories.contains_key(&category.id) {
            return Err(CategoryError::CategoryNotFound(category.id.to_string()));
        }
        categories.insert(category.id.clone(), category);
        Ok(())
    }

    async fn delete_category(&mut self, id: &CategoryId) -> Result<()> {
        let mut categories = self.categories.write().await;
        let mut category = categories
            .get(id)
            .cloned()
            .ok_or_else(|| CategoryError::CategoryNotFound(id.to_string()))?;

        category.status = crate::models::CategoryStatus::Deleted;
        categories.insert(id.clone(), category);
        Ok(())
    }

    async fn list_categories(&self, scope: &CategoryScope) -> Result<Vec<Category>> {
        let categories = self.categories.read().await;
        let result: Vec<Category> = categories
            .values()
            .filter(|c| &c.scope == scope && c.status == crate::models::CategoryStatus::Active)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_children(&self, parent_id: &CategoryId) -> Result<Vec<Category>> {
        let category = self.get_category(parent_id).await?;
        let mut children = Vec::new();
        for child_id in &category.children_ids {
            if let Ok(child) = self.get_category(child_id).await {
                children.push(child);
            }
        }
        Ok(children)
    }

    async fn navigate_path(&self, path: &str, scope: &CategoryScope) -> Result<Category> {
        self.get_category_by_path(path, scope).await
    }

    async fn browse_path(&self, path: &str, scope: &CategoryScope) -> Result<Vec<Category>> {
        let category = self.get_category_by_path(path, scope).await?;
        self.get_children(&category.id).await
    }

    async fn search_categories(
        &self,
        query: &str,
        scope: &CategoryScope,
        limit: usize,
    ) -> Result<Vec<Category>> {
        let categories = self.categories.read().await;
        let query_lower = query.to_lowercase();

        let mut results: Vec<Category> = categories
            .values()
            .filter(|c| {
                &c.scope == scope
                    && c.status == crate::models::CategoryStatus::Active
                    && (c.name.to_lowercase().contains(&query_lower)
                        || c.summary
                            .as_ref()
                            .is_some_and(|s| s.to_lowercase().contains(&query_lower)))
            })
            .cloned()
            .collect();

        // Sort by relevance (exact name match first)
        results.sort_by(|a, b| {
            let a_exact = a.name.to_lowercase() == query_lower;
            let b_exact = b.name.to_lowercase() == query_lower;
            b_exact.cmp(&a_exact).then_with(|| a.name.cmp(&b.name))
        });

        results.truncate(limit);
        Ok(results)
    }

    async fn get_tree(
        &self,
        path: &str,
        scope: &CategoryScope,
        depth: usize,
    ) -> Result<CategoryTreeNode> {
        let root_category = self.get_category_by_path(path, scope).await?;
        let mut node = CategoryTreeNode::new(root_category.clone());

        if depth > 0 {
            let children = self.get_children(&root_category.id).await?;
            for child in children {
                let child_tree = self.get_tree(&child.path, scope, depth - 1).await?;
                node.add_child(child_tree);
            }
        }

        Ok(node)
    }

    async fn update_summary(&mut self, id: &CategoryId, summary: String) -> Result<()> {
        let mut categories = self.categories.write().await;
        let category = categories
            .get_mut(id)
            .ok_or_else(|| CategoryError::CategoryNotFound(id.to_string()))?;
        category.update_summary(summary);
        Ok(())
    }

    async fn move_category(
        &mut self,
        id: &CategoryId,
        new_parent_path: &str,
        scope: &CategoryScope,
    ) -> Result<()> {
        let mut category = self.get_category(id).await?;
        let new_parent = self.get_category_by_path(new_parent_path, scope).await?;

        // Check for circular reference
        if category.id == new_parent.id {
            return Err(CategoryError::CircularReference);
        }

        // Remove from old parent
        if let Some(old_parent_id) = &category.parent_id {
            self.update_parent_child(old_parent_id, category.id.clone(), false)
                .await?;
        }

        // Update category
        let new_path = format!(
            "{}/{}",
            new_parent.path.trim_end_matches('/'),
            category.name
        );
        category.path = new_path;
        category.parent_id = Some(new_parent.id.clone());

        // Add to new parent
        self.update_parent_child(&new_parent.id, category.id.clone(), true)
            .await?;

        // Update category
        self.update_category(category).await?;

        Ok(())
    }

    async fn increment_item_count(&mut self, id: &CategoryId) -> Result<()> {
        let mut categories = self.categories.write().await;
        let category = categories
            .get_mut(id)
            .ok_or_else(|| CategoryError::CategoryNotFound(id.to_string()))?;
        category.increment_item_count();
        Ok(())
    }

    async fn decrement_item_count(&mut self, id: &CategoryId) -> Result<()> {
        let mut categories = self.categories.write().await;
        let category = categories
            .get_mut(id)
            .ok_or_else(|| CategoryError::CategoryNotFound(id.to_string()))?;
        category.decrement_item_count();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_category() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        let category = manager
            .create_category("/preferences/communication", scope.clone())
            .await
            .unwrap();

        assert_eq!(category.path, "/preferences/communication");
        assert_eq!(category.name, "communication");
        assert!(!category.is_root());

        // Check that parent was created
        let parent = manager
            .get_category_by_path("/preferences", &scope)
            .await
            .unwrap();
        assert_eq!(parent.name, "preferences");
        assert!(parent.is_root());
    }

    #[tokio::test]
    async fn test_get_category() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        let created = manager
            .create_category("/test", scope.clone())
            .await
            .unwrap();
        let retrieved = manager.get_category(&created.id).await.unwrap();

        assert_eq!(created.id, retrieved.id);
        assert_eq!(created.path, retrieved.path);
    }

    #[tokio::test]
    async fn test_navigate_path() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        manager
            .create_category("/preferences/communication/style", scope.clone())
            .await
            .unwrap();

        let category = manager
            .navigate_path("/preferences/communication", &scope)
            .await
            .unwrap();
        assert_eq!(category.name, "communication");
    }

    #[tokio::test]
    async fn test_browse_path() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        manager
            .create_category("/preferences/communication/style", scope.clone())
            .await
            .unwrap();

        let children = manager
            .browse_path("/preferences/communication", &scope)
            .await
            .unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "style");
    }

    #[tokio::test]
    async fn test_search_categories() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        manager
            .create_category("/preferences/communication", scope.clone())
            .await
            .unwrap();
        manager
            .create_category("/skills/programming", scope.clone())
            .await
            .unwrap();

        let results = manager
            .search_categories("communication", &scope, 10)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "communication");
    }

    #[tokio::test]
    async fn test_get_tree() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        manager
            .create_category("/preferences/communication/style", scope.clone())
            .await
            .unwrap();

        let tree = manager
            .get_tree("/preferences/communication", &scope, 2)
            .await
            .unwrap();
        assert_eq!(tree.category.name, "communication");
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].category.name, "style");
    }

    #[tokio::test]
    async fn test_item_count() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        let category = manager
            .create_category("/test", scope.clone())
            .await
            .unwrap();
        assert_eq!(category.item_count, 0);

        manager.increment_item_count(&category.id).await.unwrap();
        let updated = manager.get_category(&category.id).await.unwrap();
        assert_eq!(updated.item_count, 1);

        manager.decrement_item_count(&category.id).await.unwrap();
        let updated = manager.get_category(&category.id).await.unwrap();
        assert_eq!(updated.item_count, 0);
    }

    #[tokio::test]
    async fn test_move_category() {
        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());

        manager
            .create_category("/old_parent/child", scope.clone())
            .await
            .unwrap();
        manager
            .create_category("/new_parent", scope.clone())
            .await
            .unwrap();

        let child = manager
            .get_category_by_path("/old_parent/child", &scope)
            .await
            .unwrap();
        manager
            .move_category(&child.id, "/new_parent", &scope)
            .await
            .unwrap();

        let moved = manager.get_category(&child.id).await.unwrap();
        assert_eq!(moved.path, "/new_parent/child");

        // Verify old parent no longer has this child
        let old_parent = manager
            .get_category_by_path("/old_parent", &scope)
            .await
            .unwrap();
        assert!(!old_parent.children_ids.contains(&child.id));

        // Verify new parent has this child
        let new_parent = manager
            .get_category_by_path("/new_parent", &scope)
            .await
            .unwrap();
        assert!(new_parent.children_ids.contains(&child.id));
    }
}
