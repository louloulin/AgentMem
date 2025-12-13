//! 搜索统计模块
//!
//! 提供搜索操作的统计信息收集和查询功能

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 搜索统计信息（全局单例）
#[derive(Debug, Clone)]
pub struct SearchStatistics {
    /// 总搜索次数
    pub total_searches: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 精确查询次数（LibSQL）
    pub exact_queries: u64,
    /// 向量搜索次数
    pub vector_searches: u64,
    /// 总搜索延迟（微秒）
    pub total_latency_us: u64,
    /// 最后更新时间
    pub last_updated: Instant,
}

impl SearchStatistics {
    pub fn new() -> Self {
        Self {
            total_searches: 0,
            cache_hits: 0,
            cache_misses: 0,
            exact_queries: 0,
            vector_searches: 0,
            total_latency_us: 0,
            last_updated: Instant::now(),
        }
    }

    /// 获取缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.cache_hits as f64) / (self.total_searches as f64)
    }

    /// 获取平均搜索延迟（毫秒）
    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.total_latency_us as f64) / (self.total_searches as f64) / 1000.0
    }

    /// 获取搜索统计的公共访问方法
    pub fn get_total_searches(&self) -> u64 {
        self.total_searches
    }

    pub fn get_cache_hits(&self) -> u64 {
        self.cache_hits
    }

    pub fn get_cache_misses(&self) -> u64 {
        self.cache_misses
    }

    pub fn get_exact_queries(&self) -> u64 {
        self.exact_queries
    }

    pub fn get_vector_searches(&self) -> u64 {
        self.vector_searches
    }
}

impl Default for SearchStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// 搜索统计（全局单例）
static SEARCH_STATS: std::sync::OnceLock<Arc<RwLock<SearchStatistics>>> =
    std::sync::OnceLock::new();

/// 获取搜索统计
pub fn get_search_stats() -> Arc<RwLock<SearchStatistics>> {
    SEARCH_STATS.get_or_init(|| {
        Arc::new(RwLock::new(SearchStatistics::new()))
    }).clone()
}
