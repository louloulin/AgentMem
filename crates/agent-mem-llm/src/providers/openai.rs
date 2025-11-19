//! OpenAI LLM提供商实现

use agent_mem_traits::llm::{FunctionCall, FunctionCallResponse, FunctionDefinition};
use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::pin::Pin;

/// OpenAI API请求结构
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

/// OpenAI消息格式
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<OpenAIFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

/// OpenAI工具调用
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: OpenAIFunctionCall,
}

/// OpenAI API响应结构
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

/// OpenAI选择结构
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

/// OpenAI使用统计
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI函数定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// OpenAI函数调用
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIFunctionCall {
    pub name: String,
    pub arguments: String,
}

/// OpenAI工具定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunction,
}

/// OpenAI提供商实现
pub struct OpenAIProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
}

impl OpenAIProvider {
    /// 创建新的OpenAI提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        if config.api_key.is_none() {
            return Err(AgentMemError::config_error("OpenAI API key is required"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        Ok(Self {
            config,
            client,
            base_url,
        })
    }

    /// 将Message转换为OpenAI格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    agent_mem_traits::MessageRole::System => "system",
                    agent_mem_traits::MessageRole::User => "user",
                    agent_mem_traits::MessageRole::Assistant => "assistant",
                };

                OpenAIMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                    function_call: None,
                    tool_calls: None,
                }
            })
            .collect()
    }

    /// 构建API请求
    fn build_request(&self, messages: &[Message]) -> OpenAIRequest {
        OpenAIRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: None,
            stream: None,
            tools: None,
            tool_choice: None,
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("OpenAI API key not configured"))?;

        let request = self.build_request(messages);
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::llm_error(format!(
                "OpenAI API error {status}: {error_text}"
            )));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::parsing_error(format!("Failed to parse response: {e}")))?;

        if openai_response.choices.is_empty() {
            return Err(AgentMemError::llm_error("No choices in OpenAI response"));
        }

        Ok(openai_response.choices[0].message.content.clone())
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        use futures::stream::StreamExt;

        // 创建流式请求
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                },
                content: msg.content.clone(),
                function_call: None,
                tool_calls: None,
            })
            .collect();

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: openai_messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: None,
            stream: Some(true), // 启用流式处理
            tools: None,
            tool_choice: None,
        };

        // 发送流式请求
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.as_ref().unwrap()),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("OpenAI API request failed: {e}")))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AgentMemError::llm_error(format!(
                "OpenAI API error: {error_text}"
            )));
        }

        // 创建流式响应处理器
        let stream = response
            .bytes_stream()
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        // 解析 SSE 格式的数据
                        let chunk_str = String::from_utf8_lossy(&chunk);
                        if chunk_str.starts_with("data: ") {
                            let json_str = chunk_str.strip_prefix("data: ").unwrap_or("");
                            if json_str.trim() == "[DONE]" {
                                return Ok("".to_string()); // 流结束
                            }

                            // 解析 JSON 响应
                            match serde_json::from_str::<serde_json::Value>(json_str) {
                                Ok(json) => {
                                    if let Some(choices) = json["choices"].as_array() {
                                        if let Some(choice) = choices.first() {
                                            if let Some(delta) = choice["delta"].as_object() {
                                                if let Some(content) = delta["content"].as_str() {
                                                    return Ok(content.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // 忽略解析错误，继续处理下一个块
                                }
                            }
                        }
                        Ok("".to_string())
                    }
                    Err(e) => Err(AgentMemError::network_error(format!("Stream error: {e}"))),
                }
            })
            .filter(|result| {
                // 过滤掉空字符串
                futures::future::ready(match result {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                })
            });

        Ok(Box::pin(stream))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "openai".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            supports_streaming: true, // 现在支持流式处理
            supports_functions: true,
        }
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
    ) -> Result<FunctionCallResponse> {
        // 转换函数定义为 OpenAI 格式
        let tools: Vec<OpenAITool> = functions
            .iter()
            .map(|func| OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIFunction {
                    name: func.name.clone(),
                    description: func.description.clone(),
                    parameters: func.parameters.clone(),
                },
            })
            .collect();

        // 构建请求消息
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                },
                content: msg.content.clone(),
                function_call: None,
                tool_calls: None,
            })
            .collect();

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: openai_messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: None,
            stream: None,
            tools: Some(tools),
            tool_choice: Some("auto".to_string()),
        };

        // 发送请求
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.as_ref().unwrap()),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("OpenAI API request failed: {e}")))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AgentMemError::llm_error(format!(
                "OpenAI API error: {error_text}"
            )));
        }

        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            AgentMemError::parsing_error(format!("Failed to parse OpenAI response: {e}"))
        })?;

        if openai_response.choices.is_empty() {
            return Err(AgentMemError::llm_error("No choices in response"));
        }

        let choice = &openai_response.choices[0];
        let mut function_calls = Vec::new();
        let mut text_content = None;

        // 检查是否有工具调用
        if let Some(tool_calls) = &choice.message.tool_calls {
            for tool_call in tool_calls {
                function_calls.push(FunctionCall {
                    name: tool_call.function.name.clone(),
                    arguments: tool_call.function.arguments.clone(),
                });
            }
        }

        // 检查是否有文本内容
        if !choice.message.content.is_empty() {
            text_content = Some(choice.message.content.clone());
        }

        Ok(FunctionCallResponse {
            text: text_content,
            function_calls,
        })
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error("OpenAI API key is required"));
        }

        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }

        // 验证模型名称是否为已知的OpenAI模型
        let known_models = [
            "gpt-3.5-turbo",
            "gpt-3.5-turbo-16k",
            "gpt-4",
            "gpt-4-32k",
            "gpt-4-turbo-preview",
            "gpt-4o",
            "gpt-4o-mini",
        ];

        if !known_models.contains(&self.config.model.as_str()) {
            // 警告但不阻止，因为OpenAI可能会发布新模型
            eprintln!("Warning: Unknown OpenAI model: {}", self.config.model);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::MessageRole;

    #[test]
    fn test_openai_provider_creation() {
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = OpenAIProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_openai_provider_no_api_key() {
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            ..Default::default()
        };

        let provider = OpenAIProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_convert_messages() {
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = OpenAIProvider::new(config).unwrap();

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant".to_string(),
                timestamp: None,
            },
            Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
                timestamp: None,
            },
        ];

        let openai_messages = provider.convert_messages(&messages);
        assert_eq!(openai_messages.len(), 2);
        assert_eq!(openai_messages[0].role, "system");
        assert_eq!(openai_messages[1].role, "user");
    }

    #[test]
    fn test_validate_config() {
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = OpenAIProvider::new(config).unwrap();
        assert!(provider.validate_config().is_ok());
    }
}
