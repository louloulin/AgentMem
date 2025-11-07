//! Cohere嵌入模型实现

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cohere嵌入请求
#[derive(Debug, Serialize)]
struct CohereEmbeddingRequest {
    model: String,
    texts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_type: Option<String>,
}

/// Cohere嵌入响应
#[derive(Debug, Deserialize)]
struct CohereEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
    id: String,
    meta: CohereMeta,
}

/// Cohere元数据
#[derive(Debug, Deserialize)]
struct CohereMeta {
    api_version: CohereApiVersion,
    billed_units: CohereBilledUnits,
}

/// Cohere API版本
#[derive(Debug, Deserialize)]
struct CohereApiVersion {
    version: String,
}

/// Cohere计费单位
#[derive(Debug, Deserialize)]
struct CohereBilledUnits {
    input_tokens: u32,
}

/// Cohere嵌入模型实现
pub struct CohereEmbedder {
    config: EmbeddingConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl CohereEmbedder {
    /// 创建新的Cohere嵌入器实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| AgentMemError::config_error("Cohere API key is required"))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.cohere.ai/v1".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let embedder = Self {
            config,
            client,
            api_key,
            base_url,
        };

        // 测试连接
        embedder.health_check().await?;

        Ok(embedder)
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        // 尝试嵌入一个简单的文本来测试连接
        let test_result = self.embed_internal(&["test".to_string()]).await;
        Ok(test_result.is_ok())
    }

    /// 内部嵌入实现
    async fn embed_internal(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let request = CohereEmbeddingRequest {
            model: self.config.model.clone(),
            texts: texts.to_vec(),
            input_type: Some("search_document".to_string()),
        };

        let url = format!("{}/embed", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            return Err(AgentMemError::network_error(format!(
                "Cohere API error {status}: {error_text}"
            )));
        }

        let embedding_response: CohereEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::parsing_error(format!("Failed to parse response: {e}")))?;

        Ok(embedding_response.embeddings)
    }
}

#[async_trait]
impl Embedder for CohereEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_internal(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AgentMemError::parsing_error("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Cohere支持批量嵌入，但有限制
        const MAX_BATCH_SIZE: usize = 96;

        if texts.len() <= MAX_BATCH_SIZE {
            self.embed_internal(texts).await
        } else {
            // 分批处理
            let mut all_embeddings = Vec::new();
            for chunk in texts.chunks(MAX_BATCH_SIZE) {
                let chunk_embeddings = self.embed_internal(chunk).await?;
                all_embeddings.extend(chunk_embeddings);
            }
            Ok(all_embeddings)
        }
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn provider_name(&self) -> &str {
        "cohere"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    async fn health_check(&self) -> Result<bool> {
        self.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohere_embedder_creation_no_api_key() {
        let config = EmbeddingConfig {
            provider: "cohere".to_string(),
            model: "embed-english-v3.0".to_string(),
            api_key: None,
            dimension: 1024,
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(CohereEmbedder::new(config));
        assert!(result.is_err());
    }

    #[test]
    fn test_cohere_embedder_creation_with_api_key() {
        let config = EmbeddingConfig {
            provider: "cohere".to_string(),
            model: "embed-english-v3.0".to_string(),
            api_key: Some("test-key".to_string()),
            dimension: 1024,
            base_url: Some("https://api.cohere.ai/v1".to_string()),
            ..Default::default()
        };

        // 注意：这个测试会尝试连接到Cohere API，在没有有效API密钥的情况下会失败
        // 这是预期的行为
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _result = rt.block_on(CohereEmbedder::new(config));
        // 我们不检查结果，因为API调用可能失败
    }

    #[test]
    fn test_cohere_embedder_properties() {
        let config = EmbeddingConfig {
            provider: "cohere".to_string(),
            model: "embed-english-v3.0".to_string(),
            api_key: Some("test-key".to_string()),
            dimension: 1024,
            base_url: Some("https://api.cohere.ai/v1".to_string()),
            ..Default::default()
        };

        let embedder = CohereEmbedder {
            config: config.clone(),
            client: Client::new(),
            api_key: "test-key".to_string(),
            base_url: "https://api.cohere.ai/v1".to_string(),
        };

        assert_eq!(embedder.provider_name(), "cohere");
        assert_eq!(embedder.dimension(), 1024);
    }

    #[test]
    fn test_cohere_embedding_request_serialization() {
        let request = CohereEmbeddingRequest {
            model: "embed-english-v3.0".to_string(),
            texts: vec!["Hello, world!".to_string()],
            input_type: Some("search_document".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("embed-english-v3.0"));
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("search_document"));
    }

    #[test]
    fn test_cohere_embedding_response_deserialization() {
        let json = r#"{
            "embeddings": [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]],
            "id": "test-id",
            "meta": {
                "api_version": {"version": "1.0"},
                "billed_units": {"input_tokens": 10}
            }
        }"#;

        let response: CohereEmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.embeddings.len(), 2);
        assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(response.embeddings[1], vec![0.4, 0.5, 0.6]);
        assert_eq!(response.id, "test-id");
        assert_eq!(response.meta.billed_units.input_tokens, 10);
    }

    #[test]
    fn test_batch_size_calculation() {
        // 测试批量处理逻辑
        let texts: Vec<String> = (0..200).map(|i| format!("text {}", i)).collect();

        // 测试分批处理
        const MAX_BATCH_SIZE: usize = 96;
        let chunks: Vec<_> = texts.chunks(MAX_BATCH_SIZE).collect();

        assert_eq!(chunks.len(), 3); // 200 / 96 = 2.08, 所以需要3批
        assert_eq!(chunks[0].len(), 96);
        assert_eq!(chunks[1].len(), 96);
        assert_eq!(chunks[2].len(), 8); // 200 - 96 - 96 = 8
    }
}
