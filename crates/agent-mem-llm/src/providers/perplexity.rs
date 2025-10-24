//! Perplexity AI LLM provider implementation
//!
//! Provides integration with Perplexity AI models including
//! Sonar models and Llama-based models.

use agent_mem_traits::LLMConfig;
use agent_mem_traits::{AgentMemError, Message, MessageRole, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use tracing::{debug, error, info};

/// Perplexity API request structure (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct PerplexityRequest {
    model: String,
    messages: Vec<PerplexityMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
}

/// Perplexity message structure
#[derive(Debug, Serialize, Deserialize)]
struct PerplexityMessage {
    role: String,
    content: String,
}

/// Perplexity API response structure
#[derive(Debug, Deserialize)]
struct PerplexityResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<PerplexityChoice>,
    usage: PerplexityUsage,
}

/// Perplexity choice structure
#[derive(Debug, Deserialize)]
struct PerplexityChoice {
    index: u32,
    message: PerplexityMessage,
    finish_reason: String,
}

/// Perplexity usage statistics
#[derive(Debug, Deserialize)]
struct PerplexityUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Perplexity error response
#[derive(Debug, Deserialize)]
struct PerplexityError {
    error: PerplexityErrorDetail,
}

/// Perplexity error detail
#[derive(Debug, Deserialize)]
struct PerplexityErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    param: Option<String>,
    code: Option<String>,
}

/// Perplexity LLM provider
pub struct PerplexityProvider {
    config: LLMConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl PerplexityProvider {
    /// Create a new Perplexity provider
    pub fn new(config: LLMConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| AgentMemError::config_error("Perplexity API key is required"))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30)) // Default timeout
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.perplexity.ai".to_string());

        Ok(Self {
            config,
            client,
            api_key,
            base_url,
        })
    }

    /// Get available Perplexity models
    pub fn available_models() -> Vec<&'static str> {
        vec![
            "llama-3.1-sonar-small-128k-online",
            "llama-3.1-sonar-large-128k-online",
            "llama-3.1-sonar-huge-128k-online",
            "llama-3.1-sonar-small-128k-chat",
            "llama-3.1-sonar-large-128k-chat",
            "llama-3.1-8b-instruct",
            "llama-3.1-70b-instruct",
            "mixtral-8x7b-instruct",
            "mistral-7b-instruct",
        ]
    }

    /// Convert AgentMem message to Perplexity message
    fn convert_message(&self, message: &Message) -> PerplexityMessage {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };

        PerplexityMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }

    /// Build Perplexity API request
    fn build_request(&self, messages: &[Message]) -> Result<PerplexityRequest> {
        let perplexity_messages: Vec<PerplexityMessage> = messages
            .iter()
            .map(|msg| self.convert_message(msg))
            .collect();

        if perplexity_messages.is_empty() {
            return Err(AgentMemError::validation_error("No messages provided"));
        }

        let request = PerplexityRequest {
            model: self.config.model.clone(),
            messages: perplexity_messages,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            max_tokens: self.config.max_tokens,
            stream: Some(false),
            presence_penalty: None,
            frequency_penalty: None,
        };

        Ok(request)
    }

    /// Make API request to Perplexity
    async fn make_request(&self, request: &PerplexityRequest) -> Result<PerplexityResponse> {
        debug!("Making Perplexity API request to model: {}", request.model);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(request)
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Perplexity API request failed: {e}"))
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| {
            AgentMemError::network_error(format!("Failed to read response: {e}"))
        })?;

        if status.is_success() {
            serde_json::from_str(&response_text).map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse Perplexity response: {e}"))
            })
        } else {
            // Try to parse error response
            if let Ok(error) = serde_json::from_str::<PerplexityError>(&response_text) {
                error!(
                    "Perplexity API error: {} - {}",
                    error.error.error_type, error.error.message
                );
                Err(AgentMemError::llm_error(format!(
                    "Perplexity API error: {}",
                    error.error.message
                )))
            } else {
                error!(
                    "Perplexity API error (status {}): {}",
                    status, response_text
                );
                Err(AgentMemError::llm_error(format!(
                    "Perplexity API error: HTTP {status}"
                )))
            }
        }
    }
}

#[async_trait]
impl crate::LLMProvider for PerplexityProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        info!(
            "Generating response using Perplexity model: {}",
            self.config.model
        );

        let request = self.build_request(messages)?;
        let response = self.make_request(&request).await?;

        if response.choices.is_empty() {
            return Err(AgentMemError::llm_error("Perplexity returned no choices"));
        }

        let text = response.choices[0].message.content.clone();

        if text.is_empty() {
            return Err(AgentMemError::llm_error(
                "Perplexity returned empty response",
            ));
        }

        info!("Perplexity response generated successfully (tokens: prompt={}, completion={}, total={})", 
              response.usage.prompt_tokens, response.usage.completion_tokens, response.usage.total_tokens);

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
            provider: "perplexity".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            supports_streaming: false,
            supports_functions: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(AgentMemError::config_error(
                "Perplexity API key is required",
            ));
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
            provider: "perplexity".to_string(),
            model: "llama-3.1-sonar-small-128k-chat".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.perplexity.ai".to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        }
    }

    #[test]
    fn test_perplexity_provider_creation() {
        let config = create_test_config();
        let provider = PerplexityProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_available_models() {
        let models = PerplexityProvider::available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"llama-3.1-sonar-large-128k-online"));
        assert!(models.contains(&"mixtral-8x7b-instruct"));
    }

    #[test]
    fn test_message_conversion() {
        let config = create_test_config();
        let provider = PerplexityProvider::new(config).unwrap();

        let message = Message {
            role: MessageRole::User,
            content: "Hello, Perplexity!".to_string(),
            timestamp: None,
        };

        let perplexity_message = provider.convert_message(&message);
        assert_eq!(perplexity_message.role, "user");
        assert_eq!(perplexity_message.content, "Hello, Perplexity!");
    }

    #[test]
    fn test_model_info() {
        let config = create_test_config();
        let provider = PerplexityProvider::new(config).unwrap();

        let info = provider.get_model_info();
        assert_eq!(info.provider, "perplexity");
        assert_eq!(info.model, "llama-3.1-sonar-small-128k-chat");
        assert_eq!(info.max_tokens, 1000);
        assert!(!info.supports_streaming);
    }
}
