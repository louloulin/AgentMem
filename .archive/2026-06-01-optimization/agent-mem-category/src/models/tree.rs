//! Tree node representation for category hierarchy visualization

use super::Category;
use crate::CategoryId;
use std::collections::HashMap;

/// Tree node representing a category with its children loaded
#[derive(Debug, Clone)]
pub struct CategoryTreeNode {
    /// The category data
    pub category: Category,
    /// Child nodes (empty if not loaded)
    pub children: Vec<CategoryTreeNode>,
}

impl CategoryTreeNode {
    /// Create a new tree node from a category
    pub fn new(category: Category) -> Self {
        Self {
            category,
            children: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, node: CategoryTreeNode) {
        self.children.push(node);
    }

    /// Check if this is a leaf node (no children loaded or no children exist)
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Get the depth of the tree (1 for single node)
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            return 1;
        }
        1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
    }

    /// Get the total number of nodes in the tree
    pub fn size(&self) -> usize {
        1 + self.children.iter().map(|c| c.size()).sum::<usize>()
    }

    /// Find a node by ID
    pub fn find_by_id(&self, id: &CategoryId) -> Option<&CategoryTreeNode> {
        if &self.category.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a node by path
    pub fn find_by_path(&self, path: &str) -> Option<&CategoryTreeNode> {
        if self.category.path == path {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_path(path) {
                return Some(found);
            }
        }
        None
    }

    /// Build a tree from a flat list of categories
    pub fn build_tree(categories: Vec<Category>) -> Vec<CategoryTreeNode> {
        let mut category_map: HashMap<CategoryId, Category> = HashMap::new();
        let mut children_map: HashMap<CategoryId, Vec<Category>> = HashMap::new();

        // First pass: build maps
        for category in categories {
            let id = category.id.clone();
            if let Some(parent_id) = &category.parent_id {
                children_map
                    .entry(parent_id.clone())
                    .or_insert_with(Vec::new)
                    .push(category);
            } else {
                // Root node
                category_map.insert(id, category);
            }
        }

        // Second pass: build tree recursively
        let mut roots = Vec::new();
        for (_, category) in category_map {
            roots.push(Self::build_tree_recursive(&category, &children_map));
        }

        roots
    }

    fn build_tree_recursive(
        category: &Category,
        children_map: &HashMap<CategoryId, Vec<Category>>,
    ) -> CategoryTreeNode {
        let children = children_map.get(&category.id).cloned().unwrap_or_default();

        let mut node = CategoryTreeNode::new(category.clone());
        for child in children {
            node.add_child(Self::build_tree_recursive(&child, children_map));
        }
        node
    }

    /// Pretty print the tree
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut result = format!(
            "{}{} (items: {})\n",
            indent_str, self.category.name, self.category.item_count
        );
        for child in &self.children {
            result.push_str(&child.pretty_print(indent + 1));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CategoryScope, CategoryStatus};

    fn create_test_category(id: &str, path: &str, name: &str, parent_id: Option<&str>) -> Category {
        Category {
            id: CategoryId::from_string(id.to_string()),
            path: path.to_string(),
            name: name.to_string(),
            parent_id: parent_id.map(|p| CategoryId::from_string(p.to_string())),
            children_ids: Vec::new(),
            summary: None,
            embedding: None,
            item_count: 0,
            status: CategoryStatus::Active,
            metadata: Default::default(),
            scope: CategoryScope::new("user-123".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_tree_node_creation() {
        let category = create_test_category("id1", "/preferences", "preferences", None);
        let node = CategoryTreeNode::new(category);

        assert!(node.is_leaf());
        assert_eq!(node.depth(), 1);
        assert_eq!(node.size(), 1);
    }

    #[test]
    fn test_tree_node_with_children() {
        let parent = create_test_category("id1", "/preferences", "preferences", None);
        let child1 = create_test_category(
            "id2",
            "/preferences/communication",
            "communication",
            Some("id1"),
        );
        let child2 = create_test_category(
            "id3",
            "/preferences/programming",
            "programming",
            Some("id1"),
        );

        let mut node = CategoryTreeNode::new(parent);
        node.add_child(CategoryTreeNode::new(child1));
        node.add_child(CategoryTreeNode::new(child2));

        assert!(!node.is_leaf());
        assert_eq!(node.depth(), 2);
        assert_eq!(node.size(), 3);
    }

    #[test]
    fn test_build_tree() {
        let root = create_test_category("id1", "/", "root", None);
        let child1 = create_test_category("id2", "/preferences", "preferences", Some("id1"));
        let child2 = create_test_category("id3", "/skills", "skills", Some("id1"));
        let grandchild = create_test_category(
            "id4",
            "/preferences/communication",
            "communication",
            Some("id2"),
        );

        let categories = vec![root, child1, child2, grandchild];
        let roots = CategoryTreeNode::build_tree(categories);

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].category.name, "root");
        assert_eq!(roots[0].children.len(), 2);
        assert_eq!(roots[0].size(), 4);
    }

    #[test]
    fn test_find_by_id() {
        let root = create_test_category("id1", "/", "root", None);
        let child = create_test_category("id2", "/preferences", "preferences", Some("id1"));

        let mut node = CategoryTreeNode::new(root);
        node.add_child(CategoryTreeNode::new(child));

        let found = node.find_by_id(&CategoryId::from_string("id2".to_string()));
        assert!(found.is_some());
        assert_eq!(found.unwrap().category.name, "preferences");

        let not_found = node.find_by_id(&CategoryId::from_string("id999".to_string()));
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_path() {
        let root = create_test_category("id1", "/", "root", None);
        let child = create_test_category("id2", "/preferences", "preferences", Some("id1"));

        let mut node = CategoryTreeNode::new(root);
        node.add_child(CategoryTreeNode::new(child));

        let found = node.find_by_path("/preferences");
        assert!(found.is_some());
        assert_eq!(found.unwrap().category.name, "preferences");

        let not_found = node.find_by_path("/nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_pretty_print() {
        let root = create_test_category("id1", "/", "root", None);
        let child = create_test_category("id2", "/preferences", "preferences", Some("id1"));
        let grandchild = create_test_category(
            "id3",
            "/preferences/communication",
            "communication",
            Some("id2"),
        );

        let mut node = CategoryTreeNode::new(root);
        let mut child_node = CategoryTreeNode::new(child);
        child_node.add_child(CategoryTreeNode::new(grandchild));
        node.add_child(child_node);

        let output = node.pretty_print(0);
        assert!(output.contains("root"));
        assert!(output.contains("preferences"));
        assert!(output.contains("communication"));
    }
}
