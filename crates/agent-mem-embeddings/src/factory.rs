//! 嵌入模型工厂模式实现

use crate::config::EmbeddingConfig;
use crate::providers::{LocalEmbedder, OpenAIEmbedder};
#[cfg(feature = "fastembed")]
use crate::providers::FastEmbedProvider;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use std::sync::Arc;

/// 嵌入提供商枚举，包装不同的嵌入实现
pub enum EmbedderEnum {
    OpenAI(OpenAIEmbedder),
    #[cfg(feature = "fastembed")]
    FastEmbed(FastEmbedProvider),
    #[cfg(feature = "huggingface")]
    HuggingFace(HuggingFaceEmbedder),
    #[cfg(feature = "local")]
    Local(LocalEmbedder),
    #[cfg(feature = "cohere")]
    Cohere(CohereEmbedder),
}

#[async_trait]
impl Embedder for EmbedderEnum {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.embed(text).await,
            #[cfg(feature = "fastembed")]
            EmbedderEnum::FastEmbed(embedder) => embedder.embed(text).await,
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.embed(text).await,
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.embed(text).await,
            #[cfg(feature = "cohere")]
            EmbedderEnum::Cohere(embedder) => embedder.embed(text).await,
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.embed_batch(texts).await,
            #[cfg(feature = "fastembed")]
            EmbedderEnum::FastEmbed(embedder) => embedder.embed_batch(texts).await,
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.embed_batch(texts).await,
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.embed_batch(texts).await,
            #[cfg(feature = "cohere")]
            EmbedderEnum::Cohere(embedder) => embedder.embed_batch(texts).await,
        }
    }

    fn dimension(&self) -> usize {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.dimension(),
            #[cfg(feature = "fastembed")]
            EmbedderEnum::FastEmbed(embedder) => embedder.dimension(),
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.dimension(),
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.dimension(),
            #[cfg(feature = "cohere")]
            EmbedderEnum::Cohere(embedder) => embedder.dimension(),
        }
    }

    fn provider_name(&self) -> &str {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.provider_name(),
            #[cfg(feature = "fastembed")]
            EmbedderEnum::FastEmbed(embedder) => embedder.provider_name(),
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.provider_name(),
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.provider_name(),
            #[cfg(feature = "cohere")]
            EmbedderEnum::Cohere(embedder) => embedder.provider_name(),
        }
    }

    fn model_name(&self) -> &str {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.model_name(),
            #[cfg(feature = "fastembed")]
            EmbedderEnum::FastEmbed(embedder) => embedder.model_name(),
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.model_name(),
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.model_name(),
        }
    }

    async fn health_check(&self) -> Result<bool> {
        match self {
            EmbedderEnum::OpenAI(embedder) => embedder.health_check().await,
            #[cfg(feature = "fastembed")]
            EmbedderEnum::FastEmbed(embedder) => embedder.health_check().await,
            #[cfg(feature = "huggingface")]
            EmbedderEnum::HuggingFace(embedder) => embedder.health_check().await,
            #[cfg(feature = "local")]
            EmbedderEnum::Local(embedder) => embedder.health_check().await,
            #[cfg(feature = "cohere")]
            EmbedderEnum::Cohere(embedder) => embedder.health_check().await,
        }
    }
}

/// 嵌入工厂，用于创建不同的嵌入提供商实例
pub struct EmbeddingFactory;

/// 真实嵌入工厂，提供健康检查和重试机制
pub struct RealEmbeddingFactory;

impl EmbeddingFactory {
    /// 根据配置创建嵌入器实例
    pub async fn create_embedder(
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        // 验证配置
        config.validate()?;

        let embedder_enum = match config.provider.as_str() {
            "fastembed" => {
                #[cfg(feature = "fastembed")]
                {
                    let embedder = FastEmbedProvider::new(config.clone()).await?;
                    EmbedderEnum::FastEmbed(embedder)
                }
                #[cfg(not(feature = "fastembed"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "FastEmbed feature not enabled",
                    ));
                }
            }
            "openai" => {
                let embedder = OpenAIEmbedder::new(config.clone()).await?;
                EmbedderEnum::OpenAI(embedder)
            }
            "huggingface" => {
                #[cfg(feature = "huggingface")]
                {
                    let embedder = HuggingFaceEmbedder::new(config.clone()).await?;
                    EmbedderEnum::HuggingFace(embedder)
                }
                #[cfg(not(feature = "huggingface"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "HuggingFace feature not enabled",
                    ));
                }
            }
            "local" => {
                #[cfg(feature = "local")]
                {
                    let embedder = LocalEmbedder::new(config.clone()).await?;
                    EmbedderEnum::Local(embedder)
                }
                #[cfg(not(feature = "local"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Local feature not enabled",
                    ));
                }
            }
            "anthropic" => {
                return Err(AgentMemError::unsupported_provider(
                    "Anthropic does not provide a dedicated embedding API. Please use OpenAI, HuggingFace, Cohere, or Local embeddings instead.",
                ));
            }
            "cohere" => {
                #[cfg(feature = "cohere")]
                {
                    let embedder = CohereEmbedder::new(config.clone()).await?;
                    EmbedderEnum::Cohere(embedder)
                }
                #[cfg(not(feature = "cohere"))]
                {
                    return Err(AgentMemError::unsupported_provider(
                        "Cohere feature not enabled",
                    ));
                }
            }
            _ => return Err(AgentMemError::unsupported_provider(&config.provider)),
        };

        Ok(Arc::new(embedder_enum))
    }

    /// 获取支持的嵌入提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        let mut providers = Vec::new();

        #[cfg(feature = "fastembed")]
        providers.push("fastembed");

        providers.push("openai");

        #[cfg(feature = "huggingface")]
        providers.push("huggingface");

        #[cfg(feature = "local")]
        providers.push("local");

        #[cfg(feature = "anthropic")]
        providers.push("anthropic");

        #[cfg(feature = "cohere")]
        providers.push("cohere");

        providers
    }

    /// 检查提供商是否受支持
    pub fn is_provider_supported(provider: &str) -> bool {
        Self::supported_providers().contains(&provider)
    }

    /// 创建默认的嵌入器（使用 FastEmbed，零配置）
    ///
    /// 这是推荐的默认方案，提供：
    /// - 完全本地运行，无需 API 密钥
    /// - 高性能（延迟 < 10ms）
    /// - 零成本
    /// - 自动下载和缓存模型
    /// - 支持多语言（中文、英文等）
    #[cfg(feature = "fastembed")]
    pub async fn create_default() -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig {
            provider: "fastembed".to_string(),
            model: "multilingual-e5-small".to_string(),  // 使用多语言模型，支持中文
            dimension: 384,
            batch_size: 256,
            ..Default::default()
        };
        Self::create_embedder(&config).await
    }

    /// 创建 FastEmbed 嵌入器
    ///
    /// # 参数
    /// - `model`: 模型名称，支持的模型包括：
    ///   - "bge-small-en-v1.5" (384维，推荐)
    ///   - "bge-base-en-v1.5" (768维)
    ///   - "bge-large-en-v1.5" (1024维)
    ///   - "all-MiniLM-L6-v2" (384维，轻量级)
    ///   - "all-MiniLM-L12-v2" (384维)
    ///   - "mxbai-embed-large-v1" (1024维)
    ///   - "nomic-embed-text-v1" (768维)
    ///   - "nomic-embed-text-v1.5" (768维)
    ///   - "multilingual-e5-small" (384维，多语言)
    ///   - "multilingual-e5-base" (768维，多语言)
    ///   - "multilingual-e5-large" (1024维，多语言)
    #[cfg(feature = "fastembed")]
    pub async fn create_fastembed(
        model: &str,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let dimension = match model {
            "bge-small-en-v1.5" | "all-MiniLM-L6-v2" | "all-MiniLM-L12-v2" | "multilingual-e5-small" => 384,
            "bge-base-en-v1.5" | "nomic-embed-text-v1" | "nomic-embed-text-v1.5" | "multilingual-e5-base" => 768,
            "bge-large-en-v1.5" | "mxbai-embed-large-v1" | "multilingual-e5-large" => 1024,
            _ => 768, // 默认维度
        };

        let config = EmbeddingConfig {
            provider: "fastembed".to_string(),
            model: model.to_string(),
            dimension,
            batch_size: 256,
            ..Default::default()
        };
        Self::create_embedder(&config).await
    }

    /// 创建默认的OpenAI嵌入器
    pub async fn create_openai_embedder(
        api_key: String,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::openai(Some(api_key));
        Self::create_embedder(&config).await
    }

    /// 创建OpenAI 3-small嵌入器
    pub async fn create_openai_3_small(api_key: String) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::openai_3_small(Some(api_key));
        Self::create_embedder(&config).await
    }

    /// 创建OpenAI 3-large嵌入器
    pub async fn create_openai_3_large(api_key: String) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::openai_3_large(Some(api_key));
        Self::create_embedder(&config).await
    }

    /// 创建HuggingFace嵌入器
    #[cfg(feature = "huggingface")]
    pub async fn create_huggingface_embedder(
        model: &str,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::huggingface(model);
        Self::create_embedder(&config).await
    }

    /// 创建本地嵌入器
    #[cfg(feature = "local")]
    pub async fn create_local_embedder(
        model_path: &str,
        dimension: usize,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig::local(model_path, dimension);
        Self::create_embedder(&config).await
    }

    /// 创建Anthropic嵌入器
    #[cfg(feature = "anthropic")]
    pub async fn create_anthropic_embedder(
        api_key: String,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig {
            provider: "anthropic".to_string(),
            model: "claude-embedding".to_string(),
            api_key: Some(api_key),
            dimension: 1536,
            ..Default::default()
        };
        Self::create_embedder(&config).await
    }

    /// 创建Cohere嵌入器
    #[cfg(feature = "cohere")]
    pub async fn create_cohere_embedder(
        api_key: String,
        model: Option<&str>,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        let config = EmbeddingConfig {
            provider: "cohere".to_string(),
            model: model.unwrap_or("embed-english-v3.0").to_string(),
            api_key: Some(api_key),
            dimension: 1024,
            base_url: Some("https://api.cohere.ai/v1".to_string()),
            ..Default::default()
        };
        Self::create_embedder(&config).await
    }

    /// 从环境变量创建嵌入器
    ///
    /// 环境变量：
    /// - `EMBEDDING_PROVIDER`: 提供商名称（默认: "fastembed"）
    /// - `FASTEMBED_MODEL`: FastEmbed 模型名称（默认: "multilingual-e5-small"）
    /// - `OPENAI_API_KEY`: OpenAI API 密钥
    /// - `HUGGINGFACE_MODEL`: HuggingFace 模型名称
    /// - `LOCAL_MODEL_PATH`: 本地模型路径
    /// - `LOCAL_MODEL_DIMENSION`: 本地模型维度
    /// - `COHERE_API_KEY`: Cohere API 密钥
    /// - `COHERE_MODEL`: Cohere 模型名称
    pub async fn from_env() -> Result<Arc<dyn Embedder + Send + Sync>> {
        let provider = std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| {
            #[cfg(feature = "fastembed")]
            {
                "fastembed".to_string()
            }
            #[cfg(not(feature = "fastembed"))]
            {
                "openai".to_string()
            }
        });

        match provider.as_str() {
            "fastembed" => {
                #[cfg(feature = "fastembed")]
                {
                    let model = std::env::var("FASTEMBED_MODEL")
                        .unwrap_or_else(|_| "bge-small-en-v1.5".to_string());  // 更稳定的默认模型
                    Self::create_fastembed(&model).await
                }
                #[cfg(not(feature = "fastembed"))]
                {
                    Err(AgentMemError::unsupported_provider(
                        "FastEmbed feature not enabled",
                    ))
                }
            }
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
                    AgentMemError::config_error("OPENAI_API_KEY environment variable not set")
                })?;
                Self::create_openai_embedder(api_key).await
            }
            "huggingface" => {
                #[cfg(feature = "huggingface")]
                {
                    let model = std::env::var("HUGGINGFACE_MODEL")
                        .unwrap_or_else(|_| "sentence-transformers/all-MiniLM-L6-v2".to_string());
                    Self::create_huggingface_embedder(&model).await
                }
                #[cfg(not(feature = "huggingface"))]
                {
                    Err(AgentMemError::unsupported_provider(
                        "HuggingFace feature not enabled",
                    ))
                }
            }
            "local" => {
                #[cfg(feature = "local")]
                {
                    let model_path = std::env::var("LOCAL_MODEL_PATH").map_err(|_| {
                        AgentMemError::config_error("LOCAL_MODEL_PATH environment variable not set")
                    })?;
                    let dimension = std::env::var("LOCAL_MODEL_DIMENSION")
                        .unwrap_or_else(|_| "768".to_string())
                        .parse::<usize>()
                        .map_err(|_| {
                            AgentMemError::config_error("Invalid LOCAL_MODEL_DIMENSION")
                        })?;
                    Self::create_local_embedder(&model_path, dimension).await
                }
                #[cfg(not(feature = "local"))]
                {
                    Err(AgentMemError::unsupported_provider(
                        "Local feature not enabled",
                    ))
                }
            }
            "anthropic" => {
                #[cfg(feature = "anthropic")]
                {
                    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
                        AgentMemError::config_error(
                            "ANTHROPIC_API_KEY environment variable not set",
                        )
                    })?;
                    Self::create_anthropic_embedder(api_key).await
                }
                #[cfg(not(feature = "anthropic"))]
                {
                    Err(AgentMemError::unsupported_provider(
                        "Anthropic feature not enabled",
                    ))
                }
            }
            "cohere" => {
                #[cfg(feature = "cohere")]
                {
                    let api_key = std::env::var("COHERE_API_KEY").map_err(|_| {
                        AgentMemError::config_error("COHERE_API_KEY environment variable not set")
                    })?;
                    let model = std::env::var("COHERE_MODEL").ok();
                    Self::create_cohere_embedder(api_key, model.as_deref()).await
                }
                #[cfg(not(feature = "cohere"))]
                {
                    Err(AgentMemError::unsupported_provider(
                        "Cohere feature not enabled",
                    ))
                }
            }
            _ => Err(AgentMemError::unsupported_provider(&provider)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_providers() {
        let providers = EmbeddingFactory::supported_providers();
        assert!(!providers.is_empty());
        assert!(providers.contains(&"openai"));
    }

    #[test]
    fn test_is_provider_supported() {
        assert!(EmbeddingFactory::is_provider_supported("openai"));
        assert!(!EmbeddingFactory::is_provider_supported(
            "unsupported_provider"
        ));
    }

    #[test]
    fn test_create_embedder_unsupported() {
        let config = EmbeddingConfig {
            provider: "unsupported".to_string(),
            ..Default::default()
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(EmbeddingFactory::create_embedder(&config));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_openai_embedder_no_key() {
        let config = EmbeddingConfig::openai(None);
        let result = EmbeddingFactory::create_embedder(&config).await;
        assert!(result.is_err());
    }
}

impl RealEmbeddingFactory {
    /// 创建嵌入提供商，带有健康检查和重试机制
    pub async fn create_with_retry(
        config: &EmbeddingConfig,
        max_retries: u32,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        for attempt in 1..=max_retries {
            match Self::create_with_fallback(config).await {
                Ok(embedder) => {
                    // 验证嵌入提供商
                    if let Err(e) = Self::validate_embedder(&embedder).await {
                        tracing::warn!("Embedder validation failed on attempt {}: {}", attempt, e);
                        if attempt < max_retries {
                            let delay = std::time::Duration::from_secs(2_u64.pow(attempt - 1));
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(e);
                    }
                    return Ok(embedder);
                }
                Err(e) => {
                    tracing::warn!("Embedder creation failed on attempt {}: {}", attempt, e);
                    if attempt < max_retries {
                        let delay = std::time::Duration::from_secs(2_u64.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(AgentMemError::config_error(
            "Failed to create embedder after all retries",
        ))
    }

    /// 创建嵌入提供商，带有回退机制
    pub async fn create_with_fallback(
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn Embedder + Send + Sync>> {
        // 首先尝试主要配置
        match EmbeddingFactory::create_embedder(config).await {
            Ok(embedder) => Ok(embedder),
            Err(e) => {
                tracing::warn!("Primary embedder creation failed: {}", e);

                // 如果主要提供商失败，尝试回退到OpenAI
                if config.provider != "openai" {
                    tracing::info!("Attempting fallback to OpenAI embeddings");
                    let fallback_config = EmbeddingConfig {
                        provider: "openai".to_string(),
                        model: "text-embedding-3-small".to_string(),
                        api_key: config.api_key.clone(),
                        dimension: config.dimension,
                        ..Default::default()
                    };

                    match EmbeddingFactory::create_embedder(&fallback_config).await {
                        Ok(embedder) => {
                            tracing::info!("Successfully created fallback OpenAI embedder");
                            Ok(embedder)
                        }
                        Err(fallback_err) => {
                            tracing::error!(
                                "Fallback embedder creation also failed: {}",
                                fallback_err
                            );
                            Err(e) // 返回原始错误
                        }
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 验证嵌入提供商是否正常工作
    async fn validate_embedder(embedder: &Arc<dyn Embedder + Send + Sync>) -> Result<()> {
        // 健康检查
        let health_check_timeout = std::time::Duration::from_secs(10);
        let health_result =
            tokio::time::timeout(health_check_timeout, embedder.health_check()).await;

        match health_result {
            Ok(Ok(true)) => {
                tracing::debug!("Embedder health check passed");
            }
            Ok(Ok(false)) => {
                return Err(AgentMemError::config_error("Embedder health check failed"));
            }
            Ok(Err(e)) => {
                return Err(AgentMemError::config_error(&format!(
                    "Embedder health check error: {}",
                    e
                )));
            }
            Err(_) => {
                return Err(AgentMemError::config_error("Embedder health check timeout"));
            }
        }

        // 测试嵌入生成
        let test_text = "test embedding";
        let embed_timeout = std::time::Duration::from_secs(30);
        let embed_result = tokio::time::timeout(embed_timeout, embedder.embed(test_text)).await;

        match embed_result {
            Ok(Ok(embedding)) => {
                if embedding.is_empty() {
                    return Err(AgentMemError::config_error(
                        "Embedder returned empty embedding",
                    ));
                }
                if embedding.len() != embedder.dimension() {
                    return Err(AgentMemError::config_error(&format!(
                        "Embedding dimension mismatch: expected {}, got {}",
                        embedder.dimension(),
                        embedding.len()
                    )));
                }
                tracing::debug!("Embedder test embedding successful");
                Ok(())
            }
            Ok(Err(e)) => Err(AgentMemError::config_error(&format!(
                "Embedder test embedding failed: {}",
                e
            ))),
            Err(_) => Err(AgentMemError::config_error(
                "Embedder test embedding timeout",
            )),
        }
    }

    /// 获取支持的嵌入提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        vec!["openai", "huggingface", "local", "cohere"]
    }

    /// 检查提供商是否受支持
    pub fn is_provider_supported(provider: &str) -> bool {
        Self::supported_providers().contains(&provider)
    }
}
