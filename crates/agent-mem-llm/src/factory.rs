//! LLM工厂模式实现

#[cfg(feature = "azure")]
use crate::providers::AzureProvider;
use crate::providers::DeepSeekProvider;
#[cfg(feature = "gemini")]
use crate::providers::GeminiProvider;
use crate::providers::LiteLLMProvider;
use crate::providers::OllamaProvider; // 移除条件编译，确保总是可用
use crate::providers::ZhipuProvider;
use crate::providers::HuaweiMaasProvider;
use crate::providers::{AnthropicProvider, OpenAIProvider};
use crate::providers::{ClaudeProvider, CohereProvider, MistralProvider, PerplexityProvider};

use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, warn};

/// LLM提供商枚举，包装不同的提供商实现
pub enum LLMProviderEnum {
    #[cfg(feature = "openai")]
    OpenAI(OpenAIProvider),
    #[cfg(feature = "anthropic")]
    Anthropic(AnthropicProvider),
    #[cfg(feature = "azure")]
    Azure(AzureProvider),
    #[cfg(feature = "gemini")]
    Gemini(GeminiProvider),
    #[cfg(feature = "ollama")]
    Ollama(OllamaProvider),
    Claude(ClaudeProvider),
    Cohere(CohereProvider),
    LiteLLM(LiteLLMProvider),
    Mistral(MistralProvider),
    Perplexity(PerplexityProvider),
    DeepSeek(DeepSeekProvider),
    Zhipu(ZhipuProvider),
    HuaweiMaas(HuaweiMaasProvider),
}

#[async_trait]
impl LLMProvider for LLMProviderEnum {
    async fn generate(&self, messages: &[Message]) -> Result<String> {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.generate(messages).await,
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.generate(messages).await,
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.generate(messages).await,
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.generate(messages).await,
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.generate(messages).await,
            LLMProviderEnum::Claude(provider) => provider.generate(messages).await,
            LLMProviderEnum::Cohere(provider) => provider.generate(messages).await,
            LLMProviderEnum::LiteLLM(provider) => {
                // Convert messages to LiteLLM format
                let litellm_messages: Vec<crate::providers::litellm::LiteLLMMessage> = messages
                    .iter()
                    .map(|m| crate::providers::litellm::LiteLLMMessage {
                        role: match m.role {
                            agent_mem_traits::MessageRole::System => "system".to_string(),
                            agent_mem_traits::MessageRole::User => "user".to_string(),
                            agent_mem_traits::MessageRole::Assistant => "assistant".to_string(),
                        },
                        content: m.content.clone(),
                    })
                    .collect();
                provider.generate_response(&litellm_messages).await
            }
            LLMProviderEnum::Mistral(provider) => provider.generate(messages).await,
            LLMProviderEnum::Perplexity(provider) => provider.generate(messages).await,
            LLMProviderEnum::DeepSeek(provider) => provider.generate(messages).await,
            LLMProviderEnum::Zhipu(provider) => provider.generate(messages).await,
            LLMProviderEnum::HuaweiMaas(provider) => provider.generate(messages).await,
        }
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.generate_stream(messages).await,
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Claude(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Cohere(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::LiteLLM(_provider) => {
                // LiteLLM doesn't support streaming yet, return error
                Err(AgentMemError::LLMError(
                    "LiteLLM streaming not supported".to_string(),
                ))
            }
            LLMProviderEnum::Mistral(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Perplexity(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::DeepSeek(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::Zhipu(provider) => provider.generate_stream(messages).await,
            LLMProviderEnum::HuaweiMaas(provider) => provider.generate_stream(messages).await,
        }
    }

    fn get_model_info(&self) -> ModelInfo {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.get_model_info(),
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.get_model_info(),
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.get_model_info(),
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.get_model_info(),
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.get_model_info(),
            LLMProviderEnum::Claude(provider) => provider.get_model_info(),
            LLMProviderEnum::Cohere(provider) => provider.get_model_info(),
            LLMProviderEnum::LiteLLM(provider) => ModelInfo {
                provider: "LiteLLM".to_string(),
                model: provider.get_model().to_string(),
                max_tokens: provider.get_max_tokens().unwrap_or(4096),
                supports_streaming: false,
                supports_functions: false,
            },
            LLMProviderEnum::Mistral(provider) => provider.get_model_info(),
            LLMProviderEnum::Perplexity(provider) => provider.get_model_info(),
            LLMProviderEnum::DeepSeek(provider) => provider.get_model_info(),
            LLMProviderEnum::Zhipu(provider) => provider.get_model_info(),
            LLMProviderEnum::HuaweiMaas(provider) => provider.get_model_info(),
        }
    }

    fn validate_config(&self) -> Result<()> {
        match self {
            #[cfg(feature = "openai")]
            LLMProviderEnum::OpenAI(provider) => provider.validate_config(),
            #[cfg(feature = "anthropic")]
            LLMProviderEnum::Anthropic(provider) => provider.validate_config(),
            #[cfg(feature = "azure")]
            LLMProviderEnum::Azure(provider) => provider.validate_config(),
            #[cfg(feature = "gemini")]
            LLMProviderEnum::Gemini(provider) => provider.validate_config(),
            #[cfg(feature = "ollama")]
            LLMProviderEnum::Ollama(provider) => provider.validate_config(),
            LLMProviderEnum::Claude(provider) => provider.validate_config(),
            LLMProviderEnum::Cohere(provider) => provider.validate_config(),
            LLMProviderEnum::LiteLLM(_provider) => {
                // Basic validation - LiteLLM handles most validation internally
                Ok(())
            }
            LLMProviderEnum::Mistral(provider) => provider.validate_config(),
            LLMProviderEnum::Perplexity(provider) => provider.validate_config(),
            LLMProviderEnum::DeepSeek(provider) => provider.validate_config(),
            LLMProviderEnum::Zhipu(provider) => provider.validate_config(),
            LLMProviderEnum::HuaweiMaas(provider) => provider.validate_config(),
        }
    }
}

/// LLM工厂，用于创建不同的LLM提供商实例
pub struct LLMFactory;

impl LLMFactory {
    /// 根据配置创建LLM提供商实例
    pub fn create_provider(config: &LLMConfig) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let provider_enum = match config.provider.as_str() {
            "openai" => {
                #[cfg(feature = "openai")]
                {
                    let provider = OpenAIProvider::new(config.clone())?;
                    LLMProviderEnum::OpenAI(provider)
                }
                #[cfg(not(feature = "openai"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "OpenAI feature not enabled",
                    ));
                }
            }
            "anthropic" => {
                #[cfg(feature = "anthropic")]
                {
                    let provider = AnthropicProvider::new(config.clone())?;
                    LLMProviderEnum::Anthropic(provider)
                }
                #[cfg(not(feature = "anthropic"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Anthropic feature not enabled",
                    ));
                }
            }
            "azure" => {
                #[cfg(feature = "azure")]
                {
                    let provider = AzureProvider::new(config.clone())?;
                    LLMProviderEnum::Azure(provider)
                }
                #[cfg(not(feature = "azure"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Azure feature not enabled",
                    ));
                }
            }
            "gemini" => {
                #[cfg(feature = "gemini")]
                {
                    let provider = GeminiProvider::new(config.clone())?;
                    LLMProviderEnum::Gemini(provider)
                }
                #[cfg(not(feature = "gemini"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Gemini feature not enabled",
                    ));
                }
            }
            "ollama" => {
                #[cfg(feature = "ollama")]
                {
                    let provider = OllamaProvider::new(config.clone())?;
                    LLMProviderEnum::Ollama(provider)
                }
                #[cfg(not(feature = "ollama"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Ollama feature not enabled",
                    ));
                }
            }
            "claude" => {
                let provider = ClaudeProvider::new(config.clone())?;
                LLMProviderEnum::Claude(provider)
            }
            "cohere" => {
                let provider = CohereProvider::new(config.clone())?;
                LLMProviderEnum::Cohere(provider)
            }
            "litellm" => {
                let litellm_config = crate::providers::litellm::LiteLLMConfig {
                    model: config.model.clone(),
                    api_key: config.api_key.clone(),
                    api_base: config.base_url.clone(),
                    temperature: config.temperature,
                    max_tokens: config.max_tokens,
                    ..Default::default()
                };
                let provider = LiteLLMProvider::new(litellm_config)?;
                LLMProviderEnum::LiteLLM(provider)
            }
            "mistral" => {
                let provider = MistralProvider::new(config.clone())?;
                LLMProviderEnum::Mistral(provider)
            }
            "perplexity" => {
                let provider = PerplexityProvider::new(config.clone())?;
                LLMProviderEnum::Perplexity(provider)
            }
            "deepseek" => {
                let provider = DeepSeekProvider::from_config(config.clone())?;
                LLMProviderEnum::DeepSeek(provider)
            }
            "zhipu" => {
                let provider = ZhipuProvider::new(config.clone())?;
                LLMProviderEnum::Zhipu(provider)
            }
            "huawei_maas" => {
                let provider = HuaweiMaasProvider::new(config.clone())?;
                LLMProviderEnum::HuaweiMaas(provider)
            }
            _ => return Err(AgentMemError::unsupported_provider(&config.provider)),
        };

        Ok(Arc::new(provider_enum))
    }

    /// 获取支持的提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        let mut providers = Vec::new();

        #[cfg(feature = "openai")]
        providers.push("openai");

        #[cfg(feature = "anthropic")]
        providers.push("anthropic");

        #[cfg(feature = "azure")]
        providers.push("azure");

        #[cfg(feature = "gemini")]
        providers.push("gemini");

        #[cfg(feature = "ollama")]
        providers.push("ollama");

        // New providers (always available)
        providers.push("claude");
        providers.push("cohere");
        providers.push("litellm");
        providers.push("mistral");
        providers.push("perplexity");
        providers.push("deepseek");
        providers.push("zhipu");
        providers.push("huawei_maas");

        providers
    }

    /// 检查提供商是否受支持
    pub fn is_provider_supported(provider: &str) -> bool {
        Self::supported_providers().contains(&provider)
    }

    /// 创建默认的OpenAI提供商（如果启用）
    #[cfg(feature = "openai")]
    pub fn create_openai_provider(api_key: &str) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建默认的Anthropic提供商（如果启用）
    #[cfg(feature = "anthropic")]
    pub fn create_anthropic_provider(api_key: &str) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建本地Ollama提供商（如果启用）
    #[cfg(feature = "ollama")]
    pub fn create_ollama_provider(
        base_url: Option<&str>,
        model: &str,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "ollama".to_string(),
            model: model.to_string(),
            base_url: Some(base_url.unwrap_or("http://localhost:11434").to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Claude提供商
    pub fn create_claude_provider(
        api_key: &str,
        model: Option<&str>,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "claude".to_string(),
            model: model.unwrap_or("claude-3-haiku-20240307").to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Cohere提供商
    pub fn create_cohere_provider(
        api_key: &str,
        model: Option<&str>,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "cohere".to_string(),
            model: model.unwrap_or("command-r").to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Mistral提供商
    pub fn create_mistral_provider(
        api_key: &str,
        model: Option<&str>,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "mistral".to_string(),
            model: model.unwrap_or("mistral-small-latest").to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }

    /// 创建Perplexity提供商
    pub fn create_perplexity_provider(
        api_key: &str,
        model: Option<&str>,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let config = LLMConfig {
            provider: "perplexity".to_string(),
            model: model
                .unwrap_or("llama-3.1-sonar-small-128k-chat")
                .to_string(),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        };
        Self::create_provider(&config)
    }
}

/// 真实的 LLM 提供商工厂，用于替换 Mock 实现
/// 提供健康检查、重试机制和降级策略
pub struct RealLLMFactory;

impl RealLLMFactory {
    /// 创建真实的 LLM 提供商，带有降级机制
    pub async fn create_with_fallback(
        config: &LLMConfig,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        info!("Creating real LLM provider: {}", config.provider);

        // 尝试创建主要提供商
        match Self::create_primary_provider(config).await {
            Ok(provider) => {
                info!(
                    "Successfully created primary LLM provider: {}",
                    config.provider
                );
                Ok(provider)
            }
            Err(e) => {
                warn!(
                    "Failed to create primary provider {}: {}",
                    config.provider, e
                );

                // 尝试降级到本地 Ollama
                if config.provider != "ollama" {
                    info!("Attempting fallback to local Ollama");
                    Self::create_ollama_fallback().await
                } else {
                    error!("All LLM providers failed, no fallback available");
                    Err(e)
                }
            }
        }
    }

    /// 创建主要的 LLM 提供商
    async fn create_primary_provider(
        config: &LLMConfig,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        match config.provider.as_str() {
            "openai" => {
                let provider = OpenAIProvider::new(config.clone())?;
                // 验证连接
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "anthropic" => {
                let provider = AnthropicProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "ollama" => {
                let provider = OllamaProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "claude" => {
                let provider = ClaudeProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "cohere" => {
                let provider = CohereProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "mistral" => {
                let provider = MistralProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "perplexity" => {
                let provider = PerplexityProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "deepseek" => {
                let provider = DeepSeekProvider::from_config(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "zhipu" => {
                let provider = ZhipuProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            "huawei_maas" => {
                let provider = HuaweiMaasProvider::new(config.clone())?;
                Self::validate_provider(&provider).await?;
                Ok(Arc::new(provider))
            }
            // LiteLLM 需要特殊配置，暂时跳过
            // "litellm" => { ... },
            _ => Err(AgentMemError::config_error(format!(
                "Unsupported LLM provider: {}",
                config.provider
            ))),
        }
    }

    /// 创建 Ollama 降级提供商
    async fn create_ollama_fallback() -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let fallback_config = LLMConfig {
            provider: "ollama".to_string(),
            model: "llama2".to_string(), // 使用常见的本地模型
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
        };

        let provider = OllamaProvider::new(fallback_config)?;

        // 尝试验证 Ollama 连接
        match Self::validate_provider(&provider).await {
            Ok(_) => {
                info!("Successfully connected to fallback Ollama provider");
                Ok(Arc::new(provider))
            }
            Err(e) => {
                error!("Fallback Ollama provider also failed: {}", e);
                Err(AgentMemError::llm_error(
                    "All LLM providers failed, including fallback",
                ))
            }
        }
    }

    /// 验证提供商连接
    async fn validate_provider(provider: &dyn LLMProvider) -> Result<()> {
        use agent_mem_traits::{Message, MessageRole};

        let test_messages = vec![Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            timestamp: None,
        }];

        // 尝试简单的生成请求来验证连接
        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            provider.generate(&test_messages),
        )
        .await
        {
            Ok(Ok(_)) => {
                info!("LLM provider validation successful");
                Ok(())
            }
            Ok(Err(e)) => {
                warn!("LLM provider validation failed: {}", e);
                Err(e)
            }
            Err(_) => {
                warn!("LLM provider validation timed out");
                Err(AgentMemError::llm_error("Provider validation timeout"))
            }
        }
    }

    /// 创建带重试机制的 LLM 提供商
    pub async fn create_with_retry(
        config: &LLMConfig,
        max_retries: u32,
    ) -> Result<Arc<dyn LLMProvider + Send + Sync>> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            info!(
                "Attempting to create LLM provider (attempt {}/{})",
                attempt, max_retries
            );

            match Self::create_with_fallback(config).await {
                Ok(provider) => {
                    info!(
                        "✅ Successfully created LLM provider on attempt {}",
                        attempt
                    );
                    return Ok(provider);
                }
                Err(e) => {
                    warn!(
                        "❌ Failed to create LLM provider on attempt {}: {}",
                        attempt, e
                    );
                    last_error = Some(e);

                    if attempt < max_retries {
                        let delay = std::time::Duration::from_secs(2_u64.pow(attempt - 1)); // 指数退避
                        info!("⏳ Waiting {:?} before retry...", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AgentMemError::config_error("Failed to create LLM provider after all retries")
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_providers() {
        let providers = LLMFactory::supported_providers();
        assert!(!providers.is_empty());

        // 检查默认启用的提供商
        #[cfg(feature = "openai")]
        assert!(providers.contains(&"openai"));

        #[cfg(feature = "anthropic")]
        assert!(providers.contains(&"anthropic"));
    }

    #[test]
    fn test_is_provider_supported() {
        #[cfg(feature = "openai")]
        assert!(LLMFactory::is_provider_supported("openai"));

        #[cfg(feature = "anthropic")]
        assert!(LLMFactory::is_provider_supported("anthropic"));

        assert!(!LLMFactory::is_provider_supported("unsupported_provider"));
    }

    #[test]
    fn test_create_provider_unsupported() {
        let config = LLMConfig {
            provider: "unsupported".to_string(),
            ..Default::default()
        };

        let result = LLMFactory::create_provider(&config);
        assert!(result.is_err());
    }

    #[cfg(feature = "openai")]
    #[test]
    fn test_create_openai_provider() {
        let result = LLMFactory::create_openai_provider("test-key");
        assert!(result.is_ok());
    }

    #[cfg(feature = "anthropic")]
    #[test]
    fn test_create_anthropic_provider() {
        let result = LLMFactory::create_anthropic_provider("test-key");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_claude_provider() {
        let result = LLMFactory::create_claude_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_cohere_provider() {
        let result = LLMFactory::create_cohere_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_mistral_provider() {
        let result = LLMFactory::create_mistral_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_perplexity_provider() {
        let result = LLMFactory::create_perplexity_provider("test-key", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_providers_supported() {
        let providers = LLMFactory::supported_providers();
        assert!(providers.contains(&"claude"));
        assert!(providers.contains(&"cohere"));
        assert!(providers.contains(&"mistral"));
        assert!(providers.contains(&"perplexity"));
    }
}
