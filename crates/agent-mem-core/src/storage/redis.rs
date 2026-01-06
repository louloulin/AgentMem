// Redis cache backend implementation

use async_trait::async_trait;
use chrono::Utc;
use redis::{AsyncCommands, Client, RedisResult};
use serde_json;
use std::collections::HashMap;

use super::{CacheBackend, CacheStatistics, RedisConfig};
use crate::hierarchy::HierarchicalMemory;
use crate::{CoreError, CoreResult};

/// Redis cache backend
pub struct RedisCache {
    client: Client,
    config: RedisConfig,
}

impl RedisCache {
    /// Create new Redis cache backend
    pub async fn new(config: RedisConfig) -> CoreResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| CoreError::CacheError(format!("Failed to create Redis client: {}", e)))?;

        // Test connection
        let mut conn = client
            .get_async_connection()
            .await
            .map_err(|e| CoreError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis ping failed: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Get Redis connection
    async fn get_connection(&self) -> CoreResult<redis::aio::Connection> {
        self.client
            .get_async_connection()
            .await
            .map_err(|e| CoreError::CacheError(format!("Failed to get Redis connection: {}", e)))
    }

    /// Serialize memory to JSON string
    fn serialize_memory(&self, memory: &HierarchicalMemory) -> CoreResult<String> {
        serde_json::to_string(memory).map_err(|e| {
            CoreError::SerializationError(format!("Failed to serialize memory: {}", e))
        })
    }

    /// Deserialize memory from JSON string
    fn deserialize_memory(&self, data: &str) -> CoreResult<HierarchicalMemory> {
        serde_json::from_str(data).map_err(|e| {
            CoreError::SerializationError(format!("Failed to deserialize memory: {}", e))
        })
    }

    /// Generate cache key for memory
    fn cache_key(&self, key: &str) -> String {
        format!("agentmem:memory:{}", key)
    }
}

#[async_trait]
impl CacheBackend for RedisCache {
    async fn get(&self, key: &str) -> CoreResult<Option<HierarchicalMemory>> {
        let mut conn = self.get_connection().await?;
        let cache_key = self.cache_key(key);

        let result: RedisResult<String> = conn.get(&cache_key).await;

        match result {
            Ok(data) => {
                let memory = self.deserialize_memory(&data)?;
                Ok(Some(memory))
            }
            Err(e) if e.kind() == redis::ErrorKind::TypeError => {
                // Key doesn't exist
                Ok(None)
            }
            Err(e) => Err(CoreError::CacheError(format!("Redis get failed: {}", e))),
        }
    }

    async fn set(
        &self,
        key: &str,
        memory: &HierarchicalMemory,
        ttl: Option<u64>,
    ) -> CoreResult<()> {
        let mut conn = self.get_connection().await?;
        let cache_key = self.cache_key(key);
        let data = self.serialize_memory(memory)?;
        let ttl = ttl.unwrap_or(self.config.default_ttl);

        let _: () = conn
            .set_ex(&cache_key, data, ttl)
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis set failed: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> CoreResult<bool> {
        let mut conn = self.get_connection().await?;
        let cache_key = self.cache_key(key);

        let result: i32 = conn
            .del(&cache_key)
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis delete failed: {}", e)))?;

        Ok(result > 0)
    }

    async fn exists(&self, key: &str) -> CoreResult<bool> {
        let mut conn = self.get_connection().await?;
        let cache_key = self.cache_key(key);

        let result: bool = conn
            .exists(&cache_key)
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis exists failed: {}", e)))?;

        Ok(result)
    }

    async fn mset(&self, entries: HashMap<String, HierarchicalMemory>) -> CoreResult<()> {
        let mut conn = self.get_connection().await?;
        let mut pipe = redis::pipe();

        for (key, memory) in entries {
            let cache_key = self.cache_key(&key);
            let data = self.serialize_memory(&memory)?;
            pipe.set_ex(&cache_key, data, self.config.default_ttl);
        }

        let _: () = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis mset failed: {}", e)))?;

        Ok(())
    }

    async fn mget(&self, keys: &[String]) -> CoreResult<HashMap<String, HierarchicalMemory>> {
        let mut conn = self.get_connection().await?;
        let cache_keys: Vec<String> = keys.iter().map(|k| self.cache_key(k)).collect();

        let results: Vec<Option<String>> = conn
            .get(&cache_keys)
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis mget failed: {}", e)))?;

        let mut memories = HashMap::new();
        for (i, result) in results.into_iter().enumerate() {
            if let Some(data) = result {
                let memory = self.deserialize_memory(&data)?;
                memories.insert(keys[i].clone(), memory);
            }
        }

        Ok(memories)
    }

    async fn clear(&self) -> CoreResult<()> {
        let mut conn = self.get_connection().await?;

        // Get all keys matching our pattern
        let keys: Vec<String> = conn
            .keys("agentmem:memory:*")
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis keys failed: {}", e)))?;

        if !keys.is_empty() {
            let _: i32 = conn
                .del(&keys)
                .await
                .map_err(|e| CoreError::CacheError(format!("Redis clear failed: {}", e)))?;
        }

        Ok(())
    }

    async fn get_cache_stats(&self) -> CoreResult<CacheStatistics> {
        let mut conn = self.get_connection().await?;

        // Get Redis info
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis info failed: {}", e)))?;

        // Parse memory usage from info
        let memory_usage = info
            .lines()
            .find(|line| line.starts_with("used_memory:"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        // Count our keys
        let keys: Vec<String> = conn
            .keys("agentmem:memory:*")
            .await
            .map_err(|e| CoreError::CacheError(format!("Redis keys failed: {}", e)))?;

        let total_entries = keys.len() as u64;

        // Calculate cache size (approximate)
        let mut cache_size = 0u64;
        if !keys.is_empty() {
            // Sample a few keys to estimate average size
            let sample_size = std::cmp::min(10, keys.len());
            let sample_keys = &keys[0..sample_size];

            let values: Vec<Option<String>> = conn
                .get(sample_keys)
                .await
                .map_err(|e| CoreError::CacheError(format!("Redis sample get failed: {}", e)))?;

            let total_sample_size: usize =
                values.into_iter().filter_map(|v| v).map(|v| v.len()).sum();

            if sample_size > 0 {
                let avg_size = total_sample_size / sample_size;
                cache_size = (avg_size * keys.len()) as u64;
            }
        }

        // Note: Redis doesn't provide hit/miss statistics by default
        // In a production environment, you might want to implement
        // custom counters or use Redis modules like RedisInsight
        Ok(CacheStatistics {
            total_entries,
            hit_rate: 0.0,   // Would need custom tracking
            miss_rate: 0.0,  // Would need custom tracking
            total_hits: 0,   // Would need custom tracking
            total_misses: 0, // Would need custom tracking
            cache_size,
            memory_usage,
            last_updated: Utc::now(),
        })
    }
}
