//! Google Gemini LLM 提供商实现
//!
//! Google Gemini 是 Google 的下一代多模态 AI 模型，
//! 支持文本、图像、音频和视频的理解和生成。

use agent_mem_traits::{
    AgentMemError, LLMConfig, LLMProvider, Message, MessageRole, ModelInfo, Result,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Gemini 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiMessage {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

/// Gemini 消息部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiPart {
    pub text: String,
}

/// Gemini 请求
#[derive(Debug, Clone, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiMessage>,
    #[serde(rename = "generationConfig")]
    pub generation_config: GeminiGenerationConfig,
}

/// Gemini 生成配置
#[derive(Debug, Clone, Serialize)]
pub struct GeminiGenerationConfig {
    pub temperature: f32,
    #[serde(rename = "topP")]
    pub top_p: f32,
    #[serde(rename = "topK")]
    pub top_k: i32,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
}

/// Gemini 响应
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<GeminiUsageMetadata>,
}

/// Gemini 候选响应
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiMessage,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

/// Gemini 使用元数据
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: u32,
}

/// Google Gemini提供商实现
pub struct GeminiProvider {
    config: LLMConfig,
    client: Client,
    base_url: String,
}

impl GeminiProvider {
    /// 创建新的Gemini提供商实例
    pub fn new(config: LLMConfig) -> Result<Self> {
        // 验证必需的配置
        if config.api_key.is_none() {
            return Err(AgentMemError::config_error("Google AI API key is required"));
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
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());

        Ok(Self {
            config,
            client,
            base_url,
        })
    }

    pub fn convert_messages(&self, messages: &[Message]) -> Vec<GeminiMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "user", // Gemini treats system as user
                    MessageRole::User => "user",
                    MessageRole::Assistant => "model",
                };

                GeminiMessage {
                    role: role.to_string(),
                    parts: vec![GeminiPart {
                        text: msg.content.clone(),
                    }],
                }
            })
            .collect()
    }

    pub fn build_api_url(&self, endpoint: &str) -> String {
        format!(
            "{}/models/{}:{}",
            self.base_url, self.config.model, endpoint
        )
    }

    async fn make_request(&self, request: GeminiRequest) -> Result<GeminiResponse> {
        let url = self.build_api_url("generateContent");

        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("API key is required"))?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .query(&[("key", api_key)])
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentMemError::llm_error(format!("Request failed: {e}")))?;

        if response.status().is_success() {
            let gemini_response: GeminiResponse = response.json().await.map_err(|e| {
                AgentMemError::llm_error(format!("Failed to parse response: {e}"))
            })?;

            Ok(gemini_response)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(AgentMemError::llm_error(format!(
                "HTTP error {status}: {error_text}"
            )))
        }
    }

    pub fn extract_response_text(&self, response: &GeminiResponse) -> Result<String> {
        if response.candidates.is_empty() {
            return Err(AgentMemError::llm_error("No candidates in response"));
        }

        let candidate = &response.candidates[0];

        // 检查完成原因
        if let Some(finish_reason) = &candidate.finish_reason {
            if finish_reason != "STOP" {
                return Err(AgentMemError::llm_error(format!(
                    "Generation stopped due to: {finish_reason}"
                )));
            }
        }

        // 提取文本内容
        let text_parts: Vec<String> = candidate
            .content
            .parts
            .iter()
            .map(|part| part.text.clone())
            .collect();

        if text_parts.is_empty() {
            return Err(AgentMemError::llm_error("No text content in response"));
        }

        Ok(text_parts.join(" "))
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        let gemini_messages = self.convert_messages(messages);

        let request = GeminiRequest {
            contents: gemini_messages,
            generation_config: GeminiGenerationConfig {
                temperature: self.config.temperature.unwrap_or(0.7),
                top_p: self.config.top_p.unwrap_or(0.9),
                top_k: 40, // Gemini 特有参数，使用默认值
                max_output_tokens: self.config.max_tokens.unwrap_or(8192),
            },
        };

        let response = self.make_request(request).await?;
        self.extract_response_text(&response)
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        use futures::stream::StreamExt;

        // 转换消息格式
        let gemini_messages = self.convert_messages(messages);

        // 构建请求
        let request = GeminiRequest {
            contents: gemini_messages,
            generation_config: GeminiGenerationConfig {
                temperature: self.config.temperature.unwrap_or(0.7),
                top_p: self.config.top_p.unwrap_or(0.9),
                top_k: 40,
                max_output_tokens: self.config.max_tokens.unwrap_or(8192),
            },
        };

        // 构建流式 API URL (使用 streamGenerateContent 端点)
        let url = self.build_api_url("streamGenerateContent");

        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("API key is required"))?
            .clone();

        // 发送流式请求
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .query(&[("key", &api_key)])
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Gemini API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AgentMemError::llm_error(format!(
                "Gemini API error: {error_text}"
            )));
        }

        // 创建流式响应处理器
        // Gemini 使用换行分隔的 JSON 格式 (NDJSON)
        let stream = response
            .bytes_stream()
            .map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        // 解析 NDJSON 格式的数据
                        let chunk_str = String::from_utf8_lossy(&chunk);

                        // Gemini 返回多行 JSON，每行是一个完整的响应
                        for line in chunk_str.lines() {
                            let line = line.trim();
                            if line.is_empty() {
                                continue;
                            }

                            // 解析 JSON 响应
                            match serde_json::from_str::<GeminiResponse>(line) {
                                Ok(response) => {
                                    if !response.candidates.is_empty() {
                                        let candidate = &response.candidates[0];
                                        if !candidate.content.parts.is_empty() {
                                            let text = &candidate.content.parts[0].text;
                                            if !text.is_empty() {
                                                return Ok(text.clone());
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // 忽略解析错误，继续处理下一行
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
            provider: "gemini".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(8192),
            supports_streaming: true, // 现在支持流式处理
            supports_functions: true,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_none() {
            return Err(AgentMemError::config_error("Google AI API key is required"));
        }

        if self.config.model.is_empty() {
            return Err(AgentMemError::config_error("Model name is required"));
        }

        // 验证模型名称是否为已知的Gemini模型
        let known_models = [
            "gemini-pro",
            "gemini-pro-vision",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
        ];

        if !known_models.contains(&self.config.model.as_str()) {
            eprintln!("Warning: Unknown Gemini model: {}", self.config.model);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_creation() {
        let config = LLMConfig {
            provider: "gemini".to_string(),
            model: "gemini-pro".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = GeminiProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_gemini_provider_no_api_key() {
        let config = LLMConfig {
            provider: "gemini".to_string(),
            model: "gemini-pro".to_string(),
            api_key: None,
            ..Default::default()
        };

        let provider = GeminiProvider::new(config);
        assert!(provider.is_err());
    }
}
