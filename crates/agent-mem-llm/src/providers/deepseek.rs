//! DeepSeek LLM Provider
//!
//! DeepSeek API 集成，提供高质量的中英文语言模型服务

use agent_mem_traits::{AgentMemError, LLMConfig, Message, ModelInfo, Result};
use async_trait::async_trait;
use futures::stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// DeepSeek API 配置
#[derive(Debug, Clone)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout: Duration,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            timeout: Duration::from_secs(120), // 增加到 120 秒
        }
    }
}

/// DeepSeek API 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekMessage {
    pub role: String,
    pub content: String,
}

/// DeepSeek API 请求体
#[derive(Debug, Serialize)]
pub struct DeepSeekRequest {
    pub model: String,
    pub messages: Vec<DeepSeekMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

/// DeepSeek API 响应
#[derive(Debug, Deserialize)]
pub struct DeepSeekResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<DeepSeekChoice>,
    pub usage: DeepSeekUsage,
}

#[derive(Debug, Deserialize)]
pub struct DeepSeekChoice {
    pub index: u32,
    pub message: DeepSeekMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeepSeekUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// DeepSeek LLM 提供商
pub struct DeepSeekProvider {
    config: DeepSeekConfig,
    client: Client,
}

impl DeepSeekProvider {
    /// 创建新的 DeepSeek 提供商实例
    pub fn new(config: DeepSeekConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| AgentMemError::LLMError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self { config, client })
    }

    /// 创建带有 API 密钥的 DeepSeek 提供商
    pub fn with_api_key(api_key: String) -> Result<Self> {
        let config = DeepSeekConfig {
            api_key,
            ..Default::default()
        };
        Self::new(config)
    }

    /// 发送聊天完成请求（带重试机制）
    pub async fn chat_completion(
        &self,
        messages: Vec<DeepSeekMessage>,
    ) -> Result<DeepSeekResponse> {
        let request = DeepSeekRequest {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: false,
        };

        // 重试机制
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.send_request(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        // 指数退避
                        let delay =
                            std::time::Duration::from_millis(1000 * (2_u64.pow(attempt as u32)));
                        tokio::time::sleep(delay).await;
                        println!("DeepSeek API 请求失败，第 {} 次重试...", attempt + 1);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 发送单次请求
    async fn send_request(&self, request: &DeepSeekRequest) -> Result<DeepSeekResponse> {
        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AgentMemError::LLMError(format!(
                        "Request timeout after {}s: {}",
                        self.config.timeout.as_secs(),
                        e
                    ))
                } else if e.is_connect() {
                    AgentMemError::LLMError(format!("Connection failed: {e}"))
                } else {
                    AgentMemError::LLMError(format!("Request failed: {e}"))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::LLMError(format!(
                "DeepSeek API error {status}: {error_text}"
            )));
        }

        let deepseek_response: DeepSeekResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to parse response: {e}")))?;

        Ok(deepseek_response)
    }

    /// 生成文本响应
    pub async fn generate_text(&self, prompt: &str) -> Result<String> {
        let messages = vec![DeepSeekMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let response = self.chat_completion(messages).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(AgentMemError::LLMError(
                "No response choices returned".to_string(),
            ))
        }
    }

    /// 生成结构化 JSON 响应
    pub async fn generate_json<T>(&self, prompt: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json_prompt = format!(
            "{prompt}\n\nPlease respond with valid JSON only, no additional text or formatting."
        );

        let response_text = self.generate_text(&json_prompt).await?;

        // 尝试清理响应文本，移除可能的代码块标记
        let cleaned_text = response_text
            .trim()
            .strip_prefix("```json")
            .unwrap_or(&response_text)
            .strip_suffix("```")
            .unwrap_or(&response_text)
            .trim();

        serde_json::from_str(cleaned_text)
            .map_err(|e| AgentMemError::LLMError(format!("Failed to parse JSON response: {e}")))
    }

    /// 生成系统和用户消息的响应
    pub async fn generate_with_system(&self, system: &str, user: &str) -> Result<String> {
        let messages = vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: user.to_string(),
            },
        ];

        let response = self.chat_completion(messages).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(AgentMemError::LLMError(
                "No response choices returned".to_string(),
            ))
        }
    }

    /// 从 LLMConfig 创建 DeepSeekProvider
    pub fn from_config(config: LLMConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .ok_or_else(|| AgentMemError::config_error("DeepSeek provider requires api_key"))?;

        let deepseek_config = DeepSeekConfig {
            api_key,
            base_url: config
                .base_url
                .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string()),
            model: config.model,
            temperature: config.temperature.unwrap_or(0.7),
            max_tokens: config.max_tokens.unwrap_or(4096),
            timeout: Duration::from_secs(120), // 默认 120 秒超时
        };

        Self::new(deepseek_config)
    }
}

#[async_trait]
impl crate::LLMProvider for DeepSeekProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let deepseek_messages: Vec<DeepSeekMessage> = messages
            .iter()
            .map(|msg| DeepSeekMessage {
                role: match msg.role {
                    agent_mem_traits::MessageRole::System => "system".to_string(),
                    agent_mem_traits::MessageRole::User => "user".to_string(),
                    agent_mem_traits::MessageRole::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        let response = self.chat_completion(deepseek_messages).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(AgentMemError::LLMError(
                "No response choices returned".to_string(),
            ))
        }
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        // DeepSeek 流式响应暂未实现，返回空流
        Ok(Box::new(stream::empty()))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "deepseek".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            supports_streaming: false,
            supports_functions: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_empty() {
            return Err(AgentMemError::config_error("DeepSeek API key is required"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deepseek_config_default() {
        let config = DeepSeekConfig::default();
        assert_eq!(config.base_url, "https://api.deepseek.com/v1");
        assert_eq!(config.model, "deepseek-chat");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 4096);
    }

    #[tokio::test]
    async fn test_deepseek_provider_creation() {
        let config = DeepSeekConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let provider = DeepSeekProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_deepseek_with_api_key() {
        let provider = DeepSeekProvider::with_api_key("test-key".to_string());
        assert!(provider.is_ok());
    }
}
