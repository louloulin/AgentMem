//! Storage capability for plugins

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Storage capability allows plugins to persist data
#[derive(Clone)]
pub struct StorageCapability {
    store: Arc<RwLock<HashMap<String, String>>>,
}

impl StorageCapability {
    /// Create a new storage capability
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a value with a key
    pub async fn set(&self, key: String, value: String) -> Result<()> {
        let mut store = self.store.write().await;
        store.insert(key, value);
        Ok(())
    }

    /// Get a value by key
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let store = self.store.read().await;
        Ok(store.get(key).cloned())
    }

    /// Delete a value by key
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut store = self.store.write().await;
        Ok(store.remove(key).is_some())
    }

    /// List all keys
    pub async fn list_keys(&self) -> Result<Vec<String>> {
        let store = self.store.read().await;
        Ok(store.keys().cloned().collect())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let store = self.store.read().await;
        Ok(store.contains_key(key))
    }

    /// Clear all data
    pub async fn clear(&self) -> Result<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }

    /// Get the number of stored items
    pub async fn count(&self) -> Result<usize> {
        let store = self.store.read().await;
        Ok(store.len())
    }
}

impl Default for StorageCapability {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_set_and_get() {
        let storage = StorageCapability::new();

        storage
            .set("key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        let value = storage.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_storage_delete() {
        let storage = StorageCapability::new();

        storage
            .set("key1".to_string(), "value1".to_string())
            .await
            .unwrap();
        assert!(storage.exists("key1").await.unwrap());

        let deleted = storage.delete("key1").await.unwrap();
        assert!(deleted);
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_storage_list_keys() {
        let storage = StorageCapability::new();

        storage
            .set("key1".to_string(), "value1".to_string())
            .await
            .unwrap();
        storage
            .set("key2".to_string(), "value2".to_string())
            .await
            .unwrap();
        storage
            .set("key3".to_string(), "value3".to_string())
            .await
            .unwrap();

        let keys = storage.list_keys().await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[tokio::test]
    async fn test_storage_clear() {
        let storage = StorageCapability::new();

        storage
            .set("key1".to_string(), "value1".to_string())
            .await
            .unwrap();
        storage
            .set("key2".to_string(), "value2".to_string())
            .await
            .unwrap();

        assert_eq!(storage.count().await.unwrap(), 2);

        storage.clear().await.unwrap();
        assert_eq!(storage.count().await.unwrap(), 0);
    }
}
