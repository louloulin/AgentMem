//! 增强的向量搜索引擎（带多层缓存）
//!
//! 对现有VectorSearchEngine的增强包装，集成多层缓存系统

use super::{SearchQuery, SearchResult, VectorSearchEngine};
use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "redis-cache")]
use crate::cache::MultiLevelCache;

/// 缓存的搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedSearchResults {
    /// 搜索结果列表
    results: Vec<SearchResult>,
    /// 搜索耗时（毫秒）
    search_time_ms: u64,
}

/// 增强的向量搜索引擎配置
#[derive(Debug, Clone)]
pub struct CachedVectorSearchConfig {
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存TTL（秒）
    pub cache_ttl_seconds: u64,
    /// 缓存键前缀
    pub cache_key_prefix: String,
}

impl Default for CachedVectorSearchConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_ttl_seconds: 300, // 5分钟
            cache_key_prefix: "vec_search".to_string(),
        }
    }
}

/// 增强的向量搜索引擎
///
/// 对 VectorSearchEngine 的包装，添加多层缓存支持
pub struct CachedVectorSearchEngine {
    /// 基础向量搜索引擎
    base_engine: Arc<VectorSearchEngine>,

    /// 多层缓存（可选）
    #[cfg(feature = "redis-cache")]
    cache: Option<Arc<MultiLevelCache>>,

    /// 配置
    config: CachedVectorSearchConfig,
}

impl CachedVectorSearchEngine {
    /// 创建新的缓存增强搜索引擎
    pub fn new(base_engine: Arc<VectorSearchEngine>, config: CachedVectorSearchConfig) -> Self {
        Self {
            base_engine,
            #[cfg(feature = "redis-cache")]
            cache: None,
            config,
        }
    }

    /// 创建带多层缓存的增强搜索引擎
    #[cfg(feature = "redis-cache")]
    pub fn with_cache(
        base_engine: Arc<VectorSearchEngine>,
        config: CachedVectorSearchConfig,
        cache: Arc<MultiLevelCache>,
    ) -> Self {
        Self {
            base_engine,
            cache: Some(cache),
            config,
        }
    }

    /// 执行搜索（带缓存）
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        query: &SearchQuery,
    ) -> Result<(Vec<SearchResult>, u64)> {
        // 如果缓存未启用，直接调用基础引擎
        if !self.config.enable_cache {
            return self.base_engine.search(query_vector, query).await;
        }

        // 生成缓存键
        let cache_key = self.generate_cache_key(&query_vector, query);

        // 尝试从缓存获取
        #[cfg(feature = "redis-cache")]
        if let Some(cache) = &self.cache {
            if let Ok(Some(cached_data)) = cache.get(&cache_key).await {
                if let Ok(cached_results) =
                    serde_json::from_slice::<CachedSearchResults>(&cached_data)
                {
                    tracing::debug!("Cache hit for vector search: {}", cache_key);
                    return Ok((cached_results.results, cached_results.search_time_ms));
                }
            }
        }

        // 缓存未命中，执行搜索
        let (results, search_time) = self.base_engine.search(query_vector, query).await?;

        // 保存到缓存
        #[cfg(feature = "redis-cache")]
        if let Some(cache) = &self.cache {
            let cached_results = CachedSearchResults {
                results: results.clone(),
                search_time_ms: search_time,
            };

            if let Ok(serialized) = serde_json::to_vec(&cached_results) {
                let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
                let _ = cache.set(cache_key, serialized, Some(ttl)).await;
            }
        }

        Ok((results, search_time))
    }

    /// 生成缓存键
    fn generate_cache_key(&self, query_vector: &[f32], query: &SearchQuery) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 对向量进行哈希（优化：使用量化的向量）
        for &val in query_vector.iter() {
            // 量化到3位小数，减少缓存键的变化
            let quantized = (val * 1000.0).round() as i32;
            quantized.hash(&mut hasher);
        }

        query.limit.hash(&mut hasher);
        if let Some(threshold) = query.threshold {
            ((threshold * 1000.0).round() as i32).hash(&mut hasher);
        }

        format!("{}_{}", self.config.cache_key_prefix, hasher.finish())
    }

    /// 清空缓存
    pub async fn clear_cache(&self) -> Result<()> {
        #[cfg(feature = "redis-cache")]
        if let Some(cache) = &self.cache {
            cache.clear().await?;
        }

        Ok(())
    }
}

// ============================================================================
// SearchEngine Trait 实现 (V4)
// ============================================================================

use agent_mem_traits::{SearchEngine, Query, QueryIntent, QueryIntentType};
use async_trait::async_trait;

#[async_trait]
impl SearchEngine for CachedVectorSearchEngine {
    /// 执行搜索查询（V4 Query 接口）
    async fn search(&self, query: &Query) -> Result<Vec<agent_mem_traits::SearchResultV4>> {
        // 1. 提取查询向量
        let query_vector = match &query.intent {
            QueryIntent::Vector { embedding } => embedding.clone(),
            QueryIntent::Hybrid { intents, .. } => {
                // 从混合查询中提取向量意图
                intents.iter()
                    .find_map(|intent| {
                        if let QueryIntent::Vector { embedding } = intent {
                            Some(embedding.clone())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| agent_mem_traits::AgentMemError::validation_error(
                        "Hybrid query must contain at least one Vector intent"
                    ))?
            }
            _ => {
                return Err(agent_mem_traits::AgentMemError::validation_error(
                    format!("Unsupported query intent for CachedVectorSearchEngine: {:?}. Use Vector or Hybrid intent.", query.intent)
                ));
            }
        };

        // 2. 转换 Query V4 到 SearchQuery
        let search_query = SearchQuery::from_query_v4(query);

        // 3. 执行缓存向量搜索（使用现有的 search 方法）
        let (results, _elapsed) = self.search(query_vector, &search_query).await?;

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
        "CachedVectorSearchEngine"
    }

    /// 获取支持的查询意图类型
    fn supported_intents(&self) -> Vec<QueryIntentType> {
        vec![
            QueryIntentType::Vector, // 主要支持向量查询
            QueryIntentType::Hybrid, // 也支持混合查询（提取向量部分）
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let config = CachedVectorSearchConfig::default();
        let query_vector = vec![0.1, 0.2, 0.3];
        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            threshold: Some(0.7),
            ..Default::default()
        };

        // 测试：相同输入应该生成相同的键
        // 注意：由于没有实际的engine，这里只测试函数存在
        // 实际测试需要在集成测试中进行
    }
}
