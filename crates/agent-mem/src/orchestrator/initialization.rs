//! Orchestrator Initialization - 初始化模块
//!
//! 负责创建和初始化所有组件，包括Intelligence组件、Embedder、VectorStore等

use std::sync::Arc;
use tracing::{info, warn};

use agent_mem_embeddings::EmbeddingFactory;
use agent_mem_llm::LLMFactory;
use agent_mem_traits::{Embedder, LLMConfig, LLMProvider};
use agent_mem_intelligence::{
    AdvancedFactExtractor, BatchConfig, BatchEntityExtractor, BatchImportanceEvaluator,
    ConflictResolver, EnhancedDecisionEngine, EnhancedImportanceEvaluator, FactExtractor,
    MemoryDecisionEngine, TimeoutConfig,
};
use agent_mem_intelligence::clustering::{dbscan::DBSCANClusterer, kmeans::KMeansClusterer};
use agent_mem_intelligence::MemoryReasoner;
use agent_mem_core::storage::libsql::{LibSqlConnectionManager, LibSqlMemoryRepository, LibSqlMemoryOperations};
use agent_mem_core::operations::MemoryOperations;

use super::core::OrchestratorConfig;
use agent_mem_traits::{AgentMemError, Result};

/// Intelligence组件集合
pub struct IntelligenceComponents {
    pub fact_extractor: Option<Arc<FactExtractor>>,
    pub advanced_fact_extractor: Option<Arc<AdvancedFactExtractor>>,
    pub batch_entity_extractor: Option<Arc<BatchEntityExtractor>>,
    pub batch_importance_evaluator: Option<Arc<BatchImportanceEvaluator>>,
    pub decision_engine: Option<Arc<MemoryDecisionEngine>>,
    pub enhanced_decision_engine: Option<Arc<EnhancedDecisionEngine>>,
    pub importance_evaluator: Option<Arc<EnhancedImportanceEvaluator>>,
    pub conflict_resolver: Option<Arc<ConflictResolver>>,
    pub llm_provider: Option<Arc<dyn LLMProvider + Send + Sync>>,
}

/// 初始化模块
pub struct InitializationModule;

impl InitializationModule {
    /// 创建 Intelligence 组件
    pub async fn create_intelligence_components(
        config: &OrchestratorConfig,
    ) -> Result<IntelligenceComponents> {
        // 创建 LLM Provider
        let llm_provider = Self::create_llm_provider(config).await?;

        if llm_provider.is_none() {
            warn!("LLM Provider 未配置，Intelligence 组件将不可用");
            return Ok(IntelligenceComponents {
                fact_extractor: None,
                advanced_fact_extractor: None,
                batch_entity_extractor: None,
                batch_importance_evaluator: None,
                decision_engine: None,
                enhanced_decision_engine: None,
                importance_evaluator: None,
                conflict_resolver: None,
                llm_provider: None,
            });
        }

        let llm = llm_provider.clone().unwrap();

        // 创建各个 Intelligence 组件
        let fact_extractor = Some(Arc::new(FactExtractor::new(llm.clone())));
        let advanced_fact_extractor = Some(Arc::new(AdvancedFactExtractor::new(llm.clone())));

        // 创建批量处理组件
        let batch_entity_extractor = Some(Arc::new(BatchEntityExtractor::new(
            llm.clone(),
            TimeoutConfig::default(),
            BatchConfig::default(),
        )));
        let batch_importance_evaluator = Some(Arc::new(BatchImportanceEvaluator::new(
            llm.clone(),
            TimeoutConfig::default(),
            BatchConfig::default(),
        )));

        let decision_engine = Some(Arc::new(MemoryDecisionEngine::new(llm.clone())));
        let enhanced_decision_engine = Some(Arc::new(EnhancedDecisionEngine::new(
            llm.clone(),
            Default::default(),
        )));
        let importance_evaluator = Some(Arc::new(EnhancedImportanceEvaluator::new(
            llm.clone(),
            Default::default(),
        )));
        let conflict_resolver = Some(Arc::new(ConflictResolver::new(
            llm.clone(),
            Default::default(),
        )));

        info!("Intelligence 组件创建成功（包含批量处理）");

        Ok(IntelligenceComponents {
            fact_extractor,
            advanced_fact_extractor,
            batch_entity_extractor,
            batch_importance_evaluator,
            decision_engine,
            enhanced_decision_engine,
            importance_evaluator,
            conflict_resolver,
            llm_provider,
        })
    }

    /// 创建 LLM Provider
    pub async fn create_llm_provider(
        config: &OrchestratorConfig,
    ) -> Result<Option<Arc<dyn LLMProvider + Send + Sync>>> {
        // 检查是否启用智能功能
        if !config.enable_intelligent_features {
            info!("智能功能未启用，跳过 LLM Provider 创建");
            return Ok(None);
        }

        // 检查是否配置了 LLM Provider
        let provider = match &config.llm_provider {
            Some(p) => p.clone(),
            None => {
                // 尝试从环境变量获取
                match std::env::var("LLM_PROVIDER") {
                    Ok(p) => p,
                    Err(_) => {
                        info!("未配置 LLM Provider，使用默认值: openai");
                        "openai".to_string()
                    }
                }
            }
        };

        // 检查是否配置了 LLM Model
        let model = match &config.llm_model {
            Some(m) => m.clone(),
            None => {
                // 尝试从环境变量获取
                match std::env::var("LLM_MODEL") {
                    Ok(m) => m,
                    Err(_) => {
                        info!("未配置 LLM Model，使用默认值: gpt-4");
                        "gpt-4".to_string()
                    }
                }
            }
        };

        // 智能检测 API Key
        let (final_provider, final_model, api_key) = {
            // 首先尝试当前 provider 的 API Key
            let api_key = match provider.to_lowercase().as_str() {
                "zhipu" => std::env::var("ZHIPU_API_KEY")
                    .or_else(|_| std::env::var("LLM_API_KEY"))
                    .ok(),
                "openai" => std::env::var("OPENAI_API_KEY")
                    .or_else(|_| std::env::var("LLM_API_KEY"))
                    .ok(),
                "anthropic" => std::env::var("ANTHROPIC_API_KEY")
                    .or_else(|_| std::env::var("LLM_API_KEY"))
                    .ok(),
                "deepseek" => std::env::var("DEEPSEEK_API_KEY")
                    .or_else(|_| std::env::var("LLM_API_KEY"))
                    .ok(),
                _ => {
                    // 对于未知的 provider，尝试所有常见的 API Key 环境变量
                    std::env::var("ZHIPU_API_KEY")
                        .or_else(|_| std::env::var("OPENAI_API_KEY"))
                        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
                        .or_else(|_| std::env::var("DEEPSEEK_API_KEY"))
                        .or_else(|_| std::env::var("LLM_API_KEY"))
                        .ok()
                }
            };

            // 如果找到了 API Key，直接返回
            if let Some(key) = api_key {
                (provider.clone(), model.clone(), Some(key))
            } else {
                // 自动检测其他可用的 provider（按优先级）
                info!("当前 provider ({}) 的 API Key 未找到，尝试自动检测其他可用的 provider", provider);

                // 检测 Zhipu
                if let Ok(zhipu_key) = std::env::var("ZHIPU_API_KEY") {
                    let zhipu_model = std::env::var("ZHIPU_MODEL").unwrap_or_else(|_| "glm-4.6".to_string());
                    info!("✅ 检测到 ZHIPU_API_KEY，自动切换到 zhipu provider");
                    return Self::create_llm_provider_with_config("zhipu", &zhipu_model, Some(zhipu_key)).await;
                }

                // 检测 OpenAI
                if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
                    let openai_model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
                    info!("✅ 检测到 OPENAI_API_KEY，自动切换到 openai provider");
                    return Self::create_llm_provider_with_config("openai", &openai_model, Some(openai_key)).await;
                }

                // 检测 Anthropic
                if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
                    let anthropic_model = std::env::var("ANTHROPIC_MODEL")
                        .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
                    info!("✅ 检测到 ANTHROPIC_API_KEY，自动切换到 anthropic provider");
                    return Self::create_llm_provider_with_config("anthropic", &anthropic_model, Some(anthropic_key)).await;
                }

                // 检测 DeepSeek
                if let Ok(deepseek_key) = std::env::var("DEEPSEEK_API_KEY") {
                    let deepseek_model = std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());
                    info!("✅ 检测到 DEEPSEEK_API_KEY，自动切换到 deepseek provider");
                    return Self::create_llm_provider_with_config("deepseek", &deepseek_model, Some(deepseek_key)).await;
                }

                // 检测通用 LLM_API_KEY
                if let Ok(llm_key) = std::env::var("LLM_API_KEY") {
                    info!("✅ 检测到 LLM_API_KEY，使用当前 provider ({})", provider);
                    return Self::create_llm_provider_with_config(&provider, &model, Some(llm_key)).await;
                }

                // 所有检测都失败
                let env_vars = match provider.to_lowercase().as_str() {
                    "zhipu" => "ZHIPU_API_KEY 或 LLM_API_KEY",
                    "openai" => "OPENAI_API_KEY 或 LLM_API_KEY",
                    "anthropic" => "ANTHROPIC_API_KEY 或 LLM_API_KEY",
                    "deepseek" => "DEEPSEEK_API_KEY 或 LLM_API_KEY",
                    _ => "ZHIPU_API_KEY, OPENAI_API_KEY, ANTHROPIC_API_KEY, DEEPSEEK_API_KEY 或 LLM_API_KEY",
                };
                warn!(
                    "未找到任何 LLM API Key 环境变量 (当前 provider: {}, 需要: {})",
                    provider, env_vars
                );
                warn!("LLM API Key 未配置，LLM Provider 将不可用");
                return Ok(None);
            }
        };

        // 创建 LLM Config
        let llm_config = LLMConfig {
            provider: final_provider.clone(),
            model: final_model.clone(),
            api_key,
            base_url: None,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            response_format: None,
        };

        // 使用 LLMFactory 创建 Provider
        match LLMFactory::create_provider(&llm_config) {
            Ok(llm_provider) => {
                info!("成功创建 LLM Provider: {} ({})", final_provider, final_model);
                Ok(Some(llm_provider))
            }
            Err(e) => {
                warn!("创建 LLM Provider 失败: {}", e);
                Ok(None)
            }
        }
    }

    /// 辅助函数：使用指定的配置创建 LLM Provider
    async fn create_llm_provider_with_config(
        provider: &str,
        model: &str,
        api_key: Option<String>,
    ) -> Result<Option<Arc<dyn LLMProvider + Send + Sync>>> {
        let llm_config = LLMConfig {
            provider: provider.to_string(),
            model: model.to_string(),
            api_key,
            base_url: None,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            response_format: None,
        };

        match LLMFactory::create_provider(&llm_config) {
            Ok(llm_provider) => {
                info!("成功创建 LLM Provider: {} ({})", provider, model);
                Ok(Some(llm_provider))
            }
            Err(e) => {
                warn!("创建 LLM Provider 失败: {}", e);
                Ok(None)
            }
        }
    }

    /// 创建 Embedder
    pub async fn create_embedder(
        config: &OrchestratorConfig,
    ) -> Result<Option<Arc<dyn Embedder + Send + Sync>>> {
        // 检查是否配置了 Embedder Provider
        let provider = match &config.embedder_provider {
            Some(p) => p.clone(),
            None => {
                // 尝试从环境变量获取
                match std::env::var("EMBEDDING_PROVIDER") {
                    Ok(p) => p,
                    Err(_) => {
                        info!("未配置 Embedding Provider，使用默认值: fastembed");
                        "fastembed".to_string()
                    }
                }
            }
        };

        // 使用 EmbeddingFactory 创建 Embedder
        match provider.as_str() {
            "fastembed" => {
                #[cfg(feature = "fastembed")]
                {
                    // 获取模型名称（从配置或环境变量）
                    let model = match &config.embedder_model {
                        Some(m) => m.clone(),
                        None => {
                            // 尝试从环境变量获取
                            match std::env::var("FASTEMBED_MODEL") {
                                Ok(m) => m,
                                Err(_) => {
                                    info!("未配置 Embedder Model，使用默认值: multilingual-e5-small");
                                    "multilingual-e5-small".to_string()
                                }
                            }
                        }
                    };

                    match EmbeddingFactory::create_fastembed(&model).await {
                        Ok(embedder) => {
                            let dim = embedder.dimension();
                            info!("成功创建 FastEmbed Embedder ({}, {}维)", model, dim);
                            Ok(Some(embedder))
                        }
                        Err(e) => {
                            warn!("创建 FastEmbed Embedder 失败: {}", e);
                            Ok(None)
                        }
                    }
                }
                #[cfg(not(feature = "fastembed"))]
                {
                    warn!("FastEmbed 特性未启用，无法创建 Embedder");
                    Ok(None)
                }
            }
            "openai" => {
                // 获取 API Key
                let api_key = match std::env::var("OPENAI_API_KEY") {
                    Ok(key) => key,
                    Err(_) => {
                        warn!("未找到 OPENAI_API_KEY 环境变量");
                        return Ok(None);
                    }
                };

                match EmbeddingFactory::create_openai_embedder(api_key).await {
                    Ok(embedder) => {
                        info!("成功创建 OpenAI Embedder (text-embedding-ada-002, 1536维)");
                        Ok(Some(embedder))
                    }
                    Err(e) => {
                        warn!("创建 OpenAI Embedder 失败: {}", e);
                        Ok(None)
                    }
                }
            }
            _ => {
                warn!("不支持的 Embedding Provider: {}", provider);
                Ok(None)
            }
        }
    }

    /// 创建多模态处理组件
    pub async fn create_multimodal_components(
        _config: &OrchestratorConfig,
    ) -> Result<(
        Option<Arc<agent_mem_intelligence::multimodal::image::ImageProcessor>>,
        Option<Arc<agent_mem_intelligence::multimodal::audio::AudioProcessor>>,
        Option<Arc<agent_mem_intelligence::multimodal::video::VideoProcessor>>,
        Option<Arc<agent_mem_intelligence::multimodal::MultimodalProcessorManager>>,
    )> {
        info!("Phase 2: 创建多模态处理组件");

        // 创建图像处理器
        let image_processor =
            Arc::new(agent_mem_intelligence::multimodal::image::ImageProcessor::new());
        info!("✅ ImageProcessor 创建成功");

        // 创建音频处理器
        let audio_processor =
            Arc::new(agent_mem_intelligence::multimodal::audio::AudioProcessor::new());
        info!("✅ AudioProcessor 创建成功");

        // 创建视频处理器
        let video_processor =
            Arc::new(agent_mem_intelligence::multimodal::video::VideoProcessor::new());
        info!("✅ VideoProcessor 创建成功");

        // 创建多模态处理器管理器
        let mut multimodal_manager =
            agent_mem_intelligence::multimodal::MultimodalProcessorManager::new();

        // 注册所有处理器
        use agent_mem_intelligence::multimodal::{ContentType, ProcessorType};
        multimodal_manager.register_processor(
            ContentType::Image,
            ProcessorType::Image(agent_mem_intelligence::multimodal::image::ImageProcessor::new()),
        );
        multimodal_manager.register_processor(
            ContentType::Audio,
            ProcessorType::Audio(agent_mem_intelligence::multimodal::audio::AudioProcessor::new()),
        );
        multimodal_manager.register_processor(
            ContentType::Video,
            ProcessorType::Video(agent_mem_intelligence::multimodal::video::VideoProcessor::new()),
        );
        multimodal_manager.register_processor(
            ContentType::Text,
            ProcessorType::Text(agent_mem_intelligence::multimodal::text::TextProcessor::new()),
        );

        let multimodal_manager_arc = Arc::new(multimodal_manager);
        info!("✅ MultimodalProcessorManager 创建成功，已注册 4 种内容类型处理器");

        Ok((
            Some(image_processor),
            Some(audio_processor),
            Some(video_processor),
            Some(multimodal_manager_arc),
        ))
    }

    /// 创建 OpenAI 多模态 API 客户端
    #[cfg(feature = "multimodal")]
    pub async fn create_openai_multimodal_clients(
        _config: &OrchestratorConfig,
    ) -> Result<(
        Option<Arc<agent_mem_intelligence::multimodal::OpenAIVisionClient>>,
        Option<Arc<agent_mem_intelligence::multimodal::OpenAIWhisperClient>>,
    )> {
        use agent_mem_intelligence::multimodal::ai_models::{AIModelConfig, AIModelProvider};

        info!("创建 OpenAI 多模态 API 客户端");

        // 尝试从环境变量获取 API Key
        let api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                warn!("未找到 OPENAI_API_KEY 环境变量，OpenAI 多模态功能将不可用");
                return Ok((None, None));
            }
        };

        // 创建 Vision 配置
        let vision_config = AIModelConfig {
            provider: AIModelProvider::OpenAI,
            model_name: Some("gpt-4-vision-preview".to_string()),
            api_key: Some(api_key.clone()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            timeout_seconds: 60,
            max_retries: 3,
        };

        // 创建 Whisper 配置
        let whisper_config = AIModelConfig {
            provider: AIModelProvider::OpenAI,
            model_name: Some("whisper-1".to_string()),
            api_key: Some(api_key),
            base_url: Some("https://api.openai.com/v1".to_string()),
            timeout_seconds: 60,
            max_retries: 3,
        };

        // 创建客户端
        let openai_vision =
            match agent_mem_intelligence::multimodal::OpenAIVisionClient::new(vision_config) {
                Ok(client) => {
                    info!("✅ OpenAIVisionClient 创建成功");
                    Some(Arc::new(client))
                }
                Err(e) => {
                    warn!("创建 OpenAIVisionClient 失败: {}", e);
                    None
                }
            };

        let openai_whisper =
            match agent_mem_intelligence::multimodal::OpenAIWhisperClient::new(whisper_config) {
                Ok(client) => {
                    info!("✅ OpenAIWhisperClient 创建成功");
                    Some(Arc::new(client))
                }
                Err(e) => {
                    warn!("创建 OpenAIWhisperClient 失败: {}", e);
                    None
                }
            };

        Ok((openai_vision, openai_whisper))
    }

    /// 创建聚类和推理组件
    pub async fn create_clustering_reasoning_components(
        _config: &OrchestratorConfig,
    ) -> Result<(
        Option<Arc<DBSCANClusterer>>,
        Option<Arc<KMeansClusterer>>,
        Option<Arc<MemoryReasoner>>,
    )> {
        info!("Phase 3: 创建聚类和推理组件");

        // 创建 DBSCAN 聚类器
        let dbscan_clusterer = Arc::new(DBSCANClusterer::new());
        info!("✅ DBSCANClusterer 创建成功");

        // 创建 K-means 聚类器
        let kmeans_clusterer = Arc::new(KMeansClusterer::default());
        info!("✅ KMeansClusterer 创建成功");

        // 创建记忆推理器
        use agent_mem_intelligence::reasoning::ReasoningConfig;
        let reasoning_config = ReasoningConfig::default();
        let memory_reasoner = Arc::new(MemoryReasoner::new(reasoning_config));
        info!("✅ MemoryReasoner 创建成功");

        Ok((
            Some(dbscan_clusterer),
            Some(kmeans_clusterer),
            Some(memory_reasoner),
        ))
    }

    /// 创建向量存储
    pub async fn create_vector_store(
        config: &OrchestratorConfig,
        embedder: Option<&Arc<dyn agent_mem_traits::Embedder + Send + Sync>>,
    ) -> Result<Option<Arc<dyn agent_mem_traits::VectorStore + Send + Sync>>> {
        info!("Phase 6: 创建向量存储");

        use agent_mem_traits::VectorStoreConfig;

        // 获取向量维度（从 Embedder 或使用默认值）
        let vector_dimension = if let Some(emb) = embedder {
            let dim = emb.dimension();
            info!("从 Embedder 获取向量维度: {}", dim);
            dim
        } else {
            let default_dim = 384; // 默认使用 384 维
            warn!("Embedder 未配置，使用默认维度: {}", default_dim);
            default_dim
        };

        // 检查是否配置了vector_store_url
        if let Some(url) = &config.vector_store_url {
            info!("使用配置的向量存储: {}", url);

            // 解析URL格式: "provider://path"
            let (provider, path) = if let Some((prov, p)) = url.split_once("://") {
                (prov, p)
            } else {
                warn!("向量存储URL格式无效: {}，使用内存存储", url);
                ("memory", "")
            };

            // 构建VectorStoreConfig
            let mut store_config = VectorStoreConfig::default();
            store_config.provider = provider.to_string();
            store_config.dimension = Some(vector_dimension);

            // 根据provider设置path或url
            match provider {
                "lancedb" => {
                    store_config.path = path.to_string();
                    store_config.table_name = "memory_vectors".to_string();
                    info!("配置LanceDB: path={}, table={}", path, store_config.table_name);
                }
                "memory" => {
                    info!("使用内存向量存储");
                }
                "chroma" | "qdrant" | "milvus" | "weaviate" => {
                    store_config.url = Some(path.to_string());
                    store_config.collection_name = Some("agent_mem".to_string());
                    info!("配置 {}: url={}, collection=agent_mem", provider, path);
                }
                _ => {
                    warn!("不支持的向量存储provider: {}，使用内存存储", provider);
                    store_config.provider = "memory".to_string();
                }
            }

            // 使用VectorStoreFactory创建向量存储
            use agent_mem_storage::VectorStoreFactory;
            match VectorStoreFactory::create_vector_store(&store_config).await {
                Ok(store) => {
                    info!("✅ 向量存储创建成功（{} 模式，维度: {}）", provider, vector_dimension);
                    Ok(Some(store))
                }
                Err(e) => {
                    warn!("创建向量存储失败: {}，降级到内存存储", e);
                    // 降级到内存存储
                    let mut fallback_config = VectorStoreConfig::default();
                    fallback_config.dimension = Some(vector_dimension);

                    use agent_mem_storage::backends::MemoryVectorStore;
                    match MemoryVectorStore::new(fallback_config).await {
                        Ok(fallback_store) => {
                            info!("✅ 降级到内存向量存储成功（维度: {}）", vector_dimension);
                            Ok(Some(Arc::new(fallback_store) as Arc<dyn agent_mem_traits::VectorStore + Send + Sync>))
                        }
                        Err(e2) => {
                            warn!("创建内存向量存储也失败: {}, 向量存储功能将不可用", e2);
                            Ok(None)
                        }
                    }
                }
            }
        } else {
            // 没有配置时，使用内存存储
            info!("未配置向量存储URL，使用内存存储");

            use agent_mem_storage::backends::MemoryVectorStore;
            let mut store_config = VectorStoreConfig::default();
            store_config.dimension = Some(vector_dimension);

            match MemoryVectorStore::new(store_config).await {
                Ok(store) => {
                    info!("✅ 向量存储创建成功（Memory 模式，维度: {}）", vector_dimension);
                    Ok(Some(Arc::new(store) as Arc<dyn agent_mem_traits::VectorStore + Send + Sync>))
                }
                Err(e) => {
                    warn!("创建向量存储失败: {}, 向量存储功能将不可用", e);
                    Ok(None)
                }
            }
        }
    }

    /// 创建 Search 组件
    #[cfg(feature = "postgres")]
    pub async fn create_search_components(
        config: &OrchestratorConfig,
        vector_store: Option<Arc<dyn agent_mem_traits::VectorStore + Send + Sync>>,
        embedder: Option<Arc<dyn agent_mem_traits::Embedder + Send + Sync>>,
    ) -> Result<(
        Option<Arc<agent_mem_core::search::HybridSearchEngine>>,
        Option<Arc<agent_mem_core::search::VectorSearchEngine>>,
        Option<Arc<agent_mem_core::search::FullTextSearchEngine>>,
    )> {
        use agent_mem_core::search::{HybridSearchEngine, VectorSearchEngine, FullTextSearchEngine};
        use std::sync::Arc;

        info!("创建 Search 组件...");

        // 创建 VectorSearchEngine
        let vector_search_engine = if let Some(store) = &vector_store {
            if let Some(emb) = &embedder {
                let dimension = emb.dimension();
                Some(Arc::new(VectorSearchEngine::new(store.clone(), dimension)))
            } else {
                warn!("Embedder 未配置，VectorSearchEngine 将不可用");
                None
            }
        } else {
            warn!("VectorStore 未配置，VectorSearchEngine 将不可用");
            None
        };

        // 创建 FullTextSearchEngine
        // 注意：FullTextSearchEngine需要PgPool，如果storage_url是PostgreSQL连接，可以创建
        let fulltext_search_engine = if let Some(storage_url) = &config.storage_url {
            // 检查storage_url是否是PostgreSQL格式
            if storage_url.starts_with("postgresql://") || storage_url.starts_with("postgres://") {
                #[cfg(feature = "postgres")]
                {
                    use sqlx::{PgPool, PgPoolOptions};
                    use std::time::Duration;
                    
                    match PgPoolOptions::new()
                        .max_connections(10)
                        .min_connections(2)
                        .acquire_timeout(Duration::from_secs(30))
                        .idle_timeout(Duration::from_secs(600))
                        .max_lifetime(Duration::from_secs(1800))
                        .connect(storage_url)
                        .await
                    {
                        Ok(pool) => {
                            let pool = Arc::new(pool);
                            let engine = FullTextSearchEngine::new(pool);
                            info!("✅ FullTextSearchEngine 创建成功（使用PostgreSQL连接池）");
                            Some(Arc::new(engine))
                        }
                        Err(e) => {
                            warn!("创建PostgreSQL连接池失败: {}，FullTextSearchEngine 将不可用", e);
                            None
                        }
                    }
                }
                #[cfg(not(feature = "postgres"))]
                {
                    warn!("PostgreSQL feature 未启用，FullTextSearchEngine 将不可用");
                    None
                }
            } else {
                warn!("存储URL不是PostgreSQL格式，FullTextSearchEngine 将不可用");
                None
            }
        } else {
            warn!("存储URL未配置，FullTextSearchEngine 将不可用");
            None
        };

        // 创建 HybridSearchEngine
        let hybrid_search_engine = if let (Some(vector_engine), Some(fulltext_engine)) = 
            (vector_search_engine.clone(), fulltext_search_engine.clone()) {
            let hybrid_engine = HybridSearchEngine::with_default_config(
                vector_engine,
                fulltext_engine,
            );
            info!("✅ HybridSearchEngine 创建成功");
            Some(Arc::new(hybrid_engine))
        } else {
            warn!("VectorSearchEngine 或 FullTextSearchEngine 未配置，HybridSearchEngine 将不可用");
            None
        };

        Ok((hybrid_search_engine, vector_search_engine, fulltext_search_engine))
    }

    /// 创建重排序器
    pub fn create_reranker() -> Option<Arc<dyn agent_mem_core::search::Reranker>> {
        use agent_mem_core::search::{RerankerFactory, InternalReranker};
        use std::sync::Arc;
        
        info!("创建重排序器...");
        
        // 默认使用内部重排序器
        let reranker = InternalReranker::new();
        info!("✅ 重排序器创建成功（内部实现）");
        
        Some(Arc::new(reranker))
    }

    /// 创建历史记录管理器
    pub async fn create_history_manager(
        _config: &OrchestratorConfig,
    ) -> Result<Option<Arc<crate::history::HistoryManager>>> {
        info!("Phase 6: 创建历史记录管理器");

        // 使用相对路径
        let history_path = "sqlite:./data/history.db";

        match crate::history::HistoryManager::new(history_path).await {
            Ok(manager) => {
                info!("✅ HistoryManager 创建成功: {}", history_path);
                Ok(Some(Arc::new(manager)))
            }
            Err(e) => {
                warn!("创建 HistoryManager 失败: {}, 历史记录功能将不可用", e);
                Ok(None)
            }
        }
    }
    
    /// 创建LibSQL Memory Operations
    /// 
    /// 用于替代InMemoryOperations，提供持久化存储
    /// 
    /// # Phase 0 Implementation (ag25.md)
    /// 这是Phase 0: 紧急修复的核心函数，确保记忆数据持久化到SQLite
    pub async fn create_libsql_operations(
        db_path: &str,
    ) -> Result<Box<dyn MemoryOperations + Send + Sync>> {
        info!("🔧 Phase 0: 创建 LibSQL Memory Operations: {}", db_path);
        
        // Step 1: 创建连接管理器
        let conn_mgr = LibSqlConnectionManager::new(db_path)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create LibSQL connection manager: {}", e)))?;
        
        info!("✅ LibSQL连接管理器创建成功");
        
        // Step 2: 获取连接
        let conn = conn_mgr.get_connection()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get LibSQL connection: {}", e)))?;
        
        info!("✅ 获取LibSQL连接成功");
        
        // Step 3: 创建repository
        let repo = LibSqlMemoryRepository::new(conn);
        info!("✅ LibSqlMemoryRepository创建成功");
        
        // Step 4: 包装为operations（实现MemoryOperations trait）
        let operations = LibSqlMemoryOperations::new(repo);
        
        info!("✅ Phase 0: LibSQL Memory Operations 创建成功 - 数据将持久化到 {}", db_path);
        Ok(Box::new(operations))
    }
}

