// Storage module for AgentMem
// Provides different storage backends for persistent memory storage

// Repository traits (always available)
pub mod traits;

// Models (always available for LibSQL and PostgreSQL)
pub mod models;

// Repository factory (always available)
pub mod factory;

// LibSQL backend (default, embedded mode)
#[cfg(feature = "libsql")]
pub mod libsql;

// PostgreSQL repositories (optional, enterprise mode)
#[cfg(feature = "postgres")]
pub mod agent_repository;
#[cfg(feature = "postgres")]
pub mod api_key_repository;
#[cfg(feature = "postgres")]
pub mod batch;
#[cfg(feature = "postgres")]
pub mod block_repository;
#[cfg(all(feature = "postgres", feature = "redis-cache"))]
pub mod hybrid_manager;
#[cfg(feature = "postgres")]
pub mod memory_repository;
#[cfg(feature = "postgres")]
pub mod message_repository;
#[cfg(feature = "postgres")]
pub mod migration;
#[cfg(feature = "postgres")]
pub mod migration_manager;
#[cfg(feature = "postgres")]
pub mod migrations;
#[cfg(feature = "postgres")]
pub mod pool_manager;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub mod query_analyzer;
#[cfg(feature = "redis-cache")]
pub mod redis;
#[cfg(feature = "postgres")]
pub mod repository;
#[cfg(feature = "postgres")]
pub mod tool_repository;
#[cfg(feature = "postgres")]
pub mod transaction;
#[cfg(feature = "postgres")]
pub mod user_repository;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::hierarchy::{HierarchicalMemory, MemoryLevel, MemoryScope};
use crate::{types::*, CoreResult};

/// Storage backend trait for persistent memory storage
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Initialize the storage backend
    async fn initialize(&self) -> CoreResult<()>;

    /// Store a memory in the backend
    async fn store_memory(&self, memory: &HierarchicalMemory) -> CoreResult<()>;

    /// Retrieve a memory by ID
    async fn get_memory(&self, id: &str) -> CoreResult<Option<HierarchicalMemory>>;

    /// Update an existing memory
    async fn update_memory(&self, memory: &HierarchicalMemory) -> CoreResult<()>;

    /// Delete a memory by ID
    async fn delete_memory(&self, id: &str) -> CoreResult<bool>;

    /// Search memories by query
    async fn search_memories(
        &self,
        query: &str,
        scope: Option<MemoryScope>,
        level: Option<MemoryLevel>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>>;

    /// Get memories by scope
    async fn get_memories_by_scope(
        &self,
        scope: MemoryScope,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>>;

    /// Get memories by level
    async fn get_memories_by_level(
        &self,
        level: MemoryLevel,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>>;

    /// Get memory statistics
    async fn get_statistics(&self) -> CoreResult<StorageStatistics>;

    /// Perform health check
    async fn health_check(&self) -> CoreResult<HealthStatus>;
}

/// Cache backend trait for fast memory access
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get a memory from cache
    async fn get(&self, key: &str) -> CoreResult<Option<HierarchicalMemory>>;

    /// Set a memory in cache
    async fn set(&self, key: &str, memory: &HierarchicalMemory, ttl: Option<u64>)
        -> CoreResult<()>;

    /// Delete a memory from cache
    async fn delete(&self, key: &str) -> CoreResult<bool>;

    /// Check if key exists in cache
    async fn exists(&self, key: &str) -> CoreResult<bool>;

    /// Set multiple memories in cache
    async fn mset(&self, entries: HashMap<String, HierarchicalMemory>) -> CoreResult<()>;

    /// Get multiple memories from cache
    async fn mget(&self, keys: &[String]) -> CoreResult<HashMap<String, HierarchicalMemory>>;

    /// Clear all cache entries
    async fn clear(&self) -> CoreResult<()>;

    /// Get cache statistics
    async fn get_cache_stats(&self) -> CoreResult<CacheStatistics>;
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// PostgreSQL configuration
    #[cfg(feature = "postgres")]
    pub postgres: PostgresConfig,
    /// Redis configuration
    #[cfg(feature = "redis-cache")]
    pub redis: RedisConfig,
    /// Cache configuration
    pub cache: CacheConfig,
}

/// PostgreSQL configuration
#[cfg(feature = "postgres")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Query timeout in seconds
    pub query_timeout: u64,
    /// Enable SSL
    pub ssl: bool,
}

/// Redis configuration
#[cfg(feature = "redis-cache")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Default TTL for cache entries in seconds
    pub default_ttl: u64,
    /// Enable cluster mode
    pub cluster: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Default TTL for cache entries in seconds
    pub default_ttl: u64,
    /// Maximum cache size (number of entries)
    pub max_size: usize,
    /// Cache eviction policy
    pub eviction_policy: EvictionPolicy,
}

/// Cache eviction policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Time To Live
    TTL,
    /// First In First Out
    FIFO,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    /// Total number of memories stored
    pub total_memories: u64,
    /// Storage size in bytes
    pub storage_size: u64,
    /// Number of memories by level
    pub memories_by_level: HashMap<MemoryLevel, u64>,
    /// Number of memories by scope
    pub memories_by_scope: HashMap<MemoryScope, u64>,
    /// Average memory size in bytes
    pub average_memory_size: f64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Total number of cache entries
    pub total_entries: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Cache miss rate (0.0 to 1.0)
    pub miss_rate: f64,
    /// Total cache hits
    pub total_hits: u64,
    /// Total cache misses
    pub total_misses: u64,
    /// Cache size in bytes
    pub cache_size: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Is the storage healthy
    pub healthy: bool,
    /// Status message
    pub message: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "postgres")]
            postgres: PostgresConfig::default(),
            #[cfg(feature = "redis-cache")]
            redis: RedisConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

#[cfg(feature = "postgres")]
impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://agentmem:password@localhost:5432/agentmem".to_string(),
            max_connections: 10,
            connection_timeout: 30,
            query_timeout: 30,
            ssl: false,
        }
    }
}

#[cfg(feature = "redis-cache")]
impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            default_ttl: 3600, // 1 hour
            cluster: false,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: 3600, // 1 hour
            max_size: 10000,
            eviction_policy: EvictionPolicy::LRU,
        }
    }
}

/// Hybrid storage manager that combines PostgreSQL and Redis
#[cfg(all(feature = "postgres", feature = "redis-cache"))]
pub struct HybridStorageManager {
    /// PostgreSQL backend for persistent storage
    postgres: Box<dyn StorageBackend>,
    /// Redis backend for caching
    redis: Box<dyn CacheBackend>,
    /// Configuration
    config: StorageConfig,
}

#[cfg(all(feature = "postgres", feature = "redis-cache"))]
impl HybridStorageManager {
    /// Create new hybrid storage manager
    pub fn new(
        postgres: Box<dyn StorageBackend>,
        redis: Box<dyn CacheBackend>,
        config: StorageConfig,
    ) -> Self {
        Self {
            postgres,
            redis,
            config,
        }
    }

    /// Initialize both storage backends
    pub async fn initialize(&self) -> CoreResult<()> {
        self.postgres.initialize().await?;
        Ok(())
    }

    /// Store memory with caching
    pub async fn store_memory(&self, memory: &HierarchicalMemory) -> CoreResult<()> {
        // Store in PostgreSQL first
        self.postgres.store_memory(memory).await?;

        // Cache in Redis if enabled
        if self.config.cache.enabled {
            let _ = self
                .redis
                .set(
                    &memory.memory.id,
                    memory,
                    Some(self.config.cache.default_ttl),
                )
                .await;
        }

        Ok(())
    }

    /// Get memory with cache-first strategy
    pub async fn get_memory(&self, id: &str) -> CoreResult<Option<HierarchicalMemory>> {
        // Try cache first if enabled
        if self.config.cache.enabled {
            if let Ok(Some(memory)) = self.redis.get(id).await {
                return Ok(Some(memory));
            }
        }

        // Fallback to PostgreSQL
        let memory = self.postgres.get_memory(id).await?;

        // Cache the result if found and caching is enabled
        if let (Some(ref mem), true) = (&memory, self.config.cache.enabled) {
            let _ = self
                .redis
                .set(id, mem, Some(self.config.cache.default_ttl))
                .await;
        }

        Ok(memory)
    }

    /// Update memory in both storage and cache
    pub async fn update_memory(&self, memory: &HierarchicalMemory) -> CoreResult<()> {
        // Update in PostgreSQL
        self.postgres.update_memory(memory).await?;

        // Update cache if enabled
        if self.config.cache.enabled {
            let _ = self
                .redis
                .set(
                    &memory.memory.id,
                    memory,
                    Some(self.config.cache.default_ttl),
                )
                .await;
        }

        Ok(())
    }

    /// Delete memory from both storage and cache
    pub async fn delete_memory(&self, id: &str) -> CoreResult<bool> {
        // Delete from PostgreSQL
        let deleted = self.postgres.delete_memory(id).await?;

        // Delete from cache if enabled
        if self.config.cache.enabled {
            let _ = self.redis.delete(id).await;
        }

        Ok(deleted)
    }

    /// Search memories (no caching for search results)
    pub async fn search_memories(
        &self,
        query: &str,
        scope: Option<MemoryScope>,
        level: Option<MemoryLevel>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>> {
        self.postgres
            .search_memories(query, scope, level, limit)
            .await
    }

    /// Get combined statistics
    pub async fn get_statistics(&self) -> CoreResult<StorageStatistics> {
        self.postgres.get_statistics().await
    }

    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> CoreResult<CacheStatistics> {
        self.redis.get_cache_stats().await
    }

    /// Perform health check on both backends
    pub async fn health_check(&self) -> CoreResult<(HealthStatus, HealthStatus)> {
        let postgres_health = self.postgres.health_check().await?;
        let redis_health = match self.redis.get_cache_stats().await {
            Ok(_) => HealthStatus {
                healthy: true,
                message: "Redis cache is healthy".to_string(),
                response_time_ms: 0,
                last_check: Utc::now(),
            },
            Err(e) => HealthStatus {
                healthy: false,
                message: format!("Redis cache error: {}", e),
                response_time_ms: 0,
                last_check: Utc::now(),
            },
        };

        Ok((postgres_health, redis_health))
    }
}
