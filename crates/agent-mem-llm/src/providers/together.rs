//! Together AI LLM 提供商实现
//!
//! Together AI 是一个开源模型托管平台，
//! 提供多种开源 LLM 的高性能推理服务。

use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::pin::Pin;

/// Together 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TogetherMessage {
    pub role: String,
    pub content: String,
}

/// Together 请求
#[derive(Debug, Clone, Serialize)]
pub struct TogetherRequest {
    pub model: String,
    pub messages: Vec<TogetherMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub repetition_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: Option<bool>,
}

/// Together 响应
#[derive(Debug, Clone, Deserialize)]
pub struct TogetherResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<TogetherChoice>,
    pub usage: Option<TogetherUsage>,
}

/// Together 选择
#[derive(Debug, Clone, Deserialize)]
pub struct TogetherChoice {
    pub index: u32,
    pub message: TogetherMessage,
    pub finish_reason: Option<String>,
}

/// Together 使用统计
#[derive(Debug, Clone, Deserialize)]
pub struct TogetherUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Together 错误响应
#[derive(Debug, Clone, Deserialize)]
pub struct TogetherError {
    pub error: TogetherErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TogetherErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

/// Together AI 提供商
#[derive(Debug)]
pub struct TogetherProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
}

impl TogetherProvider {
    /// 创建新的 Together 提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        let _api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("Together API key is required"))?;

        if config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Together 可能需要更长时间
            .build()
            .map_err(|e| AgentMemError::llm_error(format!("Failed to create HTTP client: {e}")))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.together.xyz/v1".to_string());

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
    pub fn convert_messages(&self, messages: &[Message]) -> Vec<TogetherMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                };

                TogetherMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                }
            })
            .collect()
    }

    /// 执行 API 请求
    async fn make_request(&self, request: TogetherRequest) -> Result<TogetherResponse> {
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
                let together_response: TogetherResponse = response.json().await.map_err(|e| {
                    AgentMemError::llm_error(format!("Failed to parse response: {e}"))
                })?;

                return Ok(together_response);
            } else if response.status().is_server_error() && retries < max_retries {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(1000 * retries as u64)).await;
                continue;
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                // 尝试解析 Together 错误响应
                if let Ok(together_error) = serde_json::from_str::<TogetherError>(&error_text) {
                    return Err(AgentMemError::llm_error(format!(
                        "Together API error: {} ({})",
                        together_error.error.message, together_error.error.error_type
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
    pub fn extract_response_text(&self, response: &TogetherResponse) -> Result<String> {
        if response.choices.is_empty() {
            return Err(AgentMemError::llm_error("No choices in response"));
        }

        let choice = &response.choices[0];

        // 检查完成原因
        if let Some(finish_reason) = &choice.finish_reason {
            if finish_reason != "stop" && finish_reason != "length" && finish_reason != "eos" {
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
            // Meta Llama 系列
            "meta-llama/Llama-2-7b-chat-hf",
            "meta-llama/Llama-2-13b-chat-hf",
            "meta-llama/Llama-2-70b-chat-hf",
            "meta-llama/Meta-Llama-3-8B-Instruct",
            "meta-llama/Meta-Llama-3-70B-Instruct",
            // Mistral 系列
            "mistralai/Mistral-7B-Instruct-v0.1",
            "mistralai/Mistral-7B-Instruct-v0.2",
            "mistralai/Mixtral-8x7B-Instruct-v0.1",
            // CodeLlama 系列
            "codellama/CodeLlama-7b-Instruct-hf",
            "codellama/CodeLlama-13b-Instruct-hf",
            "codellama/CodeLlama-34b-Instruct-hf",
            // 其他开源模型
            "togethercomputer/RedPajama-INCITE-Chat-3B-v1",
            "togethercomputer/RedPajama-INCITE-7B-Chat",
            "NousResearch/Nous-Hermes-2-Mixtral-8x7B-DPO",
            "teknium/OpenHermes-2.5-Mistral-7B",
        ]
    }

    /// 检查模型是否支持
    pub fn is_model_supported(&self) -> bool {
        Self::supported_models().contains(&self.config.model.as_str())
    }

    /// 获取模型的最大 token 数
    pub fn get_model_max_tokens(&self) -> usize {
        match self.config.model.as_str() {
            // Llama 2 系列
            model if model.contains("Llama-2") => 4_096,
            // Llama 3 系列
            model if model.contains("Meta-Llama-3") => 8_192,
            // Mistral 系列
            model if model.contains("Mistral-7B") => 8_192,
            model if model.contains("Mixtral-8x7B") => 32_768,
            // CodeLlama 系列
            model if model.contains("CodeLlama") => 16_384,
            // 默认值
            _ => 4_096,
        }
    }
}

#[async_trait]
impl LLMProvider for TogetherProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let together_messages = self.convert_messages(messages);

        let request = TogetherRequest {
            model: self.config.model.clone(),
            messages: together_messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: None,              // Together 特有参数，可以根据需要设置
            repetition_penalty: None, // Together 特有参数
            stop: None,
            stream: Some(false),
        };

        let response = self.make_request(request).await?;
        self.extract_response_text(&response)
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        // Together 流式生成需要额外的实现
        // 目前返回错误，表示不支持
        Err(AgentMemError::llm_error(
            "Streaming not yet implemented for Together",
        ))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            model: self.config.model.clone(),
            provider: "together".to_string(),
            max_tokens: self.get_model_max_tokens() as u32,
            supports_streaming: false, // 流式需要额外实现
            supports_functions: false, // Together 目前不支持函数调用
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error("Together API key is required"));
        }
        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }
        // 注意：Together 支持很多模型，我们不强制验证模型名称
        // 让 API 来验证模型是否存在
        Ok(())
    }
}
