//! MemVid store implementation

use crate::conversion::{FrameData, MemoryConverter};
use crate::error::{MemvidError, Result};
use crate::store_trait::MemoryStore;
use crate::MemvidConfig;
use agent_mem_traits::{Filters, Memory, MemoryId};
use async_trait::async_trait;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Re-export StoreStats from store_trait
pub use crate::store_trait::StoreStats;

/// MemVid store for AgentMem 2.0
///
/// This is the main storage backend that replaces all previous
/// database implementations with a single-file portable memory layer.
pub struct MemvidStore {
    /// MemVid instance (wrapped in Arc for sharing)
    // Note: We'll use a mock interface for now until memvid-core is integrated
    config: MemvidConfig,
    cache: Arc<RwLock<lru::LruCache<MemoryId, Memory>>>,
}

impl MemvidStore {
    /// Create a new MemVid store
    ///
    /// # Arguments
    /// * `config` - Store configuration
    ///
    /// # Example
    /// ```no_run
    /// use agent_mem_memvid::{MemvidStore, MemvidConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MemvidConfig::new("memory.mv2");
    /// let store = MemvidStore::create(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(config: MemvidConfig) -> Result<Self> {
        info!("Creating MemVid store at: {}", config.path);

        // Create cache with NonZeroUsize
        let cache_size =
            NonZeroUsize::new(config.cache_size).unwrap_or(NonZeroUsize::new(1000).unwrap());
        let cache = Arc::new(RwLock::new(lru::LruCache::new(cache_size)));

        // Initialize the MemVid file
        // TODO: Integrate with actual memvid-core API
        Self::initialize_file(&config.path).await?;

        let store = Self { config, cache };

        info!("MemVid store created successfully");
        Ok(store)
    }

    /// Open an existing MemVid store
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_string_lossy().to_string();
        let config = MemvidConfig::new(&path);

        info!("Opening MemVid store from: {}", path);

        if !Path::new(&path).exists() {
            return Err(MemvidError::Configuration(format!(
                "Store file does not exist: {}",
                path
            )));
        }

        let cache_size =
            NonZeroUsize::new(config.cache_size).unwrap_or(NonZeroUsize::new(1000).unwrap());
        let cache = Arc::new(RwLock::new(lru::LruCache::new(cache_size)));

        let store = Self { config, cache };

        info!("MemVid store opened successfully");
        Ok(store)
    }

    /// Initialize the MemVid file
    async fn initialize_file(path: &str) -> Result<()> {
        // TODO: Use memvid-core to create/open the file
        debug!("Initializing MemVid file: {}", path);

        // For now, just ensure the directory exists
        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        Ok(())
    }

    /// Add a memory to the store
    ///
    /// # Arguments
    /// * `memory` - Memory to add
    ///
    /// # Example
    /// ```no_run
    /// # use agent_mem_memvid::MemvidStore;
    /// # use agent_mem_traits::{Memory, Content};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut store = MemvidStore::create(Default::default()).await?;
    /// let memory = Memory {
    ///     id: Default::default(),
    ///     content: Content::text("Hello, world!"),
    ///     attributes: Default::default(),
    ///     relations: Default::default(),
    ///     metadata: Default::default(),
    /// };
    /// store.add(&memory).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add(&self, memory: &Memory) -> Result<()> {
        debug!("Adding memory: {}", memory.id);

        // Convert to frame
        let frame = MemoryConverter::memory_to_frame(memory)?;

        // Write to MemVid
        // TODO: Use memvid-core API
        Self::write_frame(&self.config.path, &frame).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.put(memory.id.clone(), memory.clone());

        debug!("Memory added successfully: {}", memory.id);
        Ok(())
    }

    /// Get a memory by ID
    pub async fn get(&self, id: &MemoryId) -> Result<Option<Memory>> {
        debug!("Getting memory: {}", id);

        // Check cache first (using write lock since lru::LruCache::get requires &mut self)
        {
            let mut cache = self.cache.write().await;
            if let Some(memory) = cache.get(id) {
                debug!("Memory found in cache: {}", id);
                return Ok(Some(memory.clone()));
            }
        }

        // Load from MemVid
        // TODO: Use memvid-core API
        let frame = Self::read_frame(&self.config.path, id).await?;
        if let Some(frame) = frame {
            let memory = MemoryConverter::frame_to_memory(&frame)?;

            // Update cache
            let mut cache = self.cache.write().await;
            cache.put(id.clone(), memory.clone());

            debug!("Memory found: {}", id);
            Ok(Some(memory))
        } else {
            debug!("Memory not found: {}", id);
            Ok(None)
        }
    }

    /// Update a memory
    pub async fn update(&self, memory: &Memory) -> Result<()> {
        debug!("Updating memory: {}", memory.id);

        // Check if memory exists
        if self.get(&memory.id).await?.is_none() {
            return Err(MemvidError::MemoryNotFound(memory.id.to_string()));
        }

        // Convert and write
        let frame = MemoryConverter::memory_to_frame(memory)?;
        Self::write_frame(&self.config.path, &frame).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.put(memory.id.clone(), memory.clone());

        debug!("Memory updated successfully: {}", memory.id);
        Ok(())
    }

    /// Delete a memory
    pub async fn delete(&self, id: &MemoryId) -> Result<()> {
        debug!("Deleting memory: {}", id);

        // Remove from MemVid
        // TODO: Use memvid-core API
        Self::remove_frame(&self.config.path, id).await?;

        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.pop(id);

        debug!("Memory deleted successfully: {}", id);
        Ok(())
    }

    /// List memories with filters
    pub async fn list(&self, filters: &Filters) -> Result<Vec<Memory>> {
        debug!("Listing memories with filters");

        // TODO: Use memvid-core search API
        let frames = Self::list_frames(&self.config.path, filters).await?;

        let memories: Result<Vec<Memory>> = frames
            .into_iter()
            .map(|frame| MemoryConverter::frame_to_memory(&frame))
            .collect();

        memories
    }

    /// Count total memories
    pub async fn count(&self) -> Result<usize> {
        debug!("Counting memories");

        // TODO: Use memvid-core stats API
        Ok(Self::count_frames(&self.config.path).await?)
    }

    /// Clear all memories
    pub async fn clear(&self) -> Result<()> {
        warn!("Clearing all memories");

        // Clear cache
        let mut cache = self.cache.write().await;
        cache.clear();

        // TODO: Use memvid-core clear API
        Self::clear_frames(&self.config.path).await?;

        info!("All memories cleared");
        Ok(())
    }

    // TODO: These are placeholder implementations
    // Replace with actual memvid-core API calls

    async fn write_frame(path: &str, frame: &FrameData) -> Result<()> {
        // Placeholder: Write to a simple file for now
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        let data = serde_json::to_vec(frame)?;
        file.write_all(&data).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;

        Ok(())
    }

    async fn read_frame(path: &str, id: &MemoryId) -> Result<Option<FrameData>> {
        // Placeholder: Read from simple file
        use tokio::io::{AsyncBufReadExt, BufReader};

        let file = tokio::fs::File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if let Ok(frame) = serde_json::from_str::<FrameData>(&line) {
                if frame.tags.get("memory_id").map(|v| v.as_str()) == Some(id.as_str()) {
                    return Ok(Some(frame));
                }
            }
        }

        Ok(None)
    }

    async fn remove_frame(path: &str, id: &MemoryId) -> Result<()> {
        // Placeholder: Rebuild file without the frame
        let temp_path = format!("{}.tmp", path);

        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        // Read and filter
        let input = tokio::fs::File::open(path).await?;
        let reader = BufReader::new(input);
        let mut lines = reader.lines();

        let mut output = tokio::fs::File::create(&temp_path).await?;

        while let Some(line) = lines.next_line().await? {
            if let Ok(frame) = serde_json::from_str::<FrameData>(&line) {
                if frame.tags.get("memory_id").map(|v| v.as_str()) != Some(id.as_str()) {
                    output.write_all(line.as_bytes()).await?;
                    output.write_all(b"\n").await?;
                }
            }
        }

        output.flush().await?;

        // Replace original
        tokio::fs::rename(&temp_path, path).await?;

        Ok(())
    }

    async fn list_frames(path: &str, _filters: &Filters) -> Result<Vec<FrameData>> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let mut frames = Vec::new();
        let file = tokio::fs::File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if let Ok(frame) = serde_json::from_str::<FrameData>(&line) {
                frames.push(frame);
            }
        }

        Ok(frames)
    }

    async fn count_frames(path: &str) -> Result<usize> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let file = tokio::fs::File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut count = 0;
        while lines.next_line().await?.is_some() {
            count += 1;
        }

        Ok(count)
    }

    async fn clear_frames(path: &str) -> Result<()> {
        tokio::fs::write(path, "").await?;
        Ok(())
    }
}

// Implement the MemoryStore trait
#[async_trait]
impl MemoryStore for MemvidStore {
    async fn add(&self, memory: &Memory) -> Result<()> {
        self.add(memory).await
    }

    async fn get(&self, id: &MemoryId) -> Result<Option<Memory>> {
        self.get(id).await
    }

    async fn update(&self, memory: &Memory) -> Result<()> {
        self.update(memory).await
    }

    async fn delete(&self, id: &MemoryId) -> Result<()> {
        self.delete(id).await
    }

    async fn list(&self, filters: &Filters) -> Result<Vec<Memory>> {
        self.list(filters).await
    }

    async fn count(&self) -> Result<usize> {
        self.count().await
    }

    async fn clear(&self) -> Result<()> {
        self.clear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemvidConfig;
    use agent_mem_traits::{AttributeSet, Content, MetadataV4};

    #[tokio::test]
    async fn test_create_store() {
        let config = MemvidConfig::new("test_create.mv2");
        let store = MemvidStore::create(config).await;
        assert!(store.is_ok());

        // Cleanup
        let _ = tokio::fs::remove_file("test_create.mv2").await;
    }

    #[tokio::test]
    async fn test_add_and_get_memory() {
        let config = MemvidConfig::new("test_add_get.mv2");
        let store = MemvidStore::create(config).await.unwrap();

        let memory = Memory {
            id: MemoryId::from_string("test-id".to_string()),
            content: Content::text("Test content"),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        };

        store.add(&memory).await.unwrap();

        let retrieved = store.get(&memory.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id.as_str(), "test-id");

        // Cleanup
        let _ = tokio::fs::remove_file("test_add_get.mv2").await;
    }
}
