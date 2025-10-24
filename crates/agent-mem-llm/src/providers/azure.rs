//! Azure OpenAI LLM 提供商实现
//!
//! Azure OpenAI 是微软云平台上的 OpenAI 服务，
//! 提供企业级的 GPT 模型访问和管理。

use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::time::Duration;

/// Azure OpenAI 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureMessage {
    pub role: String,
    pub content: String,
}

/// Azure OpenAI 请求
#[derive(Debug, Clone, Serialize)]
pub struct AzureRequest {
    pub messages: Vec<AzureMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: Option<bool>,
}

/// Azure OpenAI 响应
#[derive(Debug, Clone, Deserialize)]
pub struct AzureResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<AzureChoice>,
    pub usage: Option<AzureUsage>,
}

/// Azure OpenAI 选择
#[derive(Debug, Clone, Deserialize)]
pub struct AzureChoice {
    pub index: u32,
    pub message: AzureMessage,
    pub finish_reason: Option<String>,
}

/// Azure OpenAI 使用统计
#[derive(Debug, Clone, Deserialize)]
pub struct AzureUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Azure OpenAI 错误响应
#[derive(Debug, Clone, Deserialize)]
pub struct AzureError {
    pub error: AzureErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AzureErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub param: Option<String>,
}

/// Azure OpenAI提供商实现
#[derive(Debug)]
pub struct AzureProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
    api_version: String,
    deployment_name: String,
}

impl AzureProvider {
    /// 创建新的Azure OpenAI提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        let _api_key = config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("Azure OpenAI API key is required"))?;

        let base_url = config
            .base_url
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("Azure OpenAI endpoint URL is required"))?
            .clone();

        // 从模型名称中提取部署名称
        // Azure 模型格式通常是 "deployment-name" 或 "gpt-4"
        let deployment_name = if config.model.is_empty() {
            return Err(AgentMemError::config_error(
                "Deployment name (model) is required",
            ));
        } else {
            config.model.clone()
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // Azure 可能需要更长时间
            .build()
            .map_err(|e| {
                AgentMemError::llm_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let api_version = "2024-02-01".to_string(); // 使用最新的稳定版本

        Ok(Self {
            config,
            client,
            base_url,
            api_version,
            deployment_name,
        })
    }

    /// 设置API版本
    pub fn with_api_version(mut self, api_version: &str) -> Self {
        self.api_version = api_version.to_string();
        self
    }

    /// 构建 Azure OpenAI API URL
    pub fn build_api_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.base_url.trim_end_matches('/'),
            self.deployment_name,
            self.api_version
        )
    }

    /// 转换消息格式
    pub fn convert_messages(&self, messages: &[Message]) -> Vec<AzureMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                };

                AzureMessage {
                    role: role.to_string(),
                    content: msg.content.clone(),
                }
            })
            .collect()
    }

    /// 执行 API 请求
    async fn make_request(&self, request: AzureRequest) -> Result<AzureResponse> {
        let url = self.build_api_url();
        let api_key = self.config.api_key.as_ref().unwrap();

        let mut retries = 0;
        let max_retries = 3;

        loop {
            let response = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("api-key", api_key)
                .json(&request)
                .send()
                .await
                .map_err(|e| AgentMemError::llm_error(format!("Request failed: {e}")))?;

            if response.status().is_success() {
                let azure_response: AzureResponse = response.json().await.map_err(|e| {
                    AgentMemError::llm_error(format!("Failed to parse response: {e}"))
                })?;

                return Ok(azure_response);
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

                // 尝试解析 Azure 错误响应
                if let Ok(azure_error) = serde_json::from_str::<AzureError>(&error_text) {
                    return Err(AgentMemError::llm_error(format!(
                        "Azure OpenAI API error: {} ({})",
                        azure_error.error.message, azure_error.error.code
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
    pub fn extract_response_text(&self, response: &AzureResponse) -> Result<String> {
        if response.choices.is_empty() {
            return Err(AgentMemError::llm_error("No choices in response"));
        }

        let choice = &response.choices[0];

        // 检查完成原因
        if let Some(finish_reason) = &choice.finish_reason {
            if finish_reason == "content_filter" {
                return Err(AgentMemError::llm_error(
                    "Content was filtered by Azure content policy",
                ));
            } else if finish_reason != "stop" && finish_reason != "length" {
                return Err(AgentMemError::llm_error(format!(
                    "Generation stopped due to: {finish_reason}"
                )));
            }
        }

        Ok(choice.message.content.clone())
    }
}

#[async_trait]
impl LLMProvider for AzureProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let azure_messages = self.convert_messages(messages);

        let request = AzureRequest {
            messages: azure_messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: None,
            stream: Some(false),
        };

        let response = self.make_request(request).await?;
        self.extract_response_text(&response)
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        use futures::stream::StreamExt;

        // 转换消息格式
        let azure_messages = self.convert_messages(messages);

        // 构建流式请求
        let request = AzureRequest {
            messages: azure_messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            frequency_penalty: self.config.frequency_penalty,
            presence_penalty: self.config.presence_penalty,
            stop: None,
            stream: Some(true), // 启用流式处理
        };

        // 构建 API URL
        let url = self.build_api_url();
        let api_key = self.config.api_key.as_ref().unwrap().clone();

        // 发送流式请求
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("api-key", &api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Azure OpenAI API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AgentMemError::llm_error(format!(
                "Azure OpenAI API error: {error_text}"
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

                        // Azure OpenAI 使用与 OpenAI 相同的 SSE 格式
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
                    Err(e) => Err(AgentMemError::network_error(format!(
                        "Stream error: {e}"
                    ))),
                }
            })
            .filter(|result| {
                // 过滤掉空字符串
                futures::future::ready(match result {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                })
            });

        Ok(Box::new(stream))
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "azure".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            supports_streaming: true, // 现在支持流式处理
            supports_functions: true,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error(
                "Azure OpenAI API key is required",
            ));
        }

        if self.config.base_url.is_none() {
            return Err(AgentMemError::config_error(
                "Azure OpenAI endpoint URL is required",
            ));
        }

        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Deployment name is required"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_provider_creation() {
        let config = LLMConfig {
            provider: "azure".to_string(),
            model: "gpt-35-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://your-resource.openai.azure.com".to_string()),
            ..Default::default()
        };

        let provider = AzureProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_azure_provider_missing_config() {
        let config = LLMConfig {
            provider: "azure".to_string(),
            model: "gpt-35-turbo".to_string(),
            api_key: None,
            base_url: None,
            ..Default::default()
        };

        let provider = AzureProvider::new(config);
        assert!(provider.is_err());
    }
}
