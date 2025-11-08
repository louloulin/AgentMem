//! Cached Adaptive Search Engine - Week 7-8 性能优化
//! 为自适应搜索引擎添加缓存层，提升性能

use super::adaptive_router::AdaptiveRouter;
use super::adaptive_search_engine::SearchEngineBackend;
use super::{SearchQuery, SearchResult};
use crate::config::AgentMemConfig;
use crate::performance::cache::{CacheKey, QueryCache, QueryCacheConfig};
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// 带缓存的自适应搜索引擎
pub struct CachedAdaptiveEngine<S>
where
    S: SearchEngineBackend,
{
    /// 后端搜索引擎
    backend: Arc<S>,
    /// 自适应路由器
    router: Arc<AdaptiveRouter>,
    /// 查询缓存
    cache: Arc<QueryCache>,
    /// 是否启用缓存
    enable_cache: bool,
    /// 是否启用学习
    enable_learning: bool,
}

impl<S: SearchEngineBackend> CachedAdaptiveEngine<S> {
    pub fn new(
        backend: Arc<S>,
        config: AgentMemConfig,
        cache_config: QueryCacheConfig,
        enable_cache: bool,
        enable_learning: bool,
    ) -> Self {
        let router = Arc::new(AdaptiveRouter::new(config));
        let cache = Arc::new(QueryCache::new(cache_config));
        
        Self {
            backend,
            router,
            cache,
            enable_cache,
            enable_learning,
        }
    }
    
    /// 执行缓存+自适应搜索
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        let start = Instant::now();
        
        // 步骤1: 构建缓存键
        let cache_key = self.build_cache_key(&query);
        
        // 步骤2: 尝试从缓存获取
        if self.enable_cache {
            if let Some(cached_results) = self.cache.get::<Vec<SearchResult>>(&cache_key).await {
                let latency_ms = start.elapsed().as_millis() as u64;
                info!(
                    "Cache hit for query: {}, latency: {}ms",
                    query.query, latency_ms
                );
                return Ok(cached_results);
            }
        }
        
        // 步骤3: 缓存未命中，路由器决策策略
        let (strategy_id, weights) = self.router.decide_strategy(&query).await?;
        
        debug!(
            "CachedAdaptive selected strategy: {:?}, weights: v={}, f={}",
            strategy_id, weights.vector_weight, weights.fulltext_weight
        );
        
        // 步骤4: 执行搜索
        let mut query_clone = query.clone();
        query_clone.vector_weight = weights.vector_weight;
        query_clone.fulltext_weight = weights.fulltext_weight;
        
        let results = self.backend
            .search_with_weights(
                query_vector,
                query_clone.clone(),
                weights.vector_weight,
                weights.fulltext_weight,
            )
            .await?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        // 步骤5: 计算准确率
        let accuracy = self.calculate_accuracy(&results.results);
        
        // 步骤6: 写入缓存
        if self.enable_cache && !results.results.is_empty() {
            if let Err(e) = self.cache.put(cache_key, results.results.clone()).await {
                tracing::warn!("Failed to cache results: {}", e);
            }
        }
        
        // 步骤7: 异步反馈学习
        if self.enable_learning {
            let router = Arc::clone(&self.router);
            let query_for_feedback = query.clone();
            tokio::spawn(async move {
                if let Err(e) = router
                    .record_performance(&query_for_feedback, strategy_id, accuracy, latency_ms)
                    .await
                {
                    tracing::warn!("Failed to record performance: {}", e);
                }
            });
        }
        
        info!(
            "Search completed: strategy={:?}, accuracy={:.2}, latency={}ms",
            strategy_id, accuracy, latency_ms
        );
        
        Ok(results.results)
    }
    
    /// 构建缓存键
    fn build_cache_key(&self, query: &SearchQuery) -> CacheKey {
        // 构建参数结构用于哈希
        let params = (
            &query.query,
            query.limit,
            query.threshold,
            &query.filters,
        );
        
        CacheKey::new("adaptive_search", &params)
    }
    
    /// 计算准确率
    fn calculate_accuracy(&self, results: &[SearchResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }
        
        let top_k = results.len().min(5);
        let avg_score: f32 = results.iter()
            .take(top_k)
            .map(|r| r.score)
            .sum::<f32>() / top_k as f32;
        
        avg_score.clamp(0.0, 1.0)
    }
    
    /// 获取缓存统计
    pub async fn get_cache_stats(&self) -> Result<String> {
        let stats = self.cache.get_stats().await;
        
        let output = format!(
            "=== Cache Stats ===\n\
             Total Requests: {}\n\
             Cache Hits: {} ({:.2}%)\n\
             Cache Misses: {} ({:.2}%)\n\
             Total Entries: {}\n\
             Expired Entries: {}\n\
             Hit Rate: {:.2}%",
            stats.total_requests,
            stats.cache_hits,
            stats.hit_rate() * 100.0,
            stats.cache_misses,
            stats.miss_rate() * 100.0,
            stats.total_entries,
            stats.expired_entries,
            stats.hit_rate() * 100.0
        );
        
        Ok(output)
    }
    
    /// 清空缓存
    pub async fn clear_cache(&self) -> Result<()> {
        self.cache.clear().await
    }
    
    /// 预热缓存（批量加载热点查询）
    pub async fn warmup_cache(&self, hot_queries: Vec<(Vec<f32>, SearchQuery)>) -> Result<usize> {
        let mut warmed = 0;
        
        for (vector, query) in hot_queries {
            if let Ok(results) = self.search(vector, query).await {
                if !results.is_empty() {
                    warmed += 1;
                }
            }
        }
        
        info!("Cache warmed up with {} queries", warmed);
        Ok(warmed)
    }
}

/// 并发搜索优化器
pub struct ParallelSearchOptimizer {
    /// 最大并发数
    max_concurrency: usize,
}

impl ParallelSearchOptimizer {
    pub fn new(max_concurrency: usize) -> Self {
        Self { max_concurrency }
    }
    
    /// 批量并发搜索
    pub async fn batch_search<S>(
        &self,
        engine: Arc<CachedAdaptiveEngine<S>>,
        queries: Vec<(Vec<f32>, SearchQuery)>,
    ) -> Result<Vec<Vec<SearchResult>>>
    where
        S: SearchEngineBackend + 'static,
    {
        let start = Instant::now();
        let total_queries = queries.len();
        
        // 使用tokio::spawn并发执行
        let mut tasks = Vec::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrency));
        
        for (vector, query) in queries {
            let engine = Arc::clone(&engine);
            let permit = Arc::clone(&semaphore);
            
            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                engine.search(vector, query).await
            });
            
            tasks.push(task);
        }
        
        // 等待所有任务完成
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    tracing::error!("Search failed: {}", e);
                    results.push(Vec::new());
                }
                Err(e) => {
                    tracing::error!("Task panicked: {}", e);
                    results.push(Vec::new());
                }
            }
        }
        
        let elapsed = start.elapsed();
        info!(
            "Batch search completed: {} queries in {:?} ({:.2} QPS)",
            total_queries,
            elapsed,
            total_queries as f64 / elapsed.as_secs_f64()
        );
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_key_generation() {
        let query = SearchQuery {
            query: "test query".to_string(),
            limit: 10,
            threshold: Some(0.5),
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            filters: None,
        };
        
        // Mock实现
        // 实际测试需要完整的Backend实现
    }
    
    #[test]
    fn test_parallel_optimizer() {
        let optimizer = ParallelSearchOptimizer::new(10);
        assert_eq!(optimizer.max_concurrency, 10);
    }
}

