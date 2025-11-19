//! Huawei MaaS (华为) LLM提供商实现

use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result};
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

// --- 数据结构定义 ---

#[derive(Debug, Serialize)]
struct HuaweiMaasRequest {
    model: String,
    messages: Vec<HuaweiMaasMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HuaweiMaasMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasResponse {
    choices: Vec<HuaweiMaasChoice>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasChoice {
    message: HuaweiMaasMessage,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamResponse {
    choices: Vec<HuaweiMaasStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamChoice {
    delta: HuaweiMaasStreamDelta,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasStreamDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasErrorResponse {
    error: HuaweiMaasErrorDetail,
}

#[derive(Debug, Deserialize)]
struct HuaweiMaasErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
}

// --- Provider 实现 ---

/// Huawei MaaS LLM 提供商
pub struct HuaweiMaasProvider {
    config: LLMConfig,
    client: Client,
}

impl HuaweiMaasProvider {
    /// 创建新的 HuaweiMaasProvider 实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AgentMemError::LLMError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self { config, client })
    }

    /// 将内部消息格式转换为 Huawei MaaS 格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<HuaweiMaasMessage> {
        messages
            .iter()
            .map(|msg| HuaweiMaasMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for HuaweiMaasProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AgentMemError::ConfigError("Huawei MaaS API key not configured".to_string())
        })?;

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.modelarts-maas.com/v2");

        let url = format!("{}/chat/completions", base_url);

        let request = HuaweiMaasRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: Some(false),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::LLMError(format!("Huawei MaaS API error: {}", error_text)));
        }

        let maas_response: HuaweiMaasResponse = response.json().await.map_err(|e| {
            AgentMemError::LLMError(format!("Failed to parse Huawei MaaS response: {e}"))
        })?;

        maas_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| AgentMemError::LLMError("No response from Huawei MaaS".to_string()))
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AgentMemError::ConfigError("Huawei MaaS API key not configured".to_string())
        })?;

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.modelarts-maas.com/v2");

        let url = format!("{}/chat/completions", base_url);

        let request = HuaweiMaasRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: Some(true),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::LLMError(format!("Huawei MaaS API error: {}", error_text)));
        }

        let stream = response
            .bytes_stream()
            .map_err(|e| AgentMemError::LLMError(format!("Stream error: {e}")))
            .and_then(|chunk| async move {
                let s = String::from_utf8(chunk.to_vec()).map_err(|e| {
                    AgentMemError::LLMError(format!("Stream UTF-8 error: {e}"))
                })?;
                Ok(s)
            })
            .filter_map(|res| async move {
                match res {
                    Ok(s) => {
                        let mut content_parts = Vec::new();
                        for line in s.lines() {
                            if line.starts_with("data:") {
                                let data = &line[5..].trim();
                                if data == "[DONE]" {
                                    break;
                                }
                                if let Ok(stream_resp) = serde_json::from_str::<HuaweiMaasStreamResponse>(data) {
                                    if let Some(choice) = stream_resp.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            content_parts.push(content.clone());
                                        }
                                    }
                                }
                            }
                        }
                        if content_parts.is_empty() {
                            None
                        } else {
                            Some(Ok(content_parts.join("")))
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(Box::pin(stream))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "huawei_maas".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(8192),
            supports_functions: false, // 当前实现不支持
            supports_streaming: true,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::ConfigError("Huawei MaaS API key is required".to_string()));
        }
        Ok(())
    }
}

