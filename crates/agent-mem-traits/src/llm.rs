//! LLM provider trait definitions

use crate::{Message, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Function definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Function call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Function calling response
#[derive(Debug, Clone)]
pub struct FunctionCallResponse {
    pub text: Option<String>,
    pub function_calls: Vec<FunctionCall>,
}

/// Model information structure
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub provider: String,
    pub model: String,
    pub max_tokens: u32,
    pub supports_streaming: bool,
    pub supports_functions: bool,
}

/// Core trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a text response from messages
    async fn generate(&self, messages: &[Message]) -> Result<String>;

    /// Generate a streaming response (optional)
    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>>;

    /// Get model information
    fn get_model_info(&self) -> ModelInfo;

    /// Generate response with function calling support
    async fn generate_with_functions(
        &self,
        messages: &[Message],
        _functions: &[FunctionDefinition],
    ) -> Result<FunctionCallResponse> {
        // Default implementation for providers that don't support functions
        let text = self.generate(messages).await?;
        Ok(FunctionCallResponse {
            text: Some(text),
            function_calls: Vec::new(),
        })
    }

    /// Validate the provider configuration
    fn validate_config(&self) -> Result<()>;
}
