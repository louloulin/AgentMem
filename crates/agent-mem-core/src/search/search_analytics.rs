// Search Analytics Panel - 搜索分析面板
//
// 提供搜索统计和性能分析功能，用于监控和优化搜索质量

use serde::{Deserialize, Serialize};
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
}
