//! Milvus vector database backend implementation
//!
//! Provides integration with Milvus vector database for high-performance
//! vector similarity search and storage.

use agent_mem_traits::{AgentMemError, EmbeddingVectorStore, Result, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Milvus configuration
#[derive(Debug, Clone)]
pub struct MilvusConfig {
    /// Milvus server URL
    pub url: String,
    /// Database name
    pub database: String,
    /// Collection name for storing memories
    pub collection_name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Index type (e.g., "IVF_FLAT", "HNSW")
    pub index_type: String,
    /// Metric type (e.g., "L2", "IP", "COSINE")
    pub metric_type: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for MilvusConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:19530".to_string(),
            database: "default".to_string(),
            collection_name: "memory_collection".to_string(),
            dimension: 1536, // OpenAI embedding dimension
            index_type: "HNSW".to_string(),
            metric_type: "COSINE".to_string(),
            timeout_seconds: 30,
        }
    }
}

/// Milvus entity for insertion
#[derive(Debug, Serialize)]
struct MilvusEntity {
    memory_id: String,
    embedding: Vec<f32>,
    content: String,
    agent_id: String,
    user_id: String,
    created_at: i64,
}

/// Milvus search request
#[derive(Debug, Serialize)]
struct MilvusSearchRequest {
    collection_name: String,
    vectors: Vec<Vec<f32>>,
    limit: usize,
    metric_type: String,
    params: HashMap<String, serde_json::Value>,
    output_fields: Vec<String>,
}

/// Milvus search response
#[derive(Debug, Deserialize)]
struct MilvusSearchResponse {
    status: MilvusStatus,
    results: Vec<MilvusSearchResult>,
}

/// Milvus search result
#[derive(Debug, Deserialize)]
struct MilvusSearchResult {
    ids: Vec<String>,
    scores: Vec<f32>,
    fields_data: Vec<MilvusFieldData>,
}

/// Milvus field data
#[derive(Debug, Serialize, Deserialize)]
struct MilvusFieldData {
    field_name: String,
    #[serde(rename = "type")]
    field_type: i32,
    scalars: Option<MilvusScalars>,
    vectors: Option<MilvusVectorData>,
}

/// Milvus scalar data
#[derive(Debug, Serialize, Deserialize)]
struct MilvusScalars {
    string_data: Option<MilvusStringData>,
    long_data: Option<MilvusLongData>,
}

/// Milvus string data
#[derive(Debug, Serialize, Deserialize)]
struct MilvusStringData {
    data: Vec<String>,
}

/// Milvus long data
#[derive(Debug, Serialize, Deserialize)]
struct MilvusLongData {
    data: Vec<i64>,
}

/// Milvus status
#[derive(Debug, Deserialize)]
struct MilvusStatus {
    error_code: i32,
    reason: String,
}

/// Milvus query response
#[derive(Debug, Deserialize)]
struct MilvusQueryResponse {
    status: MilvusStatus,
    results: Vec<MilvusQueryResult>,
}

/// Milvus query result
#[derive(Debug, Deserialize)]
struct MilvusQueryResult {
    fields_data: Vec<MilvusFieldData>,
}

/// Milvus vector field data
#[derive(Debug, Serialize, Deserialize)]
struct MilvusVectorData {
    data: Vec<Vec<f32>>,
}

/// Milvus collection schema
#[derive(Debug, Serialize)]
struct MilvusCollectionSchema {
    name: String,
    description: String,
    fields: Vec<MilvusField>,
}

/// Milvus field definition
#[derive(Debug, Serialize)]
struct MilvusField {
    name: String,
    #[serde(rename = "type")]
    field_type: i32,
    is_primary_key: bool,
    dimension: Option<usize>,
}

/// Milvus vector store implementation
pub struct MilvusStore {
    config: MilvusConfig,
    client: Client,
}

impl MilvusStore {
    /// Create a new Milvus store
    pub fn new(config: MilvusConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self { config, client })
    }

    /// Initialize the Milvus collection
    pub async fn initialize_collection(&self) -> Result<()> {
        info!(
            "Initializing Milvus collection: {}",
            self.config.collection_name
        );

        // Check if collection exists
        let exists = self.collection_exists().await?;
        if exists {
            info!("Collection {} already exists", self.config.collection_name);
            return Ok(());
        }

        // Create collection schema
        let schema = MilvusCollectionSchema {
            name: self.config.collection_name.clone(),
            description: "AgentMem memory storage collection".to_string(),
            fields: vec![
                MilvusField {
                    name: "memory_id".to_string(),
                    field_type: 21, // VarChar
                    is_primary_key: true,
                    dimension: None,
                },
                MilvusField {
                    name: "embedding".to_string(),
                    field_type: 101, // FloatVector
                    is_primary_key: false,
                    dimension: Some(self.config.dimension),
                },
                MilvusField {
                    name: "content".to_string(),
                    field_type: 21, // VarChar
                    is_primary_key: false,
                    dimension: None,
                },
                MilvusField {
                    name: "agent_id".to_string(),
                    field_type: 21, // VarChar
                    is_primary_key: false,
                    dimension: None,
                },
                MilvusField {
                    name: "user_id".to_string(),
                    field_type: 21, // VarChar
                    is_primary_key: false,
                    dimension: None,
                },
                MilvusField {
                    name: "created_at".to_string(),
                    field_type: 5, // Int64
                    is_primary_key: false,
                    dimension: None,
                },
            ],
        };

        let create_request = serde_json::json!({
            "collection_name": self.config.collection_name,
            "schema": schema
        });

        let response = self
            .client
            .post(format!("{}/v1/collection", self.config.url))
            .header("Content-Type", "application/json")
            .json(&create_request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create collection: {e}"))
            })?;

        if response.status().is_success() {
            info!("Milvus collection created successfully");

            // Create index
            self.create_index().await?;

            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to create collection: {} - {}", status, error_text);
            Err(AgentMemError::storage_error(format!(
                "Collection creation failed: {error_text}"
            )))
        }
    }

    /// Check if collection exists
    async fn collection_exists(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!(
                "{}/v1/collection/{}",
                self.config.url, self.config.collection_name
            ))
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to check collection: {e}"))
            })?;

        Ok(response.status().is_success())
    }

    /// Create index for the collection
    async fn create_index(&self) -> Result<()> {
        info!(
            "Creating index for collection: {}",
            self.config.collection_name
        );

        let index_request = serde_json::json!({
            "collection_name": self.config.collection_name,
            "field_name": "embedding",
            "index_name": "embedding_index",
            "index_type": self.config.index_type,
            "metric_type": self.config.metric_type,
            "params": {
                "M": 16,
                "efConstruction": 200
            }
        });

        let response = self
            .client
            .post(format!("{}/v1/index", self.config.url))
            .header("Content-Type", "application/json")
            .json(&index_request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to create index: {e}")))?;

        if response.status().is_success() {
            info!("Index created successfully");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Failed to create index: {} - {}", status, error_text);
            // Index creation failure is not critical
            Ok(())
        }
    }
}

#[async_trait]
impl EmbeddingVectorStore for MilvusStore {
    async fn store_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &HashMap<String, String>,
    ) -> Result<()> {
        debug!("Storing embedding for memory: {}", memory_id);

        let entity = MilvusEntity {
            memory_id: memory_id.to_string(),
            embedding: embedding.to_vec(),
            content: metadata.get("content").unwrap_or(&String::new()).clone(),
            agent_id: metadata.get("agent_id").unwrap_or(&String::new()).clone(),
            user_id: metadata.get("user_id").unwrap_or(&String::new()).clone(),
            created_at: chrono::Utc::now().timestamp(),
        };

        let insert_request = serde_json::json!({
            "collection_name": self.config.collection_name,
            "fields_data": [
                {
                    "field_name": "memory_id",
                    "type": 21,
                    "scalars": {
                        "string_data": {
                            "data": [entity.memory_id]
                        }
                    }
                },
                {
                    "field_name": "embedding",
                    "type": 101,
                    "vectors": {
                        "float_vector": {
                            "data": entity.embedding
                        }
                    }
                },
                {
                    "field_name": "content",
                    "type": 21,
                    "scalars": {
                        "string_data": {
                            "data": [entity.content]
                        }
                    }
                },
                {
                    "field_name": "agent_id",
                    "type": 21,
                    "scalars": {
                        "string_data": {
                            "data": [entity.agent_id]
                        }
                    }
                },
                {
                    "field_name": "user_id",
                    "type": 21,
                    "scalars": {
                        "string_data": {
                            "data": [entity.user_id]
                        }
                    }
                },
                {
                    "field_name": "created_at",
                    "type": 5,
                    "scalars": {
                        "long_data": {
                            "data": [entity.created_at]
                        }
                    }
                }
            ]
        });

        let response = self
            .client
            .post(format!("{}/v1/entities", self.config.url))
            .header("Content-Type", "application/json")
            .json(&insert_request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to store embedding: {e}"))
            })?;

        if response.status().is_success() {
            debug!("Successfully stored embedding for memory: {}", memory_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to store embedding: {} - {}", status, error_text);
            Err(AgentMemError::storage_error(format!(
                "Failed to store embedding: {error_text}"
            )))
        }
    }

    async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<SearchResult>> {
        debug!("Searching for similar embeddings with limit: {}", limit);

        let search_request = MilvusSearchRequest {
            collection_name: self.config.collection_name.clone(),
            vectors: vec![query_embedding.to_vec()],
            limit,
            metric_type: self.config.metric_type.clone(),
            params: {
                let mut params = HashMap::new();
                params.insert(
                    "ef".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(64)),
                );
                params
            },
            output_fields: vec![
                "memory_id".to_string(),
                "content".to_string(),
                "agent_id".to_string(),
                "user_id".to_string(),
            ],
        };

        let response = self
            .client
            .post(format!("{}/v1/search", self.config.url))
            .header("Content-Type", "application/json")
            .json(&search_request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to search: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Search failed: {} - {}", status, error_text);
            return Err(AgentMemError::storage_error(format!(
                "Search failed: {error_text}"
            )));
        }

        let search_response: MilvusSearchResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse search response: {e}"))
        })?;

        if search_response.status.error_code != 0 {
            error!("Milvus search error: {}", search_response.status.reason);
            return Err(AgentMemError::storage_error(format!(
                "Milvus error: {}",
                search_response.status.reason
            )));
        }

        let mut results = Vec::new();

        for result in search_response.results {
            for (i, memory_id) in result.ids.iter().enumerate() {
                let score = result.scores.get(i).copied().unwrap_or(0.0);

                // Apply threshold filter
                if let Some(threshold) = threshold {
                    if score < threshold {
                        continue;
                    }
                }

                let mut metadata = HashMap::new();

                // Extract field data
                for field_data in &result.fields_data {
                    if let Some(scalars) = &field_data.scalars {
                        if let Some(string_data) = &scalars.string_data {
                            if let Some(value) = string_data.data.get(i) {
                                metadata.insert(
                                    field_data.field_name.clone(),
                                    serde_json::Value::String(value.clone()),
                                );
                            }
                        }
                    }
                }

                results.push(SearchResult {
                    id: memory_id.clone(),
                    score,
                    metadata,
                });
            }
        }

        debug!("Found {} similar embeddings", results.len());
        Ok(results)
    }

    async fn delete_embedding(&self, memory_id: &str) -> Result<()> {
        debug!("Deleting embedding for memory: {}", memory_id);

        let delete_request = serde_json::json!({
            "collection_name": self.config.collection_name,
            "expr": format!("memory_id == \"{}\"", memory_id)
        });

        let response = self
            .client
            .delete(format!("{}/v1/entities", self.config.url))
            .header("Content-Type", "application/json")
            .json(&delete_request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to delete embedding: {e}"))
            })?;

        if response.status().is_success() {
            debug!("Successfully deleted embedding for memory: {}", memory_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to delete embedding: {} - {}", status, error_text);
            Err(AgentMemError::storage_error(format!(
                "Failed to delete embedding: {error_text}"
            )))
        }
    }

    async fn update_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &HashMap<String, String>,
    ) -> Result<()> {
        debug!("Updating embedding for memory: {}", memory_id);

        // Delete existing and insert new (Milvus doesn't have direct update)
        self.delete_embedding(memory_id).await?;
        self.store_embedding(memory_id, embedding, metadata).await
    }

    async fn get_embedding(&self, memory_id: &str) -> Result<Option<Vec<f32>>> {
        debug!("Getting embedding for memory: {}", memory_id);

        let query_request = serde_json::json!({
            "collection_name": self.config.collection_name,
            "expr": format!("memory_id == \"{}\"", memory_id),
            "output_fields": ["embedding"]
        });

        let response = self
            .client
            .post(format!("{}/v1/query", self.config.url))
            .header("Content-Type", "application/json")
            .json(&query_request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to get embedding: {e}"))
            })?;

        if response.status().is_success() {
            // Parse response and extract embedding
            let query_response: MilvusQueryResponse = response.json().await.map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse query response: {e}"))
            })?;

            if query_response.status.error_code != 0 {
                return Err(AgentMemError::storage_error(format!(
                    "Milvus query failed: {}",
                    query_response.status.reason
                )));
            }

            // Extract embedding from results
            if let Some(result) = query_response.results.first() {
                for field_data in &result.fields_data {
                    if field_data.field_name == "embedding" {
                        // Check if this field contains vector data
                        if let Some(vectors) = &field_data.vectors {
                            if let Some(embedding) = vectors.data.first() {
                                return Ok(Some(embedding.clone()));
                            }
                        }
                    }
                }
            }

            Ok(None) // No embedding found
        } else if response.status().as_u16() == 404 {
            Ok(None)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to get embedding: {} - {}", status, error_text);
            Err(AgentMemError::storage_error(format!(
                "Failed to get embedding: {error_text}"
            )))
        }
    }

    async fn list_embeddings(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        debug!("Listing embeddings with prefix: {:?}", prefix);

        let expr = if let Some(prefix) = prefix {
            format!("memory_id like \"{prefix}%\"")
        } else {
            "memory_id != \"\"".to_string()
        };

        let query_request = serde_json::json!({
            "collection_name": self.config.collection_name,
            "expr": expr,
            "output_fields": ["memory_id"]
        });

        let response = self
            .client
            .post(format!("{}/v1/query", self.config.url))
            .header("Content-Type", "application/json")
            .json(&query_request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to list embeddings: {e}"))
            })?;

        if response.status().is_success() {
            // Parse response and extract memory IDs
            let query_response: MilvusQueryResponse = response.json().await.map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse list response: {e}"))
            })?;

            if query_response.status.error_code != 0 {
                return Err(AgentMemError::storage_error(format!(
                    "Milvus list query failed: {}",
                    query_response.status.reason
                )));
            }

            let mut memory_ids = Vec::new();

            // Extract memory IDs from results
            if let Some(result) = query_response.results.first() {
                for field_data in &result.fields_data {
                    if field_data.field_name == "memory_id" {
                        if let Some(scalars) = &field_data.scalars {
                            if let Some(string_data) = &scalars.string_data {
                                memory_ids.extend(string_data.data.clone());
                            }
                        }
                    }
                }
            }

            Ok(memory_ids)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to list embeddings: {} - {}", status, error_text);
            Err(AgentMemError::storage_error(format!(
                "Failed to list embeddings: {error_text}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> MilvusConfig {
        MilvusConfig {
            url: "http://localhost:19530".to_string(),
            database: "test_db".to_string(),
            collection_name: "test_collection".to_string(),
            dimension: 128,
            index_type: "HNSW".to_string(),
            metric_type: "COSINE".to_string(),
            timeout_seconds: 30,
        }
    }

    #[test]
    fn test_milvus_store_creation() {
        let config = create_test_config();
        let store = MilvusStore::new(config);
        assert!(store.is_ok());
    }

    #[test]
    fn test_config_default() {
        let config = MilvusConfig::default();
        assert_eq!(config.url, "http://localhost:19530");
        assert_eq!(config.collection_name, "memory_collection");
        assert_eq!(config.dimension, 1536);
        assert_eq!(config.index_type, "HNSW");
        assert_eq!(config.metric_type, "COSINE");
    }
}
