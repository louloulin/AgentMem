//! Chroma向量存储实现

use agent_mem_traits::{
    AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Chroma API请求结构
#[derive(Debug, Serialize)]
struct ChromaAddRequest {
    ids: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    metadatas: Option<Vec<serde_json::Value>>,
}

/// Chroma查询请求结构
#[derive(Debug, Serialize)]
struct ChromaQueryRequest {
    query_embeddings: Vec<Vec<f32>>,
    n_results: usize,
    #[serde(rename = "where")]
    where_clause: Option<serde_json::Value>,
}

/// Chroma查询响应结构
#[derive(Debug, Deserialize)]
struct ChromaQueryResponse {
    ids: Vec<Vec<String>>,
    embeddings: Option<Vec<Vec<Vec<f32>>>>,
    metadatas: Option<Vec<Vec<Option<serde_json::Value>>>>,
    distances: Vec<Vec<f32>>,
}

/// Chroma集合创建请求结构
#[derive(Debug, Serialize)]
struct ChromaCreateCollectionRequest {
    name: String,
    metadata: Option<serde_json::Value>,
}

/// Chroma集合信息响应结构
#[derive(Debug, Deserialize)]
struct ChromaCollectionResponse {
    name: String,
    id: String,
    metadata: Option<serde_json::Value>,
}

/// Chroma向量存储实现
pub struct ChromaStore {
    config: VectorStoreConfig,
    client: Client,
    base_url: String,
    collection_name: String,
}

impl ChromaStore {
    /// 创建新的Chroma存储实例
    pub async fn new(config: VectorStoreConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .url
            .clone()
            .unwrap_or_else(|| "http://localhost:8000".to_string());

        let collection_name = config
            .collection_name
            .clone()
            .unwrap_or_else(|| "default".to_string());

        let store = Self {
            config,
            client,
            base_url,
            collection_name,
        };

        // 确保集合存在
        store.ensure_collection_exists().await?;

        Ok(store)
    }

    /// 确保集合存在，如果不存在则创建
    async fn ensure_collection_exists(&self) -> Result<()> {
        // 首先检查集合是否存在
        let collections_url = format!("{}/api/v1/collections", self.base_url);
        let response = self
            .client
            .get(&collections_url)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if response.status().is_success() {
            let collections: Vec<ChromaCollectionResponse> =
                response.json().await.map_err(|e| {
                    AgentMemError::parsing_error(format!("Failed to parse collections: {e}"))
                })?;

            // 检查集合是否已存在
            if collections.iter().any(|c| c.name == self.collection_name) {
                return Ok(());
            }
        }

        // 集合不存在，创建新集合
        self.create_collection().await
    }

    /// 创建新集合
    async fn create_collection(&self) -> Result<()> {
        let request = ChromaCreateCollectionRequest {
            name: self.collection_name.clone(),
            metadata: Some(serde_json::json!({
                "description": "AgentMem memory collection",
                "created_by": "agent-mem-storage"
            })),
        };

        let url = format!("{}/api/v1/collections", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Failed to create collection {}: {} - {}",
                self.collection_name, status, error_text
            )));
        }

        Ok(())
    }

    /// 获取集合URL
    fn get_collection_url(&self) -> String {
        format!(
            "{}/api/v1/collections/{}",
            self.base_url, self.collection_name
        )
    }
}

#[async_trait]
impl VectorStore for ChromaStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = vectors.iter().map(|v| v.vector.clone()).collect();
        let metadatas: Vec<serde_json::Value> = vectors
            .iter()
            .map(|v| serde_json::to_value(&v.metadata).unwrap_or(serde_json::Value::Null))
            .collect();

        let request = ChromaAddRequest {
            ids: ids.clone(),
            embeddings,
            metadatas: Some(metadatas),
        };

        let url = format!("{}/add", self.get_collection_url());
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {status}: {error_text}"
            )));
        }

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        _threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let request = ChromaQueryRequest {
            query_embeddings: vec![query_vector],
            n_results: limit,
            where_clause: None,
        };

        let url = format!("{}/query", self.get_collection_url());
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {status}: {error_text}"
            )));
        }

        let chroma_response: ChromaQueryResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        let mut results = Vec::new();

        if let (Some(ids), Some(distances)) =
            (chroma_response.ids.first(), chroma_response.distances.first())
        {
            for (i, (id, distance)) in ids.iter().zip(distances.iter()).enumerate() {
                let vector = chroma_response
                    .embeddings
                    .as_ref()
                    .and_then(|embs| embs.first())
                    .and_then(|emb| emb.get(i))
                    .cloned()
                    .unwrap_or_default();

                let metadata = chroma_response
                    .metadatas
                    .as_ref()
                    .and_then(|metas| metas.first())
                    .and_then(|meta| meta.get(i))
                    .and_then(|m| m.as_ref())
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                // Chroma返回距离，我们需要转换为相似度
                let similarity = 1.0 / (1.0 + distance);

                results.push(VectorSearchResult {
                    id: id.clone(),
                    vector,
                    metadata,
                    similarity,
                    distance: *distance,
                });
            }
        }

        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        let url = format!("{}/delete", self.get_collection_url());
        let request = serde_json::json!({
            "ids": ids
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {status}: {error_text}"
            )));
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = vectors.iter().map(|v| v.vector.clone()).collect();
        let metadatas: Vec<serde_json::Value> = vectors
            .iter()
            .map(|v| serde_json::to_value(&v.metadata).unwrap_or(serde_json::Value::Null))
            .collect();

        let request = serde_json::json!({
            "ids": ids,
            "embeddings": embeddings,
            "metadatas": metadatas
        });

        let url = format!("{}/update", self.get_collection_url());
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {status}: {error_text}"
            )));
        }

        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let url = format!("{}/get", self.get_collection_url());
        let request = serde_json::json!({
            "ids": [id],
            "include": ["embeddings", "metadatas"]
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {status}: {error_text}"
            )));
        }

        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        // 解析响应并构建VectorData
        if let (Some(ids), Some(embeddings)) = (
            response_data["ids"].as_array(),
            response_data["embeddings"].as_array(),
        ) {
            if !ids.is_empty() && !embeddings.is_empty() {
                let vector: Vec<f32> = embeddings[0]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                let metadata = response_data["metadatas"]
                    .as_array()
                    .and_then(|metas| metas.first())
                    .and_then(|meta| serde_json::from_value(meta.clone()).ok())
                    .unwrap_or_default();

                return Ok(Some(VectorData {
                    id: id.to_string(),
                    vector,
                    metadata,
                }));
            }
        }

        Ok(None)
    }

    async fn count_vectors(&self) -> Result<usize> {
        let url = format!("{}/count", self.get_collection_url());
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Chroma API error {status}: {error_text}"
            )));
        }

        let count: usize = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        Ok(count)
    }

    async fn clear(&self) -> Result<()> {
        // 删除集合
        let url = format!(
            "{}/api/v1/collections/{}",
            self.base_url, self.collection_name
        );
        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Failed to delete collection {}: {} - {}",
                self.collection_name, status, error_text
            )));
        }

        // 重新创建集合
        self.create_collection().await
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
        self.default_health_check("Chroma").await
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        use crate::utils::VectorStoreDefaults;
        self.default_get_stats(1536).await // 默认维度
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

    #[tokio::test]
    async fn test_chroma_store_creation() {
        let config = VectorStoreConfig {
            provider: "chroma".to_string(),
            url: Some("http://localhost:8000".to_string()),
            collection_name: Some("test".to_string()),
            ..Default::default()
        };

        // 注意：这个测试需要运行中的Chroma服务器
        // 在CI环境中可能会失败，但在开发环境中很有用
        if std::env::var("CHROMA_TEST_ENABLED").is_ok() {
            let store = ChromaStore::new(config).await;
            assert!(store.is_ok());
        }
    }

    #[test]
    fn test_get_collection_url() {
        let config = VectorStoreConfig {
            provider: "chroma".to_string(),
            url: Some("http://localhost:8000".to_string()),
            collection_name: Some("test".to_string()),
            ..Default::default()
        };

        // 创建一个不会实际连接的store实例来测试URL生成
        let store = ChromaStore {
            config,
            client: Client::new(),
            base_url: "http://localhost:8000".to_string(),
            collection_name: "test".to_string(),
        };

        assert_eq!(
            store.get_collection_url(),
            "http://localhost:8000/api/v1/collections/test"
        );
    }

    #[tokio::test]
    async fn test_vector_operations_structure() {
        // 测试向量数据结构，不需要真实的Chroma服务器
        let config = VectorStoreConfig {
            provider: "chroma".to_string(),
            url: Some("http://localhost:8000".to_string()),
            collection_name: Some("test_structure".to_string()),
            ..Default::default()
        };

        let store = ChromaStore {
            config,
            client: Client::new(),
            base_url: "http://localhost:8000".to_string(),
            collection_name: "test_structure".to_string(),
        };

        // 测试向量数据结构
        let vector_data = VectorData {
            id: "test_id".to_string(),
            vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            metadata: {
                let mut map = HashMap::new();
                map.insert("content".to_string(), "test content".to_string());
                map.insert("type".to_string(), "episodic".to_string());
                map
            },
        };

        // 验证数据结构正确
        assert_eq!(vector_data.id, "test_id");
        assert_eq!(vector_data.vector.len(), 5);
        assert!(vector_data.metadata.contains_key("content"));
    }

    // 真实 Chroma 集成测试 (需要运行 Chroma 服务器)
    #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use super::*;
        use std::env;

        async fn create_real_chroma_store() -> Option<ChromaStore> {
            let chroma_url =
                env::var("CHROMA_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

            let config = VectorStoreConfig {
                provider: "chroma".to_string(),
                url: Some(chroma_url.clone()),
                collection_name: Some("test_integration".to_string()),
                ..Default::default()
            };

            match ChromaStore::new(config).await {
                Ok(store) => Some(store),
                Err(_) => {
                    println!(
                        "Skipping real Chroma test - Chroma server not available at {}",
                        chroma_url
                    );
                    None
                }
            }
        }

        #[tokio::test]
        async fn test_real_chroma_health_check() {
            let Some(store) = create_real_chroma_store().await else {
                return;
            };

            let health = store.health_check().await;
            assert!(
                health.is_ok(),
                "Chroma health check should succeed: {:?}",
                health
            );

            let health_status = health.unwrap();
            assert_eq!(health_status.status, "healthy");
            println!("Chroma Health: {:?}", health_status);
        }

        #[tokio::test]
        async fn test_real_chroma_vector_operations() {
            let Some(store) = create_real_chroma_store().await else {
                return;
            };

            // 测试数据
            let test_vectors = vec![
                VectorData {
                    id: "test_1".to_string(),
                    vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
                    metadata: {
                        let mut map = std::collections::HashMap::new();
                        map.insert("content".to_string(), "Test content 1".to_string());
                        map.insert("type".to_string(), "test".to_string());
                        map
                    },
                },
                VectorData {
                    id: "test_2".to_string(),
                    vector: vec![0.2, 0.3, 0.4, 0.5, 0.6],
                    metadata: {
                        let mut map = std::collections::HashMap::new();
                        map.insert("content".to_string(), "Test content 2".to_string());
                        map.insert("type".to_string(), "test".to_string());
                        map
                    },
                },
            ];

            // 添加向量
            let result = store.add_vectors(test_vectors.clone()).await;
            assert!(
                result.is_ok(),
                "Adding vectors should succeed: {:?}",
                result
            );
            let ids = result.unwrap();
            assert_eq!(ids.len(), 2);

            // 搜索向量
            let query_vector = vec![0.15, 0.25, 0.35, 0.45, 0.55];
            let search_result = store.search_vectors(query_vector, 10, None).await;
            assert!(
                search_result.is_ok(),
                "Vector search should succeed: {:?}",
                search_result
            );

            let results = search_result.unwrap();
            assert!(!results.is_empty(), "Search should return results");
            println!("Chroma search results: {:?}", results);

            // 清理测试数据
            let delete_result = store.delete_vectors(ids).await;
            assert!(
                delete_result.is_ok(),
                "Deleting vectors should succeed: {:?}",
                delete_result
            );
        }

        #[tokio::test]
        async fn test_real_chroma_collection_management() {
            let Some(store) = create_real_chroma_store().await else {
                return;
            };

            // 测试集合创建和管理
            let collection_info = store.get_collection_info().await;
            assert!(
                collection_info.is_ok(),
                "Getting collection info should succeed: {:?}",
                collection_info
            );

            let info = collection_info.unwrap();
            assert_eq!(info.name, "test_integration");
            println!("Chroma collection info: {:?}", info);
        }
    }
}
