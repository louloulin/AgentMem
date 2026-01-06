//! 查询缓存模块
//!
//! 提供高性能的查询缓存功能，支持：
//! - LRU 缓存策略
//! - TTL 过期机制
//! - 缓存预热
//! - 缓存统计

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 查询缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,

    /// 默认 TTL（秒）
    pub default_ttl_seconds: u64,

    /// 是否启用 LRU
    pub enable_lru: bool,

    /// 是否启用缓存预热
    pub enable_warmup: bool,

    /// 预热批次大小
    pub warmup_batch_size: usize,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            default_ttl_seconds: 300, // 5 分钟
            enable_lru: true,
            enable_warmup: false,
            warmup_batch_size: 100,
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// 缓存的值
    value: V,

    /// 创建时间
    created_at: Instant,

    /// 最后访问时间
    last_accessed: Instant,

    /// 访问次数
    access_count: u64,

    /// TTL
    ttl: Duration,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    fn access(&mut self) -> &V {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }
}

/// 缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// 查询类型
    pub query_type: String,

    /// 查询参数哈希
    pub params_hash: u64,
}

impl CacheKey {
    /// 创建新的缓存键
    pub fn new(query_type: impl Into<String>, params: &impl Hash) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.hash(&mut hasher);

        Self {
            query_type: query_type.into(),
            params_hash: hasher.finish(),
        }
    }
}

/// 查询缓存
pub struct QueryCache {
    /// 配置
    config: QueryCacheConfig,

    /// 缓存存储
    cache: Arc<RwLock<HashMap<CacheKey, CacheEntry<Vec<u8>>>>>,

    /// LRU 访问顺序
    access_order: Arc<RwLock<VecDeque<CacheKey>>>,

    /// 统计信息
    stats: Arc<RwLock<CacheStats>>,
}

impl QueryCache {
    /// 创建新的查询缓存
    pub fn new(config: QueryCacheConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 获取缓存值
    pub async fn get<V>(&self, key: &CacheKey) -> Option<V>
    where
        V: serde::de::DeserializeOwned,
    {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(key) {
            // 检查是否过期
            if entry.is_expired() {
                cache.remove(key);
                stats.expired_entries += 1;
                stats.cache_misses += 1;
                return None;
            }

            // 更新访问信息
            let data = entry.access();

            // 更新 LRU 顺序
            if self.config.enable_lru {
                let mut access_order = self.access_order.write().await;
                access_order.retain(|k| k != key);
                access_order.push_back(key.clone());
            }

            stats.cache_hits += 1;

            // 反序列化
            if let Ok(value) = bincode::deserialize(data) {
                return Some(value);
            }
        }

        stats.cache_misses += 1;
        None
    }

    /// 设置缓存值
    pub async fn put<V>(&self, key: CacheKey, value: V) -> Result<()>
    where
        V: serde::Serialize,
    {
        // 序列化值
        let data = bincode::serialize(&value).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!(
                "Failed to serialize cache value: {e}"
            ))
        })?;

        let ttl = Duration::from_secs(self.config.default_ttl_seconds);
        let entry = CacheEntry::new(data, ttl);

        let mut cache = self.cache.write().await;

        // 如果已存在，更新
        if cache.contains_key(&key) {
            cache.insert(key.clone(), entry);

            if self.config.enable_lru {
                let mut access_order = self.access_order.write().await;
                access_order.retain(|k| k != &key);
                access_order.push_back(key);
            }

            return Ok(());
        }

        // 检查是否需要淘汰
        while cache.len() >= self.config.max_entries {
            self.evict_one(&mut cache).await;
        }

        // 插入新条目
        cache.insert(key.clone(), entry);

        if self.config.enable_lru {
            let mut access_order = self.access_order.write().await;
            access_order.push_back(key);
        }

        let mut stats = self.stats.write().await;
        stats.total_entries = cache.len();

        Ok(())
    }

    /// 淘汰一个条目
    async fn evict_one(&self, cache: &mut HashMap<CacheKey, CacheEntry<Vec<u8>>>) {
        if self.config.enable_lru {
            // LRU 淘汰
            let mut access_order = self.access_order.write().await;
            if let Some(key) = access_order.pop_front() {
                cache.remove(&key);

                let mut stats = self.stats.write().await;
                stats.evicted_entries += 1;
            }
        } else {
            // 随机淘汰第一个
            if let Some(key) = cache.keys().next().cloned() {
                cache.remove(&key);

                let mut stats = self.stats.write().await;
                stats.evicted_entries += 1;
            }
        }
    }

    /// 清理过期条目
    pub async fn cleanup_expired(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let mut access_order = self.access_order.write().await;

        let expired_keys: Vec<_> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in &expired_keys {
            cache.remove(key);
            access_order.retain(|k| k != key);
        }

        let mut stats = self.stats.write().await;
        stats.expired_entries += expired_keys.len();
        stats.total_entries = cache.len();

        tracing::debug!("Cleaned up {} expired cache entries", expired_keys.len());

        Ok(())
    }

    /// 清空缓存
    pub async fn clear(&self) -> Result<()> {
        self.cache.write().await.clear();
        self.access_order.write().await.clear();

        let mut stats = self.stats.write().await;
        stats.total_entries = 0;

        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// 总请求数
    pub total_requests: u64,

    /// 缓存命中数
    pub cache_hits: u64,

    /// 缓存未命中数
    pub cache_misses: u64,

    /// 当前条目数
    pub total_entries: usize,

    /// 过期条目数
    pub expired_entries: usize,

    /// 淘汰条目数
    pub evicted_entries: usize,
}

impl CacheStats {
    /// 计算命中率
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }

    /// 计算未命中率
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }
}
