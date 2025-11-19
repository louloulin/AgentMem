//! Cohere LLM provider implementation
//!
//! Provides integration with Cohere's Command models including
//! Command R, Command R+, and Command Light.

use agent_mem_traits::LLMConfig;
use agent_mem_traits::{AgentMemError, Message, MessageRole, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use tracing::{debug, error, info};

/// Cohere chat request structure
#[derive(Debug, Serialize)]
struct CohereRequest {
    model: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_history: Option<Vec<CohereChatMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preamble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

/// Cohere chat message structure
#[derive(Debug, Serialize, Deserialize)]
struct CohereChatMessage {
    role: String,
    message: String,
}

/// Cohere API response structure
#[derive(Debug, Deserialize)]
struct CohereResponse {
    response_id: String,
    text: String,
    generation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_count: Option<CohereTokenCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<CohereMeta>,
}

/// Cohere token count structure
#[derive(Debug, Deserialize)]
struct CohereTokenCount {
    prompt_tokens: u32,
    response_tokens: u32,
    total_tokens: u32,
    billed_tokens: u32,
}

/// Cohere meta information
#[derive(Debug, Deserialize)]
struct CohereMeta {
    api_version: CohereApiVersion,
    billed_units: CohereBilledUnits,
}

/// Cohere API version
#[derive(Debug, Deserialize)]
struct CohereApiVersion {
    version: String,
}

/// Cohere billed units
#[derive(Debug, Deserialize)]
struct CohereBilledUnits {
    input_tokens: u32,
    output_tokens: u32,
}

/// Cohere error response
#[derive(Debug, Deserialize)]
struct CohereError {
    message: String,
}

/// Cohere LLM provider
pub struct CohereProvider {
    config: LLMConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl CohereProvider {
    /// Create a new Cohere provider
    pub fn new(config: LLMConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| AgentMemError::config_error("Cohere API key is required"))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30)) // Default timeout
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.cohere.ai".to_string());

        Ok(Self {
            config,
            client,
            api_key,
            base_url,
        })
    }

    /// Get available Cohere models
    pub fn available_models() -> Vec<&'static str> {
        vec![
            "command-r-plus",
            "command-r",
            "command",
            "command-nightly",
            "command-light",
            "command-light-nightly",
        ]
    }

    /// Convert AgentMem messages to Cohere format
    fn convert_messages(
        &self,
        messages: &[Message],
    ) -> (
        Option<String>,
        Option<String>,
        Option<Vec<CohereChatMessage>>,
    ) {
        let mut preamble = None;
        let mut current_message = None;
        let mut chat_history = Vec::new();

        for (i, message) in messages.iter().enumerate() {
            match message.role {
                MessageRole::System => {
                    preamble = Some(message.content.clone());
                }
                MessageRole::User => {
                    if i == messages.len() - 1 {
                        // Last message becomes the current message
                        current_message = Some(message.content.clone());
                    } else {
                        // Previous user messages go to chat history
                        chat_history.push(CohereChatMessage {
                            role: "USER".to_string(),
                            message: message.content.clone(),
                        });
                    }
                }
                MessageRole::Assistant => {
                    chat_history.push(CohereChatMessage {
                        role: "CHATBOT".to_string(),
                        message: message.content.clone(),
                    });
                }
            }
        }

        let history = if chat_history.is_empty() {
            None
        } else {
            Some(chat_history)
        };

        (preamble, current_message, history)
    }

    /// Build Cohere API request
    fn build_request(&self, messages: &[Message]) -> Result<CohereRequest> {
        let (preamble, current_message, chat_history) = self.convert_messages(messages);

        let message = current_message
            .ok_or_else(|| AgentMemError::validation_error("No user message found"))?;

        let request = CohereRequest {
            model: self.config.model.clone(),
            message,
            chat_history,
            preamble,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            p: self.config.top_p,
            k: None,              // Not available in LLMConfig
            stop_sequences: None, // Not available in LLMConfig
        };

        Ok(request)
    }

    /// Make API request to Cohere
    async fn make_request(&self, request: &CohereRequest) -> Result<CohereResponse> {
        debug!("Making Cohere API request to model: {}", request.model);

        let response = self
            .client
            .post(format!("{}/v1/chat", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(request)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Cohere API request failed: {e}")))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Failed to read response: {e}")))?;

        if status.is_success() {
            serde_json::from_str(&response_text).map_err(|e| {
                AgentMemError::parsing_error(format!("Failed to parse Cohere response: {e}"))
            })
        } else {
            // Try to parse error response
            if let Ok(error) = serde_json::from_str::<CohereError>(&response_text) {
                error!("Cohere API error: {}", error.message);
                Err(AgentMemError::llm_error(format!(
                    "Cohere API error: {}",
                    error.message
                )))
            } else {
                error!("Cohere API error (status {}): {}", status, response_text);
                Err(AgentMemError::llm_error(format!(
                    "Cohere API error: HTTP {status}"
                )))
            }
        }
    }
}

#[async_trait]
impl crate::LLMProvider for CohereProvider {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        info!(
            "Generating response using Cohere model: {}",
            self.config.model
        );

        let request = self.build_request(messages)?;
        let response = self.make_request(&request).await?;

        if response.text.is_empty() {
            return Err(AgentMemError::llm_error("Cohere returned empty response"));
        }

        if let Some(token_count) = &response.token_count {
            info!(
                "Cohere response generated successfully (tokens: prompt={}, response={}, total={})",
                token_count.prompt_tokens, token_count.response_tokens, token_count.total_tokens
            );
        }

        Ok(response.text)
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
            provider: "cohere".to_string(),
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4000),
            supports_streaming: false,
            supports_functions: false,
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(AgentMemError::config_error("Cohere API key is required"));
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
            provider: "cohere".to_string(),
            model: "command-r".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.cohere.ai".to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        }
    }

    #[test]
    fn test_cohere_provider_creation() {
        let config = create_test_config();
        let provider = CohereProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_available_models() {
        let models = CohereProvider::available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"command-r-plus"));
        assert!(models.contains(&"command-r"));
    }

    #[test]
    fn test_message_conversion() {
        let config = create_test_config();
        let provider = CohereProvider::new(config).unwrap();

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
            Message {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
                timestamp: None,
            },
            Message {
                role: MessageRole::User,
                content: "How are you?".to_string(),
                timestamp: None,
            },
        ];

        let (preamble, current_message, chat_history) = provider.convert_messages(&messages);

        assert_eq!(preamble, Some("You are a helpful assistant.".to_string()));
        assert_eq!(current_message, Some("How are you?".to_string()));
        assert!(chat_history.is_some());

        let history = chat_history.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, "USER");
        assert_eq!(history[0].message, "Hello!");
        assert_eq!(history[1].role, "CHATBOT");
        assert_eq!(history[1].message, "Hi there!");
    }

    #[test]
    fn test_model_info() {
        let config = create_test_config();
        let provider = CohereProvider::new(config).unwrap();

        let info = provider.get_model_info();
        assert_eq!(info.provider, "cohere");
        assert_eq!(info.model, "command-r");
        assert_eq!(info.max_tokens, 1000);
        assert!(!info.supports_streaming);
    }
}
