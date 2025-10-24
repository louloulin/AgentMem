//! AWS Bedrock LLM 提供商实现
//!
//! AWS Bedrock 是亚马逊的托管式基础模型服务，
//! 支持多种领先的 AI 模型，包括 Claude、Llama、Titan 等。

use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Bedrock 请求消息
#[derive(Debug, Clone, Serialize)]
pub struct BedrockMessage {
    pub role: String,
    pub content: String,
}

/// Bedrock Claude 请求格式
#[derive(Debug, Clone, Serialize)]
pub struct BedrockClaudeRequest {
    pub prompt: String,
    pub max_tokens_to_sample: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub stop_sequences: Vec<String>,
}

/// Bedrock Llama 请求格式
#[derive(Debug, Clone, Serialize)]
pub struct BedrockLlamaRequest {
    pub prompt: String,
    pub max_gen_len: u32,
    pub temperature: f32,
    pub top_p: f32,
}

/// Bedrock Titan 请求格式
#[derive(Debug, Clone, Serialize)]
pub struct BedrockTitanRequest {
    #[serde(rename = "inputText")]
    pub input_text: String,
    #[serde(rename = "textGenerationConfig")]
    pub text_generation_config: TitanTextConfig,
}

/// Titan 文本生成配置
#[derive(Debug, Clone, Serialize)]
pub struct TitanTextConfig {
    #[serde(rename = "maxTokenCount")]
    pub max_token_count: u32,
    pub temperature: f32,
    #[serde(rename = "topP")]
    pub top_p: f32,
    #[serde(rename = "stopSequences")]
    pub stop_sequences: Vec<String>,
}

/// Bedrock Claude 响应
#[derive(Debug, Clone, Deserialize)]
pub struct BedrockClaudeResponse {
    pub completion: String,
    pub stop_reason: String,
}

/// Bedrock Llama 响应
#[derive(Debug, Clone, Deserialize)]
pub struct BedrockLlamaResponse {
    pub generation: String,
    pub prompt_token_count: Option<u32>,
    pub generation_token_count: Option<u32>,
    pub stop_reason: Option<String>,
}

/// Bedrock Titan 响应
#[derive(Debug, Clone, Deserialize)]
pub struct BedrockTitanResponse {
    #[serde(rename = "outputText")]
    pub output_text: String,
    #[serde(rename = "inputTextTokenCount")]
    pub input_text_token_count: Option<u32>,
    pub results: Vec<TitanResult>,
}

/// Titan 结果
#[derive(Debug, Clone, Deserialize)]
pub struct TitanResult {
    #[serde(rename = "tokenCount")]
    pub token_count: u32,
    #[serde(rename = "outputText")]
    pub output_text: String,
    #[serde(rename = "completionReason")]
    pub completion_reason: String,
}

/// AWS Bedrock 提供商
#[derive(Debug)]
pub struct BedrockProvider {
    config: LLMConfig,
    client: Client,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl BedrockProvider {
    /// 创建新的 Bedrock 提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        let api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("AWS access key is required"))?;

        // 解析 API 密钥格式：access_key:secret_key:region
        let parts: Vec<&str> = api_key.split(':').collect();
        if parts.len() != 3 {
            return Err(AgentMemError::config_error(
                "AWS credentials format should be 'access_key:secret_key:region'",
            ));
        }

        let access_key = parts[0].to_string();
        let secret_key = parts[1].to_string();
        let region = parts[2].to_string();

        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Bedrock 可能需要更长时间
            .build()
            .map_err(|e| {
                AgentMemError::llm_error(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            config,
            client,
            region,
            access_key,
            secret_key,
        })
    }

    /// 构建 Bedrock API URL
    pub fn build_api_url(&self, model_id: &str) -> String {
        format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            self.region, model_id
        )
    }

    /// 转换消息为提示文本
    pub fn convert_messages_to_prompt(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();

        for message in messages {
            match message.role {
                MessageRole::System => {
                    prompt.push_str(&format!("System: {}\n\n", message.content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("Human: {}\n\n", message.content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!("Assistant: {}\n\n", message.content));
                }
            }
        }

        // 确保以 Assistant: 结尾，让模型继续生成
        if !prompt.ends_with("Assistant: ") {
            prompt.push_str("Assistant: ");
        }

        prompt
    }

    /// 检测模型类型
    pub fn detect_model_type(&self) -> &str {
        let model = &self.config.model;
        if model.contains("claude") {
            "claude"
        } else if model.contains("llama") {
            "llama"
        } else if model.contains("titan") {
            "titan"
        } else {
            "claude" // 默认使用 Claude 格式
        }
    }

    /// 生成 AWS 签名
    async fn sign_request(
        &self,
        _method: &str,
        url: &str,
        body: &str,
    ) -> Result<reqwest::RequestBuilder> {
        // 这里应该实现 AWS Signature Version 4
        // 为了简化，我们使用基本的请求构建
        // 在实际生产环境中，需要完整的 AWS 签名实现

        let request = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-Amz-Target", "AWSBedrockRuntimeService.InvokeModel")
            .body(body.to_string());

        Ok(request)
    }

    /// 执行 Claude 请求
    async fn invoke_claude(&self, prompt: &str) -> Result<String> {
        let request = BedrockClaudeRequest {
            prompt: prompt.to_string(),
            max_tokens_to_sample: self.config.max_tokens.unwrap_or(4000),
            temperature: self.config.temperature.unwrap_or(0.7),
            top_p: self.config.top_p.unwrap_or(0.9),
            stop_sequences: vec!["Human:".to_string()],
        };

        let body = serde_json::to_string(&request).map_err(|e| {
            AgentMemError::llm_error(format!("Failed to serialize request: {e}"))
        })?;

        let url = self.build_api_url(&self.config.model);
        let request_builder = self.sign_request("POST", &url, &body).await?;

        let response = request_builder
            .send()
            .await
            .map_err(|e| AgentMemError::llm_error(format!("Request failed: {e}")))?;

        if response.status().is_success() {
            let claude_response: BedrockClaudeResponse = response.json().await.map_err(|e| {
                AgentMemError::llm_error(format!("Failed to parse response: {e}"))
            })?;

            Ok(claude_response.completion)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(AgentMemError::llm_error(format!(
                "Bedrock API error: {error_text}"
            )))
        }
    }

    /// 执行 Llama 请求
    async fn invoke_llama(&self, prompt: &str) -> Result<String> {
        let request = BedrockLlamaRequest {
            prompt: prompt.to_string(),
            max_gen_len: self.config.max_tokens.unwrap_or(4000),
            temperature: self.config.temperature.unwrap_or(0.7),
            top_p: self.config.top_p.unwrap_or(0.9),
        };

        let body = serde_json::to_string(&request).map_err(|e| {
            AgentMemError::llm_error(format!("Failed to serialize request: {e}"))
        })?;

        let url = self.build_api_url(&self.config.model);
        let request_builder = self.sign_request("POST", &url, &body).await?;

        let response = request_builder
            .send()
            .await
            .map_err(|e| AgentMemError::llm_error(format!("Request failed: {e}")))?;

        if response.status().is_success() {
            let llama_response: BedrockLlamaResponse = response.json().await.map_err(|e| {
                AgentMemError::llm_error(format!("Failed to parse response: {e}"))
            })?;

            Ok(llama_response.generation)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(AgentMemError::llm_error(format!(
                "Bedrock API error: {error_text}"
            )))
        }
    }

    /// 执行 Titan 请求
    async fn invoke_titan(&self, prompt: &str) -> Result<String> {
        let request = BedrockTitanRequest {
            input_text: prompt.to_string(),
            text_generation_config: TitanTextConfig {
                max_token_count: self.config.max_tokens.unwrap_or(4000),
                temperature: self.config.temperature.unwrap_or(0.7),
                top_p: self.config.top_p.unwrap_or(0.9),
                stop_sequences: vec!["Human:".to_string()],
            },
        };

        let body = serde_json::to_string(&request).map_err(|e| {
            AgentMemError::llm_error(format!("Failed to serialize request: {e}"))
        })?;

        let url = self.build_api_url(&self.config.model);
        let request_builder = self.sign_request("POST", &url, &body).await?;

        let response = request_builder
            .send()
            .await
            .map_err(|e| AgentMemError::llm_error(format!("Request failed: {e}")))?;

        if response.status().is_success() {
            let titan_response: BedrockTitanResponse = response.json().await.map_err(|e| {
                AgentMemError::llm_error(format!("Failed to parse response: {e}"))
            })?;

            Ok(titan_response.output_text)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(AgentMemError::llm_error(format!(
                "Bedrock API error: {error_text}"
            )))
        }
    }
}

#[async_trait]
impl LLMProvider for BedrockProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let prompt = self.convert_messages_to_prompt(messages);

        match self.detect_model_type() {
            "claude" => self.invoke_claude(&prompt).await,
            "llama" => self.invoke_llama(&prompt).await,
            "titan" => self.invoke_titan(&prompt).await,
            _ => Err(AgentMemError::llm_error("Unsupported model type")),
        }
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        // Bedrock 流式生成需要额外的实现
        // 目前返回错误，表示不支持
        Err(AgentMemError::llm_error(
            "Streaming not yet implemented for Bedrock",
        ))
    }

    fn get_model_info(&self) -> ModelInfo {
        let max_tokens = match self.config.model.as_str() {
            model if model.contains("claude-3") => 200_000,
            model if model.contains("claude-2") => 100_000,
            model if model.contains("llama-2-70b") => 4_096,
            model if model.contains("llama-2-13b") => 4_096,
            model if model.contains("titan-text-large") => 8_000,
            model if model.contains("titan-text-express") => 8_000,
            _ => 4_096,
        };

        ModelInfo {
            model: self.config.model.clone(),
            provider: "bedrock".to_string(),
            max_tokens,
            supports_streaming: false, // Bedrock 流式需要额外实现
            supports_functions: false, // 部分模型支持，需要额外实现
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.access_key.is_empty() {
            return Err(AgentMemError::config_error("AWS access key is required"));
        }
        if self.secret_key.is_empty() {
            return Err(AgentMemError::config_error("AWS secret key is required"));
        }
        if self.region.is_empty() {
            return Err(AgentMemError::config_error("AWS region is required"));
        }
        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model ID is required"));
        }
        Ok(())
    }
}
