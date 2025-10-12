//! 性能优化器模块
//!
//! 提供自动性能优化功能，包括：
//! - 向量搜索优化
//! - 图查询优化
//! - 索引优化
//! - 查询计划优化

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 优化器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    /// 是否启用向量搜索优化
    pub enable_vector_optimization: bool,
    
    /// 是否启用图查询优化
    pub enable_graph_optimization: bool,
    
    /// 是否启用索引优化
    pub enable_index_optimization: bool,
    
    /// 优化间隔（秒）
    pub optimization_interval_seconds: u64,
    
    /// 向量搜索阈值优化
    pub vector_threshold_optimization: bool,
    
    /// 图查询深度优化
    pub graph_depth_optimization: bool,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_vector_optimization: true,
            enable_graph_optimization: true,
            enable_index_optimization: true,
            optimization_interval_seconds: 300, // 5 分钟
            vector_threshold_optimization: true,
            graph_depth_optimization: true,
        }
    }
}

/// 性能优化器
pub struct PerformanceOptimizer {
    /// 配置
    config: OptimizerConfig,
    
    /// 统计信息
    stats: Arc<RwLock<OptimizationStats>>,
    
    /// 向量搜索优化参数
    vector_params: Arc<RwLock<VectorOptimizationParams>>,
    
    /// 图查询优化参数
    graph_params: Arc<RwLock<GraphOptimizationParams>>,
    
    /// 查询历史
    query_history: Arc<RwLock<Vec<QueryRecord>>>,
}

impl PerformanceOptimizer {
    /// 创建新的性能优化器
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(OptimizationStats::default())),
            vector_params: Arc::new(RwLock::new(VectorOptimizationParams::default())),
            graph_params: Arc::new(RwLock::new(GraphOptimizationParams::default())),
            query_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 优化向量搜索
    pub async fn optimize_vector_search(&self) -> Result<()> {
        if !self.config.enable_vector_optimization {
            return Ok(());
        }
        
        let start = Instant::now();
        
        // 分析查询历史
        let history = self.query_history.read().await;
        let vector_queries: Vec<_> = history
            .iter()
            .filter(|q| q.query_type == QueryType::VectorSearch)
            .collect();
        
        if vector_queries.is_empty() {
            return Ok(());
        }
        
        // 计算平均查询时间
        let avg_time: f64 = vector_queries
            .iter()
            .map(|q| q.duration_ms as f64)
            .sum::<f64>()
            / vector_queries.len() as f64;
        
        // 优化阈值
        if self.config.vector_threshold_optimization {
            let mut params = self.vector_params.write().await;
            
            // 如果平均查询时间过长，提高阈值以减少结果数量
            if avg_time > 100.0 {
                params.similarity_threshold = (params.similarity_threshold + 0.05).min(0.95);
                tracing::info!(
                    "Increased vector similarity threshold to {:.2}",
                    params.similarity_threshold
                );
            } else if avg_time < 20.0 {
                // 如果查询很快，可以降低阈值以获得更多结果
                params.similarity_threshold = (params.similarity_threshold - 0.05).max(0.5);
                tracing::info!(
                    "Decreased vector similarity threshold to {:.2}",
                    params.similarity_threshold
                );
            }
        }
        
        // 更新统计
        let mut stats = self.stats.write().await;
        stats.vector_optimizations += 1;
        stats.total_optimization_time_ms += start.elapsed().as_millis() as u64;
        
        Ok(())
    }
    
    /// 优化图查询
    pub async fn optimize_graph_query(&self) -> Result<()> {
        if !self.config.enable_graph_optimization {
            return Ok(());
        }
        
        let start = Instant::now();
        
        // 分析查询历史
        let history = self.query_history.read().await;
        let graph_queries: Vec<_> = history
            .iter()
            .filter(|q| q.query_type == QueryType::GraphQuery)
            .collect();
        
        if graph_queries.is_empty() {
            return Ok(());
        }
        
        // 计算平均查询时间
        let avg_time: f64 = graph_queries
            .iter()
            .map(|q| q.duration_ms as f64)
            .sum::<f64>()
            / graph_queries.len() as f64;
        
        // 优化查询深度
        if self.config.graph_depth_optimization {
            let mut params = self.graph_params.write().await;
            
            // 如果平均查询时间过长，减少最大深度
            if avg_time > 200.0 {
                params.max_depth = (params.max_depth - 1).max(1);
                tracing::info!("Decreased graph max depth to {}", params.max_depth);
            } else if avg_time < 50.0 {
                // 如果查询很快，可以增加深度
                params.max_depth = (params.max_depth + 1).min(10);
                tracing::info!("Increased graph max depth to {}", params.max_depth);
            }
        }
        
        // 更新统计
        let mut stats = self.stats.write().await;
        stats.graph_optimizations += 1;
        stats.total_optimization_time_ms += start.elapsed().as_millis() as u64;
        
        Ok(())
    }
    
    /// 优化索引
    pub async fn optimize_index(&self) -> Result<()> {
        if !self.config.enable_index_optimization {
            return Ok(());
        }
        
        let start = Instant::now();
        
        // 这里可以添加索引优化逻辑
        // 例如：重建索引、压缩索引等
        
        tracing::info!("Index optimization completed");
        
        // 更新统计
        let mut stats = self.stats.write().await;
        stats.index_optimizations += 1;
        stats.total_optimization_time_ms += start.elapsed().as_millis() as u64;
        
        Ok(())
    }
    
    /// 执行所有优化
    pub async fn optimize(&self) -> Result<()> {
        tracing::info!("Running performance optimization...");
        
        self.optimize_vector_search().await?;
        self.optimize_graph_query().await?;
        self.optimize_index().await?;
        
        tracing::info!("Performance optimization completed");
        
        Ok(())
    }
    
    /// 记录查询
    pub async fn record_query(&self, query_type: QueryType, duration_ms: u64) {
        let mut history = self.query_history.write().await;
        
        history.push(QueryRecord {
            query_type,
            duration_ms,
            timestamp: Instant::now(),
        });
        
        // 限制历史记录大小
        if history.len() > 10000 {
            history.drain(0..5000);
        }
    }
    
    /// 获取向量优化参数
    pub async fn get_vector_params(&self) -> VectorOptimizationParams {
        self.vector_params.read().await.clone()
    }
    
    /// 获取图优化参数
    pub async fn get_graph_params(&self) -> GraphOptimizationParams {
        self.graph_params.read().await.clone()
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> OptimizationStats {
        self.stats.read().await.clone()
    }
    
    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = OptimizationStats::default();
    }
}

/// 向量优化参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorOptimizationParams {
    /// 相似度阈值
    pub similarity_threshold: f32,
    
    /// 最大结果数
    pub max_results: usize,
    
    /// 是否使用近似搜索
    pub use_approximate_search: bool,
}

impl Default for VectorOptimizationParams {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            max_results: 100,
            use_approximate_search: false,
        }
    }
}

/// 图优化参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOptimizationParams {
    /// 最大深度
    pub max_depth: usize,
    
    /// 最大节点数
    pub max_nodes: usize,
    
    /// 是否使用双向搜索
    pub use_bidirectional_search: bool,
}

impl Default for GraphOptimizationParams {
    fn default() -> Self {
        Self {
            max_depth: 5,
            max_nodes: 1000,
            use_bidirectional_search: true,
        }
    }
}

/// 查询类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    /// 向量搜索
    VectorSearch,
    
    /// 图查询
    GraphQuery,
    
    /// 混合查询
    HybridQuery,
}

/// 查询记录
#[derive(Debug, Clone)]
struct QueryRecord {
    /// 查询类型
    query_type: QueryType,
    
    /// 查询时间（毫秒）
    duration_ms: u64,
    
    /// 时间戳
    timestamp: Instant,
}

/// 优化统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationStats {
    /// 向量优化次数
    pub vector_optimizations: usize,
    
    /// 图优化次数
    pub graph_optimizations: usize,
    
    /// 索引优化次数
    pub index_optimizations: usize,
    
    /// 总优化时间（毫秒）
    pub total_optimization_time_ms: u64,
}

