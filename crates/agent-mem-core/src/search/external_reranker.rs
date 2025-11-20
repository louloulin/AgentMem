//! 外部重排序器
//!
//! 提供统一的Reranker trait，支持内部和外部重排序实现
//! 包括Cohere、Jina等外部API集成

use crate::search::{SearchQuery, SearchResult};
use agent_mem_traits::Result;
use async_trait::async_trait;

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

/// 重排序器工厂
pub struct RerankerFactory;

impl RerankerFactory {
    /// 创建默认的内部重排序器
    pub fn create_internal() -> Box<dyn Reranker> {
        Box::new(InternalReranker::new())
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
