//! Enhanced Hybrid Search Engine V2
//!
//! 增强的混合搜索引擎，整合：
//! - QueryClassifier: 智能查询分类
//! - AdaptiveThresholdCalculator: 自适应阈值
//! - Vector Search (LanceDB): 语义搜索
//! - BM25 (LibSQL FTS5): 全文搜索  
//! - RRF Fusion: 结果融合
//! - Performance Monitoring: 性能监控

use super::query_classifier::{QueryClassifier, QueryType, SearchStrategy};
use super::adaptive_threshold::AdaptiveThresholdCalculator;
use super::{SearchResult, SearchStats};
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 增强的混合搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedHybridConfig {
    /// 是否启用查询分类
    pub enable_query_classification: bool,
    
    /// 是否启用自适应阈值
    pub enable_adaptive_threshold: bool,
    
    /// 是否启用并行搜索
    pub enable_parallel: bool,
    
    /// 是否启用性能监控
    pub enable_metrics: bool,
    
    /// 是否启用缓存
    pub enable_cache: bool,
    
    /// RRF常数k
    pub rrf_k: f32,
}

impl Default for EnhancedHybridConfig {
    fn default() -> Self {
        Self {
            enable_query_classification: true,
            enable_adaptive_threshold: true,
            enable_parallel: true,
            enable_metrics: true,
            enable_cache: false,
            rrf_k: 60.0,
        }
    }
}

/// 搜索结果（增强版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchResult {
    /// 结果列表
    pub results: Vec<SearchResult>,
    
    /// 查询类型
    pub query_type: QueryType,
    
    /// 使用的策略
    pub strategy: SearchStrategy,
    
    /// 搜索统计
    pub stats: EnhancedSearchStats,
    
    /// 调试信息（可选）
    pub debug_info: Option<DebugInfo>,
}

/// 增强的搜索统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchStats {
    /// 总时间（毫秒）
    pub total_time_ms: u64,
    
    /// 查询分类时间（毫秒）
    pub classification_time_ms: u64,
    
    /// 向量搜索时间（毫秒）
    pub vector_search_time_ms: u64,
    
    /// BM25搜索时间（毫秒）
    pub bm25_search_time_ms: u64,
    
    /// 精确匹配时间（毫秒）
    pub exact_match_time_ms: u64,
    
    /// 融合时间（毫秒）
    pub fusion_time_ms: u64,
    
    /// 向量搜索结果数
    pub vector_results_count: usize,
    
    /// BM25搜索结果数
    pub bm25_results_count: usize,
    
    /// 精确匹配结果数
    pub exact_match_results_count: usize,
    
    /// 最终结果数
    pub final_results_count: usize,
    
    /// 使用的阈值
    pub threshold_used: f32,
}

/// 调试信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub query: String,
    pub query_features: Option<String>,
    pub vector_scores: Vec<f32>,
    pub bm25_scores: Vec<f32>,
    pub fusion_scores: Vec<f32>,
}

/// 向量搜索trait（抽象）
#[async_trait::async_trait]
pub trait VectorSearcher: Send + Sync {
    async fn search(&self, query: &str, limit: usize, threshold: f32) 
        -> Result<Vec<SearchResult>>;
}

/// BM25搜索trait（抽象）
#[async_trait::async_trait]
pub trait BM25Searcher: Send + Sync {
    async fn search(&self, query: &str, limit: usize) 
        -> Result<Vec<SearchResult>>;
}

/// 精确匹配trait（抽象）
#[async_trait::async_trait]
pub trait ExactMatcher: Send + Sync {
    async fn match_exact(&self, query: &str, limit: usize) 
        -> Result<Vec<SearchResult>>;
}

/// 增强的混合搜索引擎
pub struct EnhancedHybridSearchEngine {
    config: EnhancedHybridConfig,
    
    // 核心组件
    query_classifier: Arc<QueryClassifier>,
    threshold_calculator: Arc<AdaptiveThresholdCalculator>,
    
    // 搜索器（使用trait对象实现解耦）
    vector_searcher: Option<Arc<dyn VectorSearcher>>,
    bm25_searcher: Option<Arc<dyn BM25Searcher>>,
    exact_matcher: Option<Arc<dyn ExactMatcher>>,
    
    // 性能监控
    metrics: Arc<RwLock<SearchMetrics>>,
}

/// 搜索指标
#[derive(Debug, Clone, Default)]
pub struct SearchMetrics {
    /// 总查询数
    pub total_queries: usize,
    
    /// 各类型查询数
    pub queries_by_type: HashMap<QueryType, usize>,
    
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    
    /// P99延迟（毫秒）
    pub p99_latency_ms: u64,
    
    /// 缓存命中率
    pub cache_hit_rate: f64,
}

impl EnhancedHybridSearchEngine {
    /// 创建新的增强混合搜索引擎
    pub fn new(config: EnhancedHybridConfig) -> Self {
        let query_classifier = Arc::new(QueryClassifier::with_default_config());
        let threshold_calculator = Arc::new(AdaptiveThresholdCalculator::with_default_config());
        
        Self {
            config,
            query_classifier,
            threshold_calculator,
            vector_searcher: None,
            bm25_searcher: None,
            exact_matcher: None,
            metrics: Arc::new(RwLock::new(SearchMetrics::default())),
        }
    }
    
    /// 设置向量搜索器
    pub fn with_vector_searcher(mut self, searcher: Arc<dyn VectorSearcher>) -> Self {
        self.vector_searcher = Some(searcher);
        self
    }
    
    /// 设置BM25搜索器
    pub fn with_bm25_searcher(mut self, searcher: Arc<dyn BM25Searcher>) -> Self {
        self.bm25_searcher = Some(searcher);
        self
    }
    
    /// 设置精确匹配器
    pub fn with_exact_matcher(mut self, matcher: Arc<dyn ExactMatcher>) -> Self {
        self.exact_matcher = Some(matcher);
        self
    }
    
    /// 执行搜索
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<EnhancedSearchResult> {
        let start = Instant::now();
        
        // 1. 查询分类
        let classification_start = Instant::now();
        let (query_type, mut strategy) = if self.config.enable_query_classification {
            self.query_classifier.get_adaptive_strategy(query)
        } else {
            (QueryType::NaturalLanguage, SearchStrategy::default())
        };
        let classification_time_ms = classification_start.elapsed().as_millis() as u64;
        
        debug!("Query classified as: {:?}, strategy: {:?}", query_type, strategy);
        
        // 2. 自适应阈值计算
        if self.config.enable_adaptive_threshold {
            let features = self.query_classifier.extract_features(query);
            strategy.threshold = self.threshold_calculator
                .calculate(query, &query_type, &features)
                .await;
        }
        
        debug!("Using threshold: {}", strategy.threshold);
        
        // 3. 执行搜索
        let (results, stats) = self.execute_search(
            query,
            limit,
            &query_type,
            &strategy,
        ).await?;
        
        let total_time_ms = start.elapsed().as_millis() as u64;
        
        // 4. 更新指标
        if self.config.enable_metrics {
            self.update_metrics(query_type, total_time_ms).await;
        }
        
        // 5. 记录反馈（用于自适应调整）
        if !results.is_empty() {
            let avg_score = results.iter()
                .map(|r| r.score)
                .sum::<f32>() / results.len() as f32;
            self.threshold_calculator.record_feedback(query_type, avg_score).await;
        }
        
        Ok(EnhancedSearchResult {
            results,
            query_type,
            strategy,
            stats: EnhancedSearchStats {
                total_time_ms,
                classification_time_ms,
                ..stats
            },
            debug_info: None, // 可以在调试模式下填充
        })
    }
    
    /// 执行实际搜索
    async fn execute_search(
        &self,
        query: &str,
        limit: usize,
        query_type: &QueryType,
        strategy: &SearchStrategy,
    ) -> Result<(Vec<SearchResult>, EnhancedSearchStats)> {
        let mut stats = EnhancedSearchStats {
            total_time_ms: 0,
            classification_time_ms: 0,
            vector_search_time_ms: 0,
            bm25_search_time_ms: 0,
            exact_match_time_ms: 0,
            fusion_time_ms: 0,
            vector_results_count: 0,
            bm25_results_count: 0,
            exact_match_results_count: 0,
            final_results_count: 0,
            threshold_used: strategy.threshold,
        };
        
        // 策略1: 精确匹配优先
        if strategy.use_exact_match {
            if let Some(matcher) = &self.exact_matcher {
                let start = Instant::now();
                let results = matcher.match_exact(query, limit).await?;
                stats.exact_match_time_ms = start.elapsed().as_millis() as u64;
                stats.exact_match_results_count = results.len();
                
                if !results.is_empty() {
                    stats.final_results_count = results.len();
                    return Ok((results, stats));
                }
            }
        }
        
        // 策略2: 并行执行向量搜索和BM25搜索
        let (vector_results, bm25_results) = if self.config.enable_parallel {
            self.parallel_search(query, limit * 2, strategy, &mut stats).await?
        } else {
            self.sequential_search(query, limit * 2, strategy, &mut stats).await?
        };
        
        // 策略3: 融合结果
        let fusion_start = Instant::now();
        let fused = self.fuse_results(
            vector_results, 
            bm25_results, 
            strategy.vector_weight,
            strategy.bm25_weight
        );
        stats.fusion_time_ms = fusion_start.elapsed().as_millis() as u64;
        
        // 策略4: 过滤和截断
        let filtered: Vec<_> = fused.into_iter()
            .filter(|r| r.score >= strategy.threshold)
            .take(limit)
            .collect();
        
        stats.final_results_count = filtered.len();
        
        Ok((filtered, stats))
    }
    
    /// 并行搜索
    async fn parallel_search(
        &self,
        query: &str,
        limit: usize,
        strategy: &SearchStrategy,
        stats: &mut EnhancedSearchStats,
    ) -> Result<(Vec<SearchResult>, Vec<SearchResult>)> {
        let vector_searcher = self.vector_searcher.clone();
        let bm25_searcher = self.bm25_searcher.clone();
        let query = query.to_string();
        let threshold = strategy.threshold;
        
        let (vector_result, bm25_result) = tokio::join!(
            async {
                if !strategy.use_vector {
                    return Ok((Vec::new(), 0u64));
                }
                
                if let Some(searcher) = vector_searcher {
                    let start = Instant::now();
                    let results = searcher.search(&query, limit, threshold).await?;
                    let time = start.elapsed().as_millis() as u64;
                    Ok((results, time))
                } else {
                    Ok((Vec::new(), 0u64))
                }
            },
            async {
                if !strategy.use_bm25 {
                    return Ok((Vec::new(), 0u64));
                }
                
                if let Some(searcher) = bm25_searcher {
                    let start = Instant::now();
                    let results = searcher.search(&query, limit).await?;
                    let time = start.elapsed().as_millis() as u64;
                    Ok((results, time))
                } else {
                    Ok((Vec::new(), 0u64))
                }
            }
        );
        
        let (vector_results, vector_time) = vector_result?;
        let (bm25_results, bm25_time) = bm25_result?;
        
        stats.vector_search_time_ms = vector_time;
        stats.bm25_search_time_ms = bm25_time;
        stats.vector_results_count = vector_results.len();
        stats.bm25_results_count = bm25_results.len();
        
        Ok((vector_results, bm25_results))
    }
    
    /// 顺序搜索
    async fn sequential_search(
        &self,
        query: &str,
        limit: usize,
        strategy: &SearchStrategy,
        stats: &mut EnhancedSearchStats,
    ) -> Result<(Vec<SearchResult>, Vec<SearchResult>)> {
        let mut vector_results = Vec::new();
        let mut bm25_results = Vec::new();
        
        if strategy.use_vector {
            if let Some(searcher) = &self.vector_searcher {
                let start = Instant::now();
                vector_results = searcher.search(query, limit, strategy.threshold).await?;
                stats.vector_search_time_ms = start.elapsed().as_millis() as u64;
                stats.vector_results_count = vector_results.len();
            }
        }
        
        if strategy.use_bm25 {
            if let Some(searcher) = &self.bm25_searcher {
                let start = Instant::now();
                bm25_results = searcher.search(query, limit).await?;
                stats.bm25_search_time_ms = start.elapsed().as_millis() as u64;
                stats.bm25_results_count = bm25_results.len();
            }
        }
        
        Ok((vector_results, bm25_results))
    }
    
    /// RRF融合
    fn fuse_results(
        &self,
        vector_results: Vec<SearchResult>,
        bm25_results: Vec<SearchResult>,
        vector_weight: f32,
        bm25_weight: f32,
    ) -> Vec<SearchResult> {
        let mut score_map: HashMap<String, (SearchResult, f32)> = HashMap::new();
        let k = self.config.rrf_k;
        
        // 向量搜索结果
        for (rank, mut result) in vector_results.into_iter().enumerate() {
            let rrf_score = vector_weight / (k + (rank + 1) as f32);
            result.vector_score = Some(result.score);
            score_map.insert(result.id.clone(), (result, rrf_score));
        }
        
        // BM25搜索结果
        for (rank, result) in bm25_results.into_iter().enumerate() {
            let rrf_score = bm25_weight / (k + (rank + 1) as f32);
            
            score_map.entry(result.id.clone())
                .and_modify(|(r, score)| {
                    r.fulltext_score = Some(result.score);
                    *score += rrf_score;
                })
                .or_insert_with(|| {
                    let mut r = result.clone();
                    r.fulltext_score = Some(result.score);
                    (r, rrf_score)
                });
        }
        
        // 排序
        let mut results: Vec<_> = score_map.into_iter()
            .map(|(_, (mut r, score))| {
                r.score = score;
                r
            })
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        results
    }
    
    /// 更新指标
    async fn update_metrics(&self, query_type: QueryType, latency_ms: u64) {
        if let Ok(mut metrics) = self.metrics.try_write() {
            metrics.total_queries += 1;
            
            *metrics.queries_by_type.entry(query_type).or_insert(0) += 1;
            
            // 更新平均延迟（简单移动平均）
            metrics.avg_latency_ms = (metrics.avg_latency_ms * (metrics.total_queries - 1) as f64 
                + latency_ms as f64) / metrics.total_queries as f64;
            
            // 更新P99延迟（简化版本）
            if latency_ms > metrics.p99_latency_ms {
                metrics.p99_latency_ms = latency_ms;
            }
        }
    }
    
    /// 获取指标
    pub async fn get_metrics(&self) -> SearchMetrics {
        self.metrics.read().await.clone()
    }
    
    /// 重置指标
    pub async fn reset_metrics(&self) {
        *self.metrics.write().await = SearchMetrics::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enhanced_hybrid_config() {
        let config = EnhancedHybridConfig::default();
        assert!(config.enable_query_classification);
        assert!(config.enable_adaptive_threshold);
        assert_eq!(config.rrf_k, 60.0);
    }
    
    #[test]
    fn test_search_metrics() {
        let metrics = SearchMetrics::default();
        assert_eq!(metrics.total_queries, 0);
        assert_eq!(metrics.avg_latency_ms, 0.0);
    }
}

