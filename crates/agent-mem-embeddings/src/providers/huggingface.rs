//! HuggingFace嵌入提供商实现

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// HuggingFace API 请求结构
#[derive(Debug, Serialize)]
struct HuggingFaceRequest {
    inputs: Vec<String>,
    options: HuggingFaceOptions,
}

/// HuggingFace API 选项
#[derive(Debug, Serialize)]
struct HuggingFaceOptions {
    wait_for_model: bool,
    use_cache: bool,
}

/// HuggingFace API 响应结构
#[derive(Debug, Deserialize)]
struct HuggingFaceResponse(Vec<Vec<f32>>);

/// HuggingFace嵌入提供商
pub struct HuggingFaceEmbedder {
    config: EmbeddingConfig,
    client: Client,
    api_url: String,
}

impl HuggingFaceEmbedder {
    /// 创建新的HuggingFace嵌入器实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        // 验证模型名称
        if config.model.is_empty() {
            return Err(AgentMemError::config_error(
                "HuggingFace model name is required",
            ));
        }

        // 验证 API key（如果需要）
        if config.api_key.is_none() {
            warn!("No HuggingFace API key provided, using public inference API (may have rate limits)");
        }

        let api_url = format!(
            "https://api-inference.huggingface.co/pipeline/feature-extraction/{}",
            config.model
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        info!(
            "Initialized HuggingFace embedder with model: {}",
            config.model
        );

        Ok(Self {
            config,
            client,
            api_url,
        })
    }

    /// 生成真实的嵌入向量
    async fn generate_real_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = HuggingFaceRequest {
            inputs: vec![text.to_string()],
            options: HuggingFaceOptions {
                wait_for_model: true,
                use_cache: true,
            },
        };

        debug!(
            "Sending embedding request to HuggingFace API for text length: {}",
            text.len()
        );

        let mut request_builder = self
            .client
            .post(&self.api_url)
            .header("Content-Type", "application/json")
            .json(&request);

        // 如果有 API key，添加认证头
        if let Some(api_key) = &self.config.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = request_builder.send().await.map_err(|e| {
            AgentMemError::network_error(format!("HuggingFace API request failed: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("HuggingFace API error {}: {}", status, error_text);
            return Err(AgentMemError::network_error(format!(
                "HuggingFace API error {status}: {error_text}"
            )));
        }

        let hf_response: HuggingFaceResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse HuggingFace response: {e}"))
        })?;

        if hf_response.0.is_empty() {
            return Err(AgentMemError::embedding_error(
                "Empty response from HuggingFace API",
            ));
        }

        let embedding = hf_response.0.into_iter().next().ok_or_else(|| {
            AgentMemError::embedding_error("No embedding in HuggingFace response")
        })?;

        // 验证嵌入维度
        if embedding.len() != self.config.dimension {
            warn!(
                "Expected dimension {}, got {}. Adjusting...",
                self.config.dimension,
                embedding.len()
            );

            // 如果维度不匹配，进行调整
            let mut adjusted_embedding = embedding;
            adjusted_embedding.resize(self.config.dimension, 0.0);
            return Ok(adjusted_embedding);
        }

        debug!("Generated embedding with dimension: {}", embedding.len());
        Ok(embedding)
    }
}

#[async_trait]
impl Embedder for HuggingFaceEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // 使用真实的 HuggingFace API
        self.generate_real_embedding(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        // 简单的顺序处理，实际实现应该支持批量处理
        for text in texts {
            let embedding = self.embed(text).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn provider_name(&self) -> &str {
        "huggingface"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    async fn health_check(&self) -> Result<bool> {
        // 真实的健康检查：尝试生成一个简单的嵌入
        match self.generate_real_embedding("health check").await {
            Ok(_) => {
                debug!("HuggingFace health check passed");
                Ok(true)
            }
            Err(e) => {
                warn!("HuggingFace health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_huggingface_embedder_creation() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let result = HuggingFaceEmbedder::new(config).await;
        assert!(result.is_ok());

        let embedder = result.unwrap();
        assert_eq!(embedder.provider_name(), "huggingface");
        assert_eq!(
            embedder.model_name(),
            "sentence-transformers/all-MiniLM-L6-v2"
        );
        assert_eq!(embedder.dimension(), 768);
    }

    #[tokio::test]
    async fn test_huggingface_embedder_empty_model() {
        let config = EmbeddingConfig {
            provider: "huggingface".to_string(),
            model: "".to_string(),
            ..Default::default()
        };

        let result = HuggingFaceEmbedder::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires HuggingFace API access
    async fn test_embed_single_text() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let embedder = HuggingFaceEmbedder::new(config).await.unwrap();

        let result = embedder.embed("test text").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    #[ignore] // Requires HuggingFace API access
    async fn test_embed_batch() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let embedder = HuggingFaceEmbedder::new(config).await.unwrap();

        let texts = vec![
            "first text".to_string(),
            "second text".to_string(),
            "third text".to_string(),
        ];

        let result = embedder.embed_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 768);
        assert_eq!(embeddings[1].len(), 768);
        assert_eq!(embeddings[2].len(), 768);
    }

    #[tokio::test]
    #[ignore] // Requires HuggingFace API access
    async fn test_health_check() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        let embedder = HuggingFaceEmbedder::new(config).await.unwrap();

        let result = embedder.health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
