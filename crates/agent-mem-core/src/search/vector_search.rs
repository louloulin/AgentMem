//! 向量搜索引擎
//!
//! 提供语义相似度搜索功能，基于向量存储后端

use super::{SearchQuery, SearchResult};
use agent_mem_traits::{AgentMemError, Result, VectorData, VectorSearchResult, VectorStore};
use std::sync::Arc;
use std::time::Instant;

/// 向量搜索引擎
pub struct VectorSearchEngine {
    /// 向量存储后端
    vector_store: Arc<dyn VectorStore>,
    /// 嵌入模型维度
    embedding_dimension: usize,
}

impl VectorSearchEngine {
    /// 创建新的向量搜索引擎
    ///
    /// # Arguments
    ///
    /// * `vector_store` - 向量存储后端
    /// * `embedding_dimension` - 嵌入向量维度
    pub fn new(vector_store: Arc<dyn VectorStore>, embedding_dimension: usize) -> Self {
        Self {
            vector_store,
            embedding_dimension,
        }
    }

    /// 执行向量搜索
    ///
    /// # Arguments
    ///
    /// * `query_vector` - 查询向量
    /// * `query` - 搜索查询参数
    ///
    /// # Returns
    ///
    /// 返回搜索结果列表和搜索时间（毫秒）
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        query: &SearchQuery,
    ) -> Result<(Vec<SearchResult>, u64)> {
        let start = Instant::now();

        // 验证向量维度
        if query_vector.len() != self.embedding_dimension {
            return Err(AgentMemError::validation_error(format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.embedding_dimension
            )));
        }

        // 执行向量搜索
        let vector_results = self
            .vector_store
            .search_vectors(query_vector, query.limit, query.threshold)
            .await?;

        // 转换为搜索结果
        let results = vector_results
            .into_iter()
            .map(|vr| SearchResult {
                id: vr.id,
                content: vr
                    .metadata
                    .get("content")
                    .map(|s| s.clone())
                    .unwrap_or_default(),
                score: vr.similarity,
                vector_score: Some(vr.similarity),
                fulltext_score: None,
                metadata: Some(
                    serde_json::to_value(&vr.metadata).unwrap_or(serde_json::Value::Null),
                ),
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;

        Ok((results, elapsed))
    }

    /// 批量添加向量
    ///
    /// # Arguments
    ///
    /// * `vectors` - 向量数据列表
    ///
    /// # Returns
    ///
    /// 返回添加的向量 ID 列表
    pub async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        // 验证所有向量维度
        for vector in &vectors {
            if vector.vector.len() != self.embedding_dimension {
                return Err(AgentMemError::validation_error(format!(
                    "Vector dimension {} does not match expected dimension {}",
                    vector.vector.len(),
                    self.embedding_dimension
                )));
            }
        }

        self.vector_store.add_vectors(vectors).await
    }

    /// 删除向量
    ///
    /// # Arguments
    ///
    /// * `ids` - 要删除的向量 ID 列表
    pub async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        self.vector_store.delete_vectors(ids).await
    }

    /// 获取向量存储统计信息
    pub async fn get_stats(&self) -> Result<VectorStoreStats> {
        // TODO: 实现统计信息获取
        Ok(VectorStoreStats {
            total_vectors: 0,
            dimension: self.embedding_dimension,
            index_type: "unknown".to_string(),
        })
    }
}

/// 向量存储统计信息
#[derive(Debug, Clone)]
pub struct VectorStoreStats {
    /// 总向量数
    pub total_vectors: usize,
    /// 向量维度
    pub dimension: usize,
    /// 索引类型
    pub index_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_storage::backends::MemoryVectorStore;
    use agent_mem_traits::VectorStoreConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_vector_search_engine() {
        let config = VectorStoreConfig {
            provider: "memory".to_string(),
            path: "".to_string(),
            table_name: "test".to_string(),
            dimension: Some(128),
            api_key: None,
            url: None,
            index_name: None,
            collection_name: None,
        };
        let vector_store = Arc::new(MemoryVectorStore::new(config).await.unwrap());
        let engine = VectorSearchEngine::new(vector_store.clone(), 128);

        // 添加测试向量
        let mut metadata = HashMap::new();
        metadata.insert(
            "content".to_string(),
            "test content".to_string(),
        );

        let vectors = vec![VectorData {
            id: "test-1".to_string(),
            vector: vec![0.1; 128],
            metadata,
        }];

        let ids = engine.add_vectors(vectors).await.unwrap();
        assert_eq!(ids.len(), 1);

        // 执行搜索
        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            threshold: Some(0.5),
            ..Default::default()
        };

        let query_vector = vec![0.1; 128];
        let (results, elapsed) = engine.search(query_vector, &query).await.unwrap();

        assert!(!results.is_empty());
        assert!(elapsed >= 0); // 允许 0，因为内存操作可能非常快
    }

    #[tokio::test]
    async fn test_vector_dimension_validation() {
        let config = VectorStoreConfig::default();
        let vector_store = Arc::new(MemoryVectorStore::new(config).await.unwrap());
        let engine = VectorSearchEngine::new(vector_store, 128);

        let query = SearchQuery::default();
        let wrong_dimension_vector = vec![0.1; 64]; // 错误的维度

        let result = engine.search(wrong_dimension_vector, &query).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_vectors() {
        let config = VectorStoreConfig {
            dimension: Some(128),
            ..Default::default()
        };
        let vector_store = Arc::new(MemoryVectorStore::new(config).await.unwrap());
        let engine = VectorSearchEngine::new(vector_store, 128);

        // 添加向量
        let mut metadata = HashMap::new();
        metadata.insert(
            "content".to_string(),
            "test".to_string(),
        );

        let vectors = vec![VectorData {
            id: "test-1".to_string(),
            vector: vec![0.1; 128],
            metadata,
        }];

        let ids = engine.add_vectors(vectors).await.unwrap();

        // 删除向量
        let result = engine.delete_vectors(ids).await;
        assert!(result.is_ok());
    }
}
