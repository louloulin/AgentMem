//! LiteLLM Provider
//!
//! 统一 LLM 接口，支持多种 LLM 提供商的统一访问

use agent_mem_traits::{AgentMemError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LiteLLM 配置
#[derive(Debug, Clone)]
pub struct LiteLLMConfig {
    pub model: String,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub api_version: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timeout: Duration,
    pub retry_config: RetryConfig,
    pub rate_limit_config: RateLimitConfig,
}

impl Default for LiteLLMConfig {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            api_base: None,
            api_version: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            timeout: Duration::from_secs(60),
            retry_config: RetryConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
        }
    }
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub backoff_factor: f32,
    pub max_backoff: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_factor: 2.0,
            max_backoff: Duration::from_secs(60),
        }
    }
}

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub concurrent_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 100000,
            concurrent_requests: 10,
        }
    }
}

/// LiteLLM 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteLLMMessage {
    pub role: String,
    pub content: String,
}

/// LiteLLM 请求体
#[derive(Debug, Serialize)]
pub struct LiteLLMRequest {
    pub model: String,
    pub messages: Vec<LiteLLMMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

/// 响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

/// LiteLLM 响应
#[derive(Debug, Deserialize)]
pub struct LiteLLMResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<LiteLLMChoice>,
    pub usage: LiteLLMUsage,
}

/// LiteLLM 选择
#[derive(Debug, Deserialize)]
pub struct LiteLLMChoice {
    pub index: u32,
    pub message: LiteLLMMessage,
    pub finish_reason: Option<String>,
}

/// LiteLLM 使用统计
#[derive(Debug, Deserialize)]
pub struct LiteLLMUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 支持的模型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportedModel {
    // OpenAI
    GPT4,
    GPT4Turbo,
    GPT35Turbo,

    // Anthropic
    Claude3Opus,
    Claude3Sonnet,
    Claude3Haiku,

    // AWS Bedrock
    BedrockClaude,
    BedrockTitan,

    // Azure OpenAI
    AzureGPT4,
    AzureGPT35,

    // Google
    Gemini15Pro,
    Gemini15Flash,

    // Others
    Groq,
    Together,
    Ollama,
}

impl SupportedModel {
    pub fn as_str(&self) -> &str {
        match self {
            SupportedModel::GPT4 => "gpt-4",
            SupportedModel::GPT4Turbo => "gpt-4-turbo",
            SupportedModel::GPT35Turbo => "gpt-3.5-turbo",
            SupportedModel::Claude3Opus => "claude-3-opus-20240229",
            SupportedModel::Claude3Sonnet => "claude-3-sonnet-20240229",
            SupportedModel::Claude3Haiku => "claude-3-haiku-20240307",
            SupportedModel::BedrockClaude => "bedrock/anthropic.claude-3-sonnet-20240229-v1:0",
            SupportedModel::BedrockTitan => "bedrock/amazon.titan-text-express-v1",
            SupportedModel::AzureGPT4 => "azure/gpt-4",
            SupportedModel::AzureGPT35 => "azure/gpt-35-turbo",
            SupportedModel::Gemini15Pro => "gemini/gemini-1.5-pro",
            SupportedModel::Gemini15Flash => "gemini/gemini-1.5-flash",
            SupportedModel::Groq => "groq/llama3-70b-8192",
            SupportedModel::Together => "together_ai/meta-llama/Llama-2-70b-chat-hf",
            SupportedModel::Ollama => "ollama/llama2",
        }
    }
}

/// LiteLLM 提供商
pub struct LiteLLMProvider {
    client: Client,
    config: LiteLLMConfig,
}

impl LiteLLMProvider {
    /// 创建新的 LiteLLM 提供商
    pub fn new(config: LiteLLMConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| AgentMemError::LLMError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self { client, config })
    }

    /// 使用默认配置创建
    pub fn with_model(model: &str) -> Result<Self> {
        let mut config = LiteLLMConfig::default();
        config.model = model.to_string();
        Self::new(config)
    }

    /// 设置 API 密钥
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.config.api_key = Some(api_key);
        self
    }

    /// 设置 API 基础 URL
    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.config.api_base = Some(api_base);
        self
    }

    /// 获取模型名称
    pub fn get_model(&self) -> &str {
        &self.config.model
    }

    /// 获取最大 token 数
    pub fn get_max_tokens(&self) -> Option<u32> {
        self.config.max_tokens
    }

    /// 生成响应
    pub async fn generate_response(&self, messages: &[LiteLLMMessage]) -> Result<String> {
        let request = LiteLLMRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: Some(false),
            response_format: None,
        };

        let response = self.send_request(&request).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(AgentMemError::LLMError("No response choices".to_string()))
        }
    }

    /// 生成结构化响应
    pub async fn generate_structured_response<T>(&self, messages: &[LiteLLMMessage]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let request = LiteLLMRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: Some(false),
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
        };

        let response = self.send_request(&request).await?;

        if let Some(choice) = response.choices.first() {
            let parsed: T = serde_json::from_str(&choice.message.content).map_err(|e| {
                AgentMemError::LLMError(format!("Failed to parse JSON response: {e}"))
            })?;
            Ok(parsed)
        } else {
            Err(AgentMemError::LLMError("No response choices".to_string()))
        }
    }

    /// 发送请求
    async fn send_request(&self, request: &LiteLLMRequest) -> Result<LiteLLMResponse> {
        let url = self
            .config
            .api_base
            .as_ref()
            .map(|base| format!("{base}/chat/completions"))
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());

        let mut req_builder = self.client.post(&url).json(request);

        // 添加认证头
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::LLMError(format!(
                "API error: {error_text}"
            )));
        }

        let llm_response: LiteLLMResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to parse response: {e}")))?;

        Ok(llm_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_litellm_config_default() {
        let config = LiteLLMConfig::default();
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_tokens, Some(4096));
    }

    #[test]
    fn test_supported_model_as_str() {
        assert_eq!(SupportedModel::GPT4.as_str(), "gpt-4");
        assert_eq!(
            SupportedModel::Claude3Sonnet.as_str(),
            "claude-3-sonnet-20240229"
        );
        assert_eq!(
            SupportedModel::BedrockClaude.as_str(),
            "bedrock/anthropic.claude-3-sonnet-20240229-v1:0"
        );
    }

    #[test]
    fn test_litellm_provider_creation() {
        let config = LiteLLMConfig::default();
        let provider = LiteLLMProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_litellm_provider_with_model() {
        let provider = LiteLLMProvider::with_model("gpt-4");
        assert!(provider.is_ok());
    }

    #[test]
    fn test_litellm_provider_with_api_key() {
        let provider = LiteLLMProvider::with_model("gpt-4")
            .unwrap()
            .with_api_key("test-key".to_string());
        assert_eq!(provider.config.api_key, Some("test-key".to_string()));
    }
}
