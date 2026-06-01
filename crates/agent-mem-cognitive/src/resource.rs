//! Resource Memory Module
//! 
//! Resource memories store multimedia content and external resources.

use serde::{Deserialize, Serialize};

/// Resource type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Text,
    Image,
    Audio,
    Video,
    Document,
    Link,
    Other,
}

/// External resource reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource ID
    pub id: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// URL or path
    pub url: String,
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Access count
    pub access_count: u64,
}

impl Resource {
    pub fn new(id: String, url: String, title: String, resource_type: ResourceType) -> Self {
        Self {
            id,
            url,
            title,
            resource_type,
            description: None,
            tags: Vec::new(),
            access_count: 0,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new(
            "res-1".to_string(),
            "https://example.com/doc.pdf".to_string(),
            "Example Document".to_string(),
            ResourceType::Document,
        ).with_tag("reference").with_tag("important");
        assert_eq!(resource.tags.len(), 2);
    }
}
