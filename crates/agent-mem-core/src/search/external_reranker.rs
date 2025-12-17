//! 外部重排序器
//!
//! 提供统一的Reranker trait，支持内部和外部重排序实现
//! 包括Cohere、Jina等外部API集成
//! Phase 2.2: 增强重排序机制，添加LLM-based重排序和缓存支持

use crate::search::{SearchQuery, SearchResult};
use agent_mem_traits::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

/// 重排序器trait
///
/// 统一的接口，支持内部和外部重排序实现
#[async_trait]
pub trait Reranker: Send + Sync {
    /// 重排序搜索结果
    ///
    /// # Arguments
    /// * `query` - 查询文本
    /// * `query_vector` - 查询向量（可选，用于内部重排序）
    /// * `documents` - 待重排序的搜索结果
    /// * `query_params` - 查询参数（包含limit等）
    ///
    /// # Returns
    /// 重排序后的搜索结果，按相关性从高到低排序
    async fn rerank(
        &self,
        query: &str,
        query_vector: Option<&[f32]>,
        documents: Vec<SearchResult>,
        query_params: &SearchQuery,
    ) -> Result<Vec<SearchResult>>;
}

/// 内部重排序器适配器
///
/// 将现有的ResultReranker适配为Reranker trait
pub struct InternalReranker {
    reranker: super::reranker::ResultReranker,
}

impl InternalReranker {
    /// 创建新的内部重排序器
    pub fn new() -> Self {
        Self {
            reranker: super::reranker::ResultReranker::with_default_config(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(config: super::reranker::RerankConfig) -> Self {
        Self {
            reranker: super::reranker::ResultReranker::new(config),
        }
    }
}

impl Default for InternalReranker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Reranker for InternalReranker {
    async fn rerank(
        &self,
        _query: &str,
        query_vector: Option<&[f32]>,
        documents: Vec<SearchResult>,
        query_params: &SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        // 内部重排序需要查询向量
        let query_vec = query_vector.ok_or_else(|| {
            agent_mem_traits::AgentMemError::InvalidInput(
                "Internal reranker requires query vector".to_string(),
            )
        })?;

        // 调用现有的ResultReranker
        self.reranker
            .rerank(documents, query_vec, query_params)
            .await
    }
}

/// Cohere重排序器
///
/// 使用Cohere API进行重排序（需要cohere crate）
#[cfg(feature = "cohere")]
pub struct CohereReranker {
    api_key: String,
    model: String,
    // client: cohere::Client,  // 需要添加cohere依赖
}

#[cfg(feature = "cohere")]
impl CohereReranker {
    /// 创建新的Cohere重排序器
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[cfg(feature = "cohere")]
#[async_trait]
impl Reranker for CohereReranker {
    async fn rerank(
        &self,
        query: &str,
        _query_vector: Option<&[f32]>,
        documents: Vec<SearchResult>,
        query_params: &SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        // TODO: 实现Cohere API调用
        // 1. 提取文档文本
        // 2. 调用Cohere rerank API
        // 3. 根据返回的relevance_score重新排序
        // 4. 返回重排序后的结果

        // 临时实现：返回原始顺序
        // 实际实现需要：
        // - 添加cohere crate依赖
        // - 实现API调用逻辑
        // - 处理错误和重试

        let top_k = query_params.limit.min(documents.len());
        Ok(documents.into_iter().take(top_k).collect())
    }
}

/// LLM-based重排序器配置
#[derive(Debug, Clone)]
pub struct LLMRerankerConfig {
    /// LLM模型名称
    pub model: String,
    /// 提示模板
    pub prompt_template: String,
    /// 启用缓存
    pub enable_cache: bool,
    /// 缓存TTL（秒）
    pub cache_ttl_seconds: u64,
    /// LLM评分权重（与向量分数结合）
    pub llm_weight: f32,
    /// 向量分数权重
    pub vector_weight: f32,
}

impl Default for LLMRerankerConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            prompt_template: "Given the query: {query}\n\nRate the relevance of the following document on a scale of 0-1:\n{document}\n\nRelevance score:".to_string(),
            enable_cache: true,
            cache_ttl_seconds: 3600, // 1小时
            llm_weight: 0.7,
            vector_weight: 0.3,
        }
    }
}

/// LLM-based重排序器
///
/// Phase 2.2: 使用LLM评估文档相关性，提供更精确的重排序
pub struct LLMReranker {
    config: LLMRerankerConfig,
    /// 重排序缓存 (query_hash + doc_ids -> reranked_results)
    cache: Arc<RwLock<HashMap<String, CachedRerankResult>>>,
    /// LLM客户端（通过agent-mem-llm）
    #[allow(dead_code)]
    llm_client: Option<Arc<dyn LLMClient>>,
}

/// LLM客户端trait（简化接口）
#[async_trait]
trait LLMClient: Send + Sync {
    async fn score_relevance(&self, query: &str, document: &str) -> Result<f32>;
}

/// 缓存的重排序结果
#[derive(Debug, Clone)]
struct CachedRerankResult {
    results: Vec<SearchResult>,
    cached_at: DateTime<Utc>,
}

impl LLMReranker {
    /// 创建新的LLM重排序器
    pub fn new(config: LLMRerankerConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            llm_client: None, // TODO: 集成agent-mem-llm
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(LLMRerankerConfig::default())
    }

    /// 生成缓存键
    fn generate_cache_key(&self, query: &str, documents: &[SearchResult]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        for doc in documents {
            hasher.update(doc.id.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// 从缓存获取结果
    async fn get_cached_result(&self, cache_key: &str) -> Option<Vec<SearchResult>> {
        if !self.config.enable_cache {
            return None;
        }

        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(cache_key) {
            let age = Utc::now() - cached.cached_at;
            if age.num_seconds() < self.config.cache_ttl_seconds as i64 {
                return Some(cached.results.clone());
            }
        }
        None
    }

    /// 缓存结果
    async fn cache_result(&self, cache_key: String, results: Vec<SearchResult>) {
        if !self.config.enable_cache {
            return;
        }

        let mut cache = self.cache.write().await;
        cache.insert(
            cache_key,
            CachedRerankResult {
                results,
                cached_at: Utc::now(),
            },
        );
    }

    /// 使用LLM评估相关性（简化实现，实际应调用LLM API）
    async fn evaluate_relevance_llm(&self, query: &str, document: &str) -> Result<f32> {
        // TODO: 集成agent-mem-llm进行实际LLM调用
        // 当前使用启发式方法作为占位符
        
        // 启发式评分：基于文本匹配
        let query_lower = query.to_lowercase();
        let doc_lower = document.to_lowercase();

        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut match_score = 0.0;

        for word in &query_words {
            if doc_lower.contains(word) {
                match_score += 1.0;
            }
        }

        let normalized_score = if query_words.is_empty() {
            0.0
        } else {
            match_score / query_words.len() as f32
        };

        // 考虑文档长度（较短的文档可能更精确）
        let length_penalty = 1.0 / (1.0 + (document.len() as f32 / 1000.0));

        Ok((normalized_score + length_penalty) / 2.0)
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// 获取缓存统计
    pub async fn get_cache_stats(&self) -> RerankCacheStats {
        let cache = self.cache.read().await;
        RerankCacheStats {
            entries: cache.len(),
            hit_rate: 0.0, // TODO: 实现命中率统计
        }
    }
}

/// 重排序缓存统计
#[derive(Debug, Clone)]
pub struct RerankCacheStats {
    pub entries: usize,
    pub hit_rate: f64,
}

#[async_trait]
impl Reranker for LLMReranker {
    async fn rerank(
        &self,
        query: &str,
        query_vector: Option<&[f32]>,
        mut documents: Vec<SearchResult>,
        query_params: &SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        // 检查缓存
        if self.config.enable_cache {
            let cache_key = self.generate_cache_key(query, &documents);
            if let Some(cached) = self.get_cached_result(&cache_key).await {
                return Ok(cached);
            }
        }

        // 使用LLM评估每个文档的相关性
        for doc in &mut documents {
            let llm_score = self
                .evaluate_relevance_llm(query, &doc.content)
                .await?;
            
            // 结合原始向量分数和LLM评分
            let vector_score = query_vector
                .and_then(|_| doc.vector_score)
                .unwrap_or(doc.score);
            
            doc.score = vector_score * self.config.vector_weight
                + llm_score * self.config.llm_weight;
        }

        // 按新分数排序
        documents.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制结果数量
        let top_k = query_params.limit.min(documents.len());
        documents.truncate(top_k);

        // 缓存结果
        if self.config.enable_cache {
            let cache_key = self.generate_cache_key(query, &documents);
            self.cache_result(cache_key, documents.clone()).await;
        }

        Ok(documents)
    }
}

/// 带缓存的重排序器包装器
///
/// Phase 2.2: 为任何重排序器添加缓存支持
pub struct CachedReranker {
    inner: Box<dyn Reranker>,
    cache: Arc<RwLock<HashMap<String, CachedRerankResult>>>,
    cache_ttl_seconds: u64,
}

impl CachedReranker {
    /// 创建带缓存的重排序器
    pub fn new(inner: Box<dyn Reranker>, cache_ttl_seconds: u64) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_seconds,
        }
    }

    fn generate_cache_key(&self, query: &str, documents: &[SearchResult]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        for doc in documents {
            hasher.update(doc.id.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    async fn get_cached(&self, cache_key: &str) -> Option<Vec<SearchResult>> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(cache_key) {
            let age = Utc::now() - cached.cached_at;
            if age.num_seconds() < self.cache_ttl_seconds as i64 {
                return Some(cached.results.clone());
            }
        }
        None
    }

    async fn cache_result(&self, cache_key: String, results: Vec<SearchResult>) {
        let mut cache = self.cache.write().await;
        cache.insert(
            cache_key,
            CachedRerankResult {
                results,
                cached_at: Utc::now(),
            },
        );
    }
}

#[async_trait]
impl Reranker for CachedReranker {
    async fn rerank(
        &self,
        query: &str,
        query_vector: Option<&[f32]>,
        documents: Vec<SearchResult>,
        query_params: &SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        let cache_key = self.generate_cache_key(query, &documents);
        
        // 检查缓存
        if let Some(cached) = self.get_cached(&cache_key).await {
            return Ok(cached);
        }

        // 调用内部重排序器
        let results = self
            .inner
            .rerank(query, query_vector, documents, query_params)
            .await?;

        // 缓存结果
        self.cache_result(cache_key, results.clone()).await;

        Ok(results)
    }
}

/// 重排序器工厂
pub struct RerankerFactory;

impl RerankerFactory {
    /// 创建默认的内部重排序器
    pub fn create_internal() -> Box<dyn Reranker> {
        Box::new(InternalReranker::new())
    }

    /// 创建带缓存的内部重排序器
    pub fn create_internal_cached(cache_ttl_seconds: u64) -> Box<dyn Reranker> {
        Box::new(CachedReranker::new(
            Self::create_internal(),
            cache_ttl_seconds,
        ))
    }

    /// 创建LLM重排序器
    pub fn create_llm(config: LLMRerankerConfig) -> Box<dyn Reranker> {
        Box::new(LLMReranker::new(config))
    }

    /// 创建带缓存的LLM重排序器
    pub fn create_llm_cached(config: LLMRerankerConfig) -> Box<dyn Reranker> {
        Box::new(CachedReranker::new(
            Self::create_llm(config.clone()),
            config.cache_ttl_seconds,
        ))
    }

    /// 创建Cohere重排序器（如果启用cohere feature）
    #[cfg(feature = "cohere")]
    pub fn create_cohere(api_key: String, model: String) -> Box<dyn Reranker> {
        Box::new(CohereReranker::new(api_key, model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchQuery;

    #[tokio::test]
    async fn test_internal_reranker() {
        let reranker = InternalReranker::new();

        let query_vector = vec![0.1; 128];
        let query = SearchQuery {
            query: "test query".to_string(),
            limit: 10,
            threshold: Some(0.5),
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            filters: None,
            metadata_filters: None,
        };

        let documents = vec![
            SearchResult {
                id: "1".to_string(),
                content: "test content 1".to_string(),
                score: 0.8,
                vector_score: Some(0.8),
                fulltext_score: None,
                metadata: None,
            },
            SearchResult {
                id: "2".to_string(),
                content: "test content 2".to_string(),
                score: 0.6,
                vector_score: Some(0.6),
                fulltext_score: None,
                metadata: None,
            },
        ];

        let result = reranker
            .rerank("test", Some(&query_vector), documents, &query)
            .await;

        assert!(result.is_ok());
        let reranked = result.unwrap();
        assert!(!reranked.is_empty());
    }
}
