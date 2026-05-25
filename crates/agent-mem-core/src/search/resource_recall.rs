//! Resource Recall Module
//!
//! Provides resource-aware search capabilities:
//! - Include source resources in search results
//! - Resource metadata search
//! - Resource content search

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Resource context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContext {
    /// Resource ID
    pub id: String,
    /// Resource URI (file://, http://, conv://, doc://)
    pub uri: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Media type (text, image, audio, video, application)
    pub media_type: String,
    /// Resource summary
    pub summary: Option<String>,
    /// When the resource was created
    pub created_at: Option<String>,
    /// When the resource was last accessed
    pub accessed_at: Option<String>,
    /// Resource metadata (author, tags, etc.)
    pub metadata: Option<serde_json::Value>,
}

/// Resource type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    File,
    Http,
    Conversation,
    Document,
    Unknown,
}

impl ResourceType {
    pub fn from_uri(uri: &str) -> Self {
        if uri.starts_with("file://") {
            ResourceType::File
        } else if uri.starts_with("http://") || uri.starts_with("https://") {
            ResourceType::Http
        } else if uri.starts_with("conv://") {
            ResourceType::Conversation
        } else if uri.starts_with("doc://") {
            ResourceType::Document
        } else {
            ResourceType::Unknown
        }
    }
}

/// Resource recall result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRecallResult {
    /// Resources associated with the search results
    pub resources: Vec<ResourceContext>,
    /// Whether resource recall was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Recall time in milliseconds
    pub recall_time_ms: u64,
}

/// Resource recall configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRecallConfig {
    /// Maximum resources to include
    pub max_resources: usize,
    /// Include resource summaries
    pub include_summaries: bool,
    /// Include resource metadata
    pub include_metadata: bool,
    /// Enable resource content search
    pub enable_content_search: bool,
}

impl Default for ResourceRecallConfig {
    fn default() -> Self {
        Self {
            max_resources: 20,
            include_summaries: true,
            include_metadata: true,
            enable_content_search: true,
        }
    }
}

/// Resource recall engine trait
#[async_trait]
pub trait ResourceRecallEngine: Send + Sync {
    /// Get resources for given memory item IDs
    async fn get_resources_for_items(
        &self,
        item_ids: &[String],
    ) -> Result<ResourceRecallResult, String>;

    /// Search resources by query
    async fn search_resources(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<ResourceRecallResult, String>;

    /// Get resource by ID
    async fn get_resource(&self, resource_id: &str) -> Result<Option<ResourceContext>, String>;
}

/// In-memory resource recall engine (for testing and simple use cases)
pub struct InMemoryResourceRecall {
    resources: Arc<RwLock<Vec<ResourceContext>>>,
    item_to_resource: Arc<RwLock<std::collections::HashMap<String, String>>>,
    config: ResourceRecallConfig,
}

impl InMemoryResourceRecall {
    pub fn new(config: ResourceRecallConfig) -> Self {
        Self {
            resources: Arc::new(RwLock::new(Vec::new())),
            item_to_resource: Arc::new(RwLock::new(std::collections::HashMap::new())),
            config,
        }
    }

    /// Add a resource
    pub async fn add_resource(&self, resource: ResourceContext) {
        let mut resources = self.resources.write().await;
        resources.push(resource);
    }

    /// Link a memory item to a resource
    pub async fn link_item_to_resource(&self, item_id: String, resource_id: String) {
        let mut item_to_resource = self.item_to_resource.write().await;
        item_to_resource.insert(item_id, resource_id);
    }

    /// Add sample data for testing
    pub async fn with_sample_data(self) -> Self {
        let sample_resources = vec![
            ResourceContext {
                id: "res-1".to_string(),
                uri: "conv://chat-2025-02-28".to_string(),
                resource_type: ResourceType::Conversation,
                media_type: "application/json".to_string(),
                summary: Some("User discussed Rust programming preferences".to_string()),
                created_at: Some("2025-02-28T10:00:00Z".to_string()),
                accessed_at: Some("2025-02-28T15:30:00Z".to_string()),
                metadata: Some(serde_json::json!({
                    "participants": ["user", "assistant"],
                    "message_count": 50
                })),
            },
            ResourceContext {
                id: "res-2".to_string(),
                uri: "file://README.md".to_string(),
                resource_type: ResourceType::Document,
                media_type: "text/markdown".to_string(),
                summary: Some("Project README with setup instructions".to_string()),
                created_at: Some("2025-01-15T08:00:00Z".to_string()),
                accessed_at: Some("2025-02-20T12:00:00Z".to_string()),
                metadata: Some(serde_json::json!({
                    "author": "dev team",
                    "size": 2048
                })),
            },
            ResourceContext {
                id: "res-3".to_string(),
                uri: "doc://design-notes".to_string(),
                resource_type: ResourceType::Document,
                media_type: "text/plain".to_string(),
                summary: Some("Architecture design notes".to_string()),
                created_at: Some("2025-02-01T09:00:00Z".to_string()),
                accessed_at: Some("2025-02-15T14:00:00Z".to_string()),
                metadata: Some(serde_json::json!({
                    "tags": ["design", "architecture"],
                    "version": "1.0"
                })),
            },
            ResourceContext {
                id: "res-4".to_string(),
                uri: "conv://chat-2025-03-01".to_string(),
                resource_type: ResourceType::Conversation,
                media_type: "application/json".to_string(),
                summary: Some("Debugging session with performance analysis".to_string()),
                created_at: Some("2025-03-01T14:00:00Z".to_string()),
                accessed_at: Some("2025-03-01T16:00:00Z".to_string()),
                metadata: Some(serde_json::json!({
                    "participants": ["user", "assistant"],
                    "message_count": 30
                })),
            },
        ];

        let sample_links = vec![
            ("item-1".to_string(), "res-1".to_string()),
            ("item-2".to_string(), "res-1".to_string()),
            ("item-3".to_string(), "res-2".to_string()),
            ("item-4".to_string(), "res-3".to_string()),
            ("item-5".to_string(), "res-4".to_string()),
        ];

        {
            let mut resources = self.resources.write().await;
            *resources = sample_resources;
        }

        {
            let mut item_to_resource = self.item_to_resource.write().await;
            for (item_id, resource_id) in sample_links {
                item_to_resource.insert(item_id, resource_id);
            }
        }

        self
    }
}

#[async_trait]
impl ResourceRecallEngine for InMemoryResourceRecall {
    async fn get_resources_for_items(
        &self,
        item_ids: &[String],
    ) -> Result<ResourceRecallResult, String> {
        let start = std::time::Instant::now();

        // Get resource IDs for the items
        let item_to_resource = self.item_to_resource.read().await;
        let resource_ids: Vec<String> = item_ids
            .iter()
            .filter_map(|id| item_to_resource.get(id).cloned())
            .collect();

        // Get the resources
        let resources = self.resources.read().await;
        let result_resources: Vec<ResourceContext> = resources
            .iter()
            .filter(|r| resource_ids.contains(&r.id))
            .cloned()
            .collect();

        let recall_time_ms = start.elapsed().as_millis() as u64;

        debug!("Resource recall for {} items found {} resources in {}ms",
            item_ids.len(), result_resources.len(), recall_time_ms);

        Ok(ResourceRecallResult {
            success: true,
            resources: result_resources,
            recall_time_ms,
            error: None,
        })
    }

    async fn search_resources(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<ResourceRecallResult, String> {
        let start = std::time::Instant::now();
        let query_lower = query.to_lowercase();

        let resources = self.resources.read().await;
        let mut results: Vec<ResourceContext> = resources
            .iter()
            .filter(|r| {
                r.uri.to_lowercase().contains(&query_lower)
                    || r.summary
                        .as_ref()
                        .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                    || r.media_type.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        results.truncate(limit);
        let recall_time_ms = start.elapsed().as_millis() as u64;

        Ok(ResourceRecallResult {
            success: true,
            resources: results,
            recall_time_ms,
            error: None,
        })
    }

    async fn get_resource(&self, resource_id: &str) -> Result<Option<ResourceContext>, String> {
        let resources = self.resources.read().await;
        let result = resources.iter().find(|r| r.id == resource_id).cloned();
        Ok(result)
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_resources_for_items() {
        let engine = InMemoryResourceRecall::new(ResourceRecallConfig::default())
            .with_sample_data()
            .await;

        let result = engine.get_resources_for_items(&["item-1".to_string(), "item-2".to_string()]).await.unwrap();

        assert!(result.success);
        assert!(!result.resources.is_empty());
    }

    #[tokio::test]
    async fn test_search_resources() {
        let engine = InMemoryResourceRecall::new(ResourceRecallConfig::default())
            .with_sample_data()
            .await;

        let result = engine.search_resources("conversation", 10).await.unwrap();

        assert!(result.success);
        assert!(!result.resources.is_empty());
    }
}
