//! Core data models for resources

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::error::ResourceId;

/// Media type enumeration for resources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    /// Plain text
    TextPlain,
    /// Markdown text
    TextMarkdown,
    /// HTML document
    TextHtml,
    /// CSV data
    TextCsv,

    /// PNG image
    ImagePng,
    /// JPEG image
    ImageJpeg,
    /// GIF image
    ImageGif,
    /// WebP image
    ImageWebp,
    /// SVG image
    ImageSvg,

    /// MP3 audio
    AudioMpeg,
    /// WAV audio
    AudioWav,
    /// OGG audio
    AudioOgg,

    /// MP4 video
    VideoMp4,
    /// WebM video
    VideoWebm,

    /// PDF document
    ApplicationPdf,
    /// JSON document
    ApplicationJson,
    /// XML document
    ApplicationXml,
    /// ZIP archive
    ApplicationZip,

    /// Unknown or unsupported type
    Unknown(String),
}

impl MediaType {
    /// Get the MIME type string
    pub fn as_mime(&self) -> &str {
        match self {
            MediaType::TextPlain => "text/plain",
            MediaType::TextMarkdown => "text/markdown",
            MediaType::TextHtml => "text/html",
            MediaType::TextCsv => "text/csv",
            MediaType::ImagePng => "image/png",
            MediaType::ImageJpeg => "image/jpeg",
            MediaType::ImageGif => "image/gif",
            MediaType::ImageWebp => "image/webp",
            MediaType::ImageSvg => "image/svg+xml",
            MediaType::AudioMpeg => "audio/mpeg",
            MediaType::AudioWav => "audio/wav",
            MediaType::AudioOgg => "audio/ogg",
            MediaType::VideoMp4 => "video/mp4",
            MediaType::VideoWebm => "video/webm",
            MediaType::ApplicationPdf => "application/pdf",
            MediaType::ApplicationJson => "application/json",
            MediaType::ApplicationXml => "application/xml",
            MediaType::ApplicationZip => "application/zip",
            MediaType::Unknown(s) => s.as_str(),
        }
    }

    /// Parse from MIME type string
    pub fn from_mime(mime: &str) -> Self {
        match mime.to_lowercase().as_str() {
            "text/plain" | "txt" => MediaType::TextPlain,
            "text/markdown" | "text/md" | "markdown" | "md" => MediaType::TextMarkdown,
            "text/html" | "html" => MediaType::TextHtml,
            "text/csv" | "csv" => MediaType::TextCsv,
            "image/png" | "png" => MediaType::ImagePng,
            "image/jpeg" | "jpg" | "jpeg" => MediaType::ImageJpeg,
            "image/gif" | "gif" => MediaType::ImageGif,
            "image/webp" | "webp" => MediaType::ImageWebp,
            "image/svg+xml" | "svg" => MediaType::ImageSvg,
            "audio/mpeg" | "mp3" => MediaType::AudioMpeg,
            "audio/wav" | "wav" => MediaType::AudioWav,
            "audio/ogg" | "ogg" => MediaType::AudioOgg,
            "video/mp4" | "mp4" => MediaType::VideoMp4,
            "video/webm" | "webm" => MediaType::VideoWebm,
            "application/pdf" | "pdf" => MediaType::ApplicationPdf,
            "application/json" | "json" => MediaType::ApplicationJson,
            "application/xml" | "xml" => MediaType::ApplicationXml,
            "application/zip" | "zip" => MediaType::ApplicationZip,
            _ => MediaType::Unknown(mime.to_string()),
        }
    }

    /// Check if this is a text type
    pub fn is_text(&self) -> bool {
        matches!(
            self,
            MediaType::TextPlain
                | MediaType::TextMarkdown
                | MediaType::TextHtml
                | MediaType::TextCsv
                | MediaType::ApplicationJson
                | MediaType::ApplicationXml
        )
    }

    /// Check if this is an image type
    pub fn is_image(&self) -> bool {
        matches!(
            self,
            MediaType::ImagePng
                | MediaType::ImageJpeg
                | MediaType::ImageGif
                | MediaType::ImageWebp
                | MediaType::ImageSvg
        )
    }

    /// Check if this is an audio type
    pub fn is_audio(&self) -> bool {
        matches!(
            self,
            MediaType::AudioMpeg | MediaType::AudioWav | MediaType::AudioOgg
        )
    }

    /// Check if this is a video type
    pub fn is_video(&self) -> bool {
        matches!(self, MediaType::VideoMp4 | MediaType::VideoWebm)
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_mime())
    }
}

/// Resource status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Resource is mounted and ready
    Mounted,
    /// Resource is being processed
    Pending,
    /// Resource mount failed
    Failed,
    /// Resource is archived
    Archived,
}

impl std::fmt::Display for ResourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceStatus::Mounted => write!(f, "mounted"),
            ResourceStatus::Pending => write!(f, "pending"),
            ResourceStatus::Failed => write!(f, "failed"),
            ResourceStatus::Archived => write!(f, "archived"),
        }
    }
}

/// Resource metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// Author of the resource
    pub author: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub modified_at: Option<DateTime<Utc>>,

    /// Tags associated with the resource
    pub tags: Vec<String>,

    /// Size in bytes
    pub size: Option<u64>,

    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ResourceMetadata {
    fn default() -> Self {
        Self {
            author: None,
            created_at: Utc::now(),
            modified_at: None,
            tags: Vec::new(),
            size: None,
            custom: HashMap::new(),
        }
    }
}

impl ResourceMetadata {
    /// Create new metadata with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Set size
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }
}

/// Resource structure representing a file-like entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique resource identifier
    pub id: ResourceId,

    /// URI pointing to the resource location
    pub uri: String,

    /// Media type of the resource
    pub media_type: MediaType,

    /// Resource metadata
    pub metadata: ResourceMetadata,

    /// Current status
    pub status: ResourceStatus,

    /// User ID that owns this resource
    pub user_id: String,

    /// Agent ID that created this resource (optional)
    pub agent_id: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Resource {
    /// Create a new resource
    pub fn new(id: ResourceId, uri: String, media_type: MediaType, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            uri,
            media_type,
            metadata: ResourceMetadata::new(),
            status: ResourceStatus::Pending,
            user_id,
            agent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set agent ID
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: ResourceMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set status
    pub fn with_status(mut self, status: ResourceStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }

    /// Mark as mounted
    pub fn mark_mounted(&mut self) {
        self.status = ResourceStatus::Mounted;
        self.updated_at = Utc::now();
    }

    /// Mark as failed
    pub fn mark_failed(&mut self) {
        self.status = ResourceStatus::Failed;
        self.updated_at = Utc::now();
    }
}

/// Resource content structure
#[derive(Debug, Clone)]
pub struct ResourceContent {
    /// Resource ID
    pub resource_id: ResourceId,

    /// Raw content bytes
    pub data: Vec<u8>,

    /// Media type
    pub media_type: MediaType,

    /// Content encoding (e.g., "utf-8", "base64")
    pub encoding: Option<String>,
}

impl ResourceContent {
    /// Create new resource content
    pub fn new(resource_id: ResourceId, data: Vec<u8>, media_type: MediaType) -> Self {
        Self {
            resource_id,
            data,
            media_type,
            encoding: None,
        }
    }

    /// Get content as string (if text type)
    pub fn as_text(&self) -> Option<String> {
        if self.media_type.is_text() {
            String::from_utf8(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Set encoding
    pub fn with_encoding(mut self, encoding: String) -> Self {
        self.encoding = Some(encoding);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_parsing() {
        assert_eq!(MediaType::from_mime("text/plain"), MediaType::TextPlain);
        assert_eq!(MediaType::from_mime("image/png"), MediaType::ImagePng);
        assert_eq!(
            MediaType::from_mime("unknown/type"),
            MediaType::Unknown("unknown/type".to_string())
        );
    }

    #[test]
    fn test_media_type_checks() {
        assert!(MediaType::TextPlain.is_text());
        assert!(MediaType::ImagePng.is_image());
        assert!(MediaType::AudioMpeg.is_audio());
        assert!(MediaType::VideoMp4.is_video());
    }

    #[test]
    fn test_resource_metadata() {
        let metadata = ResourceMetadata::new()
            .with_author("Alice".to_string())
            .with_tag("important".to_string())
            .with_size(1024);

        assert_eq!(metadata.author, Some("Alice".to_string()));
        assert_eq!(metadata.tags.len(), 1);
        assert_eq!(metadata.size, Some(1024));
    }

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new(
            ResourceId("res-123".to_string()),
            "file:///test.txt".to_string(),
            MediaType::TextPlain,
            "user-456".to_string(),
        );

        assert_eq!(resource.id.0, "res-123");
        assert_eq!(resource.uri, "file:///test.txt");
        assert_eq!(resource.status, ResourceStatus::Pending);
    }

    #[test]
    fn test_resource_status_transitions() {
        let mut resource = Resource::new(
            ResourceId("res-123".to_string()),
            "file:///test.txt".to_string(),
            MediaType::TextPlain,
            "user-456".to_string(),
        );

        resource.mark_mounted();
        assert_eq!(resource.status, ResourceStatus::Mounted);

        resource.mark_failed();
        assert_eq!(resource.status, ResourceStatus::Failed);
    }

    #[test]
    fn test_resource_content() {
        let content = ResourceContent::new(
            ResourceId("res-123".to_string()),
            b"Hello, world!".to_vec(),
            MediaType::TextPlain,
        );

        assert_eq!(content.as_text(), Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource::new(
            ResourceId("res-123".to_string()),
            "file:///test.txt".to_string(),
            MediaType::TextPlain,
            "user-456".to_string(),
        );

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: Resource = serde_json::from_str(&json).unwrap();

        assert_eq!(resource.id, deserialized.id);
        assert_eq!(resource.uri, deserialized.uri);
    }
}
