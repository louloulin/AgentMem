//! Pinecone向量存储实现

use agent_mem_traits::{
    AgentMemError, Result, VectorData, VectorSearchResult, VectorStore, VectorStoreConfig,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Pinecone向量结构
#[derive(Debug, Serialize, Deserialize)]
struct PineconeVector {
    id: String,
    values: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Pinecone批量插入请求
#[derive(Debug, Serialize)]
struct PineconeUpsertRequest {
    vectors: Vec<PineconeVector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
}

/// Pinecone查询请求
#[derive(Debug, Serialize)]
struct PineconeQueryRequest {
    vector: Vec<f32>,
    #[serde(rename = "topK")]
    top_k: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(rename = "includeValues")]
    include_values: bool,
    #[serde(rename = "includeMetadata")]
    include_metadata: bool,
}

/// Pinecone查询响应
#[derive(Debug, Deserialize)]
struct PineconeQueryResponse {
    matches: Vec<PineconeMatch>,
}

/// Pinecone匹配结果
#[derive(Debug, Deserialize)]
struct PineconeMatch {
    id: String,
    score: f32,
    #[serde(default)]
    values: Vec<f32>,
    #[serde(default)]
    metadata: HashMap<String, serde_json::Value>,
}

/// Pinecone删除请求
#[derive(Debug, Serialize)]
struct PineconeDeleteRequest {
    ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
}

/// Pinecone索引统计响应
#[derive(Debug, Deserialize)]
struct PineconeStatsResponse {
    #[serde(rename = "totalVectorCount")]
    total_vector_count: usize,
    dimension: usize,
}

/// Pinecone向量存储实现
pub struct PineconeStore {
    config: VectorStoreConfig,
    client: Client,
    base_url: String,
    api_key: String,
    index_name: String,
    namespace: Option<String>,
}

impl PineconeStore {
    /// 创建新的Pinecone存储实例
    pub async fn new(config: VectorStoreConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| AgentMemError::config_error("Pinecone API key is required"))?;

        let index_name = config
            .index_name
            .clone()
            .ok_or_else(|| AgentMemError::config_error("Pinecone index name is required"))?;

        // 构建基础URL
        let base_url = if let Some(url) = &config.url {
            url.clone()
        } else {
            // 默认使用Pinecone的标准URL格式
            format!(
                "https://{}-{}.svc.{}.pinecone.io",
                index_name,
                "default", // 项目ID，实际使用时需要配置
                "us-east1-gcp"
            ) // 区域，实际使用时需要配置
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            config,
            client,
            base_url,
            api_key,
            index_name,
            namespace: None, // 可以通过配置设置
        })
    }

    /// 获取API请求头
    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Api-Key",
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers
    }

    /// 转换VectorData到PineconeVector
    fn to_pinecone_vector(&self, data: &VectorData) -> PineconeVector {
        let metadata = if data.metadata.is_empty() {
            None
        } else {
            Some(
                data.metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect(),
            )
        };

        PineconeVector {
            id: data.id.clone(),
            values: data.vector.clone(),
            metadata,
        }
    }

    /// 转换PineconeMatch到VectorSearchResult
    fn from_pinecone_match(&self, pinecone_match: PineconeMatch) -> VectorSearchResult {
        let metadata = pinecone_match
            .metadata
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
            id: pinecone_match.id,
            vector: pinecone_match.values,
            metadata,
            similarity: pinecone_match.score,
            distance: 1.0 - pinecone_match.score,
        }
    }
}

#[async_trait]
impl VectorStore for PineconeStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let pinecone_vectors: Vec<PineconeVector> =
            vectors.iter().map(|v| self.to_pinecone_vector(v)).collect();

        let request = PineconeUpsertRequest {
            vectors: pinecone_vectors,
            namespace: self.namespace.clone(),
        };

        let url = format!("{}/vectors/upsert", self.base_url);
        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
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
                "Pinecone API error {status}: {error_text}"
            )));
        }

        // Pinecone upsert成功后返回向量ID列表
        Ok(vectors.iter().map(|v| v.id.clone()).collect())
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let request = PineconeQueryRequest {
            vector: query_vector,
            top_k: limit,
            namespace: self.namespace.clone(),
            include_values: true,
            include_metadata: true,
        };

        let url = format!("{}/query", self.base_url);
        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
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
                "Pinecone API error {status}: {error_text}"
            )));
        }

        let query_response: PineconeQueryResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        let mut results: Vec<VectorSearchResult> = query_response
            .matches
            .into_iter()
            .map(|m| self.from_pinecone_match(m))
            .collect();

        // 应用阈值过滤
        if let Some(threshold) = threshold {
            results.retain(|r| r.similarity >= threshold);
        }

        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let request = PineconeDeleteRequest {
            ids,
            namespace: self.namespace.clone(),
        };

        let url = format!("{}/vectors/delete", self.base_url);
        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
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
                "Pinecone API error {status}: {error_text}"
            )));
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        // Pinecone使用upsert操作来更新向量
        self.add_vectors(vectors).await?;
        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        // Pinecone没有直接的get操作，我们使用fetch
        let url = format!("{}/vectors/fetch", self.base_url);
        let request = serde_json::json!({
            "ids": [id],
            "namespace": self.namespace
        });

        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
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
                "Pinecone API error {status}: {error_text}"
            )));
        }

        let fetch_response: serde_json::Value = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        if let Some(vectors) = fetch_response.get("vectors") {
            if let Some(vector_data) = vectors.get(id) {
                if let (Some(values), Some(metadata)) = (
                    vector_data.get("values").and_then(|v| v.as_array()),
                    vector_data.get("metadata").and_then(|m| m.as_object()),
                ) {
                    let vector: Vec<f32> = values
                        .iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect();

                    let metadata_map: HashMap<String, String> = metadata
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
                        vector,
                        metadata: metadata_map,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn count_vectors(&self) -> Result<usize> {
        let url = format!("{}/describe_index_stats", self.base_url);
        let response = self
            .client
            .post(&url)
            .headers(self.get_headers())
            .json(&serde_json::json!({}))
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
                "Pinecone API error {status}: {error_text}"
            )));
        }

        let stats: PineconeStatsResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse response: {e}"))
        })?;

        Ok(stats.total_vector_count)
    }

    async fn clear(&self) -> Result<()> {
        // Pinecone没有直接的清空操作，需要删除所有向量
        // 这里返回错误，建议用户手动删除索引
        Err(AgentMemError::storage_error(
            "Pinecone does not support clear operation. Please delete the index manually.",
        ))
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
        self.default_health_check("Pinecone").await
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

    #[test]
    fn test_pinecone_store_creation_no_api_key() {
        let config = VectorStoreConfig {
            provider: "pinecone".to_string(),
            api_key: None,
            index_name: Some("test-index".to_string()),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(PineconeStore::new(config));
        assert!(result.is_err());
    }

    #[test]
    fn test_pinecone_store_creation_no_index() {
        let config = VectorStoreConfig {
            provider: "pinecone".to_string(),
            api_key: Some("test-key".to_string()),
            index_name: None,
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(PineconeStore::new(config));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pinecone_store_creation_success() {
        let config = VectorStoreConfig {
            provider: "pinecone".to_string(),
            api_key: Some("test-key".to_string()),
            index_name: Some("test-index".to_string()),
            url: Some("https://test-index-default.svc.us-east1-gcp.pinecone.io".to_string()),
            ..Default::default()
        };

        let result = PineconeStore::new(config).await;
        assert!(result.is_ok());

        let store = result.unwrap();
        assert_eq!(store.index_name, "test-index");
        assert_eq!(store.api_key, "test-key");
    }

    #[test]
    fn test_to_pinecone_vector() {
        let config = VectorStoreConfig {
            provider: "pinecone".to_string(),
            api_key: Some("test-key".to_string()),
            index_name: Some("test-index".to_string()),
            url: Some("https://test.pinecone.io".to_string()),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let store = rt.block_on(PineconeStore::new(config)).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());

        let vector_data = VectorData {
            id: "test-id".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata,
        };

        let pinecone_vector = store.to_pinecone_vector(&vector_data);
        assert_eq!(pinecone_vector.id, "test-id");
        assert_eq!(pinecone_vector.values, vec![1.0, 2.0, 3.0]);
        assert!(pinecone_vector.metadata.is_some());
    }

    #[test]
    fn test_from_pinecone_match() {
        let config = VectorStoreConfig {
            provider: "pinecone".to_string(),
            api_key: Some("test-key".to_string()),
            index_name: Some("test-index".to_string()),
            url: Some("https://test.pinecone.io".to_string()),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let store = rt.block_on(PineconeStore::new(config)).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert(
            "key1".to_string(),
            serde_json::Value::String("value1".to_string()),
        );

        let pinecone_match = PineconeMatch {
            id: "test-id".to_string(),
            score: 0.95,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        let result = store.from_pinecone_match(pinecone_match);
        assert_eq!(result.id, "test-id");
        assert_eq!(result.similarity, 0.95);
        assert_eq!(result.vector, vec![1.0, 2.0, 3.0]);
        assert_eq!(result.metadata.get("key1"), Some(&"value1".to_string()));
    }
}
