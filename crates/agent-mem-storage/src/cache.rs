//! 存储缓存模块
//!
//! 提供多级缓存机制，包括：
//! - 内存缓存（L1）
//! - 分布式缓存（L2）
//! - 缓存策略（LRU、TTL）
//! - 缓存预热和失效

use agent_mem_traits::{VectorData, VectorSearchResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// 缓存的数据
    data: T,
    /// 创建时间
    created_at: Instant,
    /// 最后访问时间
    last_accessed: Instant,
    /// 访问次数
    access_count: u64,
    /// TTL（生存时间）
    ttl: Option<Duration>,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.data
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 默认TTL（秒）
    pub default_ttl_seconds: Option<u64>,
    /// 是否启用LRU淘汰
    pub enable_lru: bool,
    /// 缓存统计间隔（秒）
    pub stats_interval_seconds: u64,
    /// 是否启用缓存预热
    pub enable_warmup: bool,
    /// 预热批次大小
    pub warmup_batch_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            default_ttl_seconds: Some(3600), // 1小时
            enable_lru: true,
            stats_interval_seconds: 300, // 5分钟
            enable_warmup: false,
            warmup_batch_size: 100,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 总请求数
    pub total_requests: u64,
    /// 缓存命中数
    pub cache_hits: u64,
    /// 缓存未命中数
    pub cache_misses: u64,
    /// 缓存命中率
    pub hit_rate: f64,
    /// 当前缓存条目数
    pub current_entries: usize,
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存使用率
    pub usage_rate: f64,
    /// 过期条目数
    pub expired_entries: u64,
    /// 淘汰条目数
    pub evicted_entries: u64,
}

/// LRU 缓存实现
pub struct LRUCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// 缓存数据
    cache: HashMap<K, CacheEntry<V>>,
    /// LRU 访问顺序
    access_order: VecDeque<K>,
    /// 缓存配置
    config: CacheConfig,
    /// 缓存统计
    stats: CacheStats,
}

impl<K, V> LRUCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + std::fmt::Debug,
    V: Clone,
{
    /// 创建新的LRU缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            stats: CacheStats {
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                hit_rate: 0.0,
                current_entries: 0,
                max_entries: config.max_entries,
                usage_rate: 0.0,
                expired_entries: 0,
                evicted_entries: 0,
            },
            config,
        }
    }

    /// 获取缓存项
    pub fn get(&mut self, key: &K) -> Option<V> {
        self.stats.total_requests += 1;

        // 检查是否存在
        if let Some(entry) = self.cache.get_mut(key) {
            // 检查是否过期
            if entry.is_expired() {
                self.cache.remove(key);
                self.access_order.retain(|k| k != key);
                self.stats.expired_entries += 1;
                self.stats.cache_misses += 1;
                self.update_stats();
                return None;
            }

            // 更新访问信息
            let data = entry.access().clone();

            // 更新LRU顺序
            if self.config.enable_lru {
                self.access_order.retain(|k| k != key);
                self.access_order.push_back(key.clone());
            }

            self.stats.cache_hits += 1;
            self.update_stats();
            Some(data)
        } else {
            self.stats.cache_misses += 1;
            self.update_stats();
            None
        }
    }

    /// 插入缓存项
    pub fn put(&mut self, key: K, value: V) {
        let ttl = self.config.default_ttl_seconds.map(Duration::from_secs);
        let entry = CacheEntry::new(value, ttl);

        // 如果已存在，更新并调整LRU顺序
        if self.cache.contains_key(&key) {
            self.cache.insert(key.clone(), entry);
            if self.config.enable_lru {
                self.access_order.retain(|k| k != &key);
                self.access_order.push_back(key);
            }
            return;
        }

        // 检查是否需要淘汰
        while self.cache.len() >= self.config.max_entries {
            self.evict_one();
        }

        // 插入新条目
        self.cache.insert(key.clone(), entry);
        if self.config.enable_lru {
            self.access_order.push_back(key);
        }

        self.update_stats();
    }

    /// 删除缓存项
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.cache.remove(key) {
            self.access_order.retain(|k| k != key);
            self.update_stats();
            Some(entry.data)
        } else {
            None
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.update_stats();
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// 获取缓存统计
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 清理过期条目
    pub fn cleanup_expired(&mut self) -> usize {
        let mut expired_keys = Vec::new();

        for (key, entry) in &self.cache {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        let count = expired_keys.len();
        for key in expired_keys {
            self.cache.remove(&key);
            self.access_order.retain(|k| k != &key);
        }

        self.stats.expired_entries += count as u64;
        self.update_stats();
        count
    }

    /// 淘汰一个条目
    fn evict_one(&mut self) {
        if self.config.enable_lru {
            // LRU淘汰：移除最久未访问的
            if let Some(key) = self.access_order.pop_front() {
                self.cache.remove(&key);
                self.stats.evicted_entries += 1;
            }
        } else {
            // 随机淘汰
            if let Some(key) = self.cache.keys().next().cloned() {
                self.cache.remove(&key);
                self.access_order.retain(|k| k != &key);
                self.stats.evicted_entries += 1;
            }
        }
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        self.stats.current_entries = self.cache.len();
        self.stats.usage_rate = self.stats.current_entries as f64 / self.stats.max_entries as f64;

        if self.stats.total_requests > 0 {
            self.stats.hit_rate = self.stats.cache_hits as f64 / self.stats.total_requests as f64;
        }
    }
}

/// 向量缓存管理器
pub struct VectorCacheManager {
    /// 向量数据缓存
    vector_cache: Arc<RwLock<LRUCache<String, VectorData>>>,
    /// 搜索结果缓存
    search_cache: Arc<RwLock<LRUCache<String, Vec<VectorSearchResult>>>>,
    /// 配置
    config: CacheConfig,
}

impl VectorCacheManager {
    /// 创建新的向量缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        Self {
            vector_cache: Arc::new(RwLock::new(LRUCache::new(config.clone()))),
            search_cache: Arc::new(RwLock::new(LRUCache::new(config.clone()))),
            config,
        }
    }

    /// 获取向量数据
    pub async fn get_vector(&self, id: &str) -> Option<VectorData> {
        let mut cache = self.vector_cache.write().await;
        cache.get(&id.to_string())
    }

    /// 缓存向量数据
    pub async fn put_vector(&self, id: String, data: VectorData) {
        let mut cache = self.vector_cache.write().await;
        cache.put(id, data);
    }

    /// 获取搜索结果
    pub async fn get_search_results(&self, query_hash: &str) -> Option<Vec<VectorSearchResult>> {
        let mut cache = self.search_cache.write().await;
        cache.get(&query_hash.to_string())
    }

    /// 缓存搜索结果
    pub async fn put_search_results(&self, query_hash: String, results: Vec<VectorSearchResult>) {
        let mut cache = self.search_cache.write().await;
        cache.put(query_hash, results);
    }

    /// 删除向量缓存
    pub async fn remove_vector(&self, id: &str) {
        let mut cache = self.vector_cache.write().await;
        cache.remove(&id.to_string());
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        let mut vector_cache = self.vector_cache.write().await;
        let mut search_cache = self.search_cache.write().await;
        vector_cache.clear();
        search_cache.clear();
    }

    /// 获取缓存统计
    pub async fn get_stats(&self) -> (CacheStats, CacheStats) {
        let vector_cache = self.vector_cache.read().await;
        let search_cache = self.search_cache.read().await;
        (vector_cache.stats().clone(), search_cache.stats().clone())
    }

    /// 清理过期条目
    pub async fn cleanup_expired(&self) -> (usize, usize) {
        let mut vector_cache = self.vector_cache.write().await;
        let mut search_cache = self.search_cache.write().await;
        let vector_expired = vector_cache.cleanup_expired();
        let search_expired = search_cache.cleanup_expired();
        (vector_expired, search_expired)
    }

    /// 生成搜索查询的哈希
    pub fn generate_search_hash(
        query_vector: &[f32],
        limit: usize,
        threshold: Option<f32>,
        filters: &HashMap<String, serde_json::Value>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 对查询向量进行哈希（简化处理，实际应用中可能需要更精确的方法）
        for &val in query_vector {
            val.to_bits().hash(&mut hasher);
        }

        limit.hash(&mut hasher);
        threshold.map(|t| t.to_bits()).hash(&mut hasher);

        // 对过滤器进行哈希
        let mut filter_keys: Vec<_> = filters.keys().collect();
        filter_keys.sort();
        for key in filter_keys {
            key.hash(&mut hasher);
            filters[key].to_string().hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }
}

/// 带缓存的存储包装器
pub struct CachedVectorStore {
    /// 底层存储
    inner: Arc<dyn agent_mem_traits::VectorStore + Send + Sync>,
    /// 缓存管理器
    cache_manager: VectorCacheManager,
}

impl CachedVectorStore {
    /// 创建新的带缓存存储
    pub fn new(
        inner: Arc<dyn agent_mem_traits::VectorStore + Send + Sync>,
        cache_config: CacheConfig,
    ) -> Self {
        Self {
            inner,
            cache_manager: VectorCacheManager::new(cache_config),
        }
    }

    /// 获取缓存管理器
    pub fn cache_manager(&self) -> &VectorCacheManager {
        &self.cache_manager
    }
}

#[async_trait::async_trait]
impl agent_mem_traits::VectorStore for CachedVectorStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> agent_mem_traits::Result<Vec<String>> {
        let result = self.inner.add_vectors(vectors.clone()).await?;

        // 缓存新添加的向量
        for vector in vectors {
            self.cache_manager
                .put_vector(vector.id.clone(), vector)
                .await;
        }

        Ok(result)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> agent_mem_traits::Result<Vec<VectorSearchResult>> {
        let filters = HashMap::new();
        let query_hash =
            VectorCacheManager::generate_search_hash(&query_vector, limit, threshold, &filters);

        // 尝试从缓存获取
        if let Some(cached_results) = self.cache_manager.get_search_results(&query_hash).await {
            debug!("Cache hit for search query: {}", query_hash);
            return Ok(cached_results);
        }

        // 缓存未命中，执行实际搜索
        debug!("Cache miss for search query: {}", query_hash);
        let results = self
            .inner
            .search_vectors(query_vector, limit, threshold)
            .await?;

        // 缓存搜索结果
        self.cache_manager
            .put_search_results(query_hash, results.clone())
            .await;

        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> agent_mem_traits::Result<()> {
        let result = self.inner.delete_vectors(ids.clone()).await?;

        // 从缓存中删除
        for id in ids {
            self.cache_manager.remove_vector(&id).await;
        }

        Ok(result)
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> agent_mem_traits::Result<()> {
        let result = self.inner.update_vectors(vectors.clone()).await?;

        // 更新缓存
        for vector in vectors {
            self.cache_manager
                .put_vector(vector.id.clone(), vector)
                .await;
        }

        Ok(result)
    }

    async fn get_vector(&self, id: &str) -> agent_mem_traits::Result<Option<VectorData>> {
        // 尝试从缓存获取
        if let Some(cached_vector) = self.cache_manager.get_vector(id).await {
            debug!("Cache hit for vector: {}", id);
            return Ok(Some(cached_vector));
        }

        // 缓存未命中，从底层存储获取
        debug!("Cache miss for vector: {}", id);
        let result = self.inner.get_vector(id).await?;

        // 如果找到，缓存结果
        if let Some(ref vector) = result {
            self.cache_manager
                .put_vector(id.to_string(), vector.clone())
                .await;
        }

        Ok(result)
    }

    async fn count_vectors(&self) -> agent_mem_traits::Result<usize> {
        self.inner.count_vectors().await
    }

    async fn clear(&self) -> agent_mem_traits::Result<()> {
        let result = self.inner.clear().await?;

        // 清空缓存
        self.cache_manager.clear_all().await;

        Ok(result)
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> agent_mem_traits::Result<Vec<VectorSearchResult>> {
        let query_hash =
            VectorCacheManager::generate_search_hash(&query_vector, limit, threshold, filters);

        // 尝试从缓存获取
        if let Some(cached_results) = self.cache_manager.get_search_results(&query_hash).await {
            debug!("Cache hit for filtered search query: {}", query_hash);
            return Ok(cached_results);
        }

        // 缓存未命中，执行实际搜索
        debug!("Cache miss for filtered search query: {}", query_hash);
        let results = self
            .inner
            .search_with_filters(query_vector, limit, filters, threshold)
            .await?;

        // 缓存搜索结果
        self.cache_manager
            .put_search_results(query_hash, results.clone())
            .await;

        Ok(results)
    }

    async fn health_check(&self) -> agent_mem_traits::Result<agent_mem_traits::HealthStatus> {
        self.inner.health_check().await
    }

    async fn get_stats(&self) -> agent_mem_traits::Result<agent_mem_traits::VectorStoreStats> {
        self.inner.get_stats().await
    }

    async fn add_vectors_batch(
        &self,
        batches: Vec<Vec<VectorData>>,
    ) -> agent_mem_traits::Result<Vec<Vec<String>>> {
        let result = self.inner.add_vectors_batch(batches.clone()).await?;

        // 缓存所有新添加的向量
        for batch in batches {
            for vector in batch {
                self.cache_manager
                    .put_vector(vector.id.clone(), vector)
                    .await;
            }
        }

        Ok(result)
    }

    async fn delete_vectors_batch(
        &self,
        id_batches: Vec<Vec<String>>,
    ) -> agent_mem_traits::Result<Vec<bool>> {
        let result = self.inner.delete_vectors_batch(id_batches.clone()).await?;

        // 从缓存中删除
        for batch in id_batches {
            for id in batch {
                self.cache_manager.remove_vector(&id).await;
            }
        }

        Ok(result)
    }
}
