//! Category Recall Module
//!
//! Provides category-aware search capabilities:
//! - Category embedding search (semantic similarity)
//! - Category path matching (fuzzy matching)
//! - Top-K related categories recommendation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Category search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySearchResult {
    /// Category ID
    pub id: String,
    /// Category path (e.g., "/preferences/communication/style")
    pub path: String,
    /// Category name
    pub name: String,
    /// Similarity score (0.0 - 1.0)
    pub score: f32,
    /// Parent category ID
    pub parent_id: Option<String>,
    /// Number of items in category
    pub item_count: usize,
    /// Category summary (if available)
    pub summary: Option<String>,
}

/// Category recall result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecallResult {
    /// Matching categories
    pub categories: Vec<CategorySearchResult>,
    /// Search time in milliseconds
    pub search_time_ms: u64,
    /// Whether category search was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Category filter for search
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryFilter {
    /// Include only these category IDs
    pub include_ids: Option<Vec<String>>,
    /// Include categories matching these paths
    pub include_paths: Option<Vec<String>>,
    /// Exclude these category IDs
    pub exclude_ids: Option<Vec<String>>,
    /// Minimum item count
    pub min_item_count: Option<usize>,
    /// Maximum item count
    pub max_item_count: Option<usize>,
}

/// Category recall configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecallConfig {
    /// Maximum categories to return
    pub max_categories: usize,
    /// Minimum score threshold (0.0 - 1.0)
    pub min_score: f32,
    /// Enable semantic search (embedding-based)
    pub enable_semantic: bool,
    /// Enable path matching (fuzzy)
    pub enable_path_matching: bool,
    /// Enable related categories recommendation
    pub enable_related: bool,
    /// Number of related categories to recommend
    pub related_count: usize,
}

impl Default for CategoryRecallConfig {
    fn default() -> Self {
        Self {
            max_categories: 10,
            min_score: 0.3,
            enable_semantic: true,
            enable_path_matching: true,
            enable_related: true,
            related_count: 3,
        }
    }
}

/// Category recall engine trait
#[async_trait]
pub trait CategoryRecallEngine: Send + Sync {
    /// Search categories by query
    async fn search_categories(
        &self,
        query: &str,
        scope: &CategoryScope,
        limit: usize,
    ) -> Result<CategoryRecallResult, String>;

    /// Get categories by IDs
    async fn get_categories(
        &self,
        ids: &[String],
        scope: &CategoryScope,
    ) -> Result<Vec<CategorySearchResult>, String>;

    /// Get related categories
    async fn get_related(
        &self,
        category_id: &str,
        scope: &CategoryScope,
        limit: usize,
    ) -> Result<Vec<CategorySearchResult>, String>;

    /// Filter categories by filter
    async fn filter_categories(
        &self,
        filter: &CategoryFilter,
        scope: &CategoryScope,
    ) -> Result<Vec<CategorySearchResult>, String>;
}

/// Category scope (user/agent context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScope {
    /// User ID
    pub user_id: String,
    /// Optional agent ID
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

/// In-memory category recall engine (for testing and simple use cases)
pub struct InMemoryCategoryRecall {
    categories: Arc<RwLock<Vec<CategorySearchResult>>>,
    config: CategoryRecallConfig,
}

impl InMemoryCategoryRecall {
    pub fn new(config: CategoryRecallConfig) -> Self {
        Self {
            categories: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    pub async fn add_category(&self, category: CategorySearchResult) {
        let mut categories = self.categories.write().await;
        categories.push(category);
    }

    /// Add sample categories for testing
    pub async fn with_sample_data(self) -> Self {
        let sample_categories = vec![
            CategorySearchResult {
                id: "cat-1".to_string(),
                path: "/preferences/communication/style".to_string(),
                name: "style".to_string(),
                score: 1.0,
                parent_id: Some("cat-2".to_string()),
                item_count: 5,
                summary: Some("User communication style preferences".to_string()),
            },
            CategorySearchResult {
                id: "cat-2".to_string(),
                path: "/preferences/communication".to_string(),
                name: "communication".to_string(),
                score: 1.0,
                parent_id: Some("cat-3".to_string()),
                item_count: 10,
                summary: Some("User communication preferences".to_string()),
            },
            CategorySearchResult {
                id: "cat-3".to_string(),
                path: "/preferences".to_string(),
                name: "preferences".to_string(),
                score: 1.0,
                parent_id: None,
                item_count: 20,
                summary: Some("User preferences".to_string()),
            },
            CategorySearchResult {
                id: "cat-4".to_string(),
                path: "/knowledge/programming/rust".to_string(),
                name: "rust".to_string(),
                score: 1.0,
                parent_id: Some("cat-5".to_string()),
                item_count: 15,
                summary: Some("Rust programming knowledge".to_string()),
            },
            CategorySearchResult {
                id: "cat-5".to_string(),
                path: "/knowledge/programming".to_string(),
                name: "programming".to_string(),
                score: 1.0,
                parent_id: Some("cat-6".to_string()),
                item_count: 30,
                summary: Some("Programming knowledge".to_string()),
            },
            CategorySearchResult {
                id: "cat-6".to_string(),
                path: "/knowledge".to_string(),
                name: "knowledge".to_string(),
                score: 1.0,
                parent_id: None,
                item_count: 50,
                summary: Some("General knowledge".to_string()),
            },
            CategorySearchResult {
                id: "cat-7".to_string(),
                path: "/skills/analysis/debugging".to_string(),
                name: "debugging".to_string(),
                score: 1.0,
                parent_id: Some("cat-8".to_string()),
                item_count: 8,
                summary: Some("Debugging skills".to_string()),
            },
            CategorySearchResult {
                id: "cat-8".to_string(),
                path: "/skills/analysis".to_string(),
                name: "analysis".to_string(),
                score: 1.0,
                parent_id: Some("cat-9".to_string()),
                item_count: 12,
                summary: Some("Analysis skills".to_string()),
            },
            CategorySearchResult {
                id: "cat-9".to_string(),
                path: "/skills".to_string(),
                name: "skills".to_string(),
                score: 1.0,
                parent_id: None,
                item_count: 25,
                summary: Some("User skills".to_string()),
            },
        ];

        let mut categories = self.categories.write().await;
        *categories = sample_categories;

        drop(categories);
        self
    }
}

#[async_trait]
impl CategoryRecallEngine for InMemoryCategoryRecall {
    async fn search_categories(
        &self,
        query: &str,
        _scope: &CategoryScope,
        limit: usize,
    ) -> Result<CategoryRecallResult, String> {
        let start = std::time::Instant::now();
        let query_lower = query.to_lowercase();

        let categories = self.categories.read().await;

        // Search by name, path, or summary
        let mut results: Vec<CategorySearchResult> = categories
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower)
                    || c.path.to_lowercase().contains(&query_lower)
                    || c.summary
                        .as_ref()
                        .map_or(false, |s| s.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        // Sort by score (simulated semantic similarity)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        results.truncate(limit);

        let search_time_ms = start.elapsed().as_millis() as u64;

        debug!("Category search for '{}' found {} results in {}ms", query, results.len(), search_time_ms);

        Ok(CategoryRecallResult {
            success: true,
            categories: results,
            search_time_ms,
            error: None,
        })
    }

    async fn get_categories(
        &self,
        ids: &[String],
        _scope: &CategoryScope,
    ) -> Result<Vec<CategorySearchResult>, String> {
        let categories = self.categories.read().await;

        let results: Vec<CategorySearchResult> = categories
            .iter()
            .filter(|c| ids.contains(&c.id))
            .cloned()
            .collect();

        Ok(results)
    }

    async fn get_related(
        &self,
        category_id: &str,
        _scope: &CategoryScope,
        limit: usize,
    ) -> Result<Vec<CategorySearchResult>, String> {
        let categories = self.categories.read().await;

        // Find the category
        let category = categories
            .iter()
            .find(|c| c.id == category_id)
            .ok_or_else(|| format!("Category not found: {}", category_id))?;

        // Find related categories (siblings and parent)
        let parent_id = category.parent_id.clone();
        let mut related: Vec<CategorySearchResult> = categories
            .iter()
            .filter(|c| {
                // Same parent (siblings)
                c.parent_id == parent_id && c.id != category_id
                    // Or parent
                    || parent_id.as_ref().map_or(false, |pid| c.id == *pid)
            })
            .cloned()
            .collect();

        related.truncate(limit);

        Ok(related)
    }

    async fn filter_categories(
        &self,
        filter: &CategoryFilter,
        _scope: &CategoryScope,
    ) -> Result<Vec<CategorySearchResult>, String> {
        let categories = self.categories.read().await;

        let mut results: Vec<CategorySearchResult> = categories
            .iter()
            .filter(|c| {
                // Include IDs filter
                if let Some(ref include_ids) = filter.include_ids {
                    if !include_ids.contains(&c.id) {
                        return false;
                    }
                }

                // Exclude IDs filter
                if let Some(ref exclude_ids) = filter.exclude_ids {
                    if exclude_ids.contains(&c.id) {
                        return false;
                    }
                }

                // Include paths filter
                if let Some(ref include_paths) = filter.include_paths {
                    if !include_paths.iter().any(|p| c.path.starts_with(p)) {
                        return false;
                    }
                }

                // Min item count filter
                if let Some(min) = filter.min_item_count {
                    if c.item_count < min {
                        return false;
                    }
                }

                // Max item count filter
                if let Some(max) = filter.max_item_count {
                    if c.item_count > max {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by item count descending
        results.sort_by(|a, b| b.item_count.cmp(&a.item_count));

        Ok(results)
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_categories() {
        let engine = InMemoryCategoryRecall::new(CategoryRecallConfig::default())
            .with_sample_data()
            .await;

        let scope = CategoryScope::new("user-123".to_string());
        let result = engine.search_categories("communication", &scope, 10).await.unwrap();

        assert!(result.success);
        assert!(!result.categories.is_empty());
        assert!(result.categories.iter().any(|c| c.path.contains("communication")));
    }

    #[tokio::test]
    async fn test_get_related() {
        let engine = InMemoryCategoryRecall::new(CategoryRecallConfig::default())
            .with_sample_data()
            .await;

        let scope = CategoryScope::new("user-123".to_string());
        let result = engine.get_related("cat-1", &scope, 5).await.unwrap();

        // Should find parent (cat-2) and siblings
        assert!(!result.is_empty() || result.iter().any(|c| c.id == "cat-2"));
    }

    #[tokio::test]
    async fn test_filter_categories() {
        let engine = InMemoryCategoryRecall::new(CategoryRecallConfig::default())
            .with_sample_data()
            .await;

        let scope = CategoryScope::new("user-123".to_string());
        let filter = CategoryFilter {
            include_paths: Some(vec!["/preferences".to_string()]),
            min_item_count: Some(5),
            ..Default::default()
        };

        let result = engine.filter_categories(&filter, &scope).await.unwrap();

        assert!(!result.is_empty());
        assert!(result.iter().all(|c| c.path.starts_with("/preferences")));
    }
}
