//! Elasticsearch vector database backend implementation
//!
//! Provides integration with Elasticsearch for vector similarity search
//! using dense vector fields and kNN search capabilities.

use agent_mem_traits::{AgentMemError, EmbeddingVectorStore, Result, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Elasticsearch configuration
#[derive(Debug, Clone)]
pub struct ElasticsearchConfig {
    /// Elasticsearch server URL
    pub url: String,
    /// Username for authentication
    pub username: Option<String>,
    /// Password for authentication
    pub password: Option<String>,
    /// Index name for storing memories
    pub index_name: String,
    /// Vector field name
    pub vector_field: String,
    /// Vector dimension
    pub dimension: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for ElasticsearchConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:9200".to_string(),
            username: None,
            password: None,
            index_name: "memory_index".to_string(),
            vector_field: "embedding".to_string(),
            dimension: 1536, // OpenAI embedding dimension
            timeout_seconds: 30,
        }
    }
}

/// Elasticsearch document for memory storage
#[derive(Debug, Serialize, Deserialize)]
struct ElasticsearchDocument {
    memory_id: String,
    embedding: Vec<f32>,
    content: String,
    agent_id: String,
    user_id: String,
    created_at: i64,
    #[serde(flatten)]
    metadata: HashMap<String, serde_json::Value>,
}

/// Elasticsearch search request
#[derive(Debug, Serialize)]
struct ElasticsearchSearchRequest {
    size: usize,
    query: ElasticsearchQuery,
    _source: Vec<String>,
}

/// Elasticsearch query structure
#[derive(Debug, Serialize)]
struct ElasticsearchQuery {
    knn: ElasticsearchKnnQuery,
}

/// Elasticsearch kNN query
#[derive(Debug, Serialize)]
struct ElasticsearchKnnQuery {
    field: String,
    query_vector: Vec<f32>,
    k: usize,
    num_candidates: usize,
}

/// Elasticsearch search response
#[derive(Debug, Deserialize)]
struct ElasticsearchSearchResponse {
    hits: ElasticsearchHits,
}

/// Elasticsearch hits container
#[derive(Debug, Deserialize)]
struct ElasticsearchHits {
    hits: Vec<ElasticsearchHit>,
}

/// Elasticsearch search hit
#[derive(Debug, Deserialize)]
struct ElasticsearchHit {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_score")]
    score: f32,
    #[serde(rename = "_source")]
    source: ElasticsearchDocument,
}

/// Elasticsearch error response
#[derive(Debug, Deserialize)]
struct ElasticsearchError {
    error: ElasticsearchErrorDetail,
}

/// Elasticsearch error detail
#[derive(Debug, Deserialize)]
struct ElasticsearchErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    reason: String,
}

/// Elasticsearch vector store implementation
pub struct ElasticsearchStore {
    config: ElasticsearchConfig,
    client: Client,
}

impl ElasticsearchStore {
    /// Create a new Elasticsearch store
    pub fn new(config: ElasticsearchConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self { config, client })
    }

    /// Initialize the Elasticsearch index
    pub async fn initialize_index(&self) -> Result<()> {
        info!(
            "Initializing Elasticsearch index: {}",
            self.config.index_name
        );

        // Check if index exists
        let exists = self.index_exists().await?;
        if exists {
            info!("Index {} already exists", self.config.index_name);
            return Ok(());
        }

        // Create index mapping
        let mapping = serde_json::json!({
            "mappings": {
                "properties": {
                    "memory_id": {
                        "type": "keyword"
                    },
                    self.config.vector_field.clone(): {
                        "type": "dense_vector",
                        "dims": self.config.dimension,
                        "index": true,
                        "similarity": "cosine"
                    },
                    "content": {
                        "type": "text"
                    },
                    "agent_id": {
                        "type": "keyword"
                    },
                    "user_id": {
                        "type": "keyword"
                    },
                    "created_at": {
                        "type": "long"
                    }
                }
            },
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0
            }
        });

        let response = self
            .build_request(
                reqwest::Method::PUT,
                &format!("{}/{}", self.config.url, self.config.index_name),
            )
            .header("Content-Type", "application/json")
            .json(&mapping)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to create index: {e}")))?;

        if response.status().is_success() {
            info!("Elasticsearch index created successfully");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to create index: {} - {}", status, error_text);
            Err(AgentMemError::storage_error(format!(
                "Index creation failed: {error_text}"
            )))
        }
    }

    /// Check if index exists
    async fn index_exists(&self) -> Result<bool> {
        let response = self
            .build_request(
                reqwest::Method::HEAD,
                &format!("{}/{}", self.config.url, self.config.index_name),
            )
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to check index: {e}")))?;

        Ok(response.status().is_success())
    }

    /// Build request with authentication
    fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.request(method, url);

        if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
            request = request.basic_auth(username, Some(password));
        }

        request
    }
}

#[async_trait]
impl EmbeddingVectorStore for ElasticsearchStore {
    async fn store_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &HashMap<String, String>,
    ) -> Result<()> {
        debug!("Storing embedding for memory: {}", memory_id);

        let mut doc_metadata = HashMap::new();
        for (key, value) in metadata {
            if !["memory_id", "content", "agent_id", "user_id"].contains(&key.as_str()) {
                doc_metadata.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }

        let document = ElasticsearchDocument {
            memory_id: memory_id.to_string(),
            embedding: embedding.to_vec(),
            content: metadata.get("content").unwrap_or(&String::new()).clone(),
            agent_id: metadata.get("agent_id").unwrap_or(&String::new()).clone(),
            user_id: metadata.get("user_id").unwrap_or(&String::new()).clone(),
            created_at: chrono::Utc::now().timestamp(),
            metadata: doc_metadata,
        };

        let response = self
            .build_request(
                reqwest::Method::PUT,
                &format!(
                    "{}/{}/_doc/{}",
                    self.config.url, self.config.index_name, memory_id
                ),
            )
            .header("Content-Type", "application/json")
            .json(&document)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to store embedding: {e}")))?;

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

        let search_request = ElasticsearchSearchRequest {
            size: limit,
            query: ElasticsearchQuery {
                knn: ElasticsearchKnnQuery {
                    field: self.config.vector_field.clone(),
                    query_vector: query_embedding.to_vec(),
                    k: limit,
                    num_candidates: limit * 10, // Use more candidates for better recall
                },
            },
            _source: vec![
                "memory_id".to_string(),
                "content".to_string(),
                "agent_id".to_string(),
                "user_id".to_string(),
            ],
        };

        let response = self
            .build_request(
                reqwest::Method::POST,
                &format!("{}/{}/_search", self.config.url, self.config.index_name),
            )
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

        let search_response: ElasticsearchSearchResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse search response: {e}"))
        })?;

        let results: Vec<SearchResult> = search_response
            .hits
            .hits
            .into_iter()
            .filter_map(|hit| {
                // Apply threshold filter
                if let Some(threshold) = threshold {
                    if hit.score < threshold {
                        return None;
                    }
                }

                let mut metadata = HashMap::new();
                metadata.insert(
                    "content".to_string(),
                    serde_json::Value::String(hit.source.content),
                );
                metadata.insert(
                    "agent_id".to_string(),
                    serde_json::Value::String(hit.source.agent_id),
                );
                metadata.insert(
                    "user_id".to_string(),
                    serde_json::Value::String(hit.source.user_id),
                );

                Some(SearchResult {
                    id: hit.source.memory_id,
                    score: hit.score,
                    metadata,
                })
            })
            .collect();

        debug!("Found {} similar embeddings", results.len());
        Ok(results)
    }

    async fn delete_embedding(&self, memory_id: &str) -> Result<()> {
        debug!("Deleting embedding for memory: {}", memory_id);

        let response = self
            .build_request(
                reqwest::Method::DELETE,
                &format!(
                    "{}/{}/_doc/{}",
                    self.config.url, self.config.index_name, memory_id
                ),
            )
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

        // Elasticsearch PUT will update or create
        self.store_embedding(memory_id, embedding, metadata).await
    }

    async fn get_embedding(&self, memory_id: &str) -> Result<Option<Vec<f32>>> {
        debug!("Getting embedding for memory: {}", memory_id);

        let response = self
            .build_request(
                reqwest::Method::GET,
                &format!(
                    "{}/{}/_doc/{}",
                    self.config.url, self.config.index_name, memory_id
                ),
            )
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to get embedding: {e}")))?;

        if response.status().is_success() {
            let doc_response: serde_json::Value = response.json().await.map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse document: {e}"))
            })?;

            if let Some(source) = doc_response.get("_source") {
                if let Some(embedding) = source.get(&self.config.vector_field) {
                    if let Some(embedding_array) = embedding.as_array() {
                        let embedding_vec: std::result::Result<Vec<f32>, &str> = embedding_array
                            .iter()
                            .map(|v| {
                                v.as_f64()
                                    .map(|f| f as f32)
                                    .ok_or("Invalid embedding value")
                            })
                            .collect();

                        match embedding_vec {
                            Ok(vec) => return Ok(Some(vec)),
                            Err(e) => {
                                return Err(AgentMemError::parsing_error(format!(
                                    "Failed to parse embedding: {e}"
                                )))
                            }
                        }
                    }
                }
            }

            Ok(None)
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

        let query = if let Some(prefix) = prefix {
            serde_json::json!({
                "query": {
                    "prefix": {
                        "memory_id": prefix
                    }
                },
                "_source": ["memory_id"],
                "size": 10000
            })
        } else {
            serde_json::json!({
                "query": {
                    "match_all": {}
                },
                "_source": ["memory_id"],
                "size": 10000
            })
        };

        let response = self
            .build_request(
                reqwest::Method::POST,
                &format!("{}/{}/_search", self.config.url, self.config.index_name),
            )
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to list embeddings: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to list embeddings: {} - {}", status, error_text);
            return Err(AgentMemError::storage_error(format!(
                "Failed to list embeddings: {error_text}"
            )));
        }

        let search_response: ElasticsearchSearchResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse list response: {e}"))
        })?;

        let memory_ids: Vec<String> = search_response
            .hits
            .hits
            .into_iter()
            .map(|hit| hit.source.memory_id)
            .collect();

        debug!("Found {} embeddings", memory_ids.len());
        Ok(memory_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ElasticsearchConfig {
        ElasticsearchConfig {
            url: "http://localhost:9200".to_string(),
            username: None,
            password: None,
            index_name: "test_memory_index".to_string(),
            vector_field: "embedding".to_string(),
            dimension: 128,
            timeout_seconds: 30,
        }
    }

    #[test]
    fn test_elasticsearch_store_creation() {
        let config = create_test_config();
        let store = ElasticsearchStore::new(config);
        assert!(store.is_ok());
    }

    #[test]
    fn test_config_default() {
        let config = ElasticsearchConfig::default();
        assert_eq!(config.url, "http://localhost:9200");
        assert_eq!(config.index_name, "memory_index");
        assert_eq!(config.vector_field, "embedding");
        assert_eq!(config.dimension, 1536);
    }
}
