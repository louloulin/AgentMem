/// 多模态性能优化模块
///
/// 提供多模态处理的性能优化功能，包括：
/// - 并行处理管道
/// - 嵌入缓存
/// - 批量处理
/// - 增量处理
use agent_mem_traits::{AgentMemError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::debug;

use super::cross_modal::{CrossModalAligner, CrossModalConfig, MultimodalFusionEngine};
use super::unified_retrieval::UnifiedMultimodalRetrieval;
use super::ContentType;

/// 多模态优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalOptimizationConfig {
    /// 启用并行处理
    pub enable_parallel_processing: bool,
    /// 并行工作线程数
    pub parallel_workers: usize,
    /// 启用嵌入缓存
    pub enable_embedding_cache: bool,
    /// 缓存大小（条目数）
    pub cache_size: usize,
    /// 缓存TTL（秒）
    pub cache_ttl_seconds: u64,
    /// 启用批量处理
    pub enable_batch_processing: bool,
    /// 批量大小
    pub batch_size: usize,
    /// 批量等待时间（毫秒）
    pub batch_wait_ms: u64,
    /// 启用增量处理
    pub enable_incremental_processing: bool,
    /// 增量处理阈值
    pub incremental_threshold: f32,
}

impl Default for MultimodalOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_parallel_processing: true,
            parallel_workers: num_cpus::get(),
            enable_embedding_cache: true,
            cache_size: 10000,
            cache_ttl_seconds: 3600,
            enable_batch_processing: true,
            batch_size: 32,
            batch_wait_ms: 100,
            enable_incremental_processing: true,
            incremental_threshold: 0.95,
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 嵌入向量
    embedding: Vec<f32>,
    /// 创建时间
    created_at: Instant,
    /// 访问次数
    access_count: u64,
    /// 最后访问时间
    last_accessed: Instant,
}

impl CacheEntry {
    fn new(embedding: Vec<f32>) -> Self {
        let now = Instant::now();
        Self {
            embedding,
            created_at: now,
            access_count: 1,
            last_accessed: now,
        }
    }

    fn access(&mut self) -> &Vec<f32> {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.embedding
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// 嵌入缓存管理器
pub struct EmbeddingCache {
    cache: Arc<DashMap<String, CacheEntry>>,
    config: MultimodalOptimizationConfig,
    stats: Arc<RwLock<CacheStats>>,
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl EmbeddingCache {
    /// 创建新的嵌入缓存
    pub fn new(config: MultimodalOptimizationConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 获取缓存的嵌入
    pub async fn get(&self, key: &str) -> Option<Vec<f32>> {
        if !self.config.enable_embedding_cache {
            return None;
        }

        if let Some(mut entry) = self.cache.get_mut(key) {
            // 检查是否过期
            let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
            if entry.is_expired(ttl) {
                drop(entry);
                self.cache.remove(key);
                let mut stats = self.stats.write().await;
                stats.misses += 1;
                stats.evictions += 1;
                return None;
            }

            let embedding = entry.access().clone();
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            return Some(embedding);
        }

        let mut stats = self.stats.write().await;
        stats.misses += 1;
        None
    }

    /// 存储嵌入到缓存
    pub async fn put(&self, key: String, embedding: Vec<f32>) {
        if !self.config.enable_embedding_cache {
            return;
        }

        // 检查缓存大小限制
        if self.cache.len() >= self.config.cache_size {
            self.evict_lru().await;
        }

        self.cache.insert(key, CacheEntry::new(embedding));
        let mut stats = self.stats.write().await;
        stats.total_size = self.cache.len();
    }

    /// 驱逐最少使用的条目
    async fn evict_lru(&self) {
        // 找到最少使用的条目
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = Instant::now();

        for entry in self.cache.iter() {
            if entry.value().last_accessed < oldest_time {
                oldest_time = entry.value().last_accessed;
                oldest_key = Some(entry.key().clone());
            }
        }

        if let Some(key) = oldest_key {
            self.cache.remove(&key);
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
            stats.total_size = self.cache.len();
        }
    }

    /// 清空缓存
    pub async fn clear(&self) {
        self.cache.clear();
        let mut stats = self.stats.write().await;
        stats.total_size = 0;
    }

    /// 获取缓存统计
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

/// 并行处理管道
pub struct ParallelProcessingPipeline {
    config: MultimodalOptimizationConfig,
    semaphore: Arc<Semaphore>,
    aligner: Arc<CrossModalAligner>,
    fusion_engine: Arc<MultimodalFusionEngine>,
    cache: Arc<EmbeddingCache>,
}

impl ParallelProcessingPipeline {
    /// 创建新的并行处理管道
    pub fn new(config: MultimodalOptimizationConfig, cross_modal_config: CrossModalConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.parallel_workers));
        let aligner = Arc::new(CrossModalAligner::new(cross_modal_config.clone()));
        let fusion_engine = Arc::new(MultimodalFusionEngine::new(cross_modal_config));
        let cache = Arc::new(EmbeddingCache::new(config.clone()));

        Self {
            config,
            semaphore,
            aligner,
            fusion_engine,
            cache,
        }
    }

    /// 并行对齐嵌入
    pub async fn parallel_align(
        &self,
        embeddings: Vec<(Vec<f32>, ContentType, ContentType)>,
    ) -> Result<Vec<Vec<f32>>> {
        if !self.config.enable_parallel_processing || embeddings.len() <= 1 {
            // 串行处理
            let mut results = Vec::new();
            for (embedding, source_type, target_type) in embeddings {
                let aligned =
                    self.aligner
                        .align_embeddings(&embedding, &source_type, &target_type)?;
                results.push(aligned);
            }
            return Ok(results);
        }

        // 并行处理
        let mut tasks: Vec<JoinHandle<Result<Vec<f32>>>> = Vec::new();

        for (embedding, source_type, target_type) in embeddings {
            let aligner = Arc::clone(&self.aligner);
            let semaphore = Arc::clone(&self.semaphore);
            let cache = Arc::clone(&self.cache);
            let cache_key = format!("{:?}_{:?}_{:?}", source_type, target_type, embedding.len());

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.map_err(|e| {
                    AgentMemError::internal_error(format!("Failed to acquire permit: {e}"))
                })?;

                // 检查缓存
                if let Some(cached) = cache.get(&cache_key).await {
                    return Ok(cached);
                }

                // 执行对齐
                let aligned = aligner.align_embeddings(&embedding, &source_type, &target_type)?;

                // 存入缓存
                cache.put(cache_key, aligned.clone()).await;

                Ok(aligned)
            });

            tasks.push(task);
        }

        // 等待所有任务完成
        let mut results = Vec::new();
        for task in tasks {
            let result = task
                .await
                .map_err(|e| AgentMemError::internal_error(format!("Task join error: {e}")))??;
            results.push(result);
        }

        Ok(results)
    }

    /// 获取缓存统计
    pub async fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_stats().await
    }

    /// 清空缓存
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }
}

/// 批量处理器
pub struct BatchProcessor {
    config: MultimodalOptimizationConfig,
    pipeline: Arc<ParallelProcessingPipeline>,
}

impl BatchProcessor {
    /// 创建新的批量处理器
    pub fn new(config: MultimodalOptimizationConfig, cross_modal_config: CrossModalConfig) -> Self {
        let pipeline = Arc::new(ParallelProcessingPipeline::new(
            config.clone(),
            cross_modal_config,
        ));

        Self { config, pipeline }
    }

    /// 批量对齐嵌入
    pub async fn batch_align(
        &self,
        embeddings: Vec<(Vec<f32>, ContentType, ContentType)>,
    ) -> Result<Vec<Vec<f32>>> {
        if !self.config.enable_batch_processing {
            return self.pipeline.parallel_align(embeddings).await;
        }

        // 分批处理
        let mut all_results = Vec::new();
        for chunk in embeddings.chunks(self.config.batch_size) {
            let chunk_vec = chunk.to_vec();
            let results = self.pipeline.parallel_align(chunk_vec).await?;
            all_results.extend(results);

            // 批次间等待
            if self.config.batch_wait_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.batch_wait_ms)).await;
            }
        }

        Ok(all_results)
    }

    /// 获取处理统计
    pub async fn get_stats(&self) -> BatchProcessingStats {
        let cache_stats = self.pipeline.get_cache_stats().await;
        BatchProcessingStats {
            cache_hit_rate: cache_stats.hit_rate(),
            cache_size: cache_stats.total_size,
            cache_evictions: cache_stats.evictions,
        }
    }
}

/// 批量处理统计
#[derive(Debug, Clone)]
pub struct BatchProcessingStats {
    pub cache_hit_rate: f64,
    pub cache_size: usize,
    pub cache_evictions: u64,
}

/// 增量处理器
pub struct IncrementalProcessor {
    config: MultimodalOptimizationConfig,
    previous_embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

impl IncrementalProcessor {
    /// 创建新的增量处理器
    pub fn new(config: MultimodalOptimizationConfig) -> Self {
        Self {
            config,
            previous_embeddings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查是否需要重新处理
    pub async fn should_reprocess(&self, key: &str, new_embedding: &[f32]) -> bool {
        if !self.config.enable_incremental_processing {
            return true;
        }

        let previous = self.previous_embeddings.read().await;
        if let Some(prev_embedding) = previous.get(key) {
            // 计算相似度
            let similarity = self.cosine_similarity(prev_embedding, new_embedding);
            // 如果相似度高于阈值，不需要重新处理
            similarity < self.config.incremental_threshold
        } else {
            // 没有历史记录，需要处理
            true
        }
    }

    /// 更新嵌入记录
    pub async fn update_embedding(&self, key: String, embedding: Vec<f32>) {
        let mut previous = self.previous_embeddings.write().await;
        previous.insert(key, embedding);
    }

    /// 计算余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// 清空历史记录
    pub async fn clear_history(&self) {
        let mut previous = self.previous_embeddings.write().await;
        previous.clear();
    }
}

/// 多模态优化管理器
pub struct MultimodalOptimizationManager {
    config: MultimodalOptimizationConfig,
    batch_processor: Arc<BatchProcessor>,
    incremental_processor: Arc<IncrementalProcessor>,
    retrieval: Arc<UnifiedMultimodalRetrieval>,
}

impl MultimodalOptimizationManager {
    /// 创建新的优化管理器
    pub fn new(config: MultimodalOptimizationConfig, cross_modal_config: CrossModalConfig) -> Self {
        let batch_processor = Arc::new(BatchProcessor::new(
            config.clone(),
            cross_modal_config.clone(),
        ));
        let incremental_processor = Arc::new(IncrementalProcessor::new(config.clone()));
        let retrieval = Arc::new(UnifiedMultimodalRetrieval::new(cross_modal_config));

        Self {
            config,
            batch_processor,
            incremental_processor,
            retrieval,
        }
    }

    /// 优化的批量对齐
    pub async fn optimized_batch_align(
        &self,
        embeddings: Vec<(String, Vec<f32>, ContentType, ContentType)>,
    ) -> Result<Vec<(String, Vec<f32>)>> {
        // 过滤需要处理的嵌入
        let mut to_process = Vec::new();
        let mut results = Vec::new();

        for (key, embedding, source_type, target_type) in embeddings {
            if self
                .incremental_processor
                .should_reprocess(&key, &embedding)
                .await
            {
                to_process.push((key.clone(), embedding, source_type, target_type));
            } else {
                // 使用缓存的结果
                debug!("Skipping reprocessing for key: {}", key);
            }
        }

        // 批量处理
        if !to_process.is_empty() {
            let align_input: Vec<_> = to_process
                .iter()
                .map(|(_, emb, src, tgt)| (emb.clone(), src.clone(), tgt.clone()))
                .collect();

            let aligned = self.batch_processor.batch_align(align_input).await?;

            for ((key, _, _, _), aligned_emb) in to_process.iter().zip(aligned.iter()) {
                self.incremental_processor
                    .update_embedding(key.clone(), aligned_emb.clone())
                    .await;
                results.push((key.clone(), aligned_emb.clone()));
            }
        }

        Ok(results)
    }

    /// 获取优化统计
    pub async fn get_optimization_stats(&self) -> OptimizationStats {
        let batch_stats = self.batch_processor.get_stats().await;

        OptimizationStats {
            cache_hit_rate: batch_stats.cache_hit_rate,
            cache_size: batch_stats.cache_size,
            cache_evictions: batch_stats.cache_evictions,
            parallel_workers: self.config.parallel_workers,
            batch_size: self.config.batch_size,
        }
    }
}

/// 优化统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub cache_hit_rate: f64,
    pub cache_size: usize,
    pub cache_evictions: u64,
    pub parallel_workers: usize,
    pub batch_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_cache() {
        let config = MultimodalOptimizationConfig::default();
        let cache = EmbeddingCache::new(config);

        let embedding = vec![1.0, 2.0, 3.0];
        cache.put("test_key".to_string(), embedding.clone()).await;

        let cached = cache.get("test_key").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), embedding);

        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mut config = MultimodalOptimizationConfig::default();
        config.cache_ttl_seconds = 1; // 1秒过期
        let cache = EmbeddingCache::new(config);

        let embedding = vec![1.0, 2.0, 3.0];
        cache.put("test_key".to_string(), embedding).await;

        // 等待过期
        tokio::time::sleep(Duration::from_secs(2)).await;

        let cached = cache.get("test_key").await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_incremental_processor() {
        let config = MultimodalOptimizationConfig::default();
        let processor = IncrementalProcessor::new(config);

        let embedding1 = vec![1.0, 0.0, 0.0];
        let embedding2 = vec![1.0, 0.0, 0.0]; // 相同
        let embedding3 = vec![0.0, 1.0, 0.0]; // 不同

        // 第一次应该处理
        assert!(processor.should_reprocess("key1", &embedding1).await);

        // 更新记录
        processor
            .update_embedding("key1".to_string(), embedding1.clone())
            .await;

        // 相同的嵌入不应该重新处理
        assert!(!processor.should_reprocess("key1", &embedding2).await);

        // 不同的嵌入应该重新处理
        assert!(processor.should_reprocess("key1", &embedding3).await);
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let config = MultimodalOptimizationConfig {
            batch_size: 2,
            ..Default::default()
        };
        let cross_modal_config = CrossModalConfig::default();
        let processor = BatchProcessor::new(config, cross_modal_config);

        let embeddings = vec![
            (vec![1.0, 0.0], ContentType::Text, ContentType::Image),
            (vec![0.0, 1.0], ContentType::Text, ContentType::Image),
            (vec![1.0, 1.0], ContentType::Text, ContentType::Image),
        ];

        let results = processor.batch_align(embeddings).await?;
        assert_eq!(results.len(), 3);
    }
}
