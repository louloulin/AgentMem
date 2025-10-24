//! Qdrant向量存储实现

use agent_mem_traits::{
    AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Qdrant点结构
#[derive(Debug, Serialize, Deserialize)]
struct QdrantPoint {
    id: QdrantPointId,
    vector: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<HashMap<String, serde_json::Value>>,
}

/// Qdrant点ID（支持字符串和数字）
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum QdrantPointId {
    String(String),
    Number(u64),
}

/// Qdrant批量插入请求
#[derive(Debug, Serialize)]
struct QdrantUpsertRequest {
    points: Vec<QdrantPoint>,
}

/// Qdrant搜索请求
#[derive(Debug, Serialize)]
struct QdrantSearchRequest {
    vector: Vec<f32>,
    limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    score_threshold: Option<f32>,
    with_payload: bool,
    with_vector: bool,
}

/// Qdrant搜索响应
#[derive(Debug, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantSearchResult>,
}

/// Qdrant搜索结果
#[derive(Debug, Deserialize)]
struct QdrantSearchResult {
    id: QdrantPointId,
    score: f32,
    #[serde(default)]
    vector: Vec<f32>,
    #[serde(default)]
    payload: HashMap<String, serde_json::Value>,
}

/// Qdrant删除请求
#[derive(Debug, Serialize)]
struct QdrantDeleteRequest {
    points: Vec<QdrantPointId>,
}

/// Qdrant集合信息响应
#[derive(Debug, Deserialize)]
struct QdrantCollectionInfo {
    result: QdrantCollectionResult,
}

/// Qdrant集合结果
#[derive(Debug, Deserialize)]
struct QdrantCollectionResult {
    points_count: usize,
    vectors_count: usize,
}

/// Qdrant向量存储实现
pub struct QdrantStore {
    config: VectorStoreConfig,
    client: Client,
    base_url: String,
    collection_name: String,
}

impl QdrantStore {
    /// 创建新的Qdrant存储实例
    pub async fn new(config: VectorStoreConfig) -> Result<Self> {
        let base_url = config
            .url
            .clone()
            .unwrap_or_else(|| "http://localhost:6333".to_string());

        let collection_name = config
            .collection_name
            .clone()
            .or_else(|| Some(config.table_name.clone()))
            .unwrap_or_else(|| "memories".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let store = Self {
            config,
            client,
            base_url,
            collection_name,
        };

        // 尝试创建集合（如果不存在）
        store.ensure_collection().await?;

        Ok(store)
    }

    /// 确保集合存在
    async fn ensure_collection(&self) -> Result<()> {
        // 检查集合是否存在
        let url = format!("{}/collections/{}", self.base_url, self.collection_name);
        let response = self.client.get(&url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                // 集合已存在
                Ok(())
            }
            _ => {
                // 集合不存在，创建它
                self.create_collection().await
            }
        }
    }

    /// 创建集合
    async fn create_collection(&self) -> Result<()> {
        let dimension = self.config.dimension.unwrap_or(1536);

        let create_request = serde_json::json!({
            "vectors": {
                "size": dimension,
                "distance": "Cosine"
            }
        });

        let url = format!("{}/collections/{}", self.base_url, self.collection_name);
        let response = self
            .client
            .put(&url)
            .header("Content-Type", "application/json")
            .json(&create_request)
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
                "Qdrant API error {status}: {error_text}"
            )));
        }

        Ok(())
    }

    /// 转换VectorData到QdrantPoint
    fn to_qdrant_point(&self, data: &VectorData) -> QdrantPoint {
        let payload = if data.metadata.is_empty() {
            None
        } else {
            Some(
                data.metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect(),
            )
        };

        QdrantPoint {
            id: QdrantPointId::String(data.id.clone()),
            vector: data.vector.clone(),
            payload,
        }
    }

    /// 转换QdrantSearchResult到VectorSearchResult
    fn from_qdrant_result(&self, qdrant_result: QdrantSearchResult) -> VectorSearchResult {
        let id = match qdrant_result.id {
            QdrantPointId::String(s) => s,
            QdrantPointId::Number(n) => n.to_string(),
        };

        let metadata = qdrant_result
            .payload
            .iter()
            .filter_map(|(k, v)| {
                if let serde_json::Value::String(s) = v {
                    Some((k.clone(), s.clone()))
                } else {
                    Some((k.clone(), v.to_string()))
                }
            })
            .collect();

        VectorSearchResult {
            id,
            vector: qdrant_result.vector,
            metadata,
            similarity: qdrant_result.score,
            distance: 1.0 - qdrant_result.score,
        }
    }
}

#[async_trait]
impl VectorStore for QdrantStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let qdrant_points: Vec<QdrantPoint> =
            vectors.iter().map(|v| self.to_qdrant_point(v)).collect();

        let request = QdrantUpsertRequest {
            points: qdrant_points,
        };

        let url = format!(
            "{}/collections/{}/points",
            self.base_url, self.collection_name
        );
        let response = self
            .client
            .put(&url)
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
                "Qdrant API error {status}: {error_text}"
            )));
        }

        // Qdrant upsert成功后返回向量ID列表
        Ok(vectors.iter().map(|v| v.id.clone()).collect())
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let request = QdrantSearchRequest {
            vector: query_vector,
            limit,
            score_threshold: threshold,
            with_payload: true,
            with_vector: true,
        };

        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, self.collection_name
        );
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
                "Qdrant API error {status}: {error_text}"
            )));
        }

        let search_response: QdrantSearchResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        let results: Vec<VectorSearchResult> = search_response
            .result
            .into_iter()
            .map(|r| self.from_qdrant_result(r))
            .collect();

        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let qdrant_ids: Vec<QdrantPointId> = ids.into_iter().map(QdrantPointId::String).collect();

        let request = QdrantDeleteRequest { points: qdrant_ids };

        let url = format!(
            "{}/collections/{}/points/delete",
            self.base_url, self.collection_name
        );
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
                "Qdrant API error {status}: {error_text}"
            )));
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        // Qdrant使用upsert操作来更新向量
        self.add_vectors(vectors).await?;
        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let url = format!(
            "{}/collections/{}/points/{}",
            self.base_url, self.collection_name, id
        );
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Qdrant API error {status}: {error_text}"
            )));
        }

        let point_response: serde_json::Value = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        if let Some(result) = point_response.get("result") {
            if let (Some(vector), Some(payload)) = (
                result.get("vector").and_then(|v| v.as_array()),
                result.get("payload").and_then(|p| p.as_object()),
            ) {
                let vector_data: Vec<f32> = vector
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                let metadata: HashMap<String, String> = payload
                    .iter()
                    .filter_map(|(k, v)| {
                        if let serde_json::Value::String(s) = v {
                            Some((k.clone(), s.clone()))
                        } else {
                            Some((k.clone(), v.to_string()))
                        }
                    })
                    .collect();

                return Ok(Some(VectorData {
                    id: id.to_string(),
                    vector: vector_data,
                    metadata,
                }));
            }
        }

        Ok(None)
    }

    async fn count_vectors(&self) -> Result<usize> {
        let url = format!("{}/collections/{}", self.base_url, self.collection_name);
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
                "Qdrant API error {status}: {error_text}"
            )));
        }

        let info: QdrantCollectionInfo = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        Ok(info.result.points_count)
    }

    async fn clear(&self) -> Result<()> {
        // 删除并重新创建集合
        let url = format!("{}/collections/{}", self.base_url, self.collection_name);
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
                "Qdrant API error {status}: {error_text}"
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
        self.default_health_check("Qdrant").await
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
