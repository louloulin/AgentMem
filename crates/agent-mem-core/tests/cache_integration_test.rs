//! Integration tests for cache module

use agent_mem_core::cache::{
    Cache, CacheConfig, CacheWarmer, CacheWarmingConfig, DataLoader, MemoryCache,
    MemoryCacheConfig, MultiLevelCache, MultiLevelCacheConfig, WarmingStrategy,
};
use agent_mem_traits::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_memory_cache_basic_operations() {
    let cache = MemoryCache::new(MemoryCacheConfig::default());

    // Test set and get
    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();

    let value = cache.get(&"key1".to_string()).await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));

    // Test miss
    let value = cache.get(&"nonexistent".to_string()).await.unwrap();
    assert_eq!(value, None);

    // Test delete
    let deleted = cache.delete(&"key1".to_string()).await.unwrap();
    assert!(deleted);

    let value = cache.get(&"key1".to_string()).await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_memory_cache_ttl() {
    let config = MemoryCacheConfig {
        max_entries: 100,
        max_size_bytes: 1024 * 1024,
        default_ttl: Duration::from_secs(1), // 1 second TTL
        enable_stats: true,
    };

    let cache = MemoryCache::new(config);

    // Set with short TTL
    cache
        .set(
            "key1".to_string(),
            b"value1".to_vec(),
            Some(Duration::from_secs(1)),
        )
        .await
        .unwrap();

    // Should exist immediately
    let value = cache.get(&"key1".to_string()).await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));

    // Wait for expiration (1.5 seconds to be safe)
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Should be expired
    let value = cache.get(&"key1".to_string()).await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_memory_cache_stats() {
    let cache = MemoryCache::new(MemoryCacheConfig::default());

    // Perform operations
    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();
    cache
        .set("key2".to_string(), b"value2".to_vec(), None)
        .await
        .unwrap();

    cache.get(&"key1".to_string()).await.unwrap(); // hit
    cache.get(&"key2".to_string()).await.unwrap(); // hit
    cache.get(&"key3".to_string()).await.unwrap(); // miss

    let stats = cache.stats().await.unwrap();

    assert_eq!(stats.total_sets, 2);
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.total_gets, 3);
    assert_eq!(stats.hit_rate(), 66.66666666666666);
}

#[tokio::test]
async fn test_memory_cache_eviction() {
    let config = MemoryCacheConfig {
        max_entries: 3,
        max_size_bytes: 1024 * 1024,
        default_ttl: Duration::from_secs(60),
        enable_stats: true,
    };

    let cache = MemoryCache::new(config);

    // Fill cache to capacity
    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();
    cache
        .set("key2".to_string(), b"value2".to_vec(), None)
        .await
        .unwrap();
    cache
        .set("key3".to_string(), b"value3".to_vec(), None)
        .await
        .unwrap();

    // Access key1 to make it more recently used
    cache.get(&"key1".to_string()).await.unwrap();

    // Add one more entry, should evict LRU (key2 or key3)
    cache
        .set("key4".to_string(), b"value4".to_vec(), None)
        .await
        .unwrap();

    // key1 should still exist (recently accessed)
    let value = cache.get(&"key1".to_string()).await.unwrap();
    assert!(value.is_some());

    // key4 should exist (just added)
    let value = cache.get(&"key4".to_string()).await.unwrap();
    assert!(value.is_some());

    let stats = cache.stats().await.unwrap();
    assert!(stats.evictions > 0);
}

#[tokio::test]
async fn test_multi_level_cache_l1_only() {
    let config = MultiLevelCacheConfig {
        enable_l1: true,
        enable_l2: false,
        ..Default::default()
    };

    let cache = MultiLevelCache::new(config);

    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();

    let value = cache.get(&"key1".to_string()).await.unwrap();
    assert_eq!(value, Some(b"value1".to_vec()));
}

#[tokio::test]
async fn test_multi_level_cache_stats() {
    let config = MultiLevelCacheConfig::default();
    let cache = MultiLevelCache::new(config);

    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();
    cache.get(&"key1".to_string()).await.unwrap();

    let stats = cache.stats().await.unwrap();
    assert!(stats.total_sets > 0);
    assert!(stats.hits > 0);
}

#[tokio::test]
async fn test_multi_level_cache_clear() {
    let config = MultiLevelCacheConfig::default();
    let cache = MultiLevelCache::new(config);

    cache
        .set("key1".to_string(), b"value1".to_vec(), None)
        .await
        .unwrap();
    cache
        .set("key2".to_string(), b"value2".to_vec(), None)
        .await
        .unwrap();

    cache.clear().await.unwrap();

    let value1 = cache.get(&"key1".to_string()).await.unwrap();
    let value2 = cache.get(&"key2".to_string()).await.unwrap();

    assert_eq!(value1, None);
    assert_eq!(value2, None);
}

// Mock data loader for testing
struct MockDataLoader;

#[async_trait::async_trait]
impl DataLoader for MockDataLoader {
    async fn load_data(&self, keys: Vec<String>) -> Result<HashMap<String, Vec<u8>>> {
        let mut data = HashMap::new();
        for key in keys {
            data.insert(key.clone(), format!("value_{}", key).into_bytes());
        }
        Ok(data)
    }

    async fn get_frequent_keys(&self, limit: usize) -> Result<Vec<String>> {
        Ok((0..limit).map(|i| format!("key_{}", i)).collect())
    }

    async fn get_all_keys(&self, limit: usize) -> Result<Vec<String>> {
        Ok((0..limit).map(|i| format!("key_{}", i)).collect())
    }
}

#[tokio::test]
async fn test_cache_warmer_eager() {
    let cache = Arc::new(MemoryCache::new(MemoryCacheConfig::default()));
    let loader = Arc::new(MockDataLoader);
    let config = CacheWarmingConfig {
        strategy: WarmingStrategy::Eager,
        max_items: 10,
        batch_size: 5,
        enable_stats: true,
    };

    let warmer = CacheWarmer::new(cache.clone(), loader, config);
    warmer.start().await.unwrap();

    // Check that keys were warmed
    let value = cache.get(&"key_0".to_string()).await.unwrap();
    assert!(value.is_some());
    assert_eq!(value.unwrap(), b"value_key_0");

    let stats = warmer.stats().await;
    assert_eq!(stats.total_items_warmed, 10);
    assert_eq!(stats.total_warmings, 1);
}

#[tokio::test]
async fn test_cache_warmer_stats() {
    let cache = Arc::new(MemoryCache::new(MemoryCacheConfig::default()));
    let loader = Arc::new(MockDataLoader);
    let config = CacheWarmingConfig {
        strategy: WarmingStrategy::Eager,
        max_items: 5,
        batch_size: 2,
        enable_stats: true,
    };

    let warmer = CacheWarmer::new(cache.clone(), loader, config);
    warmer.start().await.unwrap();

    let stats = warmer.stats().await;
    assert_eq!(stats.total_items_warmed, 5);
    assert!(stats.average_items_per_warming() > 0.0);
}

#[tokio::test]
async fn test_cache_config_presets() {
    // Test default config
    let default_config = CacheConfig::default();
    assert!(default_config.enable_l1);
    assert!(!default_config.enable_l2);

    // Test production config
    let prod_config = CacheConfig::production();
    assert!(prod_config.enable_l1);
    assert!(prod_config.enable_l2);
    assert_eq!(prod_config.l1_max_entries, 50000);

    // Test development config
    let dev_config = CacheConfig::development();
    assert!(dev_config.enable_l1);
    assert!(!dev_config.enable_l2);
    assert_eq!(dev_config.l1_max_entries, 1000);
}
