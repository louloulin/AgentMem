//! 性能优化模块
//!
//! 提供统一的性能优化功能，包括：
//! - 向量搜索优化
//! - 图查询优化
//! - 查询缓存管理
//! - 批量操作优化
//! - 性能监控和基准测试

pub mod batch;
pub mod benchmark;
pub mod cache;
pub mod optimizer;

pub use batch::{BatchConfig, BatchProcessor, BatchStats};
pub use benchmark::{BenchmarkConfig, BenchmarkResult, PerformanceBenchmark};
pub use cache::{CacheKey, CacheStats, QueryCache, QueryCacheConfig};
pub use optimizer::{OptimizationStats, OptimizerConfig, PerformanceOptimizer};

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 查询缓存配置
    pub cache_config: QueryCacheConfig,

    /// 批量处理配置
    pub batch_config: BatchConfig,

    /// 优化器配置
    pub optimizer_config: OptimizerConfig,

    /// 是否启用性能监控
    pub enable_monitoring: bool,

    /// 监控间隔（秒）
    pub monitoring_interval_seconds: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_config: QueryCacheConfig::default(),
            batch_config: BatchConfig::default(),
            optimizer_config: OptimizerConfig::default(),
            enable_monitoring: true,
            monitoring_interval_seconds: 60,
        }
    }
}

/// 性能管理器
///
/// 统一管理所有性能优化功能
pub struct PerformanceManager {
    /// 配置
    config: PerformanceConfig,

    /// 查询缓存
    query_cache: Arc<QueryCache>,

    /// 批量处理器
    batch_processor: Arc<BatchProcessor>,

    /// 性能优化器
    optimizer: Arc<PerformanceOptimizer>,

    /// 性能基准测试
    benchmark: Arc<RwLock<PerformanceBenchmark>>,

    /// 是否正在运行
    running: Arc<RwLock<bool>>,
}

impl PerformanceManager {
    /// 创建新的性能管理器
    pub fn new(config: PerformanceConfig) -> Self {
        let query_cache = Arc::new(QueryCache::new(config.cache_config.clone()));
        let batch_processor = Arc::new(BatchProcessor::new(config.batch_config.clone()));
        let optimizer = Arc::new(PerformanceOptimizer::new(config.optimizer_config.clone()));
        let benchmark = Arc::new(RwLock::new(PerformanceBenchmark::new(
            BenchmarkConfig::default(),
        )));

        Self {
            config,
            query_cache,
            batch_processor,
            optimizer,
            benchmark,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动性能管理器
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        *running = true;

        // 启动性能监控
        if self.config.enable_monitoring {
            self.start_monitoring().await?;
        }

        Ok(())
    }

    /// 停止性能管理器
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// 启动性能监控
    async fn start_monitoring(&self) -> Result<()> {
        let interval = self.config.monitoring_interval_seconds;
        let query_cache = self.query_cache.clone();
        let batch_processor = self.batch_processor.clone();
        let optimizer = self.optimizer.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval_timer =
                tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                let is_running = *running.read().await;
                if !is_running {
                    break;
                }

                // 收集性能统计
                let cache_stats = query_cache.get_stats().await;
                let batch_stats = batch_processor.get_stats().await;
                let optimizer_stats = optimizer.get_stats().await;

                tracing::info!(
                    "Performance Stats - Cache: {:?}, Batch: {:?}, Optimizer: {:?}",
                    cache_stats,
                    batch_stats,
                    optimizer_stats
                );
            }
        });

        Ok(())
    }

    /// 获取查询缓存
    pub fn get_query_cache(&self) -> Arc<QueryCache> {
        self.query_cache.clone()
    }

    /// 获取批量处理器
    pub fn get_batch_processor(&self) -> Arc<BatchProcessor> {
        self.batch_processor.clone()
    }

    /// 获取性能优化器
    pub fn get_optimizer(&self) -> Arc<PerformanceOptimizer> {
        self.optimizer.clone()
    }

    /// 获取性能基准测试
    pub fn get_benchmark(&self) -> Arc<RwLock<PerformanceBenchmark>> {
        self.benchmark.clone()
    }

    /// 运行性能基准测试
    pub async fn run_benchmark(&self) -> Result<BenchmarkResult> {
        let mut benchmark = self.benchmark.write().await;
        benchmark.run_all_benchmarks().await
    }

    /// 优化性能
    pub async fn optimize(&self) -> Result<()> {
        // 清理过期缓存
        self.query_cache.cleanup_expired().await?;

        // 优化批量处理
        self.batch_processor.optimize().await?;

        // 运行优化器
        self.optimizer.optimize().await?;

        Ok(())
    }

    /// 获取综合性能统计
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            cache_stats: self.query_cache.get_stats().await,
            batch_stats: self.batch_processor.get_stats().await,
            optimizer_stats: self.optimizer.get_stats().await,
        }
    }
}

/// 综合性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// 缓存统计
    pub cache_stats: CacheStats,

    /// 批量处理统计
    pub batch_stats: BatchStats,

    /// 优化器统计
    pub optimizer_stats: OptimizationStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_manager_creation() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        assert!(!*manager.running.read().await);
    }

    #[tokio::test]
    async fn test_performance_manager_start_stop() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        manager.start().await?;
        assert!(*manager.running.read().await);

        manager.stop().await?;
        assert!(!*manager.running.read().await);
    }

    #[tokio::test]
    async fn test_get_performance_stats() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        let stats = manager.get_performance_stats().await;
        assert_eq!(stats.cache_stats.total_requests, 0);
    }
}
