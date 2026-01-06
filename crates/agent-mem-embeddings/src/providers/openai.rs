//! OpenAI嵌入提供商实现

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAI嵌入API请求结构
#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
    input: Vec<String>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
    encoding_format: String,
}

/// OpenAI嵌入API响应结构
#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    model: String,
    usage: OpenAIUsage,
}

/// 单个嵌入数据
#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    object: String,
}

/// API使用统计
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// OpenAI嵌入提供商
pub struct OpenAIEmbedder {
    config: EmbeddingConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIEmbedder {
    /// 创建新的OpenAI嵌入器实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| AgentMemError::config_error("OpenAI API key is required"))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            config,
            client,
            api_key,
            base_url,
        })
    }

    /// 发送嵌入请求到OpenAI API
    async fn send_embedding_request(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let request = OpenAIEmbeddingRequest {
            input: texts,
            model: self.config.model.clone(),
            dimensions: if self.config.model.contains("text-embedding-3") {
                Some(self.config.dimension)
            } else {
                None
            },
            encoding_format: "float".to_string(),
        };

        let url = format!("{}/embeddings", self.base_url);

        let mut retries = 0;
        loop {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let embedding_response: OpenAIEmbeddingResponse =
                            resp.json().await.map_err(|e| {
                                AgentMemError::parsing_error(format!(
                                    "Failed to parse response: {e}"
                                ))
                            })?;

                        // 按索引排序并提取嵌入向量
                        let mut embeddings: Vec<(usize, Vec<f32>)> = embedding_response
                            .data
                            .into_iter()
                            .map(|data| (data.index, data.embedding))
                            .collect();

                        embeddings.sort_by_key(|(index, _)| *index);

                        let result: Vec<Vec<f32>> = embeddings
                            .into_iter()
                            .map(|(_, embedding)| embedding)
                            .collect();

                        return Ok(result);
                    } else {
                        let status = resp.status();
                        let error_text = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());

                        if retries < self.config.max_retries
                            && (status.is_server_error() || status == 429)
                        {
                            retries += 1;
                            let delay = Duration::from_millis(1000 * (1 << retries)); // 指数退避
                            tokio::time::sleep(delay).await;
                            continue;
                        }

                        return Err(AgentMemError::llm_error(format!(
                            "OpenAI API error {status}: {error_text}"
                        )));
                    }
                }
                Err(e) => {
                    if retries < self.config.max_retries {
                        retries += 1;
                        let delay = Duration::from_millis(1000 * (1 << retries));
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Err(AgentMemError::network_error(format!("Request failed: {e}")));
                }
            }
        }
    }

    /// 将文本分批处理
    fn split_into_batches(&self, texts: &[String]) -> Vec<Vec<String>> {
        texts
            .chunks(self.config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

#[async_trait]
impl Embedder for OpenAIEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed_batch(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| AgentMemError::llm_error("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let batches = self.split_into_batches(texts);
        let mut all_embeddings = Vec::new();

        for batch in batches {
            let batch_embeddings = self.send_embedding_request(batch).await?;
            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    async fn health_check(&self) -> Result<bool> {
        // 发送一个简单的嵌入请求来检查健康状态
        match self.embed("health check").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_embedder_creation_no_api_key() {
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            api_key: None,
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(OpenAIEmbedder::new(config));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_openai_embedder_creation_with_api_key() {
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let result = OpenAIEmbedder::new(config).await;
        assert!(result.is_ok());

        let embedder = result.unwrap();
        assert_eq!(embedder.provider_name(), "openai");
        assert_eq!(embedder.model_name(), "text-embedding-ada-002");
        assert_eq!(embedder.dimension(), 1536);
    }

    #[test]
    fn test_split_into_batches() {
        let config = EmbeddingConfig {
            batch_size: 2,
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let embedder = rt.block_on(OpenAIEmbedder::new(config)).unwrap();

        let texts = vec![
            "text1".to_string(),
            "text2".to_string(),
            "text3".to_string(),
            "text4".to_string(),
            "text5".to_string(),
        ];

        let batches = embedder.split_into_batches(&texts);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 2);
        assert_eq!(batches[2].len(), 1);
    }

    #[test]
    fn test_embedding_request_serialization() {
        let request = OpenAIEmbeddingRequest {
            input: vec!["test".to_string()],
            model: "text-embedding-ada-002".to_string(),
            dimensions: None,
            encoding_format: "float".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"input\":[\"test\"]"));
        assert!(json.contains("\"model\":\"text-embedding-ada-002\""));
        assert!(json.contains("\"encoding_format\":\"float\""));
        assert!(!json.contains("\"dimensions\""));
    }

    #[test]
    fn test_embedding_request_with_dimensions() {
        let request = OpenAIEmbeddingRequest {
            input: vec!["test".to_string()],
            model: "text-embedding-3-small".to_string(),
            dimensions: Some(1536),
            encoding_format: "float".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"dimensions\":1536"));
    }
}
