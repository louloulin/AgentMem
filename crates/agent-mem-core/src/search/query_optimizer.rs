// Query Optimizer - 智能查询计划优化
//
// 根据查询特征和数据统计，动态选择最优搜索策略

use crate::search::SearchQuery;
use agent_mem_traits::Result;
use std::sync::{Arc, RwLock};

/// 索引类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// 无索引（线性扫描）
    None,
    /// 精确搜索（适合小数据集）
    Flat,
    /// IVF索引（聚类索引）
    IVF,
    /// HNSW索引（层次图索引）
    HNSW,
    /// 混合索引（IVF + HNSW）
    IVF_HNSW,
}

/// 索引统计信息
#[derive(Debug, Clone)]
pub struct IndexStatistics {
    /// 总向量数
    pub total_vectors: usize,
    /// 向量维度
    pub dimension: usize,
    /// 平均向量范数
    pub avg_vector_norm: f32,
    /// 当前索引类型
    pub index_type: IndexType,
    /// 最后更新时间
    pub last_updated: std::time::Instant,
}

impl Default for IndexStatistics {
    fn default() -> Self {
        Self {
            total_vectors: 0,
            dimension: 1536, // 默认OpenAI embedding维度
            avg_vector_norm: 1.0,
            index_type: IndexType::Flat,
            last_updated: std::time::Instant::now(),
        }
    }
}

impl IndexStatistics {
    /// 创建新的统计信息
    pub fn new(total_vectors: usize, dimension: usize) -> Self {
        Self {
            total_vectors,
            dimension,
            avg_vector_norm: 1.0,
            index_type: if total_vectors < 10_000 {
                IndexType::Flat
            } else if total_vectors < 100_000 {
                IndexType::HNSW
            } else {
                IndexType::IVF_HNSW
            },
            last_updated: std::time::Instant::now(),
        }
    }

    /// 更新统计信息
    pub fn update(&mut self, total_vectors: usize) {
        self.total_vectors = total_vectors;
        self.index_type = if total_vectors < 10_000 {
            IndexType::Flat
        } else if total_vectors < 100_000 {
            IndexType::HNSW
        } else {
            IndexType::IVF_HNSW
        };
        self.last_updated = std::time::Instant::now();
    }
}

/// 搜索策略
#[derive(Debug, Clone, PartialEq)]
pub enum SearchStrategy {
    /// 精确搜索（线性扫描）
    Exact,
    /// HNSW近似搜索
    HNSW {
        /// 搜索时的探索深度
        ef_search: usize,
    },
    /// IVF搜索
    IVF {
        /// 探测的聚类数
        nprobe: usize,
    },
    /// 混合搜索（IVF + HNSW）
    Hybrid { nprobe: usize, ef_search: usize },
}

/// 优化后的搜索计划
#[derive(Debug, Clone)]
pub struct OptimizedSearchPlan {
    /// 搜索策略
    pub strategy: SearchStrategy,
    /// 是否需要重排序
    pub should_rerank: bool,
    /// 重排序因子（召回候选数 = limit * rerank_factor）
    pub rerank_factor: usize,
    /// 预期延迟（毫秒）
    pub estimated_latency_ms: u64,
    /// 预期召回率
    pub estimated_recall: f32,
}

/// 查询优化器配置
#[derive(Debug, Clone)]
pub struct QueryOptimizerConfig {
    /// 小数据集阈值（使用精确搜索）
    pub small_dataset_threshold: usize,
    /// 中数据集阈值（使用HNSW）
    pub medium_dataset_threshold: usize,
    /// 默认ef_search（平衡模式）
    pub default_ef_search: usize,
    /// 高精度ef_search
    pub high_precision_ef_search: usize,
    /// 默认nprobe
    pub default_nprobe: usize,
    /// 重排序阈值（数据集大小）
    pub rerank_threshold: usize,
    /// 默认重排序因子
    pub default_rerank_factor: usize,
}

impl Default for QueryOptimizerConfig {
    fn default() -> Self {
        Self {
            small_dataset_threshold: 10_000,
            medium_dataset_threshold: 100_000,
            default_ef_search: 100,
            high_precision_ef_search: 200,
            default_nprobe: 10,
            rerank_threshold: 10_000, // 与small_dataset_threshold一致，小数据集不需要重排序
            default_rerank_factor: 3,
        }
    }
}

/// 查询优化器
pub struct QueryOptimizer {
    /// 索引统计信息
    stats: Arc<RwLock<IndexStatistics>>,
    /// 配置
    config: QueryOptimizerConfig,
}

impl QueryOptimizer {
    /// 创建新的查询优化器
    pub fn new(stats: Arc<RwLock<IndexStatistics>>, config: QueryOptimizerConfig) -> Self {
        Self { stats, config }
    }

    /// 使用默认配置创建
    pub fn with_default_config(stats: Arc<RwLock<IndexStatistics>>) -> Self {
        Self::new(stats, QueryOptimizerConfig::default())
    }

    /// 优化查询，生成最优搜索计划
    pub fn optimize_query(&self, query: &SearchQuery) -> Result<OptimizedSearchPlan> {
        let stats = self.stats.read().unwrap();

        // 根据数据规模和查询要求选择策略
        let strategy = self.select_strategy(&stats, query);

        // 判断是否需要重排序
        let should_rerank = stats.total_vectors >= self.config.rerank_threshold;

        // 估算性能指标
        let estimated_latency_ms = self.estimate_latency(&stats, &strategy, should_rerank);
        let estimated_recall = self.estimate_recall(&strategy);

        Ok(OptimizedSearchPlan {
            strategy,
            should_rerank,
            rerank_factor: self.config.default_rerank_factor,
            estimated_latency_ms,
            estimated_recall,
        })
    }

    /// 选择搜索策略
    fn select_strategy(&self, stats: &IndexStatistics, query: &SearchQuery) -> SearchStrategy {
        // 小数据集：使用精确搜索
        if stats.total_vectors < self.config.small_dataset_threshold {
            return SearchStrategy::Exact;
        }

        // 根据索引类型和查询要求选择
        match stats.index_type {
            IndexType::None | IndexType::Flat => SearchStrategy::Exact,

            IndexType::HNSW => {
                let ef_search = if query.threshold.is_some() || query.filters.is_some() {
                    // 有过滤条件，使用更高的ef_search
                    self.config.high_precision_ef_search
                } else {
                    self.config.default_ef_search
                };

                SearchStrategy::HNSW { ef_search }
            }

            IndexType::IVF => SearchStrategy::IVF {
                nprobe: self.config.default_nprobe,
            },

            IndexType::IVF_HNSW => SearchStrategy::Hybrid {
                nprobe: self.config.default_nprobe,
                ef_search: self.config.default_ef_search,
            },
        }
    }

    /// 估算查询延迟（毫秒）
    fn estimate_latency(
        &self,
        stats: &IndexStatistics,
        strategy: &SearchStrategy,
        rerank: bool,
    ) -> u64 {
        let base_latency = match strategy {
            SearchStrategy::Exact => {
                // 线性扫描：O(n)
                (stats.total_vectors as f64 * 0.0001) as u64 // 每个向量0.1μs
            }
            SearchStrategy::HNSW { .. } => {
                // HNSW：O(log n)
                ((stats.total_vectors as f64).ln() * 2.0) as u64
            }
            SearchStrategy::IVF { nprobe } => {
                // IVF：O(nprobe * cluster_size)
                let cluster_size = stats.total_vectors / 100; // 假设100个聚类
                (nprobe * cluster_size) as u64 / 10000
            }
            SearchStrategy::Hybrid { .. } => {
                // 混合：最快
                ((stats.total_vectors as f64).ln() * 1.5) as u64
            }
        };

        // 重排序额外开销
        let rerank_overhead = if rerank { 5 } else { 0 };

        base_latency + rerank_overhead
    }

    /// 估算召回率
    fn estimate_recall(&self, strategy: &SearchStrategy) -> f32 {
        match strategy {
            SearchStrategy::Exact => 1.0, // 100%召回
            SearchStrategy::HNSW { ef_search } => {
                // ef_search越大，召回率越高
                if *ef_search >= self.config.high_precision_ef_search {
                    0.98 // 98%
                } else {
                    0.95 // 95%
                }
            }
            SearchStrategy::IVF { nprobe } => {
                // nprobe越大，召回率越高
                if *nprobe >= self.config.default_nprobe {
                    0.93 // 93%
                } else {
                    0.90 // 90%
                }
            }
            SearchStrategy::Hybrid { .. } => 0.95, // 95%
        }
    }

    /// 更新统计信息
    pub fn update_statistics(&self, total_vectors: usize) {
        let mut stats = self.stats.write().unwrap();
        stats.update(total_vectors);
    }

    /// 获取当前统计信息
    pub fn get_statistics(&self) -> IndexStatistics {
        self.stats.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_statistics_creation() {
        let stats = IndexStatistics::new(5_000, 1536);
        assert_eq!(stats.total_vectors, 5_000);
        assert_eq!(stats.dimension, 1536);
        assert_eq!(stats.index_type, IndexType::Flat);
    }

    #[test]
    fn test_index_type_selection() {
        let mut stats = IndexStatistics::new(5_000, 1536);
        assert_eq!(stats.index_type, IndexType::Flat);

        stats.update(50_000);
        assert_eq!(stats.index_type, IndexType::HNSW);

        stats.update(150_000);
        assert_eq!(stats.index_type, IndexType::IVF_HNSW);
    }

    #[test]
    fn test_query_optimizer_small_dataset() {
        let stats = Arc::new(RwLock::new(IndexStatistics::new(5_000, 1536)));
        let optimizer = QueryOptimizer::with_default_config(stats);

        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            ..Default::default()
        };

        let plan = optimizer.optimize_query(&query).unwrap();
        assert!(matches!(plan.strategy, SearchStrategy::Exact));
        assert!(!plan.should_rerank); // 小数据集不需要重排序
        assert_eq!(plan.estimated_recall, 1.0); // 精确搜索100%召回
    }

    #[test]
    fn test_query_optimizer_large_dataset() {
        let stats = Arc::new(RwLock::new(IndexStatistics::new(50_000, 1536)));
        let optimizer = QueryOptimizer::with_default_config(stats);

        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            ..Default::default()
        };

        let plan = optimizer.optimize_query(&query).unwrap();
        assert!(matches!(plan.strategy, SearchStrategy::HNSW { .. }));
        assert!(plan.should_rerank); // 大数据集需要重排序
        assert!(plan.estimated_recall >= 0.95); // 高召回率
    }

    #[test]
    fn test_latency_estimation() {
        let stats = Arc::new(RwLock::new(IndexStatistics::new(100_000, 1536)));
        let optimizer = QueryOptimizer::with_default_config(stats);

        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            ..Default::default()
        };

        let plan = optimizer.optimize_query(&query).unwrap();
        assert!(plan.estimated_latency_ms < 50); // 应该很快
    }
}

/// 结果重排序器
///
/// 用于在初始检索后对结果进行精确重排序，提高最终结果的质量
#[derive(Debug, Clone)]
pub struct ResultReranker {
    config: RerankerConfig,
}

/// 重排序器配置
#[derive(Debug, Clone)]
pub struct RerankerConfig {
    /// 时间衰减因子（0.0-1.0，越大时间影响越大）
    pub time_decay_factor: f32,
    /// 重要性权重（0.0-1.0）
    pub importance_weight: f32,
    /// 内容长度惩罚因子
    pub length_penalty_factor: f32,
}

impl Default for RerankerConfig {
    fn default() -> Self {
        Self {
            time_decay_factor: 0.1,
            importance_weight: 0.2,
            length_penalty_factor: 0.05,
        }
    }
}

impl ResultReranker {
    /// 创建新的重排序器
    pub fn new(config: RerankerConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(RerankerConfig::default())
    }
}

#[cfg(test)]
mod reranker_tests {
    use super::*;

    #[test]
    fn test_result_reranker_creation() {
        let reranker = ResultReranker::with_default_config();
        assert_eq!(reranker.config.time_decay_factor, 0.1);
        assert_eq!(reranker.config.importance_weight, 0.2);
    }
}
