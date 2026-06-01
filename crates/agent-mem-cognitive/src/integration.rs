//! Integration tests for AgentMem Cognitive
//! 
//! Tests the integration between all components

use crate::{ConfigManager, UnifiedMemoryManager, InMemoryStorage, StorageManager, StoredMemory};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryTier;
    use crate::TieredMemoryItem;

    #[tokio::test]
    async fn test_config_and_manager_integration() {
        let config = crate::UnifiedConfig::default();
        let manager = UnifiedMemoryManager::new(config);
        
        manager.add("id1".into(), "Hello world".into(), 0.8);
        
        let result = manager.access("id1");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Hello world");
    }
    
    #[tokio::test]
    async fn test_storage_integration() {
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
        
        manager.save_memory(&memory).await.unwrap();
        
        let loaded = manager.load_memory("test1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().content, "Test content");
    }
    
    #[test]
    fn test_config_serialization_roundtrip() {
        let manager = ConfigManager::with_defaults();
        let json = manager.to_json().unwrap();
        let loaded = ConfigManager::from_json(&json).unwrap();
        
        assert_eq!(
            manager.config().hierarchy.working_capacity,
            loaded.config().hierarchy.working_capacity
        );
    }
    
    #[tokio::test]
    async fn test_memory_search_across_tiers() {
        let config = crate::UnifiedConfig::default();
        let manager = UnifiedMemoryManager::new(config);
        
        manager.add("rust1".into(), "Rust programming".into(), 0.9);
        manager.add("python1".into(), "Python programming".into(), 0.7);
        manager.add("go1".into(), "Go programming".into(), 0.6);
        
        let results = manager.search("programming", 10);
        assert_eq!(results.len(), 3);
    }
    
    #[tokio::test]
    async fn test_memory_access_tracking() {
        let config = crate::UnifiedConfig::default();
        let manager = UnifiedMemoryManager::new(config);
        
        manager.add("test1".into(), "Test content".into(), 0.5);
        
        // Access multiple times
        manager.access("test1");
        manager.access("test1");
        manager.access("test1");
        
        // Should still be accessible
        let result = manager.access("test1");
        assert!(result.is_some());
    }
    
    #[test]
    fn test_tiered_memory_item_creation() {
        let item = TieredMemoryItem::new(
            "id1".to_string(),
            "Test content".to_string(),
            MemoryTier::Working,
        );
        
        assert_eq!(item.tier, MemoryTier::Working);
        assert_eq!(item.importance, 0.5); // default
        assert_eq!(item.access_count, 0);
    }
    
    #[tokio::test]
    async fn test_storage_delete() {
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
        
        manager.save_memory(&memory).await.unwrap();
        manager.delete_memory("test1").await.unwrap();
        
        let loaded = manager.load_memory("test1").await.unwrap();
        assert!(loaded.is_none());
    }
    
    #[tokio::test]
    async fn test_storage_list() {
        let storage = InMemoryStorage::new();
        let manager = StorageManager::new(storage);
        
        for i in 0..5 {
            let memory = StoredMemory {
                id: format!("test{}", i),
                tier: "Working".to_string(),
                importance: 0.8,
                access_count: 5,
                last_accessed: 1234567890,
                content: format!("Content {}", i),
                archived_at: None,
            };
            manager.save_memory(&memory).await.unwrap();
        }
        
        let keys = manager.list_memories().await.unwrap();
        assert_eq!(keys.len(), 5);
    }
    
    #[test]
    fn test_config_validation() {
        let manager = ConfigManager::with_defaults();
        assert!(manager.validate().is_ok());
    }
    
    #[test]
    fn test_config_validation_failure() {
        let mut manager = ConfigManager::with_defaults();
        manager.config_mut().hierarchy.working_capacity = 0;
        assert!(manager.validate().is_err());
    }
}
