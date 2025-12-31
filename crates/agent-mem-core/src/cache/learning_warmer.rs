//! 基于学习的智能缓存预热
//!
//! 利用学习引擎的统计数据来智能预热缓存，提升冷启动性能

use super::WarmingStats;
use crate::search::learning::{LearningEngine, QueryPattern};
use crate::search::QueryFeatures;
use agent_mem_traits::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 学习驱动的缓存预热器配置
#[derive(Debug, Clone)]
pub struct LearningWarmingConfig {
    /// 预热的查询模式数量（Top N）
    pub top_patterns: usize,

    /// 每个模式预热的查询数量
    pub queries_per_pattern: usize,

    /// 最小查询次数阈值（过滤掉不常用的模式）
    pub min_query_count: usize,

    /// 最小有效性阈值（只预热效果好的模式）
    pub min_effectiveness: f32,

    /// 预热间隔
    pub warming_interval: Duration,

    /// 是否在启动时预热
    pub warm_on_startup: bool,
}

impl Default for LearningWarmingConfig {
    fn default() -> Self {
        Self {
            top_patterns: 20,                           // Top 20 热门模式
            queries_per_pattern: 3,                     // 每个模式3个代表性查询
            min_query_count: 10,                        // 至少10次查询
            min_effectiveness: 0.7,                     // 效果≥70%
            warming_interval: Duration::from_secs(300), // 5分钟
            warm_on_startup: true,
        }
    }
}

/// 模式预热候选
#[derive(Debug, Clone)]
struct PatternCandidate {
    pattern: QueryPattern,
    query_count: usize,
    avg_effectiveness: f32,
    representative_features: Vec<QueryFeatures>,
}

/// 基于学习的缓存预热器
///
/// 从学习引擎获取热门查询模式，生成代表性查询，提前预热缓存
pub struct LearningBasedCacheWarmer {
    /// 学习引擎
    learning_engine: Arc<LearningEngine>,

    /// 配置
    config: LearningWarmingConfig,

    /// 预热统计
    stats: Arc<RwLock<WarmingStats>>,

    /// 上次预热的模式（避免重复）
    last_warmed_patterns: Arc<RwLock<HashMap<String, Instant>>>,
}

impl LearningBasedCacheWarmer {
    /// 创建新的学习驱动预热器
    pub fn new(learning_engine: Arc<LearningEngine>, config: LearningWarmingConfig) -> Self {
        Self {
            learning_engine,
            config,
            stats: Arc::new(RwLock::new(WarmingStats::default())),
            last_warmed_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取热门查询模式
    async fn get_hot_patterns(&self) -> Result<Vec<PatternCandidate>> {
        // 从学习引擎获取所有模式统计
        let all_stats = self.learning_engine.get_all_pattern_stats().await;

        // 过滤和排序
        let mut candidates: Vec<PatternCandidate> = all_stats
            .into_iter()
            .filter(|(_, stats)| {
                stats.total_queries >= self.config.min_query_count
                    && stats.avg_effectiveness >= self.config.min_effectiveness
            })
            .map(|(pattern, stats)| {
                // 从平均权重推断特征
                let features = QueryFeatures {
                    has_exact_terms: pattern == QueryPattern::ExactMatch,
                    semantic_complexity: 0.5,
                    has_temporal_indicator: pattern == QueryPattern::TemporalQuery,
                    entity_count: 2,
                    query_length: 50,
                    is_question: pattern == QueryPattern::SemanticQuestion,
                };

                PatternCandidate {
                    pattern,
                    query_count: stats.total_queries,
                    avg_effectiveness: stats.avg_effectiveness,
                    representative_features: vec![features],
                }
            })
            .collect();

        // 按查询次数 * 效果排序
        candidates.sort_by(|a, b| {
            let score_a = a.query_count as f32 * a.avg_effectiveness;
            let score_b = b.query_count as f32 * b.avg_effectiveness;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 取Top N
        candidates.truncate(self.config.top_patterns);

        debug!("Found {} hot patterns for cache warming", candidates.len());

        Ok(candidates)
    }

    /// 为模式生成代表性查询特征
    fn generate_representative_queries(
        &self,
        pattern: &QueryPattern,
        base_features: &QueryFeatures,
    ) -> Vec<QueryFeatures> {
        let mut queries = Vec::new();

        // 基础查询
        queries.push(base_features.clone());

        // 根据模式类型生成变体
        match pattern {
            QueryPattern::ExactMatch => {
                // 精确匹配：生成不同长度的变体
                let mut variant1 = base_features.clone();
                variant1.query_length = (base_features.query_length as f32 * 0.8) as usize;
                queries.push(variant1);

                let mut variant2 = base_features.clone();
                variant2.query_length = (base_features.query_length as f32 * 1.2) as usize;
                queries.push(variant2);
            }
            QueryPattern::SemanticQuestion => {
                // 语义问句：生成不同复杂度的变体
                let mut variant1 = base_features.clone();
                variant1.semantic_complexity *= 0.8;
                queries.push(variant1);

                let mut variant2 = base_features.clone();
                variant2.semantic_complexity *= 1.2;
                queries.push(variant2);
            }
            QueryPattern::TemporalQuery => {
                // 时间查询：保持时间指示符
                let mut variant = base_features.clone();
                variant.has_temporal_indicator = true;
                queries.push(variant);
            }
            _ => {
                // 其他模式：生成通用变体
                let mut variant = base_features.clone();
                variant.entity_count = (variant.entity_count + 1).min(10);
                queries.push(variant);
            }
        }

        // 限制数量
        queries.truncate(self.config.queries_per_pattern);
        queries
    }

    /// 执行预热（需要具体的搜索引擎实例）
    ///
    /// 注意：这个方法是框架方法，实际使用时需要提供搜索引擎
    pub async fn warm_cache_with_engine<F, Fut>(&self, search_fn: F) -> Result<WarmingStats>
    where
        F: Fn(QueryFeatures) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let start = Instant::now();
        let mut items_warmed = 0;
        let mut failed = 0;

        info!("Starting learning-based cache warming");

        // 1. 获取热门模式
        let hot_patterns = self.get_hot_patterns().await?;

        if hot_patterns.is_empty() {
            warn!("No hot patterns found for warming");
            return Ok(WarmingStats::default());
        }

        info!("Warming {} hot patterns", hot_patterns.len());

        // 2. 为每个模式预热
        for candidate in hot_patterns {
            // 检查是否最近已预热
            if self.should_skip_pattern(&candidate.pattern).await {
                debug!("Skipping recently warmed pattern: {:?}", candidate.pattern);
                continue;
            }

            // 生成代表性查询
            let queries = self.generate_representative_queries(
                &candidate.pattern,
                &candidate.representative_features[0],
            );

            // 执行预热
            for features in queries {
                match search_fn(features).await {
                    Ok(_) => {
                        items_warmed += 1;
                    }
                    Err(e) => {
                        warn!("Failed to warm query: {}", e);
                        failed += 1;
                    }
                }
            }

            // 记录已预热
            self.mark_pattern_warmed(&candidate.pattern).await;
        }

        let elapsed = start.elapsed();

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_warmings += 1;
        stats.total_items_warmed += items_warmed;
        stats.total_warming_time_ms += elapsed.as_millis() as u64;
        stats.failed_warmings += failed;
        stats.last_warming_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                tracing::warn!("System time is before UNIX epoch: {e}, using 0 as timestamp");
                std::time::Duration::ZERO
            })
            .unwrap_or_default()
            .as_secs();

        let final_stats = stats.clone();
        drop(stats);

        info!(
            "Cache warming completed: {} items in {:?}, {} failed",
            items_warmed, elapsed, failed
        );

        Ok(final_stats)
    }

    /// 检查是否应该跳过该模式（最近已预热）
    async fn should_skip_pattern(&self, pattern: &QueryPattern) -> bool {
        let last_warmed = self.last_warmed_patterns.read().await;

        if let Some(last_time) = last_warmed.get(pattern.as_str()) {
            // 如果距离上次预热不到预热间隔的一半，跳过
            last_time.elapsed() < self.config.warming_interval / 2
        } else {
            false
        }
    }

    /// 标记模式已预热
    async fn mark_pattern_warmed(&self, pattern: &QueryPattern) {
        let mut last_warmed = self.last_warmed_patterns.write().await;
        last_warmed.insert(pattern.as_str().to_string(), Instant::now());
    }

    /// 获取预热统计
    pub async fn get_stats(&self) -> WarmingStats {
        self.stats.read().await.clone()
    }

    /// 重置统计
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = WarmingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::LearningConfig;

    #[test]
    fn test_config_default() {
        let config = LearningWarmingConfig::default();
        assert_eq!(config.top_patterns, 20);
        assert_eq!(config.queries_per_pattern, 3);
        assert_eq!(config.min_query_count, 10);
        assert_eq!(config.min_effectiveness, 0.7);
    }

    #[test]
    fn test_representative_query_generation() {
        let learning_engine = Arc::new(LearningEngine::new(LearningConfig::default()));
        let warmer =
            LearningBasedCacheWarmer::new(learning_engine, LearningWarmingConfig::default());

        let base_features = QueryFeatures {
            has_exact_terms: true,
            semantic_complexity: 0.5,
            has_temporal_indicator: false,
            entity_count: 2,
            query_length: 50,
            is_question: false,
        };

        let queries =
            warmer.generate_representative_queries(&QueryPattern::ExactMatch, &base_features);

        // 应该生成多个变体
        assert!(queries.len() > 1);
        assert!(queries.len() <= 3);
    }
}
