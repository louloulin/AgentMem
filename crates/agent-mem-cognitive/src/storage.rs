//! Storage Backend for AgentMem Cognitive
//! 
//! Provides persistence for memory data

use crate::error::{MemoryError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Storage backend trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Save data
    async fn save(&self, key: &str, data: &[u8]) -> Result<()>;
    
    /// Load data
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Delete data
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// List all keys with prefix
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
    
    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// Get all keys
    async fn keys(&self) -> Result<Vec<String>>;
    
    /// Clear all data
    async fn clear(&self) -> Result<()>;
}

/// In-memory storage backend (for testing/development)
pub struct InMemoryStorage {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorage {
    async fn save(&self, key: &str, data: &[u8]) -> Result<()> {
        let mut storage = self.data.write().await;
        storage.insert(key.to_string(), data.to_vec());
        Ok(())
    }
    
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let storage = self.data.read().await;
        Ok(storage.get(key).cloned())
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let mut storage = self.data.write().await;
        storage.remove(key);
        Ok(())
    }
    
    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let storage = self.data.read().await;
        let keys: Vec<String> = storage
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        let storage = self.data.read().await;
        Ok(storage.contains_key(key))
    }
    
    async fn keys(&self) -> Result<Vec<String>> {
        let storage = self.data.read().await;
        Ok(storage.keys().cloned().collect())
    }
    
    async fn clear(&self) -> Result<()> {
        let mut storage = self.data.write().await;
        storage.clear();
        Ok(())
    }
}

/// File-based storage backend
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
    
    fn key_to_path(&self, key: &str) -> PathBuf {
        // Sanitize key for filesystem
        let sanitized = key.replace(['/', '\\', ':'], "_");
        self.base_path.join(format!("{}.json", sanitized))
    }
}

#[async_trait]
impl StorageBackend for FileStorage {
    async fn save(&self, key: &str, data: &[u8]) -> Result<()> {
        let path = self.key_to_path(key);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                MemoryError::storage(format!("Failed to create directory: {}", e))
            })?;
        }
        
        fs::write(&path, data).await.map_err(|e| {
            MemoryError::storage(format!("Failed to write file: {}", e))
        })?;
        
        Ok(())
    }
    
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.key_to_path(key);
        
        match fs::read(&path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::storage(format!("Failed to read file: {}", e))),
        }
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.key_to_path(key);
        
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(MemoryError::storage(format!("Failed to delete file: {}", e))),
        }
    }
    
    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut results = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await.map_err(|e| {
            MemoryError::storage(format!("Failed to read directory: {}", e))
        })?;
        
        let prefix_sanitized = prefix.replace(['/', '\\', ':'], "_");
        
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            MemoryError::storage(format!("Failed to read entry: {}", e))
        })? {
            let name = entry.file_name().to_string_lossy().to_string();
            // Remove .json extension and check prefix
            if let Some(stem) = name.strip_suffix(".json") {
                if stem.starts_with(&prefix_sanitized) {
                    // Restore original key format
                    let key = stem.replace('_', "/");
                    results.push(key);
                }
            }
        }
        
        Ok(results)
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }
    
    async fn keys(&self) -> Result<Vec<String>> {
        self.list("").await
    }
    
    async fn clear(&self) -> Result<()> {
        for key in self.list("").await? {
            self.delete(&key).await?;
        }
        Ok(())
    }
}

/// Memory item for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMemory {
    pub id: String,
    pub tier: String,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: i64,
    pub content: String,
    pub archived_at: Option<i64>,
}

/// Storage manager for unified memory
pub struct StorageManager<S: StorageBackend> {
    backend: Arc<S>,
}

impl<S: StorageBackend> StorageManager<S> {
    pub fn new(backend: S) -> Self {
        Self {
            backend: Arc::new(backend),
        }
    }
    
    /// Save a memory item
    pub async fn save_memory(&self, memory: &StoredMemory) -> Result<()> {
        let key = format!("memory:{}", memory.id);
        let data = serde_json::to_vec(memory)
            .map_err(|e| MemoryError::serialization(format!("Failed to serialize: {}", e)))?;
        self.backend.save(&key, &data).await
    }
    
    /// Load a memory item
    pub async fn load_memory(&self, id: &str) -> Result<Option<StoredMemory>> {
        let key = format!("memory:{}", id);
        match self.backend.load(&key).await? {
            Some(data) => {
                let memory = serde_json::from_slice(&data)
                    .map_err(|e| MemoryError::serialization(format!("Failed to deserialize: {}", e)))?;
                Ok(Some(memory))
            }
            None => Ok(None),
        }
    }
    
    /// Delete a memory item
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        let key = format!("memory:{}", id);
        self.backend.delete(&key).await
    }
    
    /// List all memory IDs
    pub async fn list_memories(&self) -> Result<Vec<String>> {
        self.backend.list("memory:").await
    }
    
    /// Get the backend
    pub fn backend(&self) -> Arc<S> {
        Arc::clone(&self.backend)
    }
}

/// Type alias for InMemoryStorage manager
pub type InMemoryStorageManager = StorageManager<InMemoryStorage>;
/// Type alias for FileStorage manager
pub type FileStorageManager = StorageManager<FileStorage>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryStorage::new();
        
        // Test save and load
        storage.save("key1", b"value1").await.unwrap();
        let value = storage.load("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
        
        // Test list
        storage.save("prefix:key2", b"value2").await.unwrap();
        let keys = storage.list("prefix:").await.unwrap();
        assert!(keys.contains(&"prefix:key2".to_string()));
        
        // Test exists
        assert!(storage.exists("key1").await.unwrap());
        assert!(!storage.exists("nonexistent").await.unwrap());
        
        // Test delete
        storage.delete("key1").await.unwrap();
        assert!(storage.load("key1").await.unwrap().is_none());
        
        // Test clear
        storage.clear().await.unwrap();
        assert!(storage.keys().await.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_storage_manager() {
        let storage = InMemoryStorage::new();
        let manager = StorageManager::new(storage);
        
        let memory = StoredMemory {
            id: "test1".to_string(),
            tier: "Working".to_string(),
            importance: 0.8,
            access_count: 5,
            last_accessed: 1234567890,
            content: "Test content".to_string(),
            archived_at: None,
        };
        
        // Save
        manager.save_memory(&memory).await.unwrap();
        
        // Load
        let loaded = manager.load_memory("test1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().content, "Test content");
        
        // List
        let ids = manager.list_memories().await.unwrap();
        assert!(ids.contains(&"memory:test1".to_string()));
    }
}
