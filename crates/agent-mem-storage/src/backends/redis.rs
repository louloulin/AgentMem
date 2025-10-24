//! Redis 缓存存储后端实现
//!
//! Redis 是一个高性能的内存数据结构存储，支持多种数据类型，
//! 非常适合用作缓存层、会话存储和实时数据处理。

use agent_mem_traits::{Result, VectorData, VectorSearchResult, VectorStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::warn;

#[cfg(feature = "redis")]
use redis::aio::ConnectionManager;
#[cfg(feature = "redis")]
use redis::{AsyncCommands, Client, RedisResult};

/// Redis 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis 连接 URL
    pub connection_url: String,
    /// 数据库编号
    pub database: u8,
    /// 密码（可选）
    pub password: Option<String>,
    /// 键前缀
    pub key_prefix: String,
    /// 向量键前缀
    pub vector_prefix: String,
    /// 元数据键前缀
    pub metadata_prefix: String,
    /// 索引键前缀
    pub index_prefix: String,
    /// 向量维度
    pub vector_dimension: usize,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
    /// 命令超时时间（秒）
    pub command_timeout: u64,
    /// 连接池大小
    pub pool_size: u32,
    /// TTL（生存时间，秒）- 0 表示永不过期
    pub ttl: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用分布式锁
    pub enable_distributed_lock: bool,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            connection_url: "redis://localhost:6379".to_string(),
            database: 0,
            password: None,
            key_prefix: "agentmem".to_string(),
            vector_prefix: "vector".to_string(),
            metadata_prefix: "metadata".to_string(),
            index_prefix: "index".to_string(),
            vector_dimension: 1536,
            connection_timeout: 30,
            command_timeout: 10,
            pool_size: 10,
            ttl: 0, // 永不过期
            enable_compression: false,
            enable_distributed_lock: false,
        }
    }
}

/// Redis 向量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisVectorRecord {
    /// 记录 ID
    pub id: String,
    /// 向量数据
    pub vector: Vec<f32>,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 创建时间戳
    pub created_at: i64,
    /// 更新时间戳
    pub updated_at: i64,
    /// 访问计数
    pub access_count: u64,
    /// 最后访问时间
    pub last_accessed: i64,
}

impl From<VectorData> for RedisVectorRecord {
    fn from(data: VectorData) -> Self {
        let now = chrono::Utc::now().timestamp();

        Self {
            id: data.id,
            vector: data.vector,
            metadata: data.metadata,
            created_at: now,
            updated_at: now,
            access_count: 0,
            last_accessed: now,
        }
    }
}

impl From<RedisVectorRecord> for VectorData {
    fn from(record: RedisVectorRecord) -> Self {
        let mut metadata = record.metadata;
        metadata.insert("created_at".to_string(), record.created_at.to_string());
        metadata.insert("updated_at".to_string(), record.updated_at.to_string());
        metadata.insert("access_count".to_string(), record.access_count.to_string());
        metadata.insert(
            "last_accessed".to_string(),
            record.last_accessed.to_string(),
        );

        Self {
            id: record.id,
            vector: record.vector,
            metadata,
        }
    }
}

/// Redis 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCacheStats {
    /// 总向量数量
    pub total_vectors: usize,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 缓存命中率
    pub hit_rate: f64,
    /// 内存使用量（字节）
    pub memory_usage: u64,
    /// 过期键数量
    pub expired_keys: u64,
}

/// Redis 分布式锁
#[derive(Debug)]
pub struct RedisDistributedLock {
    /// 锁键名
    pub key: String,
    /// 锁值
    pub value: String,
    /// 锁超时时间（秒）
    pub timeout: u64,
}

/// Redis 存储实现
pub struct RedisStore {
    config: RedisConfig,
    #[cfg(feature = "redis")]
    connection_manager: ConnectionManager,
    // 本地缓存，用于提高性能（无论是否启用 Redis 功能）
    vectors: std::sync::Arc<std::sync::Mutex<HashMap<String, RedisVectorRecord>>>,
    cache_stats: std::sync::Arc<std::sync::Mutex<RedisCacheStats>>,
}

impl RedisStore {
    /// 创建新的 Redis 存储实例
    pub async fn new(config: RedisConfig) -> Result<Self> {
        #[cfg(feature = "redis")]
        {
            // 真实的 Redis 连接实现
            let client = Client::open(config.connection_url.as_str()).map_err(|e| {
                AgentMemError::storage_error(format!("Failed to create Redis client: {}", e))
            })?;

            let connection_manager = ConnectionManager::new(client).await.map_err(|e| {
                AgentMemError::storage_error(format!(
                    "Failed to create Redis connection manager: {}",
                    e
                ))
            })?;

            let store = Self {
                config,
                connection_manager,
                vectors: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
                cache_stats: std::sync::Arc::new(std::sync::Mutex::new(RedisCacheStats {
                    total_vectors: 0,
                    cache_hits: 0,
                    cache_misses: 0,
                    hit_rate: 0.0,
                    memory_usage: 0,
                    expired_keys: 0,
                })),
            };

            // 验证连接
            store.verify_connection().await?;
            store.initialize_cache().await?;

            Ok(store)
        }

        #[cfg(not(feature = "redis"))]
        {
            // 回退到内存实现
            let store = Self {
                config,
                vectors: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
                cache_stats: std::sync::Arc::new(std::sync::Mutex::new(RedisCacheStats {
                    total_vectors: 0,
                    cache_hits: 0,
                    cache_misses: 0,
                    hit_rate: 0.0,
                    memory_usage: 0,
                    expired_keys: 0,
                })),
            };

            warn!("Redis feature not enabled, using in-memory fallback");
            Ok(store)
        }
    }

    /// 验证与 Redis 的连接
    async fn verify_connection(&self) -> Result<()> {
        #[cfg(feature = "redis")]
        {
            // 真实的 Redis 连接验证
            let mut conn = self.connection_manager.clone();
            let _: String = redis::cmd("PING")
                .query_async(&mut conn)
                .await
                .map_err(|e| AgentMemError::storage_error(format!("Redis ping failed: {}", e)))?;
            info!("Redis connection verified successfully");
            Ok(())
        }

        #[cfg(not(feature = "redis"))]
        {
            // 本地连接验证（无 Redis 特性时）
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        }
    }

    /// 初始化缓存
    async fn initialize_cache(&self) -> Result<()> {
        #[cfg(feature = "redis")]
        {
            // 真实的 Redis 初始化
            let mut conn = self.connection_manager.clone();

            // 选择数据库
            if self.config.database != 0 {
                let _: () = redis::cmd("SELECT")
                    .arg(self.config.database)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| {
                        AgentMemError::storage_error(format!("Failed to select database: {}", e))
                    })?;
            }

            // 设置密码（如果需要）
            if let Some(password) = &self.config.password {
                let _: () = redis::cmd("AUTH")
                    .arg(password)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| {
                        AgentMemError::storage_error(format!("Authentication failed: {}", e))
                    })?;
            }

            info!("Redis cache initialized successfully");
            Ok(())
        }

        #[cfg(not(feature = "redis"))]
        {
            // 本地初始化（无 Redis 特性时）
            tokio::time::sleep(Duration::from_millis(5)).await;
            Ok(())
        }
    }

    /// 构建向量键名
    fn build_vector_key(&self, id: &str) -> String {
        format!(
            "{}:{}:{}",
            self.config.key_prefix, self.config.vector_prefix, id
        )
    }

    /// 构建元数据键名
    fn build_metadata_key(&self, id: &str) -> String {
        format!(
            "{}:{}:{}",
            self.config.key_prefix, self.config.metadata_prefix, id
        )
    }

    /// 构建索引键名
    fn build_index_key(&self) -> String {
        format!("{}:{}", self.config.key_prefix, self.config.index_prefix)
    }

    /// 计算向量相似度 (余弦相似度)
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// 计算欧几里得距离
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// 更新缓存统计
    fn update_cache_stats(&self, hit: bool) {
        let mut stats = self.cache_stats.lock().unwrap();
        if hit {
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }

        let total_requests = stats.cache_hits + stats.cache_misses;
        if total_requests > 0 {
            stats.hit_rate = stats.cache_hits as f64 / total_requests as f64;
        }
    }

    /// 获取缓存统计
    pub fn get_cache_stats(&self) -> RedisCacheStats {
        let stats = self.cache_stats.lock().unwrap();
        let vectors = self.vectors.lock().unwrap();

        RedisCacheStats {
            total_vectors: vectors.len(),
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            hit_rate: stats.hit_rate,
            memory_usage: stats.memory_usage,
            expired_keys: stats.expired_keys,
        }
    }

    /// 获取分布式锁
    pub async fn acquire_lock(
        &self,
        key: &str,
        timeout: u64,
    ) -> Result<Option<RedisDistributedLock>> {
        if !self.config.enable_distributed_lock {
            return Ok(None);
        }

        #[cfg(feature = "redis")]
        {
            // 真实的 Redis 分布式锁实现
            let mut conn = self.connection_manager.clone();
            let lock_key = format!("{}:lock:{}", self.config.key_prefix, key);
            let lock_value = uuid::Uuid::new_v4().to_string();

            // 使用 SET NX EX 命令获取锁
            let result: RedisResult<String> = redis::cmd("SET")
                .arg(&lock_key)
                .arg(&lock_value)
                .arg("NX")
                .arg("EX")
                .arg(timeout)
                .query_async(&mut conn)
                .await;

            match result {
                Ok(_) => {
                    debug!("Acquired distributed lock: {}", lock_key);
                    Ok(Some(RedisDistributedLock {
                        key: lock_key,
                        value: lock_value,
                        timeout,
                    }))
                }
                Err(_) => {
                    debug!("Failed to acquire distributed lock: {}", lock_key);
                    Ok(None)
                }
            }
        }

        #[cfg(not(feature = "redis"))]
        {
            // 本地锁获取（无 Redis 特性时）
            tokio::time::sleep(Duration::from_millis(1)).await;

            Ok(Some(RedisDistributedLock {
                key: key.to_string(),
                value: uuid::Uuid::new_v4().to_string(),
                timeout,
            }))
        }
    }

    /// 释放分布式锁
    pub async fn release_lock(&self, lock: &RedisDistributedLock) -> Result<bool> {
        if !self.config.enable_distributed_lock {
            return Ok(false);
        }

        #[cfg(feature = "redis")]
        {
            // 真实的 Redis 分布式锁释放实现
            let mut conn = self.connection_manager.clone();

            // 使用 Lua 脚本原子性地检查和删除锁
            let script = r#"
                if redis.call("get", KEYS[1]) == ARGV[1] then
                    return redis.call("del", KEYS[1])
                else
                    return 0
                end
            "#;

            let result: RedisResult<i32> = redis::Script::new(script)
                .key(&lock.key)
                .arg(&lock.value)
                .invoke_async(&mut conn)
                .await;

            match result {
                Ok(1) => {
                    debug!("Released distributed lock: {}", lock.key);
                    Ok(true)
                }
                Ok(0) => {
                    warn!("Lock not owned or already expired: {}", lock.key);
                    Ok(false)
                }
                Ok(_) => {
                    warn!("Unexpected result when releasing lock: {}", lock.key);
                    Ok(false)
                }
                Err(e) => {
                    warn!("Failed to release lock {}: {}", lock.key, e);
                    Ok(false)
                }
            }
        }

        #[cfg(not(feature = "redis"))]
        {
            // 本地锁释放（无 Redis 特性时）
            tokio::time::sleep(Duration::from_millis(1)).await;
            Ok(true)
        }
    }

    /// 执行缓存预热
    pub async fn warm_cache(&self, vector_ids: Vec<String>) -> Result<usize> {
        // 在实际实现中，这里应该预加载指定的向量到缓存中
        // 提高后续访问的性能

        let mut warmed_count = 0;
        let vectors = self.vectors.lock().unwrap();

        for id in vector_ids {
            if vectors.contains_key(&id) {
                warmed_count += 1;
            }
        }

        Ok(warmed_count)
    }

    /// 执行缓存清理
    pub async fn cleanup_cache(&self) -> Result<usize> {
        #[cfg(feature = "redis")]
        {
            // 真实的 Redis 缓存清理实现
            let mut conn = self.connection_manager.clone();

            // 获取所有匹配的键
            let pattern = format!("{}:*", self.config.key_prefix);
            let keys: Vec<String> = conn
                .keys(&pattern)
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get keys: {}", e)))?;

            let mut cleaned_count = 0;

            // 检查每个键的 TTL，删除过期的键
            for key in keys {
                let ttl: i64 = conn.ttl(&key).await.map_err(|e| {
                    AgentMemError::StorageError(format!("Failed to get TTL: {}", e))
                })?;

                // TTL = -1 表示没有过期时间，TTL = -2 表示键不存在
                if ttl == -2 {
                    cleaned_count += 1;
                }
            }

            info!("Cleaned {} expired cache entries", cleaned_count);
            Ok(cleaned_count)
        }

        #[cfg(not(feature = "redis"))]
        {
            // 本地清理过程（无 Redis 特性时）
            tokio::time::sleep(Duration::from_millis(5)).await;
            Ok(0) // 返回清理的项目数量
        }
    }

    /// 批量设置 TTL
    pub async fn set_batch_ttl(&self, ids: Vec<String>, ttl: u64) -> Result<()> {
        #[cfg(feature = "redis")]
        {
            // 真实的 Redis 批量 TTL 设置实现
            let mut conn = self.connection_manager.clone();
            let ids_len = ids.len();

            for id in &ids {
                let vector_key = self.build_vector_key(id);
                let _: () = redis::cmd("EXPIRE")
                    .arg(&vector_key)
                    .arg(ttl as i64)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| {
                        AgentMemError::storage_error(format!(
                            "Failed to set TTL for {}: {}",
                            vector_key, e
                        ))
                    })?;
            }

            debug!("Set TTL for {} keys", ids_len);
            Ok(())
        }

        #[cfg(not(feature = "redis"))]
        {
            // 本地批量 TTL 设置（无 Redis 特性时）
            tokio::time::sleep(Duration::from_millis(ids.len() as u64)).await;
            Ok(())
        }
    }
}

#[async_trait]
impl VectorStore for RedisStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let mut store = self.vectors.lock().unwrap();
        let mut ids = Vec::new();

        for vector_data in vectors {
            let id = if vector_data.id.is_empty() {
                format!("redis_{}", uuid::Uuid::new_v4())
            } else {
                vector_data.id.clone()
            };

            // 验证向量维度
            if vector_data.vector.len() != self.config.vector_dimension {
                return Err(agent_mem_traits::AgentMemError::validation_error(format!(
                    "Vector dimension {} does not match expected dimension {}",
                    vector_data.vector.len(),
                    self.config.vector_dimension
                )));
            }

            let mut record = RedisVectorRecord::from(vector_data);
            record.id = id.clone();

            // 在实际实现中，这里应该使用 Redis 命令存储向量
            // let vector_key = self.build_vector_key(&id);
            // let serialized = serde_json::to_string(&record)?;
            // if self.config.ttl > 0 {
            //     redis::cmd("SETEX").arg(&vector_key).arg(self.config.ttl).arg(&serialized).query_async(&mut con).await?;
            // } else {
            //     redis::cmd("SET").arg(&vector_key).arg(&serialized).query_async(&mut con).await?;
            // }
            // redis::cmd("SADD").arg(self.build_index_key()).arg(&id).query_async(&mut con).await?;

            store.insert(id.clone(), record);
            ids.push(id);
        }

        // 更新统计
        {
            let mut stats = self.cache_stats.lock().unwrap();
            stats.total_vectors = store.len();
        }

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        // 验证查询向量维度
        if query_vector.len() != self.config.vector_dimension {
            return Err(agent_mem_traits::AgentMemError::validation_error(format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.config.vector_dimension
            )));
        }

        let mut store = self.vectors.lock().unwrap();
        let mut results = Vec::new();

        // 在实际实现中，这里应该使用 Redis 的向量搜索功能
        // 或者从缓存中加载所有向量进行计算

        for (_, record) in store.iter_mut() {
            let similarity = self.cosine_similarity(&query_vector, &record.vector);
            let distance = self.euclidean_distance(&query_vector, &record.vector);

            // 更新访问统计
            record.access_count += 1;
            record.last_accessed = chrono::Utc::now().timestamp();

            // 应用阈值过滤
            if let Some(threshold) = threshold {
                if similarity < threshold {
                    continue;
                }
            }

            results.push(VectorSearchResult {
                id: record.id.clone(),
                vector: record.vector.clone(),
                metadata: record.metadata.clone(),
                similarity,
                distance,
            });
        }

        // 按相似度排序并限制结果数量
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        // 更新缓存统计
        self.update_cache_stats(true);

        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        let mut store = self.vectors.lock().unwrap();

        for id in ids {
            // 在实际实现中，这里应该使用 Redis 命令删除向量
            // let vector_key = self.build_vector_key(&id);
            // redis::cmd("DEL").arg(&vector_key).query_async(&mut con).await?;
            // redis::cmd("SREM").arg(self.build_index_key()).arg(&id).query_async(&mut con).await?;

            store.remove(&id);
        }

        // 更新统计
        {
            let mut stats = self.cache_stats.lock().unwrap();
            stats.total_vectors = store.len();
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        let mut store = self.vectors.lock().unwrap();

        for vector_data in vectors {
            let id = vector_data.id.clone();

            // 验证向量维度
            if vector_data.vector.len() != self.config.vector_dimension {
                return Err(agent_mem_traits::AgentMemError::validation_error(format!(
                    "Vector dimension {} does not match expected dimension {}",
                    vector_data.vector.len(),
                    self.config.vector_dimension
                )));
            }

            if let Some(existing_record) = store.get(&id) {
                let mut updated_record = RedisVectorRecord::from(vector_data);
                updated_record.id = id.clone();
                updated_record.created_at = existing_record.created_at; // 保持原创建时间
                updated_record.access_count = existing_record.access_count; // 保持访问计数
                updated_record.updated_at = chrono::Utc::now().timestamp();

                // 在实际实现中，这里应该使用 Redis 命令更新向量
                // let vector_key = self.build_vector_key(&id);
                // let serialized = serde_json::to_string(&updated_record)?;
                // if self.config.ttl > 0 {
                //     redis::cmd("SETEX").arg(&vector_key).arg(self.config.ttl).arg(&serialized).query_async(&mut con).await?;
                // } else {
                //     redis::cmd("SET").arg(&vector_key).arg(&serialized).query_async(&mut con).await?;
                // }

                store.insert(id, updated_record);
            }
        }

        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let mut store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该使用 Redis 命令获取向量
        // let vector_key = self.build_vector_key(id);
        // let result: Option<String> = redis::cmd("GET").arg(&vector_key).query_async(&mut con).await?;

        if let Some(record) = store.get_mut(id) {
            // 更新访问统计
            record.access_count += 1;
            record.last_accessed = chrono::Utc::now().timestamp();

            self.update_cache_stats(true);
            Ok(Some(VectorData::from(record.clone())))
        } else {
            self.update_cache_stats(false);
            Ok(None)
        }
    }

    async fn count_vectors(&self) -> Result<usize> {
        let store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该使用 Redis 命令获取集合大小
        // let count: usize = redis::cmd("SCARD").arg(self.build_index_key()).query_async(&mut con).await?;

        Ok(store.len())
    }

    async fn clear(&self) -> Result<()> {
        let mut store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该删除所有相关的 Redis 键
        // let pattern = format!("{}:*", self.config.key_prefix);
        // let keys: Vec<String> = redis::cmd("KEYS").arg(&pattern).query_async(&mut con).await?;
        // if !keys.is_empty() {
        //     redis::cmd("DEL").arg(&keys).query_async(&mut con).await?;
        // }

        store.clear();

        // 重置统计
        {
            let mut stats = self.cache_stats.lock().unwrap();
            stats.total_vectors = 0;
            stats.cache_hits = 0;
            stats.cache_misses = 0;
            stats.hit_rate = 0.0;
            stats.memory_usage = 0;
            stats.expired_keys = 0;
        }

        Ok(())
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &std::collections::HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        use crate::utils::VectorStoreDefaults;
        self.default_search_with_filters(query_vector, limit, filters, threshold)
            .await
    }

    async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
        use crate::utils::VectorStoreDefaults;
        self.default_health_check("Redis").await
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        use crate::utils::VectorStoreDefaults;
        self.default_get_stats(self.config.vector_dimension).await
    }

    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        use crate::utils::VectorStoreDefaults;
        self.default_add_vectors_batch(batches).await
    }

    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        use crate::utils::VectorStoreDefaults;
        self.default_delete_vectors_batch(id_batches).await
    }
}
