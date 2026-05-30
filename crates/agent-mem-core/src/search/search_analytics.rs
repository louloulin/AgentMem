// Search Analytics Panel - 搜索分析面板
//
// 提供搜索统计和性能分析功能，用于监控和优化搜索质量

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 搜索分析器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnalyticsConfig {
    /// 是否启用分析
    pub enabled: bool,
    /// 历史记录保留数量
    pub history_size: usize,
    /// 性能指标窗口大小
    pub perf_window_size: usize,
}

impl Default for SearchAnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_size: 1000,
            perf_window_size: 100,
        }
    }
}

/// 搜索事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchEventType {
    /// 常规搜索
    Search,
    /// 重排序搜索
    RerankedSearch,
    /// 批量搜索
    BatchSearch,
    /// 自适应搜索
    AdaptiveSearch,
}

/// 搜索事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvent {
    /// 事件 ID
    pub id: String,
    /// 事件类型
    pub event_type: SearchEventType,
    /// 查询文本
    pub query: String,
    /// 查询长度
    pub query_length: usize,
    /// 返回结果数
    pub result_count: usize,
    /// 响应时间 (毫秒)
    pub response_time_ms: u64,
    /// 平均得分
    pub avg_score: f32,
    /// 最高得分
    pub max_score: f32,
    /// 是否使用缓存
    pub cache_hit: bool,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 第一相关结果的位置 (用于MRR计算，0表示无相关结果)
    pub first_relevant_position: Option<u32>,
    /// 各结果的相关性得分列表 (用于NDCG计算)
    pub relevance_scores: Option<Vec<f32>>,
}

/// 性能指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 总搜索次数
    pub total_searches: u64,
    /// 总响应时间 (毫秒)
    pub total_response_time_ms: u64,
    /// 最小响应时间 (毫秒)
    pub min_response_time_ms: u64,
    /// 最大响应时间 (毫秒)
    pub max_response_time_ms: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 结果数为0的搜索次数
    pub empty_result_searches: u64,
}

/// 查询模式统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryPatternStats {
    /// 短查询 (1-3词) 次数
    pub short_queries: u64,
    /// 中查询 (4-10词) 次数
    pub medium_queries: u64,
    /// 长查询 (10+词) 次数
    pub long_queries: u64,
    /// 平均查询长度
    pub avg_query_length: f64,
}

/// 结果分布统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultDistribution {
    /// 高分结果数 (>0.8)
    pub high_score_results: u64,
    /// 中分结果数 (0.5-0.8)
    pub medium_score_results: u64,
    /// 低分结果数 (0.3-0.5)
    pub low_score_results: u64,
    /// 无结果搜索数
    pub zero_result_searches: u64,
}

/// 搜索质量指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// 平均结果得分
    pub avg_result_score: f64,
    /// 最高结果得分
    pub max_result_score: f64,
    /// 结果得分标准差
    pub score_std_dev: f64,
    /// Mean Reciprocal Rank (MRR) - 检索质量指标
    pub mean_reciprocal_rank: f64,
    /// Normalized Discounted Cumulative Gain (NDCG) - 排名质量指标
    pub ndcg: f64,
    /// MRR计算样本数
    pub mrr_sample_count: u64,
    /// NDCG计算样本数
    pub ndcg_sample_count: u64,
}

/// 搜索分析面板
pub struct SearchAnalytics {
    config: SearchAnalyticsConfig,
    /// 搜索事件历史
    events: Arc<RwLock<Vec<SearchEvent>>>,
    /// 性能指标
    performance: Arc<RwLock<PerformanceMetrics>>,
    /// 查询模式统计
    query_patterns: Arc<RwLock<QueryPatternStats>>,
    /// 结果分布
    result_distribution: Arc<RwLock<ResultDistribution>>,
    /// 质量指标
    quality_metrics: Arc<RwLock<QualityMetrics>>,
    /// 按小时统计
    hourly_stats: Arc<RwLock<HashMap<u32, u64>>>,
}

impl Default for SearchAnalytics {
    fn default() -> Self {
        Self::new(SearchAnalyticsConfig::default())
    }
}

impl SearchAnalytics {
    /// 创建新的分析器
    pub fn new(config: SearchAnalyticsConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
            performance: Arc::new(RwLock::new(PerformanceMetrics::default())),
            query_patterns: Arc::new(RwLock::new(QueryPatternStats::default())),
            result_distribution: Arc::new(RwLock::new(ResultDistribution::default())),
            quality_metrics: Arc::new(RwLock::new(QualityMetrics::default())),
            hourly_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录搜索事件
    pub async fn record_search(&self, event: SearchEvent) {
        if !self.config.enabled {
            return;
        }

        // 更新历史记录
        {
            let mut events = self.events.write().await;
            events.push(event.clone());
            // 保持历史大小
            if events.len() > self.config.history_size {
                events.remove(0);
            }
        }

        // 更新性能指标
        {
            let mut perf = self.performance.write().await;
            perf.total_searches += 1;
            perf.total_response_time_ms += event.response_time_ms;
            
            if event.response_time_ms > 0 {
                if perf.min_response_time_ms == 0 || event.response_time_ms < perf.min_response_time_ms {
                    perf.min_response_time_ms = event.response_time_ms;
                }
            }
            if event.response_time_ms > perf.max_response_time_ms {
                perf.max_response_time_ms = event.response_time_ms;
            }
            
            if event.cache_hit {
                perf.cache_hits += 1;
            } else {
                perf.cache_misses += 1;
            }
            
            if event.result_count == 0 {
                perf.empty_result_searches += 1;
            }
        }

        // 更新查询模式统计
        {
            let mut patterns = self.query_patterns.write().await;
            let len = event.query_length;
            if len <= 3 {
                patterns.short_queries += 1;
            } else if len <= 10 {
                patterns.medium_queries += 1;
            } else {
                patterns.long_queries += 1;
            }
            
            // 更新平均长度
            let total = patterns.short_queries + patterns.medium_queries + patterns.long_queries;
            if total > 0 {
                patterns.avg_query_length = 
                    (patterns.short_queries as f64 * 2.0 
                    + patterns.medium_queries as f64 * 7.0 
                    + patterns.long_queries as f64 * 15.0) 
                    / total as f64;
            }
        }

        // 更新结果分布
        {
            let mut dist = self.result_distribution.write().await;
            dist.zero_result_searches += 1;
            
            if event.result_count > 0 {
                dist.zero_result_searches -= 1; // 撤销刚才的计数
                
                if event.max_score > 0.8 {
                    dist.high_score_results += 1;
                } else if event.max_score > 0.5 {
                    dist.medium_score_results += 1;
                } else if event.max_score >= 0.3 {
                    dist.low_score_results += 1;
                } else {
                    dist.zero_result_searches += 1;
                }
            }
        }

        // 更新质量指标
        {
            let mut quality = self.quality_metrics.write().await;
            let total_score = quality.avg_result_score * (perf::change_time_total_searches(&self.performance).await - 1) as f64
                + event.avg_score as f64;
            let searches = perf::change_time_total_searches(&self.performance).await;
            quality.avg_result_score = total_score / searches as f64;

            if event.max_score > quality.max_result_score as f32 {
                quality.max_result_score = event.max_score as f64;
            }

            // 更新MRR (Mean Reciprocal Rank)
            if let Some(first_relevant_pos) = event.first_relevant_position {
                if first_relevant_pos > 0 {
                    quality.mrr_sample_count += 1;
                    let reciprocal_rank = 1.0 / first_relevant_pos as f64;
                    quality.mean_reciprocal_rank =
                        (quality.mean_reciprocal_rank * (quality.mrr_sample_count - 1) as f64 + reciprocal_rank)
                        / quality.mrr_sample_count as f64;
                }
            }

            // 更新NDCG (Normalized Discounted Cumulative Gain)
            if let Some(ref relevance_scores) = event.relevance_scores {
                if !relevance_scores.is_empty() {
                    quality.ndcg_sample_count += 1;
                    let dcg = Self::calculate_dcg(relevance_scores);
                    // 计算理想DCG - 将相关性得分降序排列
                    let mut ideal_relevance: Vec<f32> = relevance_scores.iter().cloned().collect();
                    ideal_relevance.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
                    let idcg = Self::calculate_dcg(&ideal_relevance);
                    let ndcg = if idcg > 0.0 { dcg / idcg } else { 0.0 };
                    quality.ndcg =
                        (quality.ndcg * (quality.ndcg_sample_count - 1) as f64 + ndcg)
                        / quality.ndcg_sample_count as f64;
                }
            }
        }

        // 更新小时统计
        {
            let mut hourly = self.hourly_stats.write().await;
            let hour = event.timestamp.format("%H").to_string().parse::<u32>().unwrap_or(0);
            *hourly.entry(hour).or_insert(0) += 1;
        }
    }

    /// 获取性能指标
    pub async fn get_performance(&self) -> PerformanceMetrics {
        self.performance.read().await.clone()
    }

    /// 获取查询模式统计
    pub async fn get_query_patterns(&self) -> QueryPatternStats {
        self.query_patterns.read().await.clone()
    }

    /// 获取结果分布
    pub async fn get_result_distribution(&self) -> ResultDistribution {
        self.result_distribution.read().await.clone()
    }

    /// 获取质量指标
    pub async fn get_quality_metrics(&self) -> QualityMetrics {
        self.quality_metrics.read().await.clone()
    }

    /// 获取分析报告
    pub async fn get_report(&self) -> AnalyticsReport {
        let perf = self.get_performance().await;
        let patterns = self.get_query_patterns().await;
        let dist = self.get_result_distribution().await;
        let quality = self.get_quality_metrics().await;
        
        let cache_hit_rate = if perf.cache_hits + perf.cache_misses > 0 {
            perf.cache_hits as f64 / (perf.cache_hits + perf.cache_misses) as f64
        } else {
            0.0
        };
        
        let avg_response_time = if perf.total_searches > 0 {
            perf.total_response_time_ms as f64 / perf.total_searches as f64
        } else {
            0.0
        };
        
        let mut hourly = self.hourly_stats.read().await.clone();
        let peak_hour = hourly.iter().max_by_key(|(_, v)| *v).map(|(k, _)| *k);
        
        AnalyticsReport {
            performance: perf,
            query_patterns: patterns,
            result_distribution: dist,
            quality_metrics: quality,
            cache_hit_rate,
            avg_response_time_ms: avg_response_time,
            peak_search_hour: peak_hour,
        }
    }

    /// 重置统计数据
    pub async fn reset(&self) {
        *self.events.write().await = Vec::new();
        *self.performance.write().await = PerformanceMetrics::default();
        *self.query_patterns.write().await = QueryPatternStats::default();
        *self.result_distribution.write().await = ResultDistribution::default();
        *self.quality_metrics.write().await = QualityMetrics::default();
        *self.hourly_stats.write().await = HashMap::new();
    }

    /// 计算DCG (Discounted Cumulative Gain)
    /// 使用标准DCG公式: DCG = sum(rel_i / log2(i+1), i从1到结果数)
    fn calculate_dcg(relevance_scores: &[f32]) -> f64 {
        relevance_scores
            .iter()
            .enumerate()
            .map(|(i, &rel)| {
                if rel > 0.0 {
                    rel as f64 / (i as f64 + 2.0).log2()
                } else {
                    0.0
                }
            })
            .sum()
    }
}

// 辅助函数
mod perf {
    use std::sync::Arc;
    pub(super) async fn change_time_total_searches(perf: &Arc<tokio::sync::RwLock<super::PerformanceMetrics>>) -> u64 {
        perf.read().await.total_searches
    }
}

/// 搜索分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    /// 性能指标
    pub performance: PerformanceMetrics,
    /// 查询模式统计
    pub query_patterns: QueryPatternStats,
    /// 结果分布
    pub result_distribution: ResultDistribution,
    /// 质量指标
    pub quality_metrics: QualityMetrics,
    /// 缓存命中率
    pub cache_hit_rate: f64,
    /// 平均响应时间 (毫秒)
    pub avg_response_time_ms: f64,
    /// 搜索高峰小时
    pub peak_search_hour: Option<u32>,
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    fn create_test_event(query: &str, result_count: usize, response_time_ms: u64) -> SearchEvent {
        SearchEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SearchEventType::Search,
            query: query.to_string(),
            query_length: query.split_whitespace().count(),
            result_count,
            response_time_ms,
            avg_score: 0.7,
            max_score: 0.85,
            cache_hit: false,
            timestamp: chrono::Utc::now(),
            first_relevant_position: None,
            relevance_scores: None,
        }
    }

    fn create_test_event_with_relevance(query: &str, result_count: usize, first_relevant: Option<u32>, relevance: Vec<f32>) -> SearchEvent {
        SearchEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SearchEventType::Search,
            query: query.to_string(),
            query_length: query.split_whitespace().count(),
            result_count,
            response_time_ms: 100,
            avg_score: 0.7,
            max_score: relevance.iter().cloned().fold(0.0, f32::max),
            cache_hit: false,
            timestamp: chrono::Utc::now(),
            first_relevant_position: first_relevant,
            relevance_scores: Some(relevance),
        }
    }

    #[tokio::test]
    async fn test_search_analytics_record() {
        let analytics = SearchAnalytics::default();
        
        let event = create_test_event("test query", 5, 100);
        analytics.record_search(event).await;
        
        let perf = analytics.get_performance().await;
        assert_eq!(perf.total_searches, 1);
    }

    #[tokio::test]
    async fn test_query_pattern_classification() {
        let analytics = SearchAnalytics::default();
        
        // 短查询
        analytics.record_search(create_test_event("hi", 3, 50)).await;
        
        // 中查询
        analytics.record_search(create_test_event("this is a test query", 5, 80)).await;
        
        // 长查询
        analytics.record_search(create_test_event("one two three four five six seven eight nine ten eleven", 8, 120)).await;
        
        let patterns = analytics.get_query_patterns().await;
        assert_eq!(patterns.short_queries, 1);
        assert_eq!(patterns.medium_queries, 1);
        assert_eq!(patterns.long_queries, 1);
    }

    #[tokio::test]
    async fn test_analytics_report() {
        let analytics = SearchAnalytics::default();
        
        for i in 0..5 {
            let event = create_test_event(&format!("test query {}", i), i + 1, 100 + i as u64 * 10);
            analytics.record_search(event).await;
        }
        
        let report = analytics.get_report().await;
        assert_eq!(report.performance.total_searches, 5);
        assert!(report.avg_response_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_reset() {
        let analytics = SearchAnalytics::default();

        analytics.record_search(create_test_event("test", 5, 100)).await;
        analytics.reset().await;

        let perf = analytics.get_performance().await;
        assert_eq!(perf.total_searches, 0);
    }

    #[tokio::test]
    async fn test_mrr_calculation() {
        let analytics = SearchAnalytics::default();

        // 第一条查询：相关结果在位置1，RR = 1/1 = 1
        analytics.record_search(create_test_event_with_relevance("query1", 5, Some(1), vec![1.0, 0.8, 0.6, 0.4, 0.2])).await;

        let quality = analytics.get_quality_metrics().await;
        assert!((quality.mean_reciprocal_rank - 1.0).abs() < 0.001);
        assert_eq!(quality.mrr_sample_count, 1);
    }

    #[tokio::test]
    async fn test_mrr_multiple_queries() {
        let analytics = SearchAnalytics::default();

        // 查询1：相关结果在位置1，RR = 1/1 = 1
        analytics.record_search(create_test_event_with_relevance("query1", 5, Some(1), vec![1.0, 0.8, 0.6, 0.4, 0.2])).await;
        // 查询2：相关结果在位置2，RR = 1/2 = 0.5
        analytics.record_search(create_test_event_with_relevance("query2", 5, Some(2), vec![0.8, 1.0, 0.6, 0.4, 0.2])).await;

        let quality = analytics.get_quality_metrics().await;
        // MRR = (1 + 0.5) / 2 = 0.75
        assert!((quality.mean_reciprocal_rank - 0.75).abs() < 0.001);
        assert_eq!(quality.mrr_sample_count, 2);
    }

    #[tokio::test]
    async fn test_ndcg_calculation() {
        let analytics = SearchAnalytics::default();

        // 记录一个搜索事件，计算NDCG
        analytics.record_search(create_test_event_with_relevance("test", 5, Some(2), vec![1.0, 0.8, 0.6, 0.4, 0.2])).await;

        let quality = analytics.get_quality_metrics().await;
        assert!(quality.ndcg > 0.0);
        assert!(quality.ndcg <= 1.0);
        assert_eq!(quality.ndcg_sample_count, 1);
    }

    #[tokio::test]
    async fn test_ndcg_perfect_ranking() {
        let analytics = SearchAnalytics::default();

        // 完美排序的NDCG应该接近1.0
        analytics.record_search(create_test_event_with_relevance("test", 3, Some(1), vec![1.0, 0.8, 0.6])).await;

        let quality = analytics.get_quality_metrics().await;
        // NDCG应该接近1.0（因为已经按降序排列）
        assert!(quality.ndcg > 0.9);
    }

    #[tokio::test]
    async fn test_dcg_calculation() {
        // DCG = 1/log2(2) + 0.8/log2(3) + 0.6/log2(4)
        // DCG = 1 + 0.5 + 0.3 = 1.8
        let dcg = SearchAnalytics::calculate_dcg(&vec![1.0, 0.8, 0.6]);
        assert!(dcg > 1.7 && dcg < 1.9);
    }
}
