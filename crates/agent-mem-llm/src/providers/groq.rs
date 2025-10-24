//! Groq LLM 提供商实现
//!
//! Groq 是一个高性能的 AI 推理平台，
//! 专注于提供极速的 LLM 推理服务。

use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Groq 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqMessage {
    pub role: String,
    pub content: String,
}

/// Groq 请求
#[derive(Debug, Clone, Serialize)]
pub struct GroqRequest {
    pub messages: Vec<GroqMessage>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: Option<bool>,
}

/// Groq 响应
#[derive(Debug, Clone, Deserialize)]
pub struct GroqResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<GroqChoice>,
    pub usage: Option<GroqUsage>,
}

/// Groq 选择
#[derive(Debug, Clone, Deserialize)]
pub struct GroqChoice {
    pub index: u32,
    pub message: GroqMessage,
    pub finish_reason: Option<String>,
}

/// Groq 使用统计
#[derive(Debug, Clone, Deserialize)]
pub struct GroqUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Groq 错误响应
#[derive(Debug, Clone, Deserialize)]
pub struct GroqError {
    pub error: GroqErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroqErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

/// Groq 提供商
#[derive(Debug)]
pub struct GroqProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
}

impl GroqProvider {
    /// 创建新的 Groq 提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        let _api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("Groq API key is required"))?;

        if config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30)) // Groq 以速度著称，30秒足够
            .build()
            .map_err(|e| {
                AgentMemError::llm_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string());

        Ok(Self {
            config,
            client,
            base_url,
        })
    }

    /// 构建 API URL
    pub fn build_api_url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }

    /// 转换消息格式
    pub fn convert_messages(&self, messages: &[Message]) -> Vec<GroqMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                };

                GroqMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                }
            })
            .collect()
    }

    /// 执行 API 请求
    async fn make_request(&self, request: GroqRequest) -> Result<GroqResponse> {
        let url = self.build_api_url();
        let api_key = self.config.api_key.as_ref().unwrap();

        let mut retries = 0;
        let max_retries = 3;

        loop {
            let response = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {api_key}"))
                .json(&request)
                .send()
                .await
                .map_err(|e| AgentMemError::llm_error(format!("Request failed: {e}")))?;

            if response.status().is_success() {
                let groq_response: GroqResponse = response.json().await.map_err(|e| {
                    AgentMemError::llm_error(format!("Failed to parse response: {e}"))
                })?;

                return Ok(groq_response);
            } else if response.status().is_server_error() && retries < max_retries {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(500 * retries as u64)).await; // 更短的重试间隔
                continue;
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                // 尝试解析 Groq 错误响应
                if let Ok(groq_error) = serde_json::from_str::<GroqError>(&error_text) {
                    return Err(AgentMemError::llm_error(format!(
                        "Groq API error: {} ({})",
                        groq_error.error.message, groq_error.error.error_type
                    )));
                } else {
                    return Err(AgentMemError::llm_error(format!(
                        "HTTP error {status}: {error_text}"
                    )));
                }
            }
        }
    }

    /// 提取响应文本
    pub fn extract_response_text(&self, response: &GroqResponse) -> Result<String> {
        if response.choices.is_empty() {
            return Err(AgentMemError::llm_error("No choices in response"));
        }

        let choice = &response.choices[0];

        // 检查完成原因
        if let Some(finish_reason) = &choice.finish_reason {
            if finish_reason != "stop" && finish_reason != "length" {
                return Err(AgentMemError::llm_error(format!(
                    "Generation stopped due to: {finish_reason}"
                )));
            }
        }

        Ok(choice.message.content.clone())
    }

    /// 获取支持的模型列表
    pub fn supported_models() -> Vec<&'static str> {
        vec![
            "llama3-8b-8192",
            "llama3-70b-8192",
            "mixtral-8x7b-32768",
            "gemma-7b-it",
            "gemma2-9b-it",
        ]
    }

    /// 检查模型是否支持
    pub fn is_model_supported(&self) -> bool {
        Self::supported_models().contains(&self.config.model.as_str())
    }
}

#[async_trait]
impl LLMProvider for GroqProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let groq_messages = self.convert_messages(messages);

        let request = GroqRequest {
            messages: groq_messages,
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            stop: None,
            stream: Some(false),
        };

        let response = self.make_request(request).await?;
        self.extract_response_text(&response)
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        // Groq 流式生成需要额外的实现
        // 目前返回错误，表示不支持
        Err(AgentMemError::llm_error(
            "Streaming not yet implemented for Groq",
        ))
    }

    fn get_model_info(&self) -> ModelInfo {
        let max_tokens = match self.config.model.as_str() {
            "llama3-8b-8192" => 8_192,
            "llama3-70b-8192" => 8_192,
            "mixtral-8x7b-32768" => 32_768,
            "gemma-7b-it" => 8_192,
            "gemma2-9b-it" => 8_192,
            _ => 8_192, // 默认值
        };

        ModelInfo {
            model: self.config.model.clone(),
            provider: "groq".to_string(),
            max_tokens,
            supports_streaming: false, // 流式需要额外实现
            supports_functions: false, // Groq 目前不支持函数调用
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error("Groq API key is required"));
        }
        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }
        if !self.is_model_supported() {
            return Err(AgentMemError::config_error(format!(
                "Model '{}' is not supported by Groq. Supported models: {:?}",
                self.config.model,
                Self::supported_models()
            )));
        }
        Ok(())
    }
}
