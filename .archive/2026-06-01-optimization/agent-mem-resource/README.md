# agent-mem-resource

Resource abstraction layer for AgentMem file-centric memory system.

## Overview

This crate implements the resource abstraction layer that treats all memory sources as file-like entities with URIs, media types, and metadata.

## Features

- **URI-based Resource Identification**: Support for multiple protocols
  - `file://` - Local file system
  - `http://` / `https://` - HTTP resources
  - `conv://` - Conversation history
  - `doc://` - Document references

- **Automatic Media Type Detection**:
  - Magic bytes detection (file signatures)
  - Extension-based detection
  - Content inspection

- **Resource Management**:
  - Mount resources from URIs
  - Resolve resource content
  - List and query resources
  - Unmount/archive resources

## Architecture

```text
ResourceManager
    ├── mount_resource(uri, scope) -> ResourceId
    ├── resolve_resource(resource_id) -> ResourceContent
    └── list_resources(scope) -> Vec<Resource>

Components:
    ├── MediaTypeDetector (magic bytes + extension)
    ├── URIResolver (file://, http://, conv://, doc://)
    └── ResourceStorage (persistence layer)
```

## Usage

### Basic Example

```rust
use agent_mem_resource::{ResourceManager, ResourceManagerTrait, ResourceId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create resource manager
    let manager = ResourceManager::new()?;

    // Mount a file resource
    let resource_id = manager.mount_resource(
        "file:///path/to/document.md",
        "user-123",
        Some("agent-456")
    ).await?;

    println!("Mounted resource: {}", resource_id);

    // Resolve resource content
    let content = manager.resolve_resource(&resource_id).await?;
    if let Some(text) = content.as_text() {
        println!("Content: {}", text);
    }

    // List user resources
    let resources = manager.list_resources("user-123").await?;
    println!("User has {} resources", resources.len());

    // Unmount resource
    manager.unmount_resource(&resource_id).await?;

    Ok(())
}
```

### Media Type Detection

```rust
use agent_mem_resource::{MediaTypeDetector, MediaType};

let detector = MediaTypeDetector::new();

// Detect from URI extension
let media_type = detector.detect("file:///document.pdf", None)?;
assert_eq!(media_type, MediaType::ApplicationPdf);

// Detect from magic bytes
let png_bytes = vec![0x89, 0x50, 0x4E, 0x47];
let media_type = detector.detect("file:///image.unknown", Some(&png_bytes))?;
assert_eq!(media_type, MediaType::ImagePng);
```

### URI Resolution

```rust
use agent_mem_resource::{URI, FileURIResolver, URIResolver};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = URI::parse("file:///path/to/file.txt")?;
    let resolver = FileURIResolver;

    let content = resolver.resolve(&uri).await?;
    println!("Resolved {} bytes", content.data.len());

    Ok(())
}
```

## Supported Media Types

### Text
- `text/plain` (.txt)
- `text/markdown` (.md)
- `text/html` (.html)
- `text/csv` (.csv)
- `application/json` (.json)
- `application/xml` (.xml)

### Images
- `image/png` (.png)
- `image/jpeg` (.jpg, .jpeg)
- `image/gif` (.gif)
- `image/webp` (.webp)
- `image/svg+xml` (.svg)

### Audio
- `audio/mpeg` (.mp3)
- `audio/wav` (.wav)
- `audio/ogg` (.ogg)

### Video
- `video/mp4` (.mp4)
- `video/webm` (.webm)

### Documents
- `application/pdf` (.pdf)
- `application/zip` (.zip)

## Data Model

### Resource

```rust
pub struct Resource {
    pub id: ResourceId,              // Unique identifier
    pub uri: String,                 // Resource URI
    pub media_type: MediaType,       // Media type
    pub metadata: ResourceMetadata,  // Metadata
    pub status: ResourceStatus,      // Status (Mounted, Pending, Failed)
    pub user_id: String,             // Owner user ID
    pub agent_id: Option<String>,    // Creator agent ID
    pub created_at: DateTime<Utc>,   // Creation timestamp
    pub updated_at: DateTime<Utc>,   // Update timestamp
}
```

### ResourceMetadata

```rust
pub struct ResourceMetadata {
    pub author: Option<String>,                      // Author
    pub created_at: DateTime<Utc>,                  // Creation time
    pub modified_at: Option<DateTime<Utc>>,         // Modification time
    pub tags: Vec<String>,                          // Tags
    pub size: Option<u64>,                          // Size in bytes
    pub custom: HashMap<String, Value>,             // Custom fields
}
```

## Integration with AgentMem

This crate is part of the AgentMem file-centric reform (Phase 1: Resource Abstraction Layer).

See `PROMPT.md` for the complete reform plan and integration guide.

## Testing

```bash
# Run unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_mount_file_resource
```

## License

Part of AgentMem project.
