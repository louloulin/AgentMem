//! MemVid 存储 backend - 实现 MemoryProvider trait
//!
//! ## 架构设计
//!
//! 本模块采用**高内聚、低耦合**的设计原则：
//!
//! - **MemvidStoreImpl**: 内部实现，负责与 MemVid API 交互
//! - **MemvidStore**: Public facade，实现 `MemoryProvider` trait
//! - **适配器层**: 处理类型转换和 session 隔离
//!
//! ## 依赖关系
//!
//! ```
//! 应用层
//!    ↓ 依赖抽象 (trait)
//! ┌─────────────────────────────────────┐
//! │  MemoryProvider trait (抽象)         │
//! └─────────────────────────────────────┘
//!    ↑ 实现
//! ┌─────────────────────────────────────┐
//! │  MemvidStore (facade + 适配器)      │
//! └─────────────────────────────────────┘
//!    ↓ 委托
//! ┌─────────────────────────────────────┐
//! │  MemvidStoreImpl (内部实现)          │
//! └─────────────────────────────────────┘
//!    ↓ 使用
//! ┌─────────────────────────────────────┐
//! │  memvid-core (MemVid API)           │
//! └─────────────────────────────────────┘
//! ```

use crate::error::{MemvidError, Result};
use agent_mem_traits::{AttributeSet, Content, Memory, MemoryId, MetadataV4};
use std::io;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Re-export memvid-core types
pub use memvid_core::{
    Frame, Memvid, OpenReadOptions, PutOptions, SearchHit, SearchRequest, SearchResponse, Stats,
    TimelineQuery,
};

use memvid_core::types::FrameStatus;

/// 版本信息
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: u32,
    pub timestamp: i64,
    pub status: String,
}

// ============================================================================
// 内部实现 (MemvidStoreImpl)
// ============================================================================

/// MemVid 存储的内部实现
///
/// 负责与 memvid-core API 的直接交互，不实现任何 trait，
/// 保持高内聚，专注于 MemVid 特定的操作。
pub struct MemvidStoreImpl {
    /// Path to the .mv2 file
    path: String,

    /// In-memory cache for hot data
    cache: Arc<RwLock<lru::LruCache<String, Memory>>>,
}

impl MemvidStoreImpl {
    /// Create a new MemVid file
    pub async fn create(path: impl Into<String>) -> Result<Self> {
        let path = path.into();
        info!("Creating MemVid store: {}", path);

        // Create the MemVid file
        let _mem = Memvid::create(Path::new(&path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to create: {}", e)))?;

        // Initialize cache
        let cache_size = std::num::NonZeroUsize::new(1000).unwrap();
        let cache = Arc::new(RwLock::new(lru::LruCache::new(cache_size)));

        Ok(Self { path, cache })
    }

    /// Open an existing MemVid file
    pub async fn open(path: impl Into<String>) -> Result<Self> {
        let path_str = path.into();
        info!("Opening MemVid store: {}", path_str);

        if !Path::new(&path_str).exists() {
            return Err(MemvidError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                path_str,
            )));
        }

        let _mem = Memvid::open(&path_str)
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let cache_size = std::num::NonZeroUsize::new(1000).unwrap();
        let cache = Arc::new(RwLock::new(lru::LruCache::new(cache_size)));

        Ok(Self {
            path: path_str,
            cache,
        })
    }

    /// Add a memory
    pub async fn add(&self, memory: &Memory) -> Result<()> {
        debug!("Adding memory: {}", memory.id);

        let mut mem = Memvid::open(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        // Convert content to bytes
        let content = self.memory_to_bytes(memory)?;

        let uri = format!("mv2://memory/{}", memory.id.as_str());
        let search_text = format!("{}", memory.content);

        let options = PutOptions {
            uri: Some(uri.clone()),
            title: Some(format!("Memory: {}", memory.id.as_str())),
            search_text: Some(search_text),
            ..Default::default()
        };

        mem.put_bytes_with_options(&content, options)
            .map_err(|e| MemvidError::Memvid(format!("Failed to write: {}", e)))?;

        mem.commit()
            .map_err(|e| MemvidError::Memvid(format!("Failed to commit: {}", e)))?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.put(memory.id.as_str().to_string(), memory.clone());

        Ok(())
    }

    /// Get a memory
    pub async fn get(&self, id: &MemoryId) -> Result<Option<Memory>> {
        debug!("Getting memory: {}", id);

        // Check cache first, but validate it's not stale
        {
            let mut cache = self.cache.write().await;
            if let Some(_memory) = cache.get(id.as_str()) {
                // Found in cache - but we need to verify it's still valid
                // Drop cache lock before loading from MemVid
            }
        }

        // Load from MemVid
        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let uri = format!("mv2://memory/{}", id.as_str());
        let frame = mem.frame_by_uri(&uri);

        if let Ok(frame) = frame {
            // Check if frame is deleted
            if frame.status != FrameStatus::Active {
                // Frame is deleted, remove from cache
                let mut cache = self.cache.write().await;
                cache.pop(id.as_str());
                return Ok(None);
            }

            // Try to get the text
            match mem.frame_text_by_id(frame.id) {
                Ok(text) => {
                    let memory = Memory {
                        id: id.clone(),
                        content: Content::text(text),
                        attributes: AttributeSet::new(),
                        relations: Default::default(),
                        metadata: MetadataV4::default(),
                    };

                    // Update cache
                    let mut cache = self.cache.write().await;
                    cache.put(id.as_str().to_string(), memory.clone());

                    Ok(Some(memory))
                }
                Err(_) => {
                    // Frame was deleted or text not available
                    // Remove from cache if present
                    let mut cache = self.cache.write().await;
                    cache.pop(id.as_str());
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Update a memory
    pub async fn update(&self, memory: &Memory) -> Result<()> {
        debug!("Updating memory: {}", memory.id);

        let mut mem = Memvid::open(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        // Find existing frame
        let uri = format!("mv2://memory/{}", memory.id.as_str());
        let existing = mem.frame_by_uri(&uri);

        if let Ok(frame) = existing {
            let content = self.memory_to_bytes(memory)?;

            let search_text = format!("{}", memory.content);
            let options = PutOptions {
                uri: Some(uri),
                title: Some(format!("Memory: {}", memory.id.as_str())),
                search_text: Some(search_text),
                ..Default::default()
            };

            mem.update_frame(frame.id, Some(content), options, None)
                .map_err(|e| MemvidError::Memvid(format!("Failed to update: {}", e)))?;

            mem.commit()
                .map_err(|e| MemvidError::Memvid(format!("Failed to commit: {}", e)))?;

            // Update cache
            let mut cache = self.cache.write().await;
            cache.put(memory.id.as_str().to_string(), memory.clone());

            Ok(())
        } else {
            Err(MemvidError::MemoryNotFound(format!(
                "Memory not found: {}",
                memory.id
            )))
        }
    }

    /// Delete a memory
    pub async fn delete(&self, id: &MemoryId) -> Result<()> {
        debug!("Deleting memory: {}", id);

        let mut mem = Memvid::open(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let uri = format!("mv2://memory/{}", id.as_str());
        let frame = mem.frame_by_uri(&uri);

        if let Ok(frame) = frame {
            mem.delete_frame(frame.id)
                .map_err(|e| MemvidError::Memvid(format!("Failed to delete: {}", e)))?;

            mem.commit()
                .map_err(|e| MemvidError::Memvid(format!("Failed to commit: {}", e)))?;

            // Remove from cache
            let mut cache = self.cache.write().await;
            cache.pop(id.as_str());

            Ok(())
        } else {
            // Frame not found - might already be deleted
            // Remove from cache anyway
            let mut cache = self.cache.write().await;
            cache.pop(id.as_str());

            Err(MemvidError::MemoryNotFound(format!(
                "Memory not found: {}",
                id
            )))
        }
    }

    /// List all memories
    pub async fn list(&self) -> Result<Vec<Memory>> {
        debug!("Listing memories");

        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let stats = mem
            .stats()
            .map_err(|e| MemvidError::Memvid(format!("Failed to get stats: {}", e)))?;

        let mut memories = Vec::new();

        for frame_id in 0..stats.frame_count {
            if let Ok(frame) = mem.frame_by_id(frame_id) {
                // Only include active frames
                if frame.status != FrameStatus::Active {
                    continue;
                }

                if let Ok(text) = mem.frame_text_by_id(frame_id) {
                    // Extract memory ID from URI
                    if let Some(uri) = &frame.uri {
                        if let Some(memory_id) = uri.strip_prefix("mv2://memory/") {
                            let memory = Memory {
                                id: MemoryId::from_string(memory_id.to_string()),
                                content: Content::text(text),
                                attributes: AttributeSet::new(),
                                relations: Default::default(),
                                metadata: MetadataV4::default(),
                            };
                            memories.push(memory);
                        }
                    }
                }
            }
        }

        Ok(memories)
    }

    /// Count memories
    pub async fn count(&self) -> Result<usize> {
        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let stats = mem
            .stats()
            .map_err(|e| MemvidError::Memvid(format!("Failed to get stats: {}", e)))?;

        // Count only active frames with mv2://memory/ URIs
        let mut count = 0;
        for frame_id in 0..stats.frame_count {
            if let Ok(frame) = mem.frame_by_id(frame_id) {
                // Check if frame is active and has a memory URI
                if frame.status == FrameStatus::Active {
                    if let Some(uri) = &frame.uri {
                        if uri.starts_with("mv2://memory/") {
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Search memories
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchHit>> {
        debug!("Searching: query='{}', top_k={}", query, top_k);

        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let response = mem
            .search(SearchRequest {
                query: query.to_string(),
                top_k,
                snippet_chars: 200,
                uri: Some("mv2://memory/".to_string()),
                scope: None,
                cursor: None,
                no_sketch: false,
                as_of_frame: None,
                as_of_ts: None,
            })
            .map_err(|e| MemvidError::Memvid(format!("Search failed: {}", e)))?;

        Ok(response.hits)
    }

    /// Fuzzy search for approximate matching
    pub async fn search_fuzzy(&self, query: &str, top_k: usize) -> Result<Vec<SearchHit>> {
        debug!("Fuzzy search: query='{}', top_k={}", query, top_k);

        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        // Add fuzzy operator
        let fuzzy_query = format!("{}~", query);

        let response = mem
            .search(SearchRequest {
                query: fuzzy_query,
                top_k,
                snippet_chars: 200,
                uri: Some("mv2://memory/".to_string()),
                scope: None,
                cursor: None,
                no_sketch: false,
                as_of_frame: None,
                as_of_ts: None,
            })
            .map_err(|e| MemvidError::Memvid(format!("Fuzzy search failed: {}", e)))?;

        Ok(response.hits)
    }

    /// Phrase search for exact matching
    pub async fn search_phrase(&self, phrase: &str, top_k: usize) -> Result<Vec<SearchHit>> {
        debug!("Phrase search: phrase='{}', top_k={}", phrase, top_k);

        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        // Wrap in quotes for exact phrase matching
        let phrase_query = format!("\"{}\"", phrase);

        let response = mem
            .search(SearchRequest {
                query: phrase_query,
                top_k,
                snippet_chars: 200,
                uri: Some("mv2://memory/".to_string()),
                scope: None,
                cursor: None,
                no_sketch: false,
                as_of_frame: None,
                as_of_ts: None,
            })
            .map_err(|e| MemvidError::Memvid(format!("Phrase search failed: {}", e)))?;

        Ok(response.hits)
    }

    /// Multi-term search (combines terms with OR)
    pub async fn search_multi(&self, terms: Vec<&str>, top_k: usize) -> Result<Vec<SearchHit>> {
        debug!("Multi-term search: terms={:?}, top_k={}", terms, top_k);

        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        // Combine queries with OR
        let combined_query = terms.join(" OR ");

        let response = mem
            .search(SearchRequest {
                query: combined_query,
                top_k,
                snippet_chars: 200,
                uri: Some("mv2://memory/".to_string()),
                scope: None,
                cursor: None,
                no_sketch: false,
                as_of_frame: None,
                as_of_ts: None,
            })
            .map_err(|e| MemvidError::Memvid(format!("Multi-term search failed: {}", e)))?;

        Ok(response.hits)
    }

    /// Get statistics
    pub async fn stats(&self) -> Result<Stats> {
        let mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        mem.stats()
            .map_err(|e| MemvidError::Memvid(format!("Failed to get stats: {}", e)))
    }

    // ============================================================================
    // Batch Operations
    // ============================================================================

    /// Add multiple memories in a single transaction
    pub async fn batch_add(&self, memories: &[Memory]) -> Result<Vec<MemoryId>> {
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Batch adding {} memories", memories.len());

        let mut mem = Memvid::open(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let mut ids = Vec::with_capacity(memories.len());
        let mut cache = self.cache.write().await;

        for memory in memories {
            let content = self.memory_to_bytes(memory)?;
            let uri = format!("mv2://memory/{}", memory.id.as_str());
            let search_text = format!("{}", memory.content);

            let options = PutOptions {
                uri: Some(uri.clone()),
                title: Some(format!("Memory: {}", memory.id.as_str())),
                search_text: Some(search_text),
                ..Default::default()
            };

            mem.put_bytes_with_options(&content, options)
                .map_err(|e| MemvidError::Memvid(format!("Failed to write: {}", e)))?;

            // Update cache
            cache.put(memory.id.as_str().to_string(), memory.clone());
            ids.push(memory.id.clone());
        }

        // Single commit for all operations
        mem.commit()
            .map_err(|e| MemvidError::Memvid(format!("Failed to commit: {}", e)))?;

        info!("Batch added {} memories successfully", ids.len());
        Ok(ids)
    }

    /// Get multiple memories by their IDs
    pub async fn batch_get(&self, ids: &[MemoryId]) -> Result<Vec<Option<Memory>>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Batch getting {} memories", ids.len());

        let mut results = Vec::with_capacity(ids.len());
        let mut mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        // First pass: check cache
        let mut cache = self.cache.write().await;
        let mut uncached_ids = Vec::new();
        let mut uncached_indices = Vec::new();

        for (index, id) in ids.iter().enumerate() {
            if let Some(memory) = cache.get(id.as_str()) {
                results.push(Some(memory.clone()));
            } else {
                results.push(None);
                uncached_ids.push(id.clone());
                uncached_indices.push(index);
            }
        }

        // Second pass: load uncached from MemVid
        drop(cache); // Release cache lock before MemVid operations

        for (id, index) in uncached_ids.into_iter().zip(uncached_indices.into_iter()) {
            let uri = format!("mv2://memory/{}", id.as_str());
            let frame = mem.frame_by_uri(&uri);

            if let Ok(frame) = frame {
                if frame.status != FrameStatus::Active {
                    continue;
                }

                if let Ok(text) = mem.frame_text_by_id(frame.id) {
                    let memory = Memory {
                        id: id.clone(),
                        content: Content::text(text),
                        attributes: AttributeSet::new(),
                        relations: Default::default(),
                        metadata: MetadataV4::default(),
                    };

                    // Update cache
                    let mut cache = self.cache.write().await;
                    cache.put(id.as_str().to_string(), memory.clone());
                    results[index] = Some(memory);
                }
            }
        }

        Ok(results)
    }

    /// Delete multiple memories in a single transaction
    pub async fn batch_delete(&self, ids: &[MemoryId]) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        debug!("Batch deleting {} memories", ids.len());

        let mut mem = Memvid::open(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let mut deleted_count = 0;
        let mut cache = self.cache.write().await;

        for id in ids {
            let uri = format!("mv2://memory/{}", id.as_str());
            let frame = mem.frame_by_uri(&uri);

            if let Ok(frame) = frame {
                mem.delete_frame(frame.id)
                    .map_err(|e| MemvidError::Memvid(format!("Failed to delete: {}", e)))?;

                // Remove from cache
                cache.pop(id.as_str());
                deleted_count += 1;
            }
        }

        // Single commit for all deletions
        if deleted_count > 0 {
            mem.commit()
                .map_err(|e| MemvidError::Memvid(format!("Failed to commit: {}", e)))?;
        }

        info!("Batch deleted {} memories successfully", deleted_count);
        Ok(deleted_count)
    }

    /// Update multiple memories in a single transaction
    pub async fn batch_update(&self, memories: &[Memory]) -> Result<Vec<MemoryId>> {
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Batch updating {} memories", memories.len());

        let mut mem = Memvid::open(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        let mut ids = Vec::with_capacity(memories.len());
        let mut cache = self.cache.write().await;

        for memory in memories {
            let uri = format!("mv2://memory/{}", memory.id.as_str());

            // Delete old version if it exists
            if let Ok(old_frame) = mem.frame_by_uri(&uri) {
                let _ = mem.delete_frame(old_frame.id);
            }

            let content = self.memory_to_bytes(memory)?;
            let search_text = format!("{}", memory.content);

            let options = PutOptions {
                uri: Some(uri.clone()),
                title: Some(format!("Memory: {}", memory.id.as_str())),
                search_text: Some(search_text),
                ..Default::default()
            };

            // Write new version
            mem.put_bytes_with_options(&content, options)
                .map_err(|e| MemvidError::Memvid(format!("Failed to write: {}", e)))?;

            // Update cache
            cache.put(memory.id.as_str().to_string(), memory.clone());
            ids.push(memory.id.clone());
        }

        // Single commit for all updates
        mem.commit()
            .map_err(|e| MemvidError::Memvid(format!("Failed to commit: {}", e)))?;

        info!("Batch updated {} memories successfully", ids.len());
        Ok(ids)
    }

    /// Clear all memories (for testing)
    pub async fn clear(&self) -> Result<()> {
        info!("Clearing all memories");

        let mut mem = Memvid::open(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        // Get all frame IDs
        let stats = mem
            .stats()
            .map_err(|e| MemvidError::Memvid(format!("Failed to get stats: {}", e)))?;
        let mut cleared = 0;

        for frame_id in 0..stats.frame_count {
            if let Ok(frame) = mem.frame_by_id(frame_id) {
                // Only delete memory frames
                if let Some(uri) = &frame.uri {
                    if uri.starts_with("mv2://memory/") {
                        let _ = mem.delete_frame(frame_id);
                        cleared += 1;
                    }
                }
            }
        }

        if cleared > 0 {
            mem.commit()
                .map_err(|e| MemvidError::Memvid(format!("Failed to commit: {}", e)))?;
        }

        // Clear cache
        let mut cache = self.cache.write().await;
        cache.clear();

        info!("Cleared {} memories", cleared);
        Ok(())
    }

    /// Get version info for a memory
    pub async fn get_version_info(&self, id: &MemoryId) -> Result<Option<VersionInfo>> {
        let uri = format!("mv2://memory/{}", id.as_str());

        let mem = Memvid::open_read_only(Path::new(&self.path))
            .map_err(|e| MemvidError::Memvid(format!("Failed to open: {}", e)))?;

        if let Ok(frame) = mem.frame_by_uri(&uri) {
            Ok(Some(VersionInfo {
                version: 1, // MemVid 使用增量版本号，这里简化为 1
                timestamp: frame.timestamp,
                status: "Active".to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    // Helper: Convert Memory to bytes
    fn memory_to_bytes(&self, memory: &Memory) -> Result<Vec<u8>> {
        // For now, just serialize the content
        match &memory.content {
            Content::Text(text) => Ok(text.as_bytes().to_vec()),
            Content::Structured(data) => {
                serde_json::to_vec(data).map_err(|e| MemvidError::Serialization(format!("{}", e)))
            }
            Content::Vector(_vec) => Ok(b"vector".to_vec()), // Placeholder
            Content::Multimodal(_) => Ok(b"multimodal".to_vec()), // Placeholder
            Content::Binary(data) => Ok(data.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_memvid_basic() {
        let path = "test_real_basic.mv2";

        // Create store
        let store = MemvidStoreImpl::create(path).await.unwrap();

        // Add a memory
        let memory = Memory {
            id: MemoryId::from_string("test-1".to_string()),
            content: Content::text("Hello from real MemVid!"),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        };

        store.add(&memory).await.unwrap();

        // Get it back
        let retrieved = store.get(&memory.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id.as_str(), "test-1");

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_real_memvid_count() {
        let path = "test_real_count.mv2";

        let store = MemvidStoreImpl::create(path).await.unwrap();

        // Add 5 memories
        for i in 0..5 {
            let memory = Memory {
                id: MemoryId::from_string(format!("count-{}", i)),
                content: Content::text(&format!("Memory {}", i)),
                attributes: AttributeSet::new(),
                relations: Default::default(),
                metadata: MetadataV4::default(),
            };
            store.add(&memory).await.unwrap();
        }

        // Count
        let count = store.count().await.unwrap();
        assert_eq!(count, 5);

        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}

// ============================================================================
// Public Facade: MemvidStore (实现 MemoryProvider trait)
// ============================================================================

use agent_mem_traits::{
    AgentMemError, HistoryEntry, MemoryEvent, MemoryItem, MemoryProvider, Message, Session,
};
use agent_mem_traits::{Entity, MemoryType, Relation};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// MemVid 存储的 Public Facade
///
/// 实现 `MemoryProvider` trait，提供标准的存储接口。
/// 通过适配器模式，将 trait 调用转换为内部实现。
///
/// ## 架构优势
///
/// - **依赖倒置**: 用户代码依赖 `MemoryProvider` trait，而非具体实现
/// - **高内聚**: `MemvidStoreImpl` 专注于 MemVid API 交互
/// - **低耦合**: 可以轻松替换为其他存储实现（SQLite, PostgreSQL 等）
/// - **可测试**: 通过 mock `MemoryProvider` trait 进行单元测试
///
/// ## 使用示例
///
/// ```no_run
/// use agent_mem_memvid::MemvidStore;
/// use agent_mem_traits::MemoryProvider;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // 创建存储实例
/// let store = MemvidStore::create("memory.mv2").await?;
///
/// // 使用 trait 接口（依赖抽象）
/// let messages = vec![/* ... */];
/// let session = Session::default();
/// let memories = store.add(&messages, &session).await?;
/// # Ok(())
/// # }
/// ```
pub struct MemvidStore {
    /// 内部实现
    inner: MemvidStoreImpl,
}

impl MemvidStore {
    /// 创建新的 MemVid 文件
    pub async fn create(path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            inner: MemvidStoreImpl::create(path).await?,
        })
    }

    /// 打开已存在的 MemVid 文件
    pub async fn open(path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            inner: MemvidStoreImpl::open(path).await?,
        })
    }

    /// 获取内部实现的引用（用于高级操作）
    pub fn inner(&self) -> &MemvidStoreImpl {
        &self.inner
    }

    /// 获取可变内部实现的引用（用于高级操作）
    pub fn inner_mut(&mut self) -> &mut MemvidStoreImpl {
        &mut self.inner
    }

    // ========================================================================
    // Direct API: 直接调用内部实现
    // ========================================================================

    /// 添加单个记忆
    pub async fn add(&self, memory: &Memory) -> Result<()> {
        self.inner.add(memory).await
    }

    /// 获取单个记忆
    pub async fn get(&self, id: &MemoryId) -> Result<Option<Memory>> {
        self.inner.get(id).await
    }

    /// 搜索记忆
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchHit>> {
        self.inner.search(query, top_k).await
    }

    /// 获取记忆数量
    pub async fn count(&self) -> Result<usize> {
        self.inner.count().await
    }

    // ========================================================================
    // 辅助方法：类型转换和适配
    // ========================================================================

    /// 将 Message 转换为 Memory
    fn message_to_memory(&self, msg: &Message, _session: &Session) -> Memory {
        use uuid::Uuid;
        Memory {
            id: MemoryId::from_string(Uuid::new_v4().to_string()),
            content: Content::text(&msg.content),
            attributes: AttributeSet::new(),
            relations: Default::default(),
            metadata: MetadataV4 {
                created_at: msg.timestamp.unwrap_or_else(|| Utc::now()),
                ..Default::default()
            },
        }
    }

    /// 将 Memory 转换为 MemoryItem（向后兼容）
    fn memory_to_item(&self, mem: Memory) -> MemoryItem {
        // 注意：MemoryItem 已被标记为 deprecated
        // 这里提供转换以保持向后兼容
        let created_at = mem.metadata.created_at;
        let updated_at = mem.metadata.updated_at;
        let metadata_map = if let Ok(value) = serde_json::to_value(mem.metadata) {
            if let Some(obj) = value.as_object() {
                obj.into_iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            } else {
                std::collections::HashMap::new()
            }
        } else {
            std::collections::HashMap::new()
        };

        MemoryItem {
            id: mem.id.as_str().to_string(),
            content: mem.content.to_string(),
            hash: None,
            metadata: metadata_map,
            score: None,
            created_at,
            updated_at: Some(updated_at),
            session: Session::default(),
            memory_type: MemoryType::Semantic,
            entities: vec![],
            relations: vec![],
            agent_id: "memvid".to_string(),
            user_id: None,
            importance: 0.5,
            embedding: None,
            last_accessed_at: Utc::now(),
            access_count: 0,
            expires_at: None,
            version: 1,
        }
    }

    /// 将 SearchHit 转换为 MemoryItem
    fn search_hit_to_item(&self, hit: SearchHit) -> MemoryItem {
        // Extract memory ID from URI
        let id = hit
            .uri
            .strip_prefix("mv2://memory/")
            .unwrap_or(&hit.uri)
            .to_string();

        // Calculate importance from optional score, defaulting to 0.5
        let importance = hit
            .score
            .map(|s| s.max(0.0).min(1.0))
            .unwrap_or(0.5);

        MemoryItem {
            id,
            content: hit.text,
            hash: None,
            metadata: std::collections::HashMap::new(),
            score: hit.score,
            created_at: Utc::now(),
            updated_at: None,
            session: Session::default(),
            memory_type: MemoryType::Semantic,
            entities: vec![],
            relations: vec![],
            agent_id: "memvid".to_string(),
            user_id: None,
            importance,
            embedding: None,
            last_accessed_at: Utc::now(),
            access_count: 0,
            expires_at: None,
            version: 1,
        }
    }

    /// 应用 session 隔离（通过 URI prefix）
    fn apply_session_isolation(&self, uri: &str, session: &Session) -> String {
        // 使用 session id 作为 URI prefix 实现隔离
        if session.id.is_empty() {
            uri.to_string()
        } else {
            format!(
                "mv2://session/{}/{}",
                session.id,
                uri.strip_prefix("mv2://").unwrap_or(uri)
            )
        }
    }
}

/// 实现 MemoryProvider trait
///
/// 这是核心的适配器层，将 `MemoryProvider` trait 的标准接口
/// 转换为 MemVid 特定的操作。
#[async_trait]
impl MemoryProvider for MemvidStore {
    /// 添加新记忆
    async fn add(
        &self,
        messages: &[Message],
        session: &Session,
    ) -> std::result::Result<Vec<MemoryItem>, AgentMemError> {
        let mut results = Vec::new();

        for msg in messages {
            // 1. Message → Memory 转换
            let memory = self.message_to_memory(msg, session);

            // 2. 调用内部实现添加，转换错误类型
            self.inner
                .add(&memory)
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to add memory: {}", e)))?;

            // 3. Memory → MemoryItem 转换（返回值）
            results.push(self.memory_to_item(memory));
        }

        Ok(results)
    }

    /// 获取特定记忆
    async fn get(&self, id: &str) -> std::result::Result<Option<MemoryItem>, AgentMemError> {
        let memory_id = MemoryId::from_string(id.to_string());

        match self
            .inner
            .get(&memory_id)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get memory: {}", e)))?
        {
            Some(memory) => Ok(Some(self.memory_to_item(memory))),
            None => Ok(None),
        }
    }

    /// 搜索记忆
    async fn search(
        &self,
        query: &str,
        _session: &Session,
        limit: usize,
    ) -> std::result::Result<Vec<MemoryItem>, AgentMemError> {
        // 注意：当前搜索不区分 session（session 隔离需要在查询时应用）
        // TODO: 实现基于 session 的过滤
        let hits = self
            .inner
            .search(query, limit)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to search: {}", e)))?;

        Ok(hits
            .into_iter()
            .map(|hit| self.search_hit_to_item(hit))
            .collect())
    }

    /// 更新记忆
    async fn update(&self, id: &str, data: &str) -> std::result::Result<(), AgentMemError> {
        let memory_id = MemoryId::from_string(id.to_string());

        // 获取现有记忆
        if let Some(mut existing) = self
            .inner
            .get(&memory_id)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get memory: {}", e)))?
        {
            // 更新内容
            existing.content = Content::text(data);

            // 写回
            self.inner.update(&existing).await.map_err(|e| {
                AgentMemError::StorageError(format!("Failed to update memory: {}", e))
            })?;
        }

        Ok(())
    }

    /// 删除记忆
    async fn delete(&self, id: &str) -> std::result::Result<(), AgentMemError> {
        let memory_id = MemoryId::from_string(id.to_string());
        self.inner
            .delete(&memory_id)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to delete memory: {}", e)))
    }

    /// 获取记忆历史
    async fn history(&self, id: &str) -> std::result::Result<Vec<HistoryEntry>, AgentMemError> {
        // MemVid 支持版本历史，这里提供一个基本实现
        let memory_id = MemoryId::from_string(id.to_string());

        // 尝试获取版本信息
        match self.inner.get_version_info(&memory_id).await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get version info: {}", e))
        })? {
            Some(version_info) => {
                // 转换为 HistoryEntry
                let entry = HistoryEntry {
                    id: Uuid::new_v4().to_string(),
                    memory_id: id.to_string(),
                    event: MemoryEvent::Update,
                    timestamp: Utc::now(),
                    data: Some(serde_json::json!({
                        "version": version_info.version,
                        "timestamp": version_info.timestamp
                    })),
                };
                Ok(vec![entry])
            }
            None => Ok(vec![]),
        }
    }

    /// 获取 session 的所有记忆
    async fn get_all(
        &self,
        _session: &Session,
    ) -> std::result::Result<Vec<MemoryItem>, AgentMemError> {
        // 注意：当前实现获取所有记忆，不区分 session
        // TODO: 实现基于 session 的过滤
        let count = self
            .inner
            .count()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to count: {}", e)))?;

        // 简化实现：返回最近的一些记忆
        // 实际应用中应该实现完整的分页和过滤
        if count == 0 {
            return Ok(vec![]);
        }

        // 获取前 100 个记忆（示例）
        let hits = self
            .inner
            .search("*", count.min(100))
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to search: {}", e)))?;

        Ok(hits
            .into_iter()
            .map(|hit| self.search_hit_to_item(hit))
            .collect())
    }

    /// 重置所有记忆（用于测试）
    async fn reset(&self) -> std::result::Result<(), AgentMemError> {
        self.inner
            .clear()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to clear: {}", e)))
    }
}
