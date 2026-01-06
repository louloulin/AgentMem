//! 🆕 Phase 3: HNSW索引自动调优
//!
//! 根据数据规模自动优化HNSW索引参数，充分复用现有配置

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// HNSW索引参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswParams {
    /// M参数：每个节点的最大连接数（影响索引大小和构建时间）
    pub m: usize,
    /// ef_construction：构建时的候选列表大小（影响索引质量和构建时间）
    pub ef_construction: usize,
    /// ef_search：搜索时的候选列表大小（影响搜索质量和速度）
    pub ef_search: usize,
}

impl Default for HnswParams {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
        }
    }
}

/// 数据规模分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataScale {
    /// 小规模：< 10K向量
    Small,
    /// 中等规模：10K - 100K向量
    Medium,
    /// 大规模：100K - 1M向量
    Large,
    /// 超大规模：> 1M向量
    VeryLarge,
}

impl DataScale {
    /// 根据向量数量判断规模
    pub fn from_count(count: usize) -> Self {
        match count {
            n if n < 10_000 => DataScale::Small,
            n if n < 100_000 => DataScale::Medium,
            n if n < 1_000_000 => DataScale::Large,
            _ => DataScale::VeryLarge,
        }
    }
}

/// HNSW优化器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswOptimizerConfig {
    /// 启用自动调优
    pub enable_auto_tuning: bool,
    /// 小规模数据阈值
    pub small_scale_threshold: usize,
    /// 中等规模数据阈值
    pub medium_scale_threshold: usize,
    /// 大规模数据阈值
    pub large_scale_threshold: usize,
    /// 向量维度
    pub vector_dimension: usize,
}

impl Default for HnswOptimizerConfig {
    fn default() -> Self {
        Self {
            enable_auto_tuning: true,
            small_scale_threshold: 10_000,
            medium_scale_threshold: 100_000,
            large_scale_threshold: 1_000_000,
            vector_dimension: 1536, // OpenAI默认维度
        }
    }
}

/// HNSW索引优化器
/// 
/// 根据数据规模自动调优HNSW参数，复用现有配置逻辑
pub struct HnswOptimizer {
    config: HnswOptimizerConfig,
    current_params: Arc<RwLock<HnswParams>>,
    stats: Arc<RwLock<HnswStats>>,
}

/// HNSW统计信息
#[derive(Debug, Clone)]
pub struct HnswStats {
    /// 总向量数
    pub total_vectors: usize,
    /// 索引构建时间（毫秒）
    pub build_time_ms: u64,
    /// 平均搜索时间（毫秒）
    pub avg_search_time_ms: f64,
    /// 搜索召回率
    pub recall: f64,
    /// 最后更新时间
    pub last_updated: std::time::Instant,
}

impl Default for HnswStats {
    fn default() -> Self {
        Self {
            total_vectors: 0,
            build_time_ms: 0,
            avg_search_time_ms: 0.0,
            recall: 0.0,
            last_updated: std::time::Instant::now(),
        }
    }
}

impl HnswOptimizer {
    /// 创建新的HNSW优化器
    pub fn new(config: HnswOptimizerConfig) -> Self {
        info!("Creating HNSW optimizer with config: {:?}", config);
        Self {
            config,
            current_params: Arc::new(RwLock::new(HnswParams::default())),
            stats: Arc::new(RwLock::new(HnswStats::default())),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(HnswOptimizerConfig::default())
    }

    /// 根据数据规模自动调优参数
    /// 
    /// 复用现有配置逻辑，根据数据规模选择最优参数
    pub async fn auto_tune(&self, vector_count: usize, dimension: usize) -> HnswParams {
        if !self.config.enable_auto_tuning {
            debug!("Auto-tuning disabled, using default params");
            return HnswParams::default();
        }

        let scale = DataScale::from_count(vector_count);
        let params = self.calculate_optimal_params(scale, dimension);

        // 更新当前参数
        {
            let mut current = self.current_params.write().await;
            *current = params.clone();
        }

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_vectors = vector_count;
            stats.last_updated = std::time::Instant::now();
        }

        info!(
            "Auto-tuned HNSW params for {} vectors (scale: {:?}): m={}, ef_construction={}, ef_search={}",
            vector_count, scale, params.m, params.ef_construction, params.ef_search
        );

        params
    }

    /// 计算最优参数（复用现有配置逻辑）
    fn calculate_optimal_params(&self, scale: DataScale, dimension: usize) -> HnswParams {
        // 根据向量维度调整M参数（复用现有逻辑）
        let base_m = if dimension > 512 { 16 } else { 32 };

        match scale {
            DataScale::Small => {
                // 小规模：使用较小参数，快速构建
                HnswParams {
                    m: base_m.min(16),
                    ef_construction: base_m * 8,  // 较小值，快速构建
                    ef_search: 32,                // 较小值，快速搜索
                }
            }
            DataScale::Medium => {
                // 中等规模：平衡参数
                HnswParams {
                    m: base_m,
                    ef_construction: base_m * 10, // 默认值
                    ef_search: 50,                // 默认值
                }
            }
            DataScale::Large => {
                // 大规模：使用较大参数，保证质量
                HnswParams {
                    m: base_m.max(16),
                    ef_construction: base_m * 12, // 较大值，保证质量
                    ef_search: 100,               // 较大值，保证召回率
                }
            }
            DataScale::VeryLarge => {
                // 超大规模：使用最大参数，保证高召回率
                HnswParams {
                    m: base_m.max(16),
                    ef_construction: base_m * 15, // 最大值，保证质量
                    ef_search: 200,               // 最大值，保证高召回率
                }
            }
        }
    }

    /// 获取当前参数
    pub async fn get_current_params(&self) -> HnswParams {
        self.current_params.read().await.clone()
    }

    /// 更新统计信息
    pub async fn update_stats(&self, build_time_ms: u64, avg_search_time_ms: f64, recall: f64) {
        let mut stats = self.stats.write().await;
        stats.build_time_ms = build_time_ms;
        stats.avg_search_time_ms = avg_search_time_ms;
        stats.recall = recall;
        stats.last_updated = std::time::Instant::now();
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> HnswStats {
        self.stats.read().await.clone()
    }

    /// 根据性能反馈调整参数
    pub async fn adjust_from_feedback(&self, target_recall: f64, current_recall: f64) -> Option<HnswParams> {
        if !self.config.enable_auto_tuning {
            return None;
        }

        let recall_diff = target_recall - current_recall;
        
        // 如果召回率低于目标，增加ef_search
        if recall_diff > 0.05 {
            let mut params = self.current_params.read().await.clone();
            params.ef_search = (params.ef_search as f64 * 1.2) as usize;
            params.ef_search = params.ef_search.min(500); // 限制最大值
            
            debug!("Adjusting ef_search to {} due to low recall", params.ef_search);
            
            {
                let mut current = self.current_params.write().await;
                *current = params.clone();
            }
            
            return Some(params);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auto_tune_small_scale() {
        let optimizer = HnswOptimizer::with_default_config();
        let params = optimizer.auto_tune(5_000, 1536).await;

        // 小规模应该使用较小参数
        assert!(params.m <= 16);
        assert!(params.ef_construction <= 200);
        assert!(params.ef_search <= 50);
    }

    #[tokio::test]
    async fn test_auto_tune_medium_scale() {
        let optimizer = HnswOptimizer::with_default_config();
        let params = optimizer.auto_tune(50_000, 1536).await;

        // 中等规模使用平衡参数
        assert_eq!(params.m, 16);
        assert_eq!(params.ef_construction, 160);
        assert_eq!(params.ef_search, 50);
    }

    #[tokio::test]
    async fn test_auto_tune_large_scale() {
        let optimizer = HnswOptimizer::with_default_config();
        let params = optimizer.auto_tune(500_000, 1536).await;

        // 大规模使用较大参数
        assert_eq!(params.m, 16);
        assert_eq!(params.ef_construction, 192);
        assert!(params.ef_search >= 100);
    }

    #[tokio::test]
    async fn test_auto_tune_very_large_scale() {
        let optimizer = HnswOptimizer::with_default_config();
        let params = optimizer.auto_tune(2_000_000, 1536).await;

        // 超大规模使用最大参数
        assert_eq!(params.m, 16);
        assert_eq!(params.ef_construction, 240);
        assert_eq!(params.ef_search, 200);
    }

    #[tokio::test]
    async fn test_adjust_from_feedback() {
        let optimizer = HnswOptimizer::with_default_config();
        
        // 初始化参数
        optimizer.auto_tune(100_000, 1536).await;
        let initial_params = optimizer.get_current_params().await;
        
        // 召回率低于目标，应该调整
        let adjusted = optimizer.adjust_from_feedback(0.95, 0.80).await;
        assert!(adjusted.is_some());
        
        let new_params = adjusted.unwrap();
        assert!(new_params.ef_search > initial_params.ef_search);
    }

    #[tokio::test]
    async fn test_data_scale_classification() {
        assert_eq!(DataScale::from_count(5_000), DataScale::Small);
        assert_eq!(DataScale::from_count(50_000), DataScale::Medium);
        assert_eq!(DataScale::from_count(500_000), DataScale::Large);
        assert_eq!(DataScale::from_count(2_000_000), DataScale::VeryLarge);
    }
}
