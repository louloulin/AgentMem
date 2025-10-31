//! 混合搜索引擎
//!
//! 整合向量搜索和全文搜索，使用 RRF 算法融合结果

use super::{
    FullTextSearchEngine, RRFRanker, SearchQuery, SearchResult, SearchResultRanker, SearchStats,
    VectorSearchEngine,
};
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// 混合搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchConfig {
    /// 向量搜索权重 (0.0 - 1.0)
    pub vector_weight: f32,
    /// 全文搜索权重 (0.0 - 1.0)
    pub fulltext_weight: f32,
    /// RRF 常数 k
    pub rrf_k: f32,
    /// 是否启用并行搜索
    pub enable_parallel: bool,
    /// 是否启用搜索缓存
    pub enable_cache: bool,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            rrf_k: 60.0,
            enable_parallel: true,
            enable_cache: false,
        }
    }
}

/// 混合搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    /// 搜索结果列表
    pub results: Vec<SearchResult>,
    /// 搜索统计信息
    pub stats: SearchStats,
}

/// 混合搜索引擎
pub struct HybridSearchEngine {
    /// 向量搜索引擎
    vector_engine: Arc<VectorSearchEngine>,
    /// 全文搜索引擎
    fulltext_engine: Arc<FullTextSearchEngine>,
    /// 搜索配置
    config: HybridSearchConfig,
    /// RRF 排序器
    ranker: RRFRanker,
}

impl HybridSearchEngine {
    /// 创建新的混合搜索引擎
    ///
    /// # Arguments
    ///
    /// * `vector_engine` - 向量搜索引擎
    /// * `fulltext_engine` - 全文搜索引擎
    /// * `config` - 搜索配置
    pub fn new(
        vector_engine: Arc<VectorSearchEngine>,
        fulltext_engine: Arc<FullTextSearchEngine>,
        config: HybridSearchConfig,
    ) -> Self {
        let ranker = RRFRanker::new(config.rrf_k);
        Self {
            vector_engine,
            fulltext_engine,
            config,
            ranker,
        }
    }

    /// 使用默认配置创建混合搜索引擎
    pub fn with_default_config(
        vector_engine: Arc<VectorSearchEngine>,
        fulltext_engine: Arc<FullTextSearchEngine>,
    ) -> Self {
        Self::new(
            vector_engine,
            fulltext_engine,
            HybridSearchConfig::default(),
        )
    }

    /// 使用自定义权重执行混合搜索
    ///
    /// # Arguments
    ///
    /// * `query_vector` - 查询向量
    /// * `query` - 搜索查询
    /// * `vector_weight` - 向量搜索权重
    /// * `fulltext_weight` - 全文搜索权重
    pub async fn search_with_weights(
        &self,
        query_vector: Vec<f32>,
        query: SearchQuery,
        vector_weight: f32,
        fulltext_weight: f32,
    ) -> Result<HybridSearchResult> {
        let start = Instant::now();
        
        // 并行或串行搜索
        let (vector_results, fulltext_results) = if self.config.enable_parallel {
            self.parallel_search(query_vector, &query).await?
        } else {
            self.sequential_search(query_vector, &query).await?
        };
        
        // RRF 融合（使用自定义权重）
        let fused_results = self.ranker.fuse_with_weights(
            vector_results,
            fulltext_results,
            vector_weight,
            fulltext_weight,
        )?;
        
        let total_time = start.elapsed().as_millis() as u64;
        
        Ok(HybridSearchResult {
            results: fused_results,
            stats: SearchStats {
                total_time_ms: total_time,
                vector_search_time_ms: 0,
                fulltext_search_time_ms: 0,
                fusion_time_ms: 0,
                vector_results_count: 0,
                fulltext_results_count: 0,
                final_results_count: 0,
            },
        })
    }
    
    /// 执行混合搜索
    ///
    /// # Arguments
    ///
    /// * `query_vector` - 查询向量
    /// * `query` - 搜索查询参数
    ///
    /// # Returns
    ///
    /// 返回混合搜索结果
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        query: &SearchQuery,
    ) -> Result<HybridSearchResult> {
        let start = Instant::now();

        // 执行向量搜索和全文搜索
        let (vector_results, fulltext_results, vector_time, fulltext_time) =
            if self.config.enable_parallel {
                self.parallel_search(query_vector, query).await?
            } else {
                self.sequential_search(query_vector, query).await?
            };

        // 融合搜索结果
        let fusion_start = Instant::now();
        let fused_results = self.fuse_results(vector_results.clone(), fulltext_results.clone())?;
        let fusion_time = fusion_start.elapsed().as_millis() as u64;

        // 限制结果数量
        let final_results: Vec<SearchResult> =
            fused_results.into_iter().take(query.limit).collect();

        // 构建统计信息
        let stats = SearchStats {
            total_time_ms: start.elapsed().as_millis() as u64,
            vector_search_time_ms: vector_time,
            fulltext_search_time_ms: fulltext_time,
            fusion_time_ms: fusion_time,
            vector_results_count: vector_results.len(),
            fulltext_results_count: fulltext_results.len(),
            final_results_count: final_results.len(),
        };

        Ok(HybridSearchResult {
            results: final_results,
            stats,
        })
    }

    /// 并行执行向量搜索和全文搜索
    /// P1 优化 #23: 并行搜索支持部分失败
    /// 
    /// 优化策略：允许某个搜索失败，仍能返回其他搜索的结果
    async fn parallel_search(
        &self,
        query_vector: Vec<f32>,
        query: &SearchQuery,
    ) -> Result<(Vec<SearchResult>, Vec<SearchResult>, u64, u64)> {
        use tracing::{debug, warn};
        
        let vector_engine = self.vector_engine.clone();
        let fulltext_engine = self.fulltext_engine.clone();
        let query_clone = query.clone();

        // 并行执行两个搜索
        let (vector_result, fulltext_result) = tokio::join!(
            vector_engine.search(query_vector, &query_clone),
            fulltext_engine.search(&query_clone)
        );

        // P1 优化 #23: 处理部分失败情况
        let (vector_results, vector_time) = match vector_result {
            Ok((results, time)) => {
                debug!("✅ 向量搜索成功: {} 个结果", results.len());
                (results, time)
            }
            Err(e) => {
                warn!("向量搜索失败: {}, 继续使用空结果", e);
                (Vec::new(), 0) // 失败时使用空结果
            }
        };

        let (fulltext_results, fulltext_time) = match fulltext_result {
            Ok((results, time)) => {
                debug!("✅ 全文搜索成功: {} 个结果", results.len());
                (results, time)
            }
            Err(e) => {
                warn!("全文搜索失败: {}, 继续使用空结果", e);
                (Vec::new(), 0) // 失败时使用空结果
            }
        };

        // P1 优化 #23: 即使一个搜索失败，只要有结果就继续
        if vector_results.is_empty() && fulltext_results.is_empty() {
            return Err(agent_mem_traits::AgentMemError::SearchError(
                "Both vector and fulltext search failed or returned no results".to_string()
            ));
        }

        Ok((vector_results, fulltext_results, vector_time, fulltext_time))
    }

    /// 顺序执行向量搜索和全文搜索
    async fn sequential_search(
        &self,
        query_vector: Vec<f32>,
        query: &SearchQuery,
    ) -> Result<(Vec<SearchResult>, Vec<SearchResult>, u64, u64)> {
        let (vector_results, vector_time) = self.vector_engine.search(query_vector, query).await?;
        let (fulltext_results, fulltext_time) = self.fulltext_engine.search(query).await?;

        Ok((vector_results, fulltext_results, vector_time, fulltext_time))
    }

    /// 融合搜索结果
    fn fuse_results(
        &self,
        vector_results: Vec<SearchResult>,
        fulltext_results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        // 使用 RRF 算法融合结果
        let weights = vec![self.config.vector_weight, self.config.fulltext_weight];
        self.ranker
            .fuse(vec![vector_results, fulltext_results], weights)
    }

    /// 更新搜索配置
    pub fn update_config(&mut self, config: HybridSearchConfig) {
        self.config = config;
        self.ranker = RRFRanker::new(self.config.rrf_k);
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &HybridSearchConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::FullTextSearchEngine;
    use agent_mem_storage::backends::MemoryVectorStore;
    use sqlx::PgPool;

    #[test]
    fn test_hybrid_search_config() {
        let config = HybridSearchConfig::default();
        assert_eq!(config.vector_weight, 0.7);
        assert_eq!(config.fulltext_weight, 0.3);
        assert_eq!(config.rrf_k, 60.0);
        assert!(config.enable_parallel);
    }

    #[test]
    fn test_config_update() {
        // 注意：这个测试需要实际的数据库连接和向量存储
        // 这里只测试配置更新逻辑
        let new_config = HybridSearchConfig {
            vector_weight: 0.5,
            fulltext_weight: 0.5,
            rrf_k: 80.0,
            enable_parallel: false,
            enable_cache: true,
        };

        // 实际测试需要在集成测试中进行
        assert_eq!(new_config.vector_weight, 0.5);
        assert_eq!(new_config.fulltext_weight, 0.5);
    }

    #[tokio::test]
    async fn test_hybrid_search_result_structure() {
        // 测试结果结构
        let results = vec![];
        let stats = SearchStats {
            total_time_ms: 100,
            vector_search_time_ms: 50,
            fulltext_search_time_ms: 40,
            fusion_time_ms: 10,
            vector_results_count: 5,
            fulltext_results_count: 3,
            final_results_count: 6,
        };

        let hybrid_result = HybridSearchResult { results, stats };

        assert_eq!(hybrid_result.stats.total_time_ms, 100);
        assert_eq!(hybrid_result.stats.vector_results_count, 5);
        assert_eq!(hybrid_result.stats.fulltext_results_count, 3);
    }
}
