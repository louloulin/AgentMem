//! MCP Sampling 支持
//!
//! 提供 LLM 采样功能，包括：
//! - create_message() 方法
//! - 流式响应
//! - 采样参数配置
//! - LLM 提供商集成

use super::error::{McpError, McpResult};
use agent_mem_llm::{LLMClient, LLMConfig};
use agent_mem_traits::{Message, MessageRole};
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, info};

/// 采样消息角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SamplingRole {
    /// 系统消息
    System,
    /// 用户消息
    User,
    /// 助手消息
    Assistant,
}

impl From<SamplingRole> for MessageRole {
    fn from(role: SamplingRole) -> Self {
        match role {
            SamplingRole::System => MessageRole::System,
            SamplingRole::User => MessageRole::User,
            SamplingRole::Assistant => MessageRole::Assistant,
        }
    }
}

/// 采样消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    /// 消息角色
    pub role: SamplingRole,
    /// 消息内容
    pub content: String,
}

impl SamplingMessage {
    /// 创建新的采样消息
    pub fn new(role: SamplingRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(SamplingRole::System, content)
    }

    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(SamplingRole::User, content)
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(SamplingRole::Assistant, content)
    }
}

impl From<SamplingMessage> for Message {
    fn from(msg: SamplingMessage) -> Self {
        Message {
            role: msg.role.into(),
            content: msg.content,
            timestamp: Some(Utc::now()),
        }
    }
}

/// 采样参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    /// 温度参数 (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p 采样参数 (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top-k 采样参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// 最大生成令牌数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// 是否启用流式响应
    #[serde(default)]
    pub stream: bool,

    /// 其他参数
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: None,
            max_tokens: Some(2048),
            stop_sequences: None,
            stream: false,
            metadata: HashMap::new(),
        }
    }
}

impl SamplingParams {
    /// 创建新的采样参数
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置温度
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// 设置 top_p
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// 设置 top_k
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// 设置最大令牌数
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 设置停止序列
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// 启用流式响应
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// create_message 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    /// 消息列表
    pub messages: Vec<SamplingMessage>,

    /// 采样参数
    #[serde(flatten)]
    pub params: SamplingParams,

    /// 模型名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// 系统提示词（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

/// create_message 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageResponse {
    /// 生成的消息
    pub message: SamplingMessage,

    /// 停止原因
    pub stop_reason: StopReason,

    /// 使用的令牌数
    pub usage: TokenUsage,

    /// 生成时间
    pub created_at: DateTime<Utc>,

    /// 模型名称
    pub model: String,
}

/// 停止原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// 达到最大令牌数
    MaxTokens,
    /// 遇到停止序列
    StopSequence,
    /// 自然结束
    EndTurn,
    /// 其他原因
    Other,
}

/// 令牌使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入令牌数
    pub input_tokens: usize,
    /// 输出令牌数
    pub output_tokens: usize,
    /// 总令牌数
    pub total_tokens: usize,
}

impl TokenUsage {
    /// 创建新的令牌使用统计
    pub fn new(input_tokens: usize, output_tokens: usize) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }
}

/// 流式响应块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// 增量内容
    pub delta: String,
    /// 是否是最后一块
    pub is_final: bool,
    /// 停止原因（仅在最后一块时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
}

/// Sampling 管理器
pub struct SamplingManager {
    /// LLM 客户端
    llm_client: Option<Arc<LLMClient>>,
    /// 默认采样参数
    default_params: SamplingParams,
    /// 是否启用
    enabled: bool,
}

impl SamplingManager {
    /// 创建新的 Sampling 管理器
    pub fn new(llm_config: Option<&LLMConfig>) -> McpResult<Self> {
        let llm_client = if let Some(config) = llm_config {
            Some(Arc::new(
                LLMClient::new(config)
                    .map_err(|e| McpError::Internal(format!("Failed to create LLM client: {}", e)))?,
            ))
        } else {
            None
        };

        Ok(Self {
            llm_client,
            default_params: SamplingParams::default(),
            enabled: llm_config.is_some(),
        })
    }

    /// 创建禁用的 Sampling 管理器
    pub fn disabled() -> Self {
        Self {
            llm_client: None,
            default_params: SamplingParams::default(),
            enabled: false,
        }
    }

    /// 设置默认采样参数
    pub fn set_default_params(&mut self, params: SamplingParams) {
        self.default_params = params;
    }

    /// 创建消息（非流式）
    pub async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> McpResult<CreateMessageResponse> {
        if !self.enabled {
            return Err(McpError::Internal(
                "Sampling is not enabled. Please configure an LLM provider.".to_string(),
            ));
        }

        let llm_client = self
            .llm_client
            .as_ref()
            .ok_or_else(|| McpError::Internal("LLM client not initialized".to_string()))?;

        info!(
            "Creating message with {} messages",
            request.messages.len()
        );

        // 转换消息格式
        let mut messages: Vec<Message> = request
            .messages
            .into_iter()
            .map(|m| m.into())
            .collect();

        // 如果有系统提示词，添加到消息列表开头
        if let Some(system_prompt) = request.system_prompt {
            messages.insert(
                0,
                Message {
                    role: MessageRole::System,
                    content: system_prompt,
                    timestamp: Some(Utc::now()),
                },
            );
        }

        // 生成响应
        let start_time = std::time::Instant::now();
        let response_text = llm_client
            .generate(&messages)
            .await
            .map_err(|e| McpError::Internal(format!("LLM generation failed: {}", e)))?;
        let elapsed = start_time.elapsed();

        debug!("LLM generation completed in {:?}", elapsed);

        // 估算令牌使用（简化版本）
        let input_tokens = messages.iter().map(|m| m.content.len() / 4).sum();
        let output_tokens = response_text.len() / 4;

        Ok(CreateMessageResponse {
            message: SamplingMessage::assistant(response_text),
            stop_reason: StopReason::EndTurn,
            usage: TokenUsage::new(input_tokens, output_tokens),
            created_at: Utc::now(),
            model: request
                .model
                .unwrap_or_else(|| "default".to_string()),
        })
    }

    /// 创建消息（流式）
    ///
    /// 注意：流式响应功能依赖于 LLM 提供商的支持。
    /// 目前大部分提供商尚未完全实现流式响应，会回退到非流式模式。
    pub async fn create_message_stream(
        &self,
        request: CreateMessageRequest,
    ) -> McpResult<Pin<Box<dyn Stream<Item = McpResult<StreamChunk>> + Send>>> {
        if !self.enabled {
            return Err(McpError::Internal(
                "Sampling is not enabled. Please configure an LLM provider.".to_string(),
            ));
        }

        info!(
            "Creating streaming message with {} messages",
            request.messages.len()
        );

        // 对于流式响应，我们暂时使用非流式方式实现
        // 因为 LLMClient 没有暴露 generate_stream 方法
        // 未来可以通过扩展 LLMClient API 来支持真正的流式响应

        let response = self.create_message(request).await?;

        // 将完整响应转换为单个流块
        use futures::stream;
        let chunk = StreamChunk {
            delta: response.message.content,
            is_final: true,
            stop_reason: Some(response.stop_reason),
        };

        let chunk_stream = stream::once(async move { Ok(chunk) });
        Ok(Box::pin(chunk_stream))
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 获取默认参数
    pub fn default_params(&self) -> &SamplingParams {
        &self.default_params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_message_creation() {
        let msg = SamplingMessage::user("Hello, world!");
        assert_eq!(msg.role, SamplingRole::User);
        assert_eq!(msg.content, "Hello, world!");

        let sys_msg = SamplingMessage::system("You are a helpful assistant.");
        assert_eq!(sys_msg.role, SamplingRole::System);
    }

    #[test]
    fn test_sampling_params_default() {
        let params = SamplingParams::default();
        assert_eq!(params.temperature, Some(0.7));
        assert_eq!(params.top_p, Some(0.9));
        assert_eq!(params.max_tokens, Some(2048));
        assert!(!params.stream);
    }

    #[test]
    fn test_sampling_params_builder() {
        let params = SamplingParams::new()
            .with_temperature(0.5)
            .with_max_tokens(1024)
            .with_stream(true);

        assert_eq!(params.temperature, Some(0.5));
        assert_eq!(params.max_tokens, Some(1024));
        assert!(params.stream);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_sampling_manager_disabled() {
        let manager = SamplingManager::disabled();
        assert!(!manager.is_enabled());
    }

    #[tokio::test]
    async fn test_create_message_disabled() {
        let manager = SamplingManager::disabled();
        let request = CreateMessageRequest {
            messages: vec![SamplingMessage::user("Hello")],
            params: SamplingParams::default(),
            model: None,
            system_prompt: None,
        };

        let result = manager.create_message(request).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_stop_reason_serialization() {
        let reason = StopReason::MaxTokens;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"max_tokens\"");

        let reason = StopReason::EndTurn;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"end_turn\"");
    }
}

