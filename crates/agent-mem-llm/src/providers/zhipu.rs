//! Zhipu AI (智谱AI) LLM提供商实现
//! 支持 GLM-4, GLM-4-Plus, GLM-4-Air 等模型

use agent_mem_traits::llm::{FunctionCall, FunctionCallResponse, FunctionDefinition};
use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Zhipu AI API请求结构
#[derive(Debug, Serialize)]
struct ZhipuRequest {
    model: String,
    messages: Vec<ZhipuMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ZhipuTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

/// Zhipu消息格式
#[derive(Debug, Serialize, Deserialize)]
struct ZhipuMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ZhipuToolCall>>,
}

/// Zhipu工具调用
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ZhipuFunctionCall,
}

/// Zhipu API响应结构
#[derive(Debug, Deserialize)]
struct ZhipuResponse {
    id: String,
    created: u64,
    model: String,
    choices: Vec<ZhipuChoice>,
    usage: ZhipuUsage,
}

/// Zhipu选择结构
#[derive(Debug, Deserialize)]
struct ZhipuChoice {
    index: u32,
    message: ZhipuMessage,
    finish_reason: Option<String>,
}

/// Zhipu使用统计
#[derive(Debug, Deserialize)]
struct ZhipuUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Zhipu函数定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Zhipu函数调用
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Zhipu工具定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZhipuTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ZhipuFunction,
}

/// Zhipu错误响应
#[derive(Debug, Deserialize)]
struct ZhipuErrorResponse {
    error: ZhipuErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ZhipuErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    code: Option<String>,
}

/// Zhipu AI LLM提供商
pub struct ZhipuProvider {
    config: LLMConfig,
    client: Client,
}

impl ZhipuProvider {
    /// 创建新的Zhipu提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AgentMemError::LLMError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// 将内部消息格式转换为Zhipu格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<ZhipuMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                };

                ZhipuMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                    tool_calls: None,
                }
            })
            .collect()
    }

    /// 将函数定义转换为Zhipu工具格式
    fn convert_tools(&self, functions: &[FunctionDefinition]) -> Vec<ZhipuTool> {
        functions
            .iter()
            .map(|func| ZhipuTool {
                tool_type: "function".to_string(),
                function: ZhipuFunction {
                    name: func.name.clone(),
                    description: func.description.clone(),
                    parameters: func.parameters.clone(),
                },
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for ZhipuProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::ConfigError("Zhipu API key not configured".to_string()))?;

        let base_url = self
            .config
            .base_url
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://open.bigmodel.cn/api/paas/v4");

        let url = format!("{}/chat/completions", base_url);

        let request = ZhipuRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(&messages),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if let Ok(error_response) = serde_json::from_str::<ZhipuErrorResponse>(&error_text) {
                return Err(AgentMemError::LLMError(format!(
                    "Zhipu API error: {} (type: {})",
                    error_response.error.message, error_response.error.error_type
                )));
            }

            return Err(AgentMemError::LLMError(format!(
                "Zhipu API error: {}",
                error_text
            )));
        }

        let zhipu_response: ZhipuResponse = response.json().await.map_err(|e| {
            AgentMemError::LLMError(format!("Failed to parse Zhipu response: {}", e))
        })?;

        zhipu_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| AgentMemError::LLMError("No response from Zhipu".to_string()))
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
    ) -> Result<FunctionCallResponse> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::ConfigError("Zhipu API key not configured".to_string()))?;

        let base_url = self
            .config
            .base_url
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://open.bigmodel.cn/api/paas/v4");

        let url = format!("{}/chat/completions", base_url);

        let tools = self.convert_tools(&functions);

        let request = ZhipuRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(&messages),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            stream: Some(false),
            tools: Some(tools),
            tool_choice: Some("auto".to_string()),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::LLMError(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::LLMError(format!(
                "Zhipu API error: {}",
                error_text
            )));
        }

        let zhipu_response: ZhipuResponse = response.json().await.map_err(|e| {
            AgentMemError::LLMError(format!("Failed to parse Zhipu response: {}", e))
        })?;

        let choice = zhipu_response
            .choices
            .first()
            .ok_or_else(|| AgentMemError::LLMError("No response from Zhipu".to_string()))?;

        let mut function_calls = Vec::new();

        if let Some(tool_calls) = &choice.message.tool_calls {
            for tool_call in tool_calls {
                function_calls.push(FunctionCall {
                    name: tool_call.function.name.clone(),
                    arguments: tool_call.function.arguments.clone(),
                });
            }
        }

        Ok(FunctionCallResponse {
            text: if function_calls.is_empty() {
                Some(choice.message.content.clone())
            } else {
                None
            },
            function_calls,
        })
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        Err(AgentMemError::LLMError(
            "Streaming not yet implemented for Zhipu provider".to_string(),
        ))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "zhipu".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(8192),
            supports_functions: true,
            supports_streaming: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::ConfigError(
                "Zhipu API key is required".to_string(),
            ));
        }
        Ok(())
    }
}

