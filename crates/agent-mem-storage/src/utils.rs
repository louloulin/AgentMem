//! Storage utilities and common implementations
//!
//! This module provides common implementations for VectorStore trait methods
//! to reduce code duplication across different storage backends.

use agent_mem_traits::{HealthStatus, Result, VectorData, VectorSearchResult, VectorStoreStats};
use async_trait::async_trait;
use std::collections::HashMap;

/// Default implementations for VectorStore trait methods
#[async_trait]
pub trait VectorStoreDefaults: Send + Sync {
    /// Default implementation for search_with_filters
    async fn default_search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>>
    where
        Self: agent_mem_traits::VectorStore,
    {
        // 首先进行基础向量搜索
        let mut results = self
            .search_vectors(query_vector, limit * 2, threshold)
            .await?;

        // 应用过滤器
        if !filters.is_empty() {
            results.retain(|result| {
                // 检查每个过滤条件
                filters.iter().all(|(key, expected_value)| {
                    if let Some(actual_value) = result.metadata.get(key) {
                        // 简单的字符串匹配
                        if let serde_json::Value::String(expected_str) = expected_value {
                            actual_value == expected_str
                        } else {
                            // 对于其他类型，转换为字符串比较
                            actual_value == &expected_value.to_string()
                        }
                    } else {
                        false
                    }
                })
            });
        }

        // 限制结果数量
        results.truncate(limit);
        Ok(results)
    }

    /// Default implementation for health_check
    async fn default_health_check(&self, store_name: &str) -> Result<HealthStatus>
    where
        Self: agent_mem_traits::VectorStore,
    {
        // 基本健康检查
        let vector_count = self.count_vectors().await?;

        Ok(HealthStatus {
            status: "healthy".to_string(),
            message: format!(
                "{store_name} store is healthy with {vector_count} vectors"
            ),
            timestamp: chrono::Utc::now(),
            details: HashMap::from([
                (
                    "vector_count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(vector_count)),
                ),
                (
                    "store_type".to_string(),
                    serde_json::Value::String(store_name.to_string()),
                ),
            ]),
        })
    }

    /// Default implementation for get_stats
    async fn default_get_stats(&self, dimension: usize) -> Result<VectorStoreStats>
    where
        Self: agent_mem_traits::VectorStore,
    {
        let vector_count = self.count_vectors().await?;

        Ok(VectorStoreStats {
            total_vectors: vector_count,
            dimension,
            index_size: vector_count,
        })
    }

    /// Default implementation for add_vectors_batch
    async fn default_add_vectors_batch(
        &self,
        batches: Vec<Vec<VectorData>>,
    ) -> Result<Vec<Vec<String>>>
    where
        Self: agent_mem_traits::VectorStore,
    {
        let mut all_results = Vec::new();

        for batch in batches {
            let batch_result = self.add_vectors(batch).await?;
            all_results.push(batch_result);
        }

        Ok(all_results)
    }

    /// Default implementation for delete_vectors_batch
    async fn default_delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>>
    where
        Self: agent_mem_traits::VectorStore,
    {
        let mut results = Vec::new();

        for batch in id_batches {
            let result = self.delete_vectors(batch).await;
            results.push(result.is_ok());
        }

        Ok(results)
    }
}

/// Blanket implementation for all types
impl<T> VectorStoreDefaults for T where T: agent_mem_traits::VectorStore {}
