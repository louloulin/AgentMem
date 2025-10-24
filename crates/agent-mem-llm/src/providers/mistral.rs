//! Mistral AI LLM provider implementation
//!
//! Provides integration with Mistral AI models including
//! Mistral Large, Mistral Medium, and Mistral Small.

use agent_mem_traits::LLMConfig;
use agent_mem_traits::{AgentMemError, Message, MessageRole, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use tracing::{debug, error, info};

/// Mistral API request structure
#[derive(Debug, Serialize)]
struct MistralRequest {
    model: String,
    messages: Vec<MistralMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safe_prompt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    random_seed: Option<u32>,
}

/// Mistral message structure
#[derive(Debug, Serialize, Deserialize)]
struct MistralMessage {
    role: String,
    content: String,
}

/// Mistral API response structure
#[derive(Debug, Deserialize)]
struct MistralResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<MistralChoice>,
    usage: MistralUsage,
}

/// Mistral choice structure
#[derive(Debug, Deserialize)]
struct MistralChoice {
    index: u32,
    message: MistralMessage,
    finish_reason: String,
}

/// Mistral usage statistics
#[derive(Debug, Deserialize)]
struct MistralUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Mistral error response
#[derive(Debug, Deserialize)]
struct MistralError {
    error: MistralErrorDetail,
}

/// Mistral error detail
#[derive(Debug, Deserialize)]
struct MistralErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    param: Option<String>,
    code: Option<String>,
}

/// Mistral LLM provider
pub struct MistralProvider {
    config: LLMConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl MistralProvider {
    /// Create a new Mistral provider
    pub fn new(config: LLMConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| AgentMemError::config_error("Mistral API key is required"))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30)) // Default timeout
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.mistral.ai".to_string());

        Ok(Self {
            config,
            client,
            api_key,
            base_url,
        })
    }

    /// Get available Mistral models
    pub fn available_models() -> Vec<&'static str> {
        vec![
            "mistral-large-latest",
            "mistral-large-2402",
            "mistral-medium-latest",
            "mistral-medium-2312",
            "mistral-small-latest",
            "mistral-small-2402",
            "mistral-tiny",
            "open-mistral-7b",
            "open-mixtral-8x7b",
            "open-mixtral-8x22b",
        ]
    }

    /// Convert AgentMem message to Mistral message
    fn convert_message(&self, message: &Message) -> MistralMessage {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };

        MistralMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }

    /// Build Mistral API request
    fn build_request(&self, messages: &[Message]) -> Result<MistralRequest> {
        let mistral_messages: Vec<MistralMessage> = messages
            .iter()
            .map(|msg| self.convert_message(msg))
            .collect();

        if mistral_messages.is_empty() {
            return Err(AgentMemError::validation_error("No messages provided"));
        }

        let request = MistralRequest {
            model: self.config.model.clone(),
            messages: mistral_messages,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            max_tokens: self.config.max_tokens,
            stream: Some(false),
            safe_prompt: Some(false),
            random_seed: None,
        };

        Ok(request)
    }

    /// Make API request to Mistral
    async fn make_request(&self, request: &MistralRequest) -> Result<MistralResponse> {
        debug!("Making Mistral API request to model: {}", request.model);

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Mistral API request failed: {e}"))
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| {
            AgentMemError::network_error(format!("Failed to read response: {e}"))
        })?;

        if status.is_success() {
            serde_json::from_str(&response_text).map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse Mistral response: {e}"))
            })
        } else {
            // Try to parse error response
            if let Ok(error) = serde_json::from_str::<MistralError>(&response_text) {
                error!(
                    "Mistral API error: {} - {}",
                    error.error.error_type, error.error.message
                );
                Err(AgentMemError::llm_error(format!(
                    "Mistral API error: {}",
                    error.error.message
                )))
            } else {
                error!("Mistral API error (status {}): {}", status, response_text);
                Err(AgentMemError::llm_error(format!(
                    "Mistral API error: HTTP {status}"
                )))
            }
        }
    }
}

#[async_trait]
impl crate::LLMProvider for MistralProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        info!(
            "Generating response using Mistral model: {}",
            self.config.model
        );

        let request = self.build_request(messages)?;
        let response = self.make_request(&request).await?;

        if response.choices.is_empty() {
            return Err(AgentMemError::llm_error("Mistral returned no choices"));
        }

        let text = response.choices[0].message.content.clone();

        if text.is_empty() {
            return Err(AgentMemError::llm_error("Mistral returned empty response"));
        }

        info!(
            "Mistral response generated successfully (tokens: prompt={}, completion={}, total={})",
            response.usage.prompt_tokens,
            response.usage.completion_tokens,
            response.usage.total_tokens
        );

        Ok(text)
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        // For now, fall back to non-streaming
        let response = self.generate(messages).await?;
        let stream = futures::stream::once(async move { Ok(response) });
        Ok(Box::new(Box::pin(stream)))
    }

    fn get_model_info(&self) -> agent_mem_traits::ModelInfo {
        agent_mem_traits::ModelInfo {
            provider: "mistral".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            supports_streaming: false,
            supports_functions: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(AgentMemError::config_error("Mistral API key is required"));
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
            provider: "mistral".to_string(),
            model: "mistral-small-latest".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.mistral.ai".to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        }
    }

    #[test]
    fn test_mistral_provider_creation() {
        let config = create_test_config();
        let provider = MistralProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_available_models() {
        let models = MistralProvider::available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"mistral-large-latest"));
        assert!(models.contains(&"open-mixtral-8x7b"));
    }

    #[test]
    fn test_message_conversion() {
        let config = create_test_config();
        let provider = MistralProvider::new(config).unwrap();

        let message = Message {
            role: MessageRole::User,
            content: "Hello, Mistral!".to_string(),
            timestamp: None,
        };

        let mistral_message = provider.convert_message(&message);
        assert_eq!(mistral_message.role, "user");
        assert_eq!(mistral_message.content, "Hello, Mistral!");
    }

    #[test]
    fn test_request_building() {
        let config = create_test_config();
        let provider = MistralProvider::new(config).unwrap();

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
        assert_eq!(request.model, "mistral-small-latest");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
    }

    #[test]
    fn test_model_info() {
        let config = create_test_config();
        let provider = MistralProvider::new(config).unwrap();

        let info = provider.get_model_info();
        assert_eq!(info.provider, "mistral");
        assert_eq!(info.model, "mistral-small-latest");
        assert_eq!(info.max_tokens, 1000);
        assert!(!info.supports_streaming);
    }
}
