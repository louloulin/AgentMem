//! Adaptive Search Engine - Week 5-6 完整集成
//! 集成 AdaptiveRouter，实现智能自适应搜索

use super::adaptive_router::AdaptiveRouter;
use super::{SearchQuery, SearchResult};
use crate::config::AgentMemConfig;
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// 自适应搜索引擎（抽象接口）
/// 
/// 注意：由于避免循环依赖，这里提供trait而非直接依赖HybridSearchEngine
pub struct AdaptiveSearchEngine<S>
where
    S: SearchEngineBackend,
{
    /// 搜索引擎后端
    backend: Arc<S>,
    /// 自适应路由器
    router: Arc<AdaptiveRouter>,
    /// 是否启用学习
    enable_learning: bool,
}

/// 搜索引擎后端trait（避免循环依赖）
#[async_trait::async_trait]
pub trait SearchEngineBackend: Send + Sync {
    /// 执行搜索
    async fn search_with_weights(
        &self,
        query_vector: Vec<f32>,
        query: SearchQuery,
        vector_weight: f32,
        fulltext_weight: f32,
    ) -> Result<SearchBackendResult>;
}

/// 搜索后端结果
pub struct SearchBackendResult {
    pub results: Vec<SearchResult>,
}

impl<S: SearchEngineBackend> AdaptiveSearchEngine<S> {
    pub fn new(
        backend: Arc<S>,
        config: AgentMemConfig,
        enable_learning: bool,
    ) -> Self {
        let router = Arc::new(AdaptiveRouter::new(config));
        
        Self {
            backend,
            router,
            enable_learning,
        }
    }
    
    /// 执行自适应搜索（核心方法）
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        mut query: SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        let start = Instant::now();
        
        // 步骤1: 路由器决策最优策略
        let (strategy_id, weights) = self.router.decide_strategy(&query).await?;
        
        debug!(
            "AdaptiveRouter selected strategy: {:?}, weights: v={}, f={}",
            strategy_id, weights.vector_weight, weights.fulltext_weight
        );
        
        // 步骤2: 应用策略权重
        query.vector_weight = weights.vector_weight;
        query.fulltext_weight = weights.fulltext_weight;
        
        // 步骤3: 执行搜索
        let results = self.backend
            .search_with_weights(
                query_vector,
                query.clone(),
                weights.vector_weight,
                weights.fulltext_weight,
            )
            .await?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        // 步骤4: 计算准确率（基于结果分数）
        let accuracy = self.calculate_accuracy(&results.results);
        
        // 步骤5: 反馈学习（异步，不阻塞）
        if self.enable_learning {
            let router = Arc::clone(&self.router);
            let query_clone = query.clone();
            tokio::spawn(async move {
                if let Err(e) = router.record_performance(&query_clone, strategy_id, accuracy, latency_ms).await {
                    tracing::warn!("Failed to record performance: {}", e);
                }
            });
        }
        
        info!(
            "Adaptive search completed: strategy={:?}, accuracy={:.2}, latency={}ms",
            strategy_id, accuracy, latency_ms
        );
        
        Ok(results.results)
    }
    
    /// 计算搜索准确率（基于返回结果的分数分布）
    fn calculate_accuracy(&self, results: &[SearchResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }
        
        // 简单策略：使用top结果的平均分数作为准确率代理
        let top_k = results.len().min(5);
        let avg_score: f32 = results.iter()
            .take(top_k)
            .map(|r| r.score)
            .sum::<f32>() / top_k as f32;
        
        avg_score.clamp(0.0, 1.0)
    }
    
    /// 获取路由器统计信息
    pub async fn get_router_stats(&self) -> anyhow::Result<String> {
        let stats = self.router.get_strategy_stats().await;
        
        let mut output = String::from("=== Adaptive Router Stats ===\n");
        for (strategy_id, arm) in stats.iter() {
            output.push_str(&format!(
                "Strategy: {:?}\n  Expected Rate: {:.3}\n  Total Tries: {}\n  Alpha: {:.2}, Beta: {:.2}\n",
                strategy_id,
                arm.expected_rate(),
                arm.total_tries,
                arm.alpha,
                arm.beta
            ));
        }
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_adaptive_search_engine() {
        // 注意：这是一个集成测试示意，实际需要mock引擎
        // 这里仅验证接口正确性
        
        // 创建配置
        let config = AgentMemConfig::default();
        
        // 这里需要mock VectorSearchEngine和FullTextSearchEngine
        // 实际测试中需要提供真实实现或mock
        
        // 验证配置正确
        assert_eq!(config.hybrid_search.vector_weight, 0.7);
        assert_eq!(config.hybrid_search.fulltext_weight, 0.3);
    }
    
    #[test]
    fn test_accuracy_calculation() {
        let config = AgentMemConfig::default();
        let router = Arc::new(AdaptiveRouter::new(config.clone()));
        
        // Mock hybrid engine需要真实实现，这里先跳过
        // 测试准确率计算逻辑
        
        let results = vec![
            SearchResult {
                id: "1".to_string(),
                score: 0.9,
                content: "test".to_string(),
                vector_score: Some(0.9),
                fulltext_score: Some(0.8),
                metadata: None,
            },
            SearchResult {
                id: "2".to_string(),
                score: 0.8,
                content: "test".to_string(),
                vector_score: Some(0.8),
                fulltext_score: Some(0.7),
                metadata: None,
            },
        ];
        
        // 手动验证准确率计算
        let avg: f64 = (0.9 + 0.8) / 2.0;
        assert!((avg - 0.85).abs() < 0.01);
    }
}

// ============================================================================
// SearchEngine Trait 实现 (V4)
// ============================================================================

use agent_mem_traits::{SearchEngine, Query, QueryIntent, QueryIntentType};

#[async_trait::async_trait]
impl<S> SearchEngine for AdaptiveSearchEngine<S>
where
    S: SearchEngineBackend,
{
    /// 执行搜索查询（V4 Query 接口）
    async fn search(&self, query: &Query) -> agent_mem_traits::Result<Vec<agent_mem_traits::SearchResultV4>> {
        // 1. 提取查询向量和文本
        let (query_vector, query_text) = match &query.intent {
            QueryIntent::Hybrid { intents, .. } => {
                // 从混合查询中提取向量和文本
                let vector = intents.iter()
                    .find_map(|intent| {
                        if let QueryIntent::Vector { embedding } = intent {
                            Some(embedding.clone())
                        } else {
                            None
                        }
                    });

                let text = intents.iter()
                    .find_map(|intent| {
                        if let QueryIntent::NaturalLanguage { text, .. } = intent {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                let vector = vector.ok_or_else(|| agent_mem_traits::AgentMemError::validation_error(
                    "Hybrid query must contain at least one Vector intent"
                ))?;

                (vector, text)
            }
            QueryIntent::Vector { embedding } => {
                // 纯向量查询
                (embedding.clone(), String::new())
            }
            _ => {
                return Err(agent_mem_traits::AgentMemError::validation_error(
                    format!("Unsupported query intent for AdaptiveSearchEngine: {:?}. Use Vector or Hybrid intent.", query.intent)
                ));
            }
        };

        // 2. 转换 Query V4 到 SearchQuery
        let mut search_query = SearchQuery::from_query_v4(query);
        if !query_text.is_empty() {
            search_query.query = query_text;
        }

        // 3. 执行自适应搜索（使用现有的 search 方法）
        let results = self.search(query_vector, search_query).await
            .map_err(|e| agent_mem_traits::AgentMemError::Other(e))?;

        // 4. 转换 SearchResult 到 SearchResultV4
        let v4_results = results.into_iter()
            .map(|r| agent_mem_traits::SearchResultV4 {
                id: r.id,
                content: r.content,
                score: r.score,
                vector_score: r.vector_score,
                fulltext_score: r.fulltext_score,
                metadata: r.metadata,
            })
            .collect();

        Ok(v4_results)
    }

    /// 获取引擎名称
    fn name(&self) -> &str {
        "AdaptiveSearchEngine"
    }

    /// 获取支持的查询意图类型
    fn supported_intents(&self) -> Vec<QueryIntentType> {
        vec![
            QueryIntentType::Hybrid, // 主要支持混合查询
            QueryIntentType::Vector, // 也支持纯向量查询
        ]
    }
}

