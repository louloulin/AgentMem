//! Stage 1: Resource Ingestor
//!
//! Mounts and validates resources from various URIs

use crate::error::{ExtractionError, Result};
use crate::models::{ExtractionContext, ExtractionInput, ExtractionOutput, ResourceContent};
use crate::stage::{ExtractionStage, StagePriority};
use async_trait::async_trait;
use std::path::Path;
use tracing::{debug, info};

/// Stage 1: Resource Ingestor
///
/// This stage:
/// - Validates resource URIs
/// - Loads resource content
/// - Detects media types
/// - Extracts basic metadata
pub struct ResourceIngestor;

impl ResourceIngestor {
    /// Create new resource ingestor
    pub fn new() -> Self {
        Self
    }

    /// Load resource from URI
    async fn load_resource(&self, uri: &str) -> Result<ResourceContent> {
        // Parse URI scheme
        if uri.starts_with("file://") {
            self.load_file(uri).await
        } else if uri.starts_with("http://") || uri.starts_with("https://") {
            self.load_http(uri).await
        } else if uri.starts_with("conv://") {
            self.load_conversation(uri).await
        } else if uri.starts_with("doc://") {
            self.load_document(uri).await
        } else {
            Err(ExtractionError::InvalidURI(format!(
                "Unknown URI scheme: {}",
                uri
            )))
        }
    }

    /// Load file from local filesystem
    async fn load_file(&self, uri: &str) -> Result<ResourceContent> {
        let path = uri.trim_start_matches("file://");

        // Check if path exists
        if !Path::new(path).exists() {
            return Err(ExtractionError::ResourceNotFound(format!(
                "File not found: {}",
                path
            )));
        }

        // Read file content
        let content = tokio::fs::read_to_string(path).await?;

        info!("Loaded file: {} ({} bytes)", path, content.len());

        Ok(ResourceContent::Text(content))
    }

    /// Load resource from HTTP(S)
    async fn load_http(&self, uri: &str) -> Result<ResourceContent> {
        // For now, return placeholder
        // In production, use reqwest to fetch HTTP resources
        Ok(ResourceContent::Text(format!(
            "HTTP resource placeholder: {}",
            uri
        )))
    }

    /// Load conversation from conversation URI
    async fn load_conversation(&self, uri: &str) -> Result<ResourceContent> {
        // For now, return placeholder
        // In production, integrate with conversation storage
        Ok(ResourceContent::Text(format!(
            "Conversation placeholder: {}",
            uri
        )))
    }

    /// Load document from document URI
    async fn load_document(&self, uri: &str) -> Result<ResourceContent> {
        // For now, return placeholder
        // In production, integrate with document storage
        Ok(ResourceContent::Text(format!(
            "Document placeholder: {}",
            uri
        )))
    }

    /// Detect media type from URI and content
    fn detect_media_type(&self, uri: &str, _content: &ResourceContent) -> String {
        // Check file extension
        if let Some(ext) = Path::new(uri).extension() {
            match ext.to_str() {
                Some("md") => return "text/markdown".to_string(),
                Some("txt") => return "text/plain".to_string(),
                Some("json") => return "application/json".to_string(),
                Some("pdf") => return "application/pdf".to_string(),
                Some("png") => return "image/png".to_string(),
                Some("jpg") | Some("jpeg") => return "image/jpeg".to_string(),
                _ => {}
            }
        }

        // Default to text
        "text/plain".to_string()
    }
}

impl Default for ResourceIngestor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStage for ResourceIngestor {
    fn name(&self) -> &str {
        "ResourceIngestor"
    }

    fn priority(&self) -> StagePriority {
        StagePriority::CRITICAL
    }

    async fn process(
        &self,
        input: ExtractionInput,
        mut output: ExtractionOutput,
        _context: &mut ExtractionContext,
    ) -> Result<ExtractionOutput> {
        debug!("ResourceIngestor processing: {}", input.uri);

        // Load resource content if not already provided
        let content = if let Some(content) = input.content {
            content
        } else {
            self.load_resource(&input.uri).await?
        };

        // Detect media type if not provided
        let media_type = input
            .media_type
            .unwrap_or_else(|| self.detect_media_type(&input.uri, &content));

        // Store in context for next stages
        output.metrics.resource_size_bytes = content.size();

        info!(
            "Ingested resource: {} (media_type: {}, size: {} bytes)",
            input.uri, media_type, output.metrics.resource_size_bytes
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_media_type() {
        let ingestor = ResourceIngestor::new();

        assert_eq!(
            ingestor.detect_media_type("file://test.md", &ResourceContent::Text(String::new())),
            "text/markdown"
        );

        assert_eq!(
            ingestor.detect_media_type("file://test.pdf", &ResourceContent::Text(String::new())),
            "application/pdf"
        );

        assert_eq!(
            ingestor.detect_media_type("file://test.png", &ResourceContent::Text(String::new())),
            "image/png"
        );
    }

    #[test]
    fn test_stage_priority() {
        let ingestor = ResourceIngestor::new();
        assert_eq!(ingestor.priority(), StagePriority::CRITICAL);
    }

    #[test]
    fn test_stage_name() {
        let ingestor = ResourceIngestor::new();
        assert_eq!(ingestor.name(), "ResourceIngestor");
    }
}
