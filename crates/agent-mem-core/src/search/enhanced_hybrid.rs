//! 增强的混合搜索引擎
//!
//! 在原有混合搜索基础上增加自适应权重调整和结果重排序

use super::adaptive::{AdaptiveSearchOptimizer, SearchReranker, SearchWeights};
use super::hybrid::HybridSearchEngine;
use super::{SearchQuery, SearchResult};
use agent_mem_traits::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 增强混合搜索引擎
pub struct EnhancedHybridSearchEngine {
    /// 原有的混合搜索引擎
    base_engine: Arc<HybridSearchEngine>,
    
    /// 自适应优化器
    optimizer: Arc<RwLock<AdaptiveSearchOptimizer>>,
    
    /// 重排序器
    reranker: Arc<SearchReranker>,
    
    /// 是否启用自适应权重
    enable_adaptive_weights: bool,
    
    /// 是否启用重排序
    enable_reranking: bool,
}

impl EnhancedHybridSearchEngine {
    /// 创建新的增强混合搜索引擎
    pub fn new(
        base_engine: Arc<HybridSearchEngine>,
        enable_adaptive_weights: bool,
        enable_reranking: bool,
    ) -> Self {
        Self {
            base_engine,
            optimizer: Arc::new(RwLock::new(AdaptiveSearchOptimizer::default())),
            reranker: Arc::new(SearchReranker::default()),
            enable_adaptive_weights,
            enable_reranking,
        }
    }
    
    /// 执行增强搜索
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        // 步骤1: 自适应权重调整（如果启用）
        let (optimized_query, predicted_weights) = if self.enable_adaptive_weights {
            let optimizer = self.optimizer.read().await;
            optimizer.optimize_query(&query)
        } else {
            (query.clone(), SearchWeights {
                vector_weight: 0.7,
                fulltext_weight: 0.3,
                confidence: 1.0,
            })
        };
        
        // 步骤2: 执行基础混合搜索（使用预测的权重）
        let result = self.base_engine.search_with_weights(
            query_vector,
            optimized_query.clone(),
            predicted_weights.vector_weight,
            predicted_weights.fulltext_weight,
        ).await?;
        
        let mut results = result.results;
        
        // 步骤3: 结果重排序（如果启用）
        if self.enable_reranking {
            results = self.reranker.rerank(results, &optimized_query);
        }
        
        Ok(results)
    }
    
    /// 记录搜索反馈（用于持续优化）
    pub async fn record_feedback(
        &self,
        query: &str,
        weights: SearchWeights,
        user_satisfaction: f32,
    ) {
        let mut optimizer = self.optimizer.write().await;
        optimizer.record_feedback(query, weights, user_satisfaction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 注意：这些测试需要mock HybridSearchEngine
    // 实际测试将在集成测试中进行
}

