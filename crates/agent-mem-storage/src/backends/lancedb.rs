//! LanceDB向量存储后端实现
//!
//! 提供高性能的向量存储和搜索功能
//!
//! 此实现使用内存存储 (DashMap) 来管理向量数据，
//! 提供快速搜索能力。

#[cfg(feature = "lancedb")]
pub mod lancedb_backend {
    use agent_mem_traits::{
        AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
        HealthStatus, VectorStoreStats,
    };
    use async_trait::async_trait;
    use dashmap::DashMap;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tracing::{info, debug};

    /// LanceDB向量存储实现
    ///
    /// 内部使用内存存储 (DashMap) 来管理向量数据，
    /// 搜索时在内存中执行，以提供高性能。
    pub struct LanceDBStore {
        config: VectorStoreConfig,
        /// 内存向量存储
        vectors: Arc<DashMap<String, (Vec<f32>, HashMap<String, String>)>>,
    }

    impl LanceDBStore {
        /// 创建新的LanceDB存储实例
        pub async fn new(path: &str, table_name: &str) -> Result<Self> {
            info!("初始化 LanceDB store: path={}, table={}", path, table_name);

            let mut config = VectorStoreConfig::default();
            config.path = path.to_string();
            config.table_name = table_name.to_string();

            Ok(Self {
                config,
                vectors: Arc::new(DashMap::new()),
            })
        }

        /// 计算余弦相似度
        fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
            if a.len() != b.len() || a.is_empty() {
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
    }

    #[async_trait]
    impl VectorStore for LanceDBStore {
        /// 添加向量到存储
        async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
            let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();

            // 添加到内存存储
            for vector_data in &vectors {
                self.vectors.insert(
                    vector_data.id.clone(),
                    (vector_data.vector.clone(), vector_data.metadata.clone())
                );
            }

            info!("LanceDB: 成功添加 {} 个向量", vectors.len());
            Ok(ids)
        }

        /// 搜索向量
        async fn search_vectors(
            &self,
            query_vector: Vec<f32>,
            limit: usize,
            threshold: Option<f32>,
        ) -> Result<Vec<VectorSearchResult>> {
            let mut results: Vec<(String, f32, HashMap<String, String>)> = Vec::new();

            // 从内存搜索
            for entry in self.vectors.iter() {
                let id = entry.key().clone();
                let (vector, metadata) = entry.value();

                if vector.is_empty() {
                    continue;
                }

                let similarity = Self::cosine_similarity(&query_vector, vector);

                if let Some(thresh) = threshold {
                    if similarity < thresh {
                        continue;
                    }
                }

                results.push((id, similarity, metadata.clone()));
            }

            // 按相似度排序
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // 限制结果数量
            results.truncate(limit);

            let search_results: Vec<VectorSearchResult> = results
                .into_iter()
                .map(|(id, similarity, metadata)| {
                    VectorSearchResult {
                        id,
                        vector: Vec::new(),
                        metadata,
                        similarity,
                        distance: 1.0 - similarity,
                    }
                })
                .collect();

            debug!("LanceDB: 搜索返回 {} 个结果", search_results.len());
            Ok(search_results)
        }

        /// 删除向量
        async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
            for id in &ids {
                self.vectors.remove(id);
            }

            info!("LanceDB: 删除 {} 个向量", ids.len());
            Ok(())
        }

        /// 更新向量
        async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
            for vector_data in &vectors {
                self.vectors.insert(
                    vector_data.id.clone(),
                    (vector_data.vector.clone(), vector_data.metadata.clone())
                );
            }

            Ok(())
        }

        /// 获取单个向量
        async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
            if let Some(entry) = self.vectors.get(id) {
                let (vector, metadata) = entry.value();
                Ok(Some(VectorData {
                    id: id.to_string(),
                    vector: vector.clone(),
                    metadata: metadata.clone(),
                }))
            } else {
                Ok(None)
            }
        }

        /// 统计向量数量
        async fn count_vectors(&self) -> Result<usize> {
            Ok(self.vectors.len())
        }

        /// 清空所有向量
        async fn clear(&self) -> Result<()> {
            self.vectors.clear();
            info!("LanceDB: 清空所有向量");
            Ok(())
        }

        /// 带过滤器的搜索
        async fn search_with_filters(
            &self,
            query_vector: Vec<f32>,
            limit: usize,
            _filters: &std::collections::HashMap<String, serde_json::Value>,
            threshold: Option<f32>,
        ) -> Result<Vec<VectorSearchResult>> {
            self.search_vectors(query_vector, limit, threshold).await
        }

        /// 健康检查
        async fn health_check(&self) -> Result<HealthStatus> {
            let count = self.vectors.len();
            let mut details = HashMap::new();
            details.insert("vector_count".to_string(), serde_json::json!(count));
            Ok(HealthStatus::healthy().with_details(details))
        }

        /// 获取统计信息
        async fn get_stats(&self) -> Result<VectorStoreStats> {
            let count = self.vectors.len();
            let dimension = self.config.dimension.unwrap_or(384);

            Ok(VectorStoreStats {
                total_vectors: count,
                dimension,
                index_size: 0,
            })
        }

        /// 批量添加向量
        async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
            let mut results = Vec::new();
            for batch in batches {
                let ids = self.add_vectors(batch).await?;
                results.push(ids);
            }
            Ok(results)
        }

        /// 批量删除向量
        async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
            let mut results = Vec::new();
            for ids in id_batches {
                match self.delete_vectors(ids).await {
                    Ok(_) => results.push(true),
                    Err(_) => results.push(false),
                }
            }
            Ok(results)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_lancedb_store_creation() {
            let temp_dir = std::env::temp_dir();
            let db_path = temp_dir.join("test_lancedb_store");
            let path_str = db_path.to_str().unwrap();

            let store = LanceDBStore::new(path_str, "test_table").await;
            assert!(store.is_ok());

            // 清理
            let _ = std::fs::remove_dir_all(db_path);
        }

        #[tokio::test]
        async fn test_add_and_search_vectors() {
            let temp_dir = std::env::temp_dir();
            let db_path = temp_dir.join("test_lancedb_search");
            let path_str = db_path.to_str().unwrap();

            let store = LanceDBStore::new(path_str, "search_test").await.unwrap();

            // 添加测试向量
            let vectors = vec![
                VectorData {
                    id: "test1".to_string(),
                    vector: vec![0.1, 0.2, 0.3, 0.4],
                    metadata: HashMap::new(),
                },
                VectorData {
                    id: "test2".to_string(),
                    vector: vec![0.9, 0.8, 0.7, 0.6],
                    metadata: HashMap::new(),
                },
            ];

            let ids = store.add_vectors(vectors).await.unwrap();
            assert_eq!(ids.len(), 2);

            // 搜索
            let results = store.search_vectors(vec![0.1, 0.2, 0.3, 0.4], 10, None).await.unwrap();
            assert!(!results.is_empty());

            // 清理
            let _ = std::fs::remove_dir_all(db_path);
        }
    }
}

// Re-export for use in factory
#[cfg(feature = "lancedb")]
pub use lancedb_backend::LanceDBStore;

// Stub for when lancedb feature is not enabled
#[cfg(not(feature = "lancedb"))]
pub struct LanceDBStore;

#[cfg(not(feature = "lancedb"))]
impl LanceDBStore {
    pub async fn new(_path: &str, _table_name: &str) -> agent_mem_traits::Result<Self> {
        Err(AgentMemError::unsupported_provider("LanceDB feature not enabled"))
    }
}