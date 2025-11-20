//! Ollama本地LLM提供商实现

use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

/// Ollama API请求结构
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

/// Ollama消息格式
#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<String>,
}

/// Ollama选项
#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>, // Ollama的max_tokens等价物
}

/// Ollama API响应结构
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    message: OllamaMessage,
    done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    done_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_duration: Option<u64>,
}

/// Ollama提供商实现
pub struct OllamaProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
}

impl OllamaProvider {
    /// 创建新的Ollama提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // Ollama可能需要更长时间
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());

        Ok(Self {
            config,
            client,
            base_url,
        })
    }

    /// 将Message转换为Ollama格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    agent_mem_traits::MessageRole::System => "system",
                    agent_mem_traits::MessageRole::User => "user",
                    agent_mem_traits::MessageRole::Assistant => "assistant",
                };

                OllamaMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                    thinking: None,
                }
            })
            .collect()
    }

    /// 构建API请求
    fn build_request(&self, messages: &[Message]) -> OllamaRequest {
        let options = if self.config.temperature.is_some()
            || self.config.top_p.is_some()
            || self.config.max_tokens.is_some()
        {
            Some(OllamaOptions {
                temperature: self.config.temperature,
                top_p: self.config.top_p,
                num_predict: self.config.max_tokens,
            })
        } else {
            None
        };

        OllamaRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            options,
            stream: false,
        }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let request = self.build_request(messages);
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
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
                "Ollama API error {status}: {error_text}"
            )));
        }

        // 获取响应文本
        let response_text = response
            .text()
            .await
            .map_err(|e| AgentMemError::llm_error(format!("Failed to read response: {e}")))?;

        let ollama_response: OllamaResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentMemError::llm_error(format!("Failed to parse response: {e}")))?;

        // 如果 content 为空但有 thinking，使用 thinking 内容
        let content = if ollama_response.message.content.is_empty() {
            if let Some(thinking) = &ollama_response.message.thinking {
                thinking.clone()
            } else {
                return Err(AgentMemError::llm_error(
                    "Empty response from Ollama".to_string(),
                ));
            }
        } else {
            ollama_response.message.content
        };

        Ok(content)
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        use futures::stream::StreamExt;

        // 构建流式请求
        let options = if self.config.temperature.is_some()
            || self.config.top_p.is_some()
            || self.config.max_tokens.is_some()
        {
            Some(OllamaOptions {
                temperature: self.config.temperature,
                top_p: self.config.top_p,
                num_predict: self.config.max_tokens,
            })
        } else {
            None
        };

        let request = OllamaRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            options,
            stream: true, // 启用流式处理
        };

        let url = format!("{}/api/chat", self.base_url);

        // 发送流式请求
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Ollama API request failed: {e}")))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AgentMemError::llm_error(format!(
                "Ollama API error: {error_text}"
            )));
        }

        // 创建流式响应处理器
        // Ollama 使用换行分隔的 JSON 格式 (NDJSON)
        let stream = response
            .bytes_stream()
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        // 解析 NDJSON 格式的数据
                        let chunk_str = String::from_utf8_lossy(&chunk);

                        // Ollama 返回多行 JSON，每行是一个完整的响应
                        for line in chunk_str.lines() {
                            let line = line.trim();
                            if line.is_empty() {
                                continue;
                            }

                            // 解析 JSON 响应
                            match serde_json::from_str::<OllamaResponse>(line) {
                                Ok(response) => {
                                    // 提取消息内容
                                    let content = if !response.message.content.is_empty() {
                                        response.message.content.clone()
                                    } else if let Some(thinking) = response.message.thinking {
                                        thinking
                                    } else {
                                        String::new()
                                    };

                                    if !content.is_empty() {
                                        return Ok(content);
                                    }

                                    // 如果 done 为 true，流结束
                                    if response.done {
                                        return Ok("".to_string());
                                    }
                                }
                                Err(_) => {
                                    // 忽略解析错误，继续处理下一行
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
            provider: "ollama".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(2048),
            supports_streaming: true,  // 现在支持流式处理
            supports_functions: false, // 大多数本地模型不支持函数调用
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }

        // 验证base_url格式
        if let Some(ref url) = self.config.base_url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(AgentMemError::config_error(
                    "Base URL must start with http:// or https://",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::MessageRole;

    #[test]
    fn test_ollama_provider_creation() {
        let config = LLMConfig {
            provider: "ollama".to_string(),
            model: "llama2".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            ..Default::default()
        };

        let provider = OllamaProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_convert_messages() {
        let config = LLMConfig {
            provider: "ollama".to_string(),
            model: "llama2".to_string(),
            ..Default::default()
        };

        let provider = OllamaProvider::new(config).unwrap();

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

        let ollama_messages = provider.convert_messages(&messages);
        assert_eq!(ollama_messages.len(), 2);
        assert_eq!(ollama_messages[0].role, "system");
        assert_eq!(ollama_messages[1].role, "user");
    }

    #[test]
    fn test_validate_config() {
        let config = LLMConfig {
            provider: "ollama".to_string(),
            model: "llama2".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            ..Default::default()
        };

        let provider = OllamaProvider::new(config).unwrap();
        assert!(provider.validate_config().is_ok());
    }

    #[test]
    fn test_validate_config_invalid_url() {
        let config = LLMConfig {
            provider: "ollama".to_string(),
            model: "llama2".to_string(),
            base_url: Some("invalid-url".to_string()),
            ..Default::default()
        };

        let provider = OllamaProvider::new(config).unwrap();
        assert!(provider.validate_config().is_err());
    }
}
