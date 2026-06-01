//! Resource manager implementation

use crate::detector::MediaTypeDetector;
use crate::models::{
    MediaType, Resource, ResourceContent, ResourceId, ResourceMetadata, ResourceStatus,
};
use crate::resolver::{CompositeURIResolver, URIResolver, URI};
use crate::{ResourceError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Resource manager trait
#[async_trait]
pub trait ResourceManagerTrait: Send + Sync {
    /// Mount a resource from URI
    async fn mount_resource(
        &self,
        uri: &str,
        user_id: &str,
        agent_id: Option<&str>,
    ) -> Result<ResourceId>;

    /// Resolve resource content
    async fn resolve_resource(&self, resource_id: &ResourceId) -> Result<ResourceContent>;

    /// List resources for a user
    async fn list_resources(&self, user_id: &str) -> Result<Vec<Resource>>;

    /// Get resource metadata
    async fn get_resource(&self, resource_id: &ResourceId) -> Result<Resource>;

    /// Unmount a resource
    async fn unmount_resource(&self, resource_id: &ResourceId) -> Result<()>;
}

/// In-memory resource storage (for testing and simple use cases)
type ResourceStore = Arc<RwLock<HashMap<ResourceId, Resource>>>;

/// Resource manager implementation
pub struct ResourceManager {
    /// URI resolver
    resolver: CompositeURIResolver,

    /// Media type detector
    detector: MediaTypeDetector,

    /// Resource storage
    storage: ResourceStore,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            resolver: CompositeURIResolver::new()?,
            detector: MediaTypeDetector::new(),
            storage: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate a unique resource ID
    fn generate_resource_id() -> ResourceId {
        ResourceId(format!("res-{}", Uuid::new_v4()))
    }

    /// Extract metadata from content
    fn extract_metadata(&self, content: &ResourceContent) -> ResourceMetadata {
        let mut metadata = ResourceMetadata::new();

        // Set size
        metadata = metadata.with_size(content.data.len() as u64);

        // For text content, extract additional metadata
        if content.media_type.is_text() {
            if let Some(text) = content.as_text() {
                // Count lines and words
                let lines = text.lines().count();
                let words = text.split_whitespace().count();

                metadata.custom.insert(
                    "line_count".to_string(),
                    serde_json::Value::Number(lines.into()),
                );
                metadata.custom.insert(
                    "word_count".to_string(),
                    serde_json::Value::Number(words.into()),
                );
            }
        }

        metadata
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default resource manager")
    }
}

#[async_trait]
impl ResourceManagerTrait for ResourceManager {
    /// Mount a resource from URI
    ///
    /// # Arguments
    /// * `uri` - Resource URI (file://, http://, conv://, doc://)
    /// * `user_id` - User ID that owns this resource
    /// * `agent_id` - Optional agent ID that created this resource
    ///
    /// # Returns
    /// Resource ID
    async fn mount_resource(
        &self,
        uri: &str,
        user_id: &str,
        agent_id: Option<&str>,
    ) -> Result<ResourceId> {
        // Parse URI
        let parsed_uri = URI::parse(uri)?;

        // Resolve content
        let content = self.resolver.resolve(&parsed_uri).await?;

        // Generate resource ID
        let resource_id = Self::generate_resource_id();

        // Extract metadata
        let metadata = self.extract_metadata(&content);

        // Create resource
        let mut resource = Resource::new(
            resource_id.clone(),
            uri.to_string(),
            content.media_type,
            user_id.to_string(),
        );

        resource = resource
            .with_metadata(metadata)
            .with_status(ResourceStatus::Mounted);

        if let Some(aid) = agent_id {
            resource = resource.with_agent_id(aid.to_string());
        }

        // Store resource
        let mut storage = self.storage.write().await;
        storage.insert(resource_id.clone(), resource);

        Ok(resource_id)
    }

    /// Resolve resource content
    async fn resolve_resource(&self, resource_id: &ResourceId) -> Result<ResourceContent> {
        // Get resource from storage
        let storage = self.storage.read().await;
        let resource = storage
            .get(resource_id)
            .ok_or_else(|| ResourceError::NotFound(resource_id.clone()))?;

        // Check status
        if resource.status != ResourceStatus::Mounted {
            return Err(ResourceError::ResolutionFailed(format!(
                "Resource {} is not mounted (status: {})",
                resource_id, resource.status
            )));
        }

        // Parse URI and resolve content
        let uri = URI::parse(&resource.uri)?;
        drop(storage); // Release lock before async operation

        let content = self.resolver.resolve(&uri).await?;

        Ok(content)
    }

    /// List resources for a user
    async fn list_resources(&self, user_id: &str) -> Result<Vec<Resource>> {
        let storage = self.storage.read().await;
        let resources: Vec<Resource> = storage
            .values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect();

        Ok(resources)
    }

    /// Get resource metadata
    async fn get_resource(&self, resource_id: &ResourceId) -> Result<Resource> {
        let storage = self.storage.read().await;
        let resource = storage
            .get(resource_id)
            .ok_or_else(|| ResourceError::NotFound(resource_id.clone()))?;

        Ok(resource.clone())
    }

    /// Unmount a resource
    async fn unmount_resource(&self, resource_id: &ResourceId) -> Result<()> {
        let mut storage = self.storage.write().await;

        if let Some(mut resource) = storage.remove(resource_id) {
            resource.status = ResourceStatus::Archived;
            resource.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(ResourceError::NotFound(resource_id.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_mount_file_resource() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content").unwrap();

        let path = temp_file.path().to_str().unwrap();
        let uri = format!("file://{}", path);

        let manager = ResourceManager::new().unwrap();
        let resource_id = manager
            .mount_resource(&uri, "user-123", Some("agent-456"))
            .await
            .unwrap();

        assert!(resource_id.0.starts_with("res-"));

        // Get resource
        let resource = manager.get_resource(&resource_id).await.unwrap();
        assert_eq!(resource.user_id, "user-123");
        assert_eq!(resource.agent_id, Some("agent-456".to_string()));
        assert_eq!(resource.status, ResourceStatus::Mounted);
    }

    #[tokio::test]
    async fn test_resolve_resource() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, world!").unwrap();

        let path = temp_file.path().to_str().unwrap();
        let uri = format!("file://{}", path);

        let manager = ResourceManager::new().unwrap();
        let resource_id = manager
            .mount_resource(&uri, "user-123", None)
            .await
            .unwrap();

        // Resolve content
        let content = manager.resolve_resource(&resource_id).await.unwrap();
        assert_eq!(content.as_text(), Some("Hello, world!\n".to_string()));
    }

    #[tokio::test]
    async fn test_list_resources() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        let mut temp_file2 = NamedTempFile::new().unwrap();
        writeln!(temp_file1, "Content 1").unwrap();
        writeln!(temp_file2, "Content 2").unwrap();

        let path1 = temp_file1.path().to_str().unwrap();
        let path2 = temp_file2.path().to_str().unwrap();

        let manager = ResourceManager::new().unwrap();

        // Mount two resources for user-123
        let _id1 = manager
            .mount_resource(&format!("file://{}", path1), "user-123", None)
            .await
            .unwrap();
        let _id2 = manager
            .mount_resource(&format!("file://{}", path2), "user-123", None)
            .await
            .unwrap();

        // List resources
        let resources = manager.list_resources("user-123").await.unwrap();
        assert_eq!(resources.len(), 2);

        // List resources for different user
        let resources = manager.list_resources("user-456").await.unwrap();
        assert_eq!(resources.len(), 0);
    }

    #[tokio::test]
    async fn test_unmount_resource() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Content").unwrap();

        let path = temp_file.path().to_str().unwrap();
        let uri = format!("file://{}", path);

        let manager = ResourceManager::new().unwrap();
        let resource_id = manager
            .mount_resource(&uri, "user-123", None)
            .await
            .unwrap();

        // Unmount
        manager.unmount_resource(&resource_id).await.unwrap();

        // Try to get unmounted resource (should fail)
        let result = manager.get_resource(&resource_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metadata_extraction() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1\nLine 2\nLine 3").unwrap();

        let path = temp_file.path().to_str().unwrap();
        let uri = format!("file://{}", path);

        let manager = ResourceManager::new().unwrap();
        let resource_id = manager
            .mount_resource(&uri, "user-123", None)
            .await
            .unwrap();

        // Get resource and check metadata
        let resource = manager.get_resource(&resource_id).await.unwrap();

        assert!(resource.metadata.size.is_some());
        assert!(resource.metadata.size.unwrap() > 0);

        // Check custom metadata for text files
        assert!(resource.metadata.custom.contains_key("line_count"));
        assert!(resource.metadata.custom.contains_key("word_count"));
    }
}
