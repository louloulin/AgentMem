//! URI resolution for different protocols

use crate::models::MediaType;
use crate::models::ResourceContent;
use crate::{ResourceError, ResourceId, Result};
use async_trait::async_trait;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncReadExt;

/// URI representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct URI {
    /// Full URI string
    pub full: String,

    /// Protocol scheme (e.g., "file", "http", "conv", "doc")
    pub scheme: String,

    /// Path component
    pub path: String,
}

impl URI {
    /// Parse a URI string
    pub fn parse(uri: &str) -> Result<Self> {
        // URI format: scheme://path
        let re = Regex::new(r"^([a-zA-Z][a-zA-Z0-9+.-]*)://(.+)$")
            .map_err(|e| ResourceError::InvalidUri(format!("Regex error: {}", e)))?;

        let caps = re
            .captures(uri)
            .ok_or_else(|| ResourceError::InvalidUri(format!("Invalid URI format: {}", uri)))?;

        let scheme = caps[1].to_lowercase();
        let path = caps[2].to_string();

        Ok(Self {
            full: uri.to_string(),
            scheme,
            path,
        })
    }

    /// Check if this is a file URI
    pub fn is_file(&self) -> bool {
        self.scheme == "file"
    }

    /// Check if this is an HTTP(S) URI
    pub fn is_http(&self) -> bool {
        self.scheme == "http" || self.scheme == "https"
    }

    /// Check if this is a conversation URI
    pub fn is_conversation(&self) -> bool {
        self.scheme == "conv"
    }

    /// Check if this is a document URI
    pub fn is_document(&self) -> bool {
        self.scheme == "doc"
    }
}

impl std::fmt::Display for URI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full)
    }
}

/// URI resolver trait
#[async_trait]
pub trait URIResolver: Send + Sync {
    /// Resolve URI to content
    async fn resolve(&self, uri: &URI) -> Result<ResourceContent>;

    /// Check if this resolver supports the given scheme
    fn supports(&self, scheme: &str) -> bool;
}

/// File URI resolver (file://)
pub struct FileURIResolver;

#[async_trait]
impl URIResolver for FileURIResolver {
    async fn resolve(&self, uri: &URI) -> Result<ResourceContent> {
        if !uri.is_file() {
            return Err(ResourceError::UnsupportedScheme(uri.scheme.clone()));
        }

        let path = PathBuf::from(&uri.path);

        // Read file content
        let mut file = fs::File::open(&path).await.map_err(|e| {
            ResourceError::ResolutionFailed(format!("Failed to open file {:?}: {}", path, e))
        })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data).await.map_err(|e| {
            ResourceError::ResolutionFailed(format!("Failed to read file {:?}: {}", path, e))
        })?;

        // Detect media type: try magic bytes first, then extension
        let detector = crate::detector::MediaTypeDetector::new();
        let media_type = detector.detect_from_magic_bytes(&data).unwrap_or_else(|| {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            detector.detect_from_extension(extension)
        });

        Ok(ResourceContent::new(
            ResourceId::from(uri.full.clone()),
            data,
            media_type,
        ))
    }

    fn supports(&self, scheme: &str) -> bool {
        scheme == "file"
    }
}

/// HTTP URI resolver (http://, https://)
pub struct HTTPURIResolver {
    /// HTTP client
    client: reqwest::Client,
}

impl HTTPURIResolver {
    /// Create a new HTTP resolver
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                ResourceError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client })
    }
}

impl Default for HTTPURIResolver {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP resolver")
    }
}

#[async_trait]
impl URIResolver for HTTPURIResolver {
    async fn resolve(&self, uri: &URI) -> Result<ResourceContent> {
        if !uri.is_http() {
            return Err(ResourceError::UnsupportedScheme(uri.scheme.clone()));
        }

        // Make HTTP GET request
        let response = self
            .client
            .get(&uri.full)
            .send()
            .await
            .map_err(|e| ResourceError::NetworkError(format!("HTTP request failed: {}", e)))?;

        // Check status code
        if !response.status().is_success() {
            return Err(ResourceError::NetworkError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Get content type from headers
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream");

        let media_type = MediaType::from_mime(content_type);

        // Read response body
        let data = response
            .bytes()
            .await
            .map_err(|e| ResourceError::NetworkError(format!("Failed to read response: {}", e)))?;

        Ok(ResourceContent::new(
            ResourceId::from(uri.full.clone()),
            data.to_vec(),
            media_type,
        ))
    }

    fn supports(&self, scheme: &str) -> bool {
        scheme == "http" || scheme == "https"
    }
}

/// Conversation URI resolver (conv://)
///
/// This is a placeholder for conversation history resolution.
/// In production, this would integrate with AgentMem's conversation storage.
pub struct ConversationURIResolver;

#[async_trait]
impl URIResolver for ConversationURIResolver {
    async fn resolve(&self, uri: &URI) -> Result<ResourceContent> {
        if !uri.is_conversation() {
            return Err(ResourceError::UnsupportedScheme(uri.scheme.clone()));
        }

        // Placeholder: In production, this would fetch conversation history
        // from AgentMem's conversation storage

        let conversation_id = &uri.path;

        // Mock conversation content
        let content = format!("Conversation: {}", conversation_id);
        let data = content.as_bytes().to_vec();

        Ok(ResourceContent::new(
            ResourceId::from(uri.full.clone()),
            data,
            MediaType::TextPlain,
        ))
    }

    fn supports(&self, scheme: &str) -> bool {
        scheme == "conv"
    }
}

/// Document URI resolver (doc://)
///
/// This is a placeholder for document reference resolution.
/// In production, this would integrate with AgentMem's document storage.
pub struct DocumentURIResolver;

#[async_trait]
impl URIResolver for DocumentURIResolver {
    async fn resolve(&self, uri: &URI) -> Result<ResourceContent> {
        if !uri.is_document() {
            return Err(ResourceError::UnsupportedScheme(uri.scheme.clone()));
        }

        // Placeholder: In production, this would fetch document content
        // from AgentMem's document storage

        let document_id = &uri.path;

        // Mock document content
        let content = format!("Document: {}", document_id);
        let data = content.as_bytes().to_vec();

        Ok(ResourceContent::new(
            ResourceId::from(uri.full.clone()),
            data,
            MediaType::TextPlain,
        ))
    }

    fn supports(&self, scheme: &str) -> bool {
        scheme == "doc"
    }
}

/// Composite resolver that tries multiple resolvers
pub struct CompositeURIResolver {
    resolvers: Vec<Box<dyn URIResolver>>,
}

impl CompositeURIResolver {
    /// Create a new composite resolver with default resolvers
    pub fn new() -> Result<Self> {
        let mut resolvers: Vec<Box<dyn URIResolver>> = vec![
            Box::new(FileURIResolver),
            Box::new(ConversationURIResolver),
            Box::new(DocumentURIResolver),
        ];

        // Add HTTP resolver if available
        if let Ok(http_resolver) = HTTPURIResolver::new() {
            resolvers.push(Box::new(http_resolver));
        }

        Ok(Self { resolvers })
    }

    /// Add a custom resolver
    pub fn add_resolver(&mut self, resolver: Box<dyn URIResolver>) {
        self.resolvers.push(resolver);
    }
}

impl Default for CompositeURIResolver {
    fn default() -> Self {
        Self::new().expect("Failed to create default composite resolver")
    }
}

#[async_trait]
impl URIResolver for CompositeURIResolver {
    async fn resolve(&self, uri: &URI) -> Result<ResourceContent> {
        for resolver in &self.resolvers {
            if resolver.supports(&uri.scheme) {
                return resolver.resolve(uri).await;
            }
        }

        Err(ResourceError::UnsupportedScheme(format!(
            "No resolver found for scheme: {}",
            uri.scheme
        )))
    }

    fn supports(&self, scheme: &str) -> bool {
        self.resolvers.iter().any(|r| r.supports(scheme))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_parsing() {
        let uri = URI::parse("file:///path/to/file.txt").unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.path, "/path/to/file.txt");
        assert!(uri.is_file());
    }

    #[test]
    fn test_uri_http_parsing() {
        let uri = URI::parse("https://example.com/doc.pdf").unwrap();
        assert_eq!(uri.scheme, "https");
        assert_eq!(uri.path, "example.com/doc.pdf");
        assert!(uri.is_http());
    }

    #[test]
    fn test_uri_invalid() {
        assert!(URI::parse("invalid-uri").is_err());
    }

    #[tokio::test]
    async fn test_file_resolver() {
        use crate::resolver::URIResolver;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, world!").unwrap();

        let path = temp_file.path().to_str().unwrap();
        let uri = URI::parse(&format!("file://{}", path)).unwrap();

        let resolver = FileURIResolver;
        let content = resolver.resolve(&uri).await.unwrap();

        assert_eq!(content.as_text(), Some("Hello, world!\n".to_string()));
    }

    #[tokio::test]
    async fn test_composite_resolver() {
        let resolver = CompositeURIResolver::new().unwrap();

        // Test file scheme support
        assert!(resolver.supports("file"));
        assert!(resolver.supports("http"));
        assert!(resolver.supports("conv"));
        assert!(resolver.supports("doc"));

        // Test unsupported scheme
        assert!(!resolver.supports("ftp"));
    }
}
