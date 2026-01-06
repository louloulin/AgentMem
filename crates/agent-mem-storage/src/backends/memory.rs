//! 内存向量存储实现

use agent_mem_traits::{
    AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// 内存向量存储实现
pub struct MemoryVectorStore {
    config: VectorStoreConfig,
    vectors: Arc<DashMap<String, VectorData>>,
}

impl MemoryVectorStore {
    /// 创建新的内存向量存储实例
    pub async fn new(config: VectorStoreConfig) -> Result<Self> {
        Ok(Self {
            config,
            vectors: Arc::new(DashMap::new()),
        })
    }

    /// 计算余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// 计算欧几里得距离
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

#[async_trait]
impl VectorStore for MemoryVectorStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        for vector in vectors {
            // 验证向量维度
            if let Some(expected_dim) = self.config.dimension {
                if vector.vector.len() != expected_dim {
                    return Err(AgentMemError::validation_error(format!(
                        "Vector dimension mismatch: expected {}, got {}",
                        expected_dim,
                        vector.vector.len()
                    )));
                }
            }

            let id = vector.id.clone();
            self.vectors.insert(id.clone(), vector);
            ids.push(id);
        }

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let mut results = Vec::new();

        // 验证查询向量维度
        if let Some(expected_dim) = self.config.dimension {
            if query_vector.len() != expected_dim {
                return Err(AgentMemError::validation_error(format!(
                    "Query vector dimension mismatch: expected {}, got {}",
                    expected_dim,
                    query_vector.len()
                )));
            }
        }

        for entry in self.vectors.iter() {
            let vector_data = entry.value();

            // 计算相似度和距离
            let similarity = self.cosine_similarity(&query_vector, &vector_data.vector);
            let distance = self.euclidean_distance(&query_vector, &vector_data.vector);

            // 应用阈值过滤
            if let Some(threshold) = threshold {
                if similarity < threshold {
                    continue;
                }
            }

            results.push(VectorSearchResult {
                id: vector_data.id.clone(),
                vector: vector_data.vector.clone(),
                metadata: vector_data.metadata.clone(),
                similarity,
                distance,
            });
        }

        // 按相似度排序（降序）
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制结果数量
        results.truncate(limit);

        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        for id in ids {
            self.vectors.remove(&id);
        }
        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        for vector in vectors {
            // 验证向量维度
            if let Some(expected_dim) = self.config.dimension {
                if vector.vector.len() != expected_dim {
                    return Err(AgentMemError::validation_error(format!(
                        "Vector dimension mismatch: expected {}, got {}",
                        expected_dim,
                        vector.vector.len()
                    )));
                }
            }

            let id = vector.id.clone();
            if self.vectors.contains_key(&id) {
                self.vectors.insert(id, vector);
            } else {
                return Err(AgentMemError::not_found(format!(
                    "Vector with id {id} not found"
                )));
            }
        }
        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        Ok(self.vectors.get(id).map(|entry| entry.value().clone()))
    }

    async fn count_vectors(&self) -> Result<usize> {
        Ok(self.vectors.len())
    }

    async fn clear(&self) -> Result<()> {
        self.vectors.clear();
        Ok(())
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &std::collections::HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        use crate::utils::VectorStoreDefaults;
        self.default_search_with_filters(query_vector, limit, filters, threshold)
            .await
    }

    async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
        use crate::utils::VectorStoreDefaults;
        self.default_health_check("Memory").await
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        use crate::utils::VectorStoreDefaults;
        self.default_get_stats(self.config.dimension.unwrap_or(1536))
            .await
    }

    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        use crate::utils::VectorStoreDefaults;
        self.default_add_vectors_batch(batches).await
    }

    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        use crate::utils::VectorStoreDefaults;
        self.default_delete_vectors_batch(id_batches).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    async fn create_test_store() -> MemoryVectorStore {
        let config = VectorStoreConfig {
            provider: "memory".to_string(),
            dimension: Some(3),
            ..Default::default()
        };
        MemoryVectorStore::new(config).await?
    }

    fn create_test_vector(id: &str, vector: Vec<f32>) -> VectorData {
        VectorData {
            id: id.to_string(),
            vector,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_add_and_get_vectors() {
        let store = create_test_store().await;

        let vectors = vec![
            create_test_vector("1", vec![1.0, 0.0, 0.0]),
            create_test_vector("2", vec![0.0, 1.0, 0.0]),
        ];

        let ids = store.add_vectors(vectors).await?;
        assert_eq!(ids.len(), 2);

        let vector = store.get_vector("1").await?;
        assert!(vector.is_some());
        assert_eq!(vector.unwrap().vector, vec![1.0, 0.0, 0.0]);
    }

    #[tokio::test]
    async fn test_search_vectors() {
        let store = create_test_store().await;

        let vectors = vec![
            create_test_vector("1", vec![1.0, 0.0, 0.0]),
            create_test_vector("2", vec![0.0, 1.0, 0.0]),
            create_test_vector("3", vec![0.0, 0.0, 1.0]),
        ];

        store.add_vectors(vectors).await?;

        // 搜索与第一个向量相似的向量
        let results = store
            .search_vectors(vec![1.0, 0.0, 0.0], 2, None)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "1");
        assert_eq!(results[0].similarity, 1.0); // 完全匹配
    }

    #[tokio::test]
    async fn test_delete_vectors() {
        let store = create_test_store().await;

        let vectors = vec![
            create_test_vector("1", vec![1.0, 0.0, 0.0]),
            create_test_vector("2", vec![0.0, 1.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;
        assert_eq!(store.count_vectors().await?, 2);

        store.delete_vectors(vec!["1".to_string()]).await?;
        assert_eq!(store.count_vectors().await?, 1);

        let vector = store.get_vector("1").await?;
        assert!(vector.is_none());
    }

    #[tokio::test]
    async fn test_update_vectors() {
        let store = create_test_store().await;

        let vectors = vec![create_test_vector("1", vec![1.0, 0.0, 0.0])];
        store.add_vectors(vectors).await?;

        let updated_vectors = vec![create_test_vector("1", vec![0.0, 1.0, 0.0])];
        store.update_vectors(updated_vectors).await?;

        let vector = store.get_vector("1").await?.unwrap();
        assert_eq!(vector.vector, vec![0.0, 1.0, 0.0]);
    }

    #[tokio::test]
    async fn test_clear() {
        let store = create_test_store().await;

        let vectors = vec![
            create_test_vector("1", vec![1.0, 0.0, 0.0]),
            create_test_vector("2", vec![0.0, 1.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;
        assert_eq!(store.count_vectors().await?, 2);

        store.clear().await?;
        assert_eq!(store.count_vectors().await?, 0);
    }

    #[tokio::test]
    async fn test_dimension_validation() {
        let store = create_test_store().await;

        // 尝试添加错误维度的向量
        let vectors = vec![create_test_vector("1", vec![1.0, 0.0])]; // 2维而不是3维
        let result = store.add_vectors(vectors).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let store = create_test_store().await;

        // 测试余弦相似度计算
        let sim = store.cosine_similarity(&[1.0, 0.0, 0.0], &[1.0, 0.0, 0.0]);
        assert_eq!(sim, 1.0);

        let sim = store.cosine_similarity(&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0]);
        assert_eq!(sim, 0.0);

        let sim = store.cosine_similarity(&[1.0, 0.0, 0.0], &[-1.0, 0.0, 0.0]);
        assert_eq!(sim, -1.0);
    }
}
