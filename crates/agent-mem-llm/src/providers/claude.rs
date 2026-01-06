//! Claude (Anthropic) LLM provider implementation
//!
//! Provides integration with Anthropic's Claude models including
//! Claude-3, Claude-3.5, and Claude-2 series.

use agent_mem_traits::LLMConfig;
use agent_mem_traits::{AgentMemError, Message, MessageRole, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::{debug, error, info};

/// Claude API request structure
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

/// Claude message structure
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API response structure
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: ClaudeUsage,
}

/// Claude content structure
#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Claude usage statistics
#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Claude error response
#[derive(Debug, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Claude LLM provider
pub struct ClaudeProvider {
    config: LLMConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl ClaudeProvider {
    /// Create a new Claude provider
    pub fn new(config: LLMConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| AgentMemError::config_error("Claude API key is required"))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30)) // Default timeout
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());

        Ok(Self {
            config,
            client,
            api_key,
            base_url,
        })
    }

    /// Get available Claude models
    pub fn available_models() -> Vec<&'static str> {
        vec![
            "claude-3-5-sonnet-20241022",
            "claude-3-5-sonnet-20240620",
            "claude-3-opus-20240229",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
            "claude-2.1",
            "claude-2.0",
            "claude-instant-1.2",
        ]
    }

    /// Convert AgentMem message to Claude message
    fn convert_message(&self, message: &Message) -> ClaudeMessage {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };

        ClaudeMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }

    /// Build Claude API request
    fn build_request(&self, messages: &[Message]) -> Result<ClaudeRequest> {
        let mut claude_messages = Vec::new();
        let mut system_message = None;

        // Separate system message from conversation messages
        for message in messages {
            match message.role {
                MessageRole::System => {
                    system_message = Some(message.content.clone());
                }
                _ => {
                    claude_messages.push(self.convert_message(message));
                }
            }
        }

        let request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            messages: claude_messages,
            system: system_message,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: None,          // Not available in LLMConfig
            stop_sequences: None, // Not available in LLMConfig
        };

        Ok(request)
    }

    /// Make API request to Claude
    async fn make_request(&self, request: &ClaudeRequest) -> Result<ClaudeResponse> {
        debug!("Making Claude API request to model: {}", request.model);

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Claude API request failed: {e}")))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to read response: {e}")))?;

        if status.is_success() {
            serde_json::from_str(&response_text).map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse Claude response: {e}"))
            })
        } else {
            // Try to parse error response
            if let Ok(error) = serde_json::from_str::<ClaudeError>(&response_text) {
                error!("Claude API error: {} - {}", error.error_type, error.message);
                Err(AgentMemError::llm_error(format!(
                    "Claude API error: {}",
                    error.message
                )))
            } else {
                error!("Claude API error (status {}): {}", status, response_text);
                Err(AgentMemError::llm_error(format!(
                    "Claude API error: HTTP {status}"
                )))
            }
        }
    }
}

#[async_trait]
impl crate::LLMProvider for ClaudeProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        info!(
            "Generating response using Claude model: {}",
            self.config.model
        );

        let request = self.build_request(messages)?;
        let response = self.make_request(&request).await?;

        // Extract text from response content
        let text = response
            .content
            .into_iter()
            .filter(|content| content.content_type == "text")
            .map(|content| content.text)
            .collect::<Vec<_>>()
            .join("\n");

        if text.is_empty() {
            return Err(AgentMemError::llm_error("Claude returned empty response"));
        }

        info!(
            "Claude response generated successfully (tokens: input={}, output={})",
            response.usage.input_tokens, response.usage.output_tokens
        );

        Ok(text)
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        // For now, fall back to non-streaming
        let response = self.generate(messages).await?;
        let stream = futures::stream::once(async move { Ok(response) });
        Ok(Box::pin(stream))
    }

    fn get_model_info(&self) -> agent_mem_traits::ModelInfo {
        agent_mem_traits::ModelInfo {
            provider: "anthropic".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            supports_streaming: false,
            supports_functions: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(AgentMemError::config_error("Claude API key is required"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::{LLMProvider, MessageRole};

    fn create_test_config() -> LLMConfig {
        LLMConfig {
            provider: "claude".to_string(),
            model: "claude-3-haiku-20240307".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.anthropic.com".to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        }
    }

    #[test]
    fn test_claude_provider_creation() {
        let config = create_test_config();
        let provider = ClaudeProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_available_models() {
        let models = ClaudeProvider::available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"claude-3-5-sonnet-20241022"));
        assert!(models.contains(&"claude-3-opus-20240229"));
    }

    #[test]
    fn test_message_conversion() {
        let config = create_test_config();
        let provider = ClaudeProvider::new(config).unwrap();

        let message = Message {
            role: MessageRole::User,
            content: "Hello, Claude!".to_string(),
            timestamp: None, // No timestamp for testing
        };

        let claude_message = provider.convert_message(&message);
        assert_eq!(claude_message.role, "user");
        assert_eq!(claude_message.content, "Hello, Claude!");
    }

    #[test]
    fn test_request_building() {
        let config = create_test_config();
        let provider = ClaudeProvider::new(config).unwrap();

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
                timestamp: None,
            },
            Message {
                role: MessageRole::User,
                content: "Hello!".to_string(),
                timestamp: None,
            },
        ];

        let request = provider.build_request(&messages).unwrap();
        assert_eq!(request.model, "claude-3-haiku-20240307");
        assert_eq!(
            request.system,
            Some("You are a helpful assistant.".to_string())
        );
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
    }

    #[test]
    fn test_model_info() {
        let config = create_test_config();
        let provider = ClaudeProvider::new(config).unwrap();

        let info = provider.get_model_info();
        assert_eq!(info.provider, "anthropic");
        assert_eq!(info.model, "claude-3-haiku-20240307");
        assert_eq!(info.max_tokens, 1000);
        assert!(!info.supports_streaming);
    }
}
