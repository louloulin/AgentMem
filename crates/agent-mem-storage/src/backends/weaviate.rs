//! Weaviate vector database backend implementation
//!
//! Provides integration with Weaviate vector database for storing
//! and retrieving memory embeddings with semantic search capabilities.

use agent_mem_traits::{AgentMemError, EmbeddingVectorStore, Result, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Weaviate configuration
#[derive(Debug, Clone)]
pub struct WeaviateConfig {
    /// Weaviate server URL
    pub url: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Class name for storing memories
    pub class_name: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for WeaviateConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".to_string(),
            api_key: None,
            class_name: "Memory".to_string(),
            timeout_seconds: 30,
        }
    }
}

/// Weaviate object structure
#[derive(Debug, Serialize, Deserialize)]
struct WeaviateObject {
    id: Option<String>,
    class: String,
    properties: HashMap<String, serde_json::Value>,
    vector: Option<Vec<f32>>,
}

/// Weaviate query structure
#[derive(Debug, Serialize)]
struct WeaviateQuery {
    query: String,
}

/// Weaviate search response
#[derive(Debug, Deserialize)]
struct WeaviateSearchResponse {
    data: WeaviateSearchData,
}

/// Weaviate search data
#[derive(Debug, Deserialize)]
struct WeaviateSearchData {
    #[serde(rename = "Get")]
    get: HashMap<String, Vec<WeaviateSearchResult>>,
}

/// Weaviate search result
#[derive(Debug, Deserialize)]
struct WeaviateSearchResult {
    id: Option<String>,
    content: Option<String>,
    memory_id: Option<String>,
    agent_id: Option<String>,
    user_id: Option<String>,
    #[serde(rename = "_additional")]
    additional: Option<WeaviateAdditional>,
}

/// Weaviate additional metadata
#[derive(Debug, Deserialize)]
struct WeaviateAdditional {
    distance: Option<f32>,
    certainty: Option<f32>,
}

/// Weaviate error response
#[derive(Debug, Deserialize)]
struct WeaviateError {
    error: Vec<WeaviateErrorDetail>,
}

/// Weaviate error detail
#[derive(Debug, Deserialize)]
struct WeaviateErrorDetail {
    message: String,
}

/// Weaviate vector store implementation
pub struct WeaviateStore {
    config: WeaviateConfig,
    client: Client,
}

impl WeaviateStore {
    /// Create a new Weaviate store
    pub fn new(config: WeaviateConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self { config, client })
    }

    /// Initialize the Weaviate schema
    pub async fn initialize_schema(&self) -> Result<()> {
        info!(
            "Initializing Weaviate schema for class: {}",
            self.config.class_name
        );

        let schema = serde_json::json!({
            "class": self.config.class_name,
            "description": "AgentMem memory storage",
            "vectorizer": "none",
            "properties": [
                {
                    "name": "content",
                    "dataType": ["text"],
                    "description": "Memory content"
                },
                {
                    "name": "memory_id",
                    "dataType": ["string"],
                    "description": "Memory ID"
                },
                {
                    "name": "agent_id",
                    "dataType": ["string"],
                    "description": "Agent ID"
                },
                {
                    "name": "user_id",
                    "dataType": ["string"],
                    "description": "User ID"
                },
                {
                    "name": "created_at",
                    "dataType": ["int"],
                    "description": "Creation timestamp"
                }
            ]
        });

        let mut request = self
            .client
            .post(format!("{}/v1/schema", self.config.url))
            .header("Content-Type", "application/json")
            .json(&schema);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = request.send().await.map_err(|e| {
            AgentMemError::network_error(format!("Failed to create schema: {e}"))
        })?;

        if response.status().is_success() {
            info!("Weaviate schema initialized successfully");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Schema might already exist, which is OK
            if status.as_u16() == 422 {
                warn!("Schema already exists, continuing...");
                Ok(())
            } else {
                error!("Failed to initialize schema: {} - {}", status, error_text);
                Err(AgentMemError::storage_error(format!(
                    "Schema initialization failed: {error_text}"
                )))
            }
        }
    }

    /// Build request with authentication
    fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.request(method, url);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }

        request
    }
}

#[async_trait]
impl EmbeddingVectorStore for WeaviateStore {
    async fn store_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &HashMap<String, String>,
    ) -> Result<()> {
        debug!("Storing embedding for memory: {}", memory_id);

        let mut properties = HashMap::new();
        properties.insert(
            "memory_id".to_string(),
            serde_json::Value::String(memory_id.to_string()),
        );

        // Add metadata as properties
        for (key, value) in metadata {
            properties.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        let object = WeaviateObject {
            id: Some(memory_id.to_string()),
            class: self.config.class_name.clone(),
            properties,
            vector: Some(embedding.to_vec()),
        };

        let response = self
            .build_request(
                reqwest::Method::POST,
                &format!("{}/v1/objects", self.config.url),
            )
            .header("Content-Type", "application/json")
            .json(&object)
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

        let query = serde_json::json!({
            "query": format!(
                "{{
                    Get {{
                        {} (
                            nearVector: {{
                                vector: {:?}
                                certainty: {}
                            }}
                            limit: {}
                        ) {{
                            memory_id
                            content
                            agent_id
                            user_id
                            _additional {{
                                distance
                                certainty
                            }}
                        }}
                    }}
                }}",
                self.config.class_name,
                query_embedding,
                threshold.unwrap_or(0.7),
                limit
            )
        });

        let response = self
            .build_request(
                reqwest::Method::POST,
                &format!("{}/v1/graphql", self.config.url),
            )
            .header("Content-Type", "application/json")
            .json(&query)
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

        let search_response: WeaviateSearchResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse search response: {e}"))
        })?;

        let results: Vec<SearchResult> = search_response
            .data
            .get
            .get(&self.config.class_name)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|result| {
                let memory_id = result.memory_id.as_ref()?;
                let score = result
                    .additional
                    .as_ref()
                    .and_then(|a| a.certainty)
                    .unwrap_or(0.0);

                Some(SearchResult {
                    id: memory_id.clone(),
                    score,
                    metadata: {
                        let mut meta = HashMap::new();
                        if let Some(content) = &result.content {
                            meta.insert(
                                "content".to_string(),
                                serde_json::Value::String(content.clone()),
                            );
                        }
                        if let Some(agent_id) = &result.agent_id {
                            meta.insert(
                                "agent_id".to_string(),
                                serde_json::Value::String(agent_id.clone()),
                            );
                        }
                        if let Some(user_id) = &result.user_id {
                            meta.insert(
                                "user_id".to_string(),
                                serde_json::Value::String(user_id.clone()),
                            );
                        }
                        meta
                    },
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
                &format!("{}/v1/objects/{}", self.config.url, memory_id),
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

        // Delete existing and create new (Weaviate doesn't have direct update for vectors)
        self.delete_embedding(memory_id).await?;
        self.store_embedding(memory_id, embedding, metadata).await
    }

    async fn get_embedding(&self, memory_id: &str) -> Result<Option<Vec<f32>>> {
        debug!("Getting embedding for memory: {}", memory_id);

        let response = self
            .build_request(
                reqwest::Method::GET,
                &format!("{}/v1/objects/{}", self.config.url, memory_id),
            )
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to get embedding: {e}"))
            })?;

        if response.status().is_success() {
            let object: WeaviateObject = response.json().await.map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse object: {e}"))
            })?;

            Ok(object.vector)
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

        let query = serde_json::json!({
            "query": format!(
                "{{
                    Get {{
                        {} {{
                            memory_id
                        }}
                    }}
                }}",
                self.config.class_name
            )
        });

        let response = self
            .build_request(
                reqwest::Method::POST,
                &format!("{}/v1/graphql", self.config.url),
            )
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to list embeddings: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to list embeddings: {} - {}", status, error_text);
            return Err(AgentMemError::storage_error(format!(
                "Failed to list embeddings: {error_text}"
            )));
        }

        let search_response: WeaviateSearchResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse list response: {e}"))
        })?;

        let mut memory_ids: Vec<String> = search_response
            .data
            .get
            .get(&self.config.class_name)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|result| result.memory_id.clone())
            .collect();

        // Apply prefix filter if specified
        if let Some(prefix) = prefix {
            memory_ids.retain(|id| id.starts_with(prefix));
        }

        debug!("Found {} embeddings", memory_ids.len());
        Ok(memory_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> WeaviateConfig {
        WeaviateConfig {
            url: "http://localhost:8080".to_string(),
            api_key: None,
            class_name: "TestMemory".to_string(),
            timeout_seconds: 30,
        }
    }

    #[test]
    fn test_weaviate_store_creation() {
        let config = create_test_config();
        let store = WeaviateStore::new(config);
        assert!(store.is_ok());
    }

    #[test]
    fn test_config_default() {
        let config = WeaviateConfig::default();
        assert_eq!(config.url, "http://localhost:8080");
        assert_eq!(config.class_name, "Memory");
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.api_key.is_none());
    }
}
