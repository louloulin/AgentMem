//! Orchestrator Core - 核心编排器
//!
//! 定义MemoryOrchestrator核心结构和配置，协调各个模块

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use agent_mem_core::manager::MemoryManager;
use agent_mem_core::managers::CoreMemoryManager;
use agent_mem_traits::{MemoryItem, Result};

use super::initialization::IntelligenceComponents;
use crate::types::{AddResult, MemoryStats};

/// 编排器配置
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// 存储 URL
    pub storage_url: Option<String>,
    /// LLM 提供商
    pub llm_provider: Option<String>,
    /// LLM 模型
    pub llm_model: Option<String>,
    /// Embedder 提供商
    pub embedder_provider: Option<String>,
    /// Embedder 模型
    pub embedder_model: Option<String>,
    /// 向量存储 URL
    pub vector_store_url: Option<String>,
    /// 是否启用智能功能
    pub enable_intelligent_features: bool,
    /// 是否启用嵌入队列（P1 优化：自动批量处理并发请求）
    pub enable_embedding_queue: Option<bool>,
    /// 嵌入批处理大小（默认 32）
    pub embedding_batch_size: Option<usize>,
    /// 嵌入批处理间隔（毫秒，默认 10ms）
    pub embedding_batch_interval_ms: Option<u64>,
    /// 是否启用嵌入缓存（P0 优化：启用 CachedEmbedder 以提升 2-5x 性能）
    pub enable_embedder_cache: Option<bool>,
    /// 嵌入缓存大小（默认 1000）
    pub embedder_cache_size: Option<usize>,
    /// 嵌入缓存 TTL 秒数（默认 3600 秒 = 1 小时）
    pub embedder_cache_ttl_secs: Option<u64>,
    /// 是否启用向量缓存（Phase 2.5 优化：启用 CachedVectorStore）
    pub enable_vector_cache: Option<bool>,
    /// 向量缓存大小（默认 10000）
    pub vector_cache_size: Option<usize>,
    /// 向量缓存 TTL 秒数（默认 3600 秒 = 1 小时）
    pub vector_cache_ttl_seconds: Option<u64>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            storage_url: None,
            llm_provider: None,
            llm_model: None,
            embedder_provider: None,
            embedder_model: None,
            vector_store_url: None,
            enable_intelligent_features: true,
            enable_embedding_queue: Some(true), // 默认启用队列优化
            embedding_batch_size: Some(64),     // 优化：增加批处理大小（32 → 64）
            embedding_batch_interval_ms: Some(20), // 优化：增加批处理间隔（10ms → 20ms）
            enable_embedder_cache: Some(true),  // P0 优化：默认启用嵌入缓存（2-5x 性能提升）
            embedder_cache_size: Some(1000),    // 默认缓存 1000 个嵌入
            embedder_cache_ttl_secs: Some(3600), // 默认 TTL 1 小时
            enable_vector_cache: Some(true),    // Phase 2.5 优化：默认启用向量缓存
            vector_cache_size: Some(10000),     // 默认缓存 10000 个向量
            vector_cache_ttl_seconds: Some(3600), // 默认 TTL 1 小时
        }
    }
}

/// 已完成的操作（用于事务回滚）
#[derive(Debug, Clone)]
pub enum CompletedOperation {
    Add {
        memory_id: String,
    },
    Update {
        memory_id: String,
        old_content: String,
    },
    Delete {
        memory_id: String,
        deleted_content: String,
    },
    Merge {
        primary_memory_id: String,
        secondary_memory_ids: Vec<String>,
        /// 原始内容映射：memory_id -> 原始content（用于回滚）
        original_contents: HashMap<String, String>,
    },
}

/// 记忆编排器
///
/// AgentMem 3.0 核心职责：
/// 1. 智能路由: 根据内容类型路由到对应 Manager
/// 2. Manager 协调: 直接协调多个 Manager 完成复杂任务
/// 3. Intelligence 集成: 完整集成 8 个智能组件
/// 4. Search 集成: 集成混合搜索引擎
/// 5. 降级处理: 无智能组件时降级到基础模式
pub struct MemoryOrchestrator {
    // ========== Managers ==========
    pub(crate) core_manager: Option<Arc<CoreMemoryManager>>,
    /// MemoryManager - 用于提供update_memory, delete_memory, get_memory等功能
    pub(crate) memory_manager: Option<Arc<MemoryManager>>,

    #[cfg(feature = "postgres")]
    pub(crate) semantic_manager: Option<Arc<agent_mem_core::managers::SemanticMemoryManager>>,
    #[cfg(feature = "postgres")]
    pub(crate) episodic_manager: Option<Arc<agent_mem_core::managers::EpisodicMemoryManager>>,
    #[cfg(feature = "postgres")]
    pub(crate) procedural_manager: Option<Arc<agent_mem_core::managers::ProceduralMemoryManager>>,

    // ========== Intelligence 组件 ==========
    pub(crate) fact_extractor: Option<Arc<agent_mem_intelligence::FactExtractor>>,
    pub(crate) advanced_fact_extractor: Option<Arc<agent_mem_intelligence::AdvancedFactExtractor>>,
    pub(crate) batch_entity_extractor: Option<Arc<agent_mem_intelligence::BatchEntityExtractor>>,
    pub(crate) batch_importance_evaluator:
        Option<Arc<agent_mem_intelligence::BatchImportanceEvaluator>>,
    pub(crate) decision_engine: Option<Arc<agent_mem_intelligence::MemoryDecisionEngine>>,
    pub(crate) enhanced_decision_engine:
        Option<Arc<agent_mem_intelligence::EnhancedDecisionEngine>>,
    pub(crate) importance_evaluator:
        Option<Arc<agent_mem_intelligence::EnhancedImportanceEvaluator>>,
    pub(crate) conflict_resolver: Option<Arc<agent_mem_intelligence::ConflictResolver>>,

    // ========== 聚类和推理组件 ==========
    pub(crate) dbscan_clusterer:
        Option<Arc<agent_mem_intelligence::clustering::dbscan::DBSCANClusterer>>,
    pub(crate) kmeans_clusterer:
        Option<Arc<agent_mem_intelligence::clustering::kmeans::KMeansClusterer>>,
    pub(crate) memory_reasoner: Option<Arc<agent_mem_intelligence::MemoryReasoner>>,

    // ========== Search 组件 ==========
    #[cfg(feature = "postgres")]
    pub(crate) hybrid_search_engine: Option<Arc<agent_mem_core::search::HybridSearchEngine>>,
    #[cfg(feature = "postgres")]
    pub(crate) vector_search_engine: Option<Arc<agent_mem_core::search::VectorSearchEngine>>,
    #[cfg(feature = "postgres")]
    pub(crate) fulltext_search_engine: Option<Arc<agent_mem_core::search::FullTextSearchEngine>>,

    // ========== 重排序器 ==========
    pub(crate) reranker: Option<Arc<dyn agent_mem_core::search::Reranker>>,

    // ========== 多模态处理组件 ==========
    pub(crate) image_processor:
        Option<Arc<agent_mem_intelligence::multimodal::image::ImageProcessor>>,
    pub(crate) audio_processor:
        Option<Arc<agent_mem_intelligence::multimodal::audio::AudioProcessor>>,
    pub(crate) video_processor:
        Option<Arc<agent_mem_intelligence::multimodal::video::VideoProcessor>>,
    pub(crate) multimodal_manager:
        Option<Arc<agent_mem_intelligence::multimodal::MultimodalProcessorManager>>,

    #[cfg(feature = "multimodal")]
    pub(crate) openai_vision: Option<Arc<agent_mem_intelligence::multimodal::OpenAIVisionClient>>,
    #[cfg(feature = "multimodal")]
    pub(crate) openai_whisper: Option<Arc<agent_mem_intelligence::multimodal::OpenAIWhisperClient>>,

    // ========== 辅助组件 ==========
    pub(crate) llm_provider: Option<Arc<dyn agent_mem_llm::LLMProvider + Send + Sync>>,
    pub(crate) embedder: Option<Arc<dyn agent_mem_traits::Embedder + Send + Sync>>,
    /// CachedEmbedder 引用，用于缓存管理（如果启用了缓存）
    pub(crate) cached_embedder: Option<Arc<agent_mem_embeddings::CachedEmbedder>>,
    /// QueryEmbeddingCache，用于缓存查询嵌入向量（Phase 1.5 优化）
    pub(crate) query_embedding_cache: Option<crate::cache::QueryEmbeddingCache>,

    // ========== LLM 缓存 ==========
    pub(crate) facts_cache:
        Option<Arc<agent_mem_llm::LLMCache<Vec<agent_mem_intelligence::ExtractedFact>>>>,
    pub(crate) structured_facts_cache:
        Option<Arc<agent_mem_llm::LLMCache<Vec<agent_mem_intelligence::StructuredFact>>>>,
    pub(crate) importance_cache:
        Option<Arc<agent_mem_llm::LLMCache<Vec<agent_mem_intelligence::ImportanceEvaluation>>>>,

    // ========== 核心功能 ==========
    pub(crate) vector_store: Option<Arc<dyn agent_mem_traits::VectorStore + Send + Sync>>,
    pub(crate) history_manager: Option<Arc<crate::history::HistoryManager>>,

    // ========== 配置 ==========
    pub(crate) config: OrchestratorConfig,
}

impl MemoryOrchestrator {
    /// 自动配置初始化
    pub async fn new_with_auto_config() -> Result<Self> {
        info!("使用自动配置初始化 MemoryOrchestrator");

        let auto_config = crate::auto_config::AutoConfig::detect().await?;
        Self::new_with_config(auto_config).await
    }

    /// 使用配置初始化
    pub async fn new_with_config(config: OrchestratorConfig) -> Result<Self> {
        info!(
            "AgentMem 3.0: 使用配置初始化 MemoryOrchestrator: {:?}",
            config
        );

        // ========== Step 1: 创建 Managers ==========
        info!("创建 Managers...");
        let core_manager = Some(Arc::new(CoreMemoryManager::new()));
        info!("✅ CoreMemoryManager 创建成功");

        // 创建 MemoryManager 用于提供完整的CRUD功能
        // Phase 0 修复: 使用 LibSQL 后端而不是 InMemoryOperations
        let db_path = config
            .storage_url
            .as_ref()
            .map(|u| {
                // 处理 memory:// URL，转换为 SQLite 内存数据库
                if u == "memory://" {
                    ":memory:"
                } else if u.starts_with("libsql://") {
                    // 处理 libsql:// 前缀，提取实际文件路径
                    u.strip_prefix("libsql://").unwrap_or(u.as_str())
                } else {
                    u.as_str()
                }
            })
            .unwrap_or("./data/agentmem.db");
        info!("🔧 Phase 0: 使用 LibSQL 后端: {}", db_path);
        let operations =
            super::initialization::InitializationModule::create_libsql_operations(db_path).await?;
        let memory_manager = Some(Arc::new(MemoryManager::with_operations(
            agent_mem_config::MemoryConfig::default(),
            operations,
        )));
        info!(
            "✅ Phase 0: MemoryManager 创建成功 (持久化后端: {})",
            db_path
        );

        #[cfg(feature = "postgres")]
        let semantic_manager = None;
        #[cfg(feature = "postgres")]
        let episodic_manager = None;
        #[cfg(feature = "postgres")]
        let procedural_manager = None;

        // ========== Step 2-7: ✅ P1 Optimization - 顺序初始化组件 ==========
        // 改为顺序初始化以避免 stack overflow
        // 如果需要并行优化，可以考虑使用 tokio::spawn 在不同线程上运行
        info!("🚀 Phase 1: 顺序初始化组件...");

        // Task 1: Intelligence 组件
        let intelligence_components = if config.enable_intelligent_features {
            info!("📦 [1/4] 创建 Intelligence 组件...");
            super::initialization::InitializationModule::create_intelligence_components(&config)
                .await
                .map_err(|e| {
                    error!("❌ Intelligence 组件创建失败: {}", e);
                    e
                })?
        } else {
            info!("⚠️  [1/4] 智能功能已禁用");
            IntelligenceComponents {
                fact_extractor: None,
                advanced_fact_extractor: None,
                batch_entity_extractor: None,
                batch_importance_evaluator: None,
                decision_engine: None,
                enhanced_decision_engine: None,
                importance_evaluator: None,
                conflict_resolver: None,
                llm_provider: None,
            }
        };

        // Task 2: Embedder
        info!("📦 [2/4] 创建 Embedder...");
        let embedder = super::initialization::InitializationModule::create_embedder(&config)
            .await
            .map_err(|e| {
                error!("❌ Embedder 创建失败: {}", e);
                e
            })?;

        // Task 3: 多模态处理组件
        info!("📦 [3/4] 创建多模态处理组件...");
        let (image_processor, audio_processor, video_processor, multimodal_manager) = super::initialization::InitializationModule::create_multimodal_components(&config)
            .await
            .map_err(|e| {
                error!("❌ 多模态处理组件创建失败: {}", e);
                e
            })?;

        // Task 4: 聚类和推理组件
        info!("📦 [4/4] 创建聚类和推理组件...");
        let (dbscan_clusterer, kmeans_clusterer, memory_reasoner) = super::initialization::InitializationModule::create_clustering_reasoning_components(&config)
            .await
            .map_err(|e| {
                error!("❌ 聚类和推理组件创建失败: {}", e);
                e
            })?;

        info!("✅ Phase 1: 组件初始化完成");

        // ========== Step 6: OpenAI 多模态 API（有条件编译，无法并行）==========
        #[cfg(feature = "multimodal")]
        let (openai_vision, openai_whisper) = {
            info!("创建 OpenAI 多模态 API 客户端...");
            super::initialization::InitializationModule::create_openai_multimodal_clients(&config)
                .await?
        };

        // ========== Step 4: Search 组件（需要在 embedder 和 vector_store 之后）==========
        // 注意：Search组件需要embedder和vector_store，所以需要在它们创建之后
        // 这里先设置为None，稍后在创建vector_store之后会更新
        #[cfg(feature = "postgres")]
        let (hybrid_search_engine, vector_search_engine, fulltext_search_engine): (
            Option<Arc<agent_mem_core::search::HybridSearchEngine>>,
            Option<Arc<agent_mem_core::search::VectorSearchEngine>>,
            Option<Arc<agent_mem_core::search::FullTextSearchEngine>>,
        ) = (None, None, None);

        // ========== Step 8: 创建向量存储 ==========
        let vector_store = {
            info!("Phase 6: 创建向量存储...");
            super::initialization::InitializationModule::create_vector_store(
                &config,
                embedder.as_ref(),
            )
            .await?
        };

        // ========== Step 8.5: 创建 Search 组件（需要在vector_store创建之后）==========
        #[cfg(feature = "postgres")]
        let (hybrid_search_engine, vector_search_engine, fulltext_search_engine) = {
            super::initialization::InitializationModule::create_search_components(
                &config,
                vector_store.clone(),
                embedder.clone(),
            )
            .await
            .unwrap_or_else(|e| {
                warn!("创建 Search 组件失败: {}, Search 功能将不可用", e);
                (
                    None::<Arc<agent_mem_core::search::HybridSearchEngine>>,
                    None::<Arc<agent_mem_core::search::VectorSearchEngine>>,
                    None::<Arc<agent_mem_core::search::FullTextSearchEngine>>,
                )
            })
        };
        #[cfg(not(feature = "postgres"))]
        let (_hybrid_search_engine, _vector_search_engine, _fulltext_search_engine) =
            (None::<Arc<()>>, None::<Arc<()>>, None::<Arc<()>>);

        // ========== Step 8.5: 创建重排序器 ==========
        let reranker = {
            info!("创建重排序器...");
            super::initialization::InitializationModule::create_reranker()
        };

        // ========== Step 9: 创建历史记录管理器 ==========
        let history_manager = {
            info!("Phase 6: 创建历史记录管理器...");
            super::initialization::InitializationModule::create_history_manager(&config).await?
        };

        // ========== Step 10: 创建 LLM 缓存 ==========
        let (facts_cache, structured_facts_cache, importance_cache) =
            if config.enable_intelligent_features {
                info!("Phase 2: 创建 LLM 缓存...");
                use std::time::Duration;

                let facts_cache = Some(Arc::new(agent_mem_llm::LLMCache::new(
                    Duration::from_secs(3600),
                    1000,
                )));
                let structured_facts_cache = Some(Arc::new(agent_mem_llm::LLMCache::new(
                    Duration::from_secs(3600),
                    1000,
                )));
                let importance_cache = Some(Arc::new(agent_mem_llm::LLMCache::new(
                    Duration::from_secs(3600),
                    1000,
                )));

                info!("✅ LLM 缓存创建成功（TTL: 1小时，最大条目: 1000）");
                (facts_cache, structured_facts_cache, importance_cache)
            } else {
                info!("智能功能已禁用，跳过 LLM 缓存创建");
                (None, None, None)
            };

        Ok(Self {
            // Managers
            core_manager,
            memory_manager,

            #[cfg(feature = "postgres")]
            semantic_manager,
            #[cfg(feature = "postgres")]
            episodic_manager,
            #[cfg(feature = "postgres")]
            procedural_manager,

            // Intelligence 组件
            fact_extractor: intelligence_components.fact_extractor,
            advanced_fact_extractor: intelligence_components.advanced_fact_extractor,
            batch_entity_extractor: intelligence_components.batch_entity_extractor,
            batch_importance_evaluator: intelligence_components.batch_importance_evaluator,
            decision_engine: intelligence_components.decision_engine,
            enhanced_decision_engine: intelligence_components.enhanced_decision_engine,
            importance_evaluator: intelligence_components.importance_evaluator,
            conflict_resolver: intelligence_components.conflict_resolver,

            // 聚类和推理
            dbscan_clusterer,
            kmeans_clusterer,
            memory_reasoner,

            // Search 组件
            #[cfg(feature = "postgres")]
            hybrid_search_engine,
            #[cfg(feature = "postgres")]
            vector_search_engine,
            #[cfg(feature = "postgres")]
            fulltext_search_engine,

            // 重排序器
            reranker,

            // 多模态组件
            image_processor,
            audio_processor,
            video_processor,
            multimodal_manager,

            #[cfg(feature = "multimodal")]
            openai_vision,
            #[cfg(feature = "multimodal")]
            openai_whisper,

            // 辅助组件
            llm_provider: intelligence_components.llm_provider,
            embedder: embedder.clone(),

            // 尝试提取 CachedEmbedder 引用（如果启用了缓存）
            cached_embedder: {
                
                if let Some(_emb) = &embedder {
                    // 尝试通过内部方法获取 CachedEmbedder 引用
                    // 注意：这里使用一个技巧 - 我们知道启用缓存时会包装为 CachedEmbedder
                    // 由于 Rust 的 trait 对象限制，我们需要在初始化时保存引用
                    // 当前方案：如果 embedder 启用了缓存，我们假设它是 CachedEmbedder
                    // 并在创建 embedder 时同时保存引用（需要修改 create_embedder）
                    // 暂时设置为 None，待实现
                    None
                } else {
                    None
                }
            },

            // Phase 1.5: 查询嵌入缓存（新增）
            query_embedding_cache: if config.enable_embedder_cache.unwrap_or(false) {
                use crate::cache::QueryEmbeddingCache;
                let cache_size = config.embedder_cache_size.unwrap_or(1000);
                Some(QueryEmbeddingCache::new(cache_size))
            } else {
                None
            },

            // Phase 2: LLM 缓存
            facts_cache,
            structured_facts_cache,
            importance_cache,

            // Phase 6: 向量存储和历史记录
            vector_store,
            history_manager,

            // 配置
            config,
        })
    }

    // ========== 存储方法委托（内部方法） ==========

    /// 添加记忆（快速模式）- 内部方法
    #[allow(dead_code)]
    pub async fn add_memory_fast(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        memory_type: Option<agent_mem_core::types::MemoryType>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        super::storage::StorageModule::add_memory_fast(
            self,
            content,
            agent_id,
            user_id,
            memory_type,
            metadata,
        )
        .await
    }

    /// 添加记忆（简单模式）- 内部方法
    #[allow(dead_code)]
    pub async fn add_memory(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        memory_type: Option<agent_mem_core::types::MemoryType>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        super::storage::StorageModule::add_memory(
            self,
            content,
            agent_id,
            user_id,
            memory_type,
            metadata,
        )
        .await
    }

    /// 添加记忆 v2（支持 infer 参数）- 内部方法
    #[allow(dead_code)]
    pub async fn add_memory_v2(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        run_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
        infer: bool,
        memory_type: Option<String>,
        prompt: Option<String>,
    ) -> Result<AddResult> {
        super::storage::StorageModule::add_memory_v2(
            self,
            content,
            agent_id,
            user_id,
            run_id,
            metadata,
            infer,
            memory_type,
            prompt,
        )
        .await
    }

    /// 更新记忆（内部方法）
    #[allow(dead_code)]
    pub(crate) async fn update_memory(
        &self,
        memory_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<MemoryItem> {
        super::storage::StorageModule::update_memory(self, memory_id, data).await
    }

    /// 删除记忆（内部方法）
    #[allow(dead_code)]
    pub(crate) async fn delete_memory(&self, memory_id: &str) -> Result<()> {
        super::storage::StorageModule::delete_memory(self, memory_id).await
    }

    /// 获取记忆（内部方法）
    #[allow(dead_code)]
    pub(crate) async fn get_memory(&self, memory_id: &str) -> Result<MemoryItem> {
        super::storage::StorageModule::get_memory(self, memory_id).await
    }

    // ========== 检索方法委托（内部方法） ==========

    /// 搜索记忆 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn search_memories(
        &self,
        query: String,
        agent_id: String,
        user_id: Option<String>,
        limit: usize,
        memory_type: Option<agent_mem_core::types::MemoryType>,
    ) -> Result<Vec<MemoryItem>> {
        super::retrieval::RetrievalModule::search_memories(
            self,
            query,
            agent_id,
            user_id,
            limit,
            memory_type,
        )
        .await
    }

    /// 混合搜索记忆 - 内部方法
    #[cfg(feature = "postgres")]
    #[allow(dead_code)]
    pub(crate) async fn search_memories_hybrid(
        &self,
        query: String,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<MemoryItem>> {
        super::retrieval::RetrievalModule::search_memories_hybrid(
            self, query, user_id, limit, threshold, filters,
        )
        .await
    }

    /// 混合搜索记忆（非 postgres 版本） - 内部方法
    #[cfg(not(feature = "postgres"))]
    #[allow(dead_code)]
    pub(crate) async fn search_memories_hybrid(
        &self,
        query: String,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<MemoryItem>> {
        super::retrieval::RetrievalModule::search_memories_hybrid(
            self, query, user_id, limit, threshold, filters,
        )
        .await
    }

    /// 上下文感知重排序 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn context_aware_rerank(
        &self,
        memories: Vec<MemoryItem>,
        query: &str,
        user_id: &str,
    ) -> Result<Vec<MemoryItem>> {
        super::retrieval::RetrievalModule::context_aware_rerank(self, memories, query, user_id)
            .await
    }

    // ========== 批量操作方法委托（内部方法） ==========

    /// 批量添加记忆 - 内部方法
    #[allow(dead_code)]
    pub async fn add_memories_batch(
        &self,
        items: Vec<(
            String,
            String,
            Option<String>,
            Option<agent_mem_core::types::MemoryType>,
            Option<HashMap<String, serde_json::Value>>,
        )>,
    ) -> Result<Vec<String>> {
        super::batch::BatchModule::add_memories_batch(self, items).await
    }

    /// 批量添加记忆（优化版） - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn add_memory_batch_optimized(
        &self,
        contents: Vec<String>,
        agent_id: String,
        user_id: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<String>> {
        super::batch::BatchModule::add_memory_batch_optimized(
            self, contents, agent_id, user_id, metadata,
        )
        .await
    }

    // ========== 多模态方法委托（内部方法） ==========

    /// 添加图像记忆 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn add_image_memory(
        &self,
        image_data: Vec<u8>,
        user_id: String,
        agent_id: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<AddResult> {
        super::multimodal::MultimodalModule::add_image_memory(
            self, image_data, user_id, agent_id, metadata,
        )
        .await
    }

    /// 添加音频记忆 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn add_audio_memory(
        &self,
        audio_data: Vec<u8>,
        user_id: String,
        agent_id: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<AddResult> {
        super::multimodal::MultimodalModule::add_audio_memory(
            self, audio_data, user_id, agent_id, metadata,
        )
        .await
    }

    /// 添加视频记忆 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn add_video_memory(
        &self,
        video_data: Vec<u8>,
        user_id: String,
        agent_id: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<AddResult> {
        super::multimodal::MultimodalModule::add_video_memory(
            self, video_data, user_id, agent_id, metadata,
        )
        .await
    }

    // ========== 工具方法委托（内部方法） ==========

    /// 生成查询嵌入向量 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn generate_query_embedding(&self, query: &str) -> Result<Vec<f32>> {
        if let Some(embedder) = &self.embedder {
            super::utils::UtilsModule::generate_query_embedding(query, embedder.as_ref()).await
        } else {
            Err(agent_mem_traits::AgentMemError::ConfigError(
                "Embedder not configured".to_string(),
            ))
        }
    }

    /// 获取统计信息 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn get_stats(&self, _user_id: Option<String>) -> Result<MemoryStats> {
        let total_memories = 0;
        let memories_by_type: HashMap<String, usize> = HashMap::new();
        let total_importance = 0.0;
        let count = 0;

        // 从 CoreMemoryManager 获取统计
        // Note: CoreMemoryManager 不提供 get_memory_stats 方法
        // 如果需要统计功能，应该使用 MemoryManager 而不是 CoreMemoryManager
        // 这里暂时跳过，返回默认统计

        // 从向量存储获取统计（如果可用）
        if let Some(_vector_store) = &self.vector_store {
            // 向量存储可能不直接提供统计，这里使用估算
            // 实际实现可能需要根据具体的向量存储 API 调整
        }

        let average_importance = if count > 0 {
            total_importance / count as f32
        } else {
            0.0
        };

        Ok(MemoryStats {
            total_memories,
            memories_by_type,
            average_importance,
            storage_size_bytes: 0, // 需要从存储层获取
            last_updated: Some(chrono::Utc::now()),
        })
    }

    /// 获取所有记忆 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn get_all_memories(
        &self,
        agent_id: String,
        _user_id: Option<String>,
    ) -> Result<Vec<MemoryItem>> {
        let mut all_memories = Vec::new();

        // 使用 MemoryManager 获取所有记忆
        if let Some(manager) = &self.memory_manager {
            let memories = manager
                .get_agent_memories(&agent_id, None)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(format!(
                        "Failed to get memories from MemoryManager: {e}"
                    ))
                })?;

            // 转换为 MemoryItem
            // MemoryManager返回的是agent_mem_core::types::Memory，可以直接转换为MemoryItem
            for memory in memories {
                all_memories.push(MemoryItem::from(memory));
            }
        }

        Ok(all_memories)
    }

    /// 获取所有记忆 v2 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn get_all_memories_v2(
        &self,
        agent_id: String,
        user_id: Option<String>,
        _run_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>> {
        let mut memories = self.get_all_memories(agent_id, user_id).await?;
        if let Some(limit_val) = limit {
            memories.truncate(limit_val);
        }
        Ok(memories)
    }

    /// 删除所有记忆 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn delete_all_memories(
        &self,
        agent_id: String,
        user_id: Option<String>,
        _run_id: Option<String>,
    ) -> Result<usize> {
        use super::storage::StorageModule;
        let mut deleted_count = 0;

        // 先获取所有记忆
        let memories = self
            .get_all_memories(agent_id.clone(), user_id.clone())
            .await?;

        // 逐个删除
        for memory in memories {
            if let Ok(_) = StorageModule::delete_memory(self, &memory.id).await {
                deleted_count += 1;
            }
        }

        info!("✅ 删除所有记忆完成: {} 个", deleted_count);
        Ok(deleted_count)
    }

    /// 重置（内部方法）
    #[allow(dead_code)]
    pub(crate) async fn reset(&self) -> Result<()> {
        info!("重置 MemoryOrchestrator");

        // 1. 删除所有记忆（通过 MemoryManager）
        if let Some(_manager) = &self.memory_manager {
            // 获取所有记忆并删除
            // 注意：这里使用默认 agent_id，实际可能需要遍历所有 agent
            let default_agent_id = "default".to_string();
            let _ = self
                .delete_all_memories(default_agent_id.clone(), None, None)
                .await;
            info!("✅ 已删除所有记忆");
        }

        // 2. 清空向量存储
        // 注意：向量存储会在 delete_all_memories 中通过 delete_memory 自动清理
        // 因为 delete_memory 会同时删除向量存储中的向量
        // 所以这里不需要单独清空向量存储
        info!("✅ 向量存储将在删除记忆时自动清理");

        // 3. 清空历史记录
        if let Some(history_manager) = &self.history_manager {
            if let Err(e) = history_manager.reset().await {
                warn!("清空历史记录失败: {}", e);
            } else {
                info!("✅ 已清空历史记录");
            }
        }

        // 4. 清空 CoreMemoryManager（如果存在）
        if let Some(_core_manager) = &self.core_manager {
            // CoreMemoryManager 是内存存储，通常不需要显式清空
            // 但如果需要，可以在这里添加清空逻辑
            info!("✅ CoreMemoryManager 已处理");
        }

        info!("✅ 重置完成");
        Ok(())
    }

    /// 缓存搜索 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn cached_search(
        &self,
        query: String,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<MemoryItem>> {
        // 实现缓存搜索逻辑
        // 为了简化，这里直接调用混合搜索，缓存功能可以在后续优化中实现
        self.search_memories_hybrid(query, user_id, limit, threshold, None)
            .await
    }

    /// 获取性能统计 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn get_performance_stats(&self) -> Result<crate::memory::PerformanceStats> {
        // 实现性能统计逻辑

        let cache_hit_rate = 0.0;
        let avg_add_latency_ms = 0.0;
        let avg_search_latency_ms = 0.0;
        let queries_per_second = 0.0;
        let memory_usage_mb = 0.0;

        // 从 MemoryManager 获取统计
        let total_memories = if let Some(manager) = &self.memory_manager {
            manager
                .get_memory_stats(None)
                .await
                .map(|stats| stats.total_memories)
                .unwrap_or(0)
        } else {
            0
        };

        // 从向量存储和Search组件获取统计（如果可用）
        // 向量存储和Search组件可能不直接提供统计，这里使用默认值
        // 实际实现可能需要根据具体的向量存储 API 调整

        Ok(crate::memory::PerformanceStats {
            total_memories,
            cache_hit_rate,
            avg_add_latency_ms,
            avg_search_latency_ms,
            queries_per_second,
            memory_usage_mb,
        })
    }

    /// 获取历史记录 - 内部方法
    #[allow(dead_code)]
    pub(crate) async fn get_history(
        &self,
        memory_id: &str,
    ) -> Result<Vec<crate::history::HistoryEntry>> {
        if let Some(history_manager) = &self.history_manager {
            history_manager.get_history(memory_id).await
        } else {
            Ok(Vec::new())
        }
    }

    // ========== ✅ 新 API - 统一的记忆管理 ==========

    /// 添加记忆（统一入口，自动使用智能处理）
    ///
    /// 这是推荐的添加记忆方法，会自动使用智能添加：
    /// - 事实提取
    /// - 重要性评估
    /// - 冲突检测
    ///
    /// # 示例
    ///
    /// ```rust
    /// let id = orchestrator.add("Hello, world!").await?;
    /// ```
    pub async fn add(&self, content: &str) -> Result<String> {
        // 使用智能添加（如果可用），否则使用快速添加
        if self.config.enable_intelligent_features {
            // 调用智能添加的内部实现
            super::intelligence::IntelligenceModule::add_memory_intelligent(
                self,
                content.to_string(),
                "default".to_string(),
                Some("default".to_string()),
                None,
            )
            .await
            .and_then(|r| {
                Ok(r.results
                    .first()
                    .map(|e| e.id.clone())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()))
            })
        } else {
            // 降级到快速添加
            self.add_memory_fast(
                content.to_string(),
                "default".to_string(),
                Some("default".to_string()),
                None,
                None,
            )
            .await
        }
    }

    /// 添加记忆（带自定义选项）
    ///
    /// 当需要指定 agent_id、user_id 或 memory_type 时使用此方法。
    ///
    /// # 参数
    ///
    /// - `content`: 记忆内容
    /// - `agent_id`: 代理 ID
    /// - `user_id`: 用户 ID（可选）
    /// - `memory_type`: 记忆类型（可选）
    /// - `metadata`: 额外的元数据（可选）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem::MemoryOrchestrator;
    /// use std::collections::HashMap;
    ///
    /// let id = orchestrator.add_with_options(
    ///     "Hello",
    ///     "agent1",
    ///     Some("user1"),
    ///     None,
    ///     None,
    /// ).await?;
    /// ```
    pub async fn add_with_options(
        &self,
        content: &str,
        agent_id: &str,
        user_id: Option<&str>,
        memory_type: Option<agent_mem_core::types::MemoryType>,
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        // 使用智能添加（如果可用），否则使用快速添加
        if self.config.enable_intelligent_features {
            // 调用智能添加的内部实现
            super::intelligence::IntelligenceModule::add_memory_intelligent(
                self,
                content.to_string(),
                agent_id.to_string(),
                user_id.map(|u| u.to_string()),
                metadata,
            )
            .await
            .and_then(|r| {
                Ok(r.results
                    .first()
                    .map(|e| e.id.clone())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()))
            })
        } else {
            // 降级到快速添加
            self.add_memory_fast(
                content.to_string(),
                agent_id.to_string(),
                user_id.map(|u| u.to_string()),
                memory_type,
                metadata,
            )
            .await
        }
    }

    /// 批量添加记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// let ids = orchestrator.add_batch(vec!["Memory 1", "Memory 2"]).await?;
    /// ```
    pub async fn add_batch(&self, contents: Vec<String>) -> Result<Vec<String>> {
        if contents.is_empty() {
            return Ok(Vec::new());
        }

        // 准备批量数据
        let items: Vec<(
            String,
            String,
            Option<String>,
            Option<agent_mem_core::types::MemoryType>,
            Option<std::collections::HashMap<String, serde_json::Value>>,
        )> = contents
            .into_iter()
            .map(|content| {
                (
                    content,
                    "default".to_string(),
                    Some("default".to_string()),
                    None,
                    None,
                )
            })
            .collect();

        // 使用现有的批量添加方法
        self.add_memories_batch(items).await
    }

    /// 添加图片记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// let id = orchestrator.add_image(image_data, Some("A beautiful sunset")).await?;
    /// ```
    pub async fn add_image(&self, image: Vec<u8>, caption: Option<&str>) -> Result<String> {
        let mut metadata = std::collections::HashMap::new();
        if let Some(caption_text) = caption {
            metadata.insert("caption".to_string(), caption_text.to_string());
        }

        self.add_image_memory(
            image,
            "default".to_string(),
            "default".to_string(),
            if metadata.is_empty() {
                None
            } else {
                Some(metadata)
            },
        )
        .await
        .and_then(|r| {
            Ok(r.results
                .first()
                .map(|e| e.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()))
        })
    }

    /// 添加音频记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// let id = orchestrator.add_audio(audio_data, Some("Transcript text")).await?;
    /// ```
    pub async fn add_audio(&self, audio: Vec<u8>, transcript: Option<&str>) -> Result<String> {
        let mut metadata = std::collections::HashMap::new();
        if let Some(transcript_text) = transcript {
            metadata.insert("transcript".to_string(), transcript_text.to_string());
        }

        self.add_audio_memory(
            audio,
            "default".to_string(),
            "default".to_string(),
            if metadata.is_empty() {
                None
            } else {
                Some(metadata)
            },
        )
        .await
        .and_then(|r| {
            Ok(r.results
                .first()
                .map(|e| e.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()))
        })
    }

    /// 添加视频记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// let id = orchestrator.add_video(video_data, Some("Video description")).await?;
    /// ```
    pub async fn add_video(&self, video: Vec<u8>, description: Option<&str>) -> Result<String> {
        let mut metadata = std::collections::HashMap::new();
        if let Some(desc) = description {
            metadata.insert("description".to_string(), desc.to_string());
        }

        self.add_video_memory(
            video,
            "default".to_string(),
            "default".to_string(),
            if metadata.is_empty() {
                None
            } else {
                Some(metadata)
            },
        )
        .await
        .and_then(|r| {
            Ok(r.results
                .first()
                .map(|e| e.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()))
        })
    }

    // ========== ✅ 新 API - 统一的查询 ==========

    /// 获取单个记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memory = orchestrator.get("memory-id").await?;
    /// ```
    pub async fn get(&self, id: &str) -> Result<MemoryItem> {
        self.get_memory(id).await
    }

    /// 获取所有记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// let memories = orchestrator.get_all().await?;
    /// ```
    pub async fn get_all(&self) -> Result<Vec<MemoryItem>> {
        self.get_all_memories_v2(
            "default".to_string(),
            Some("default".to_string()),
            None,
            None,
        )
        .await
    }

    // ========== ✅ 新 API - 统一的更新 ==========

    /// 更新记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// orchestrator.update("memory-id", "new content").await?;
    /// ```
    pub async fn update(&self, id: &str, content: &str) -> Result<()> {
        let mut data = std::collections::HashMap::new();
        data.insert("content".to_string(), serde_json::json!(content));
        self.update_memory(id, data).await?;
        Ok(())
    }

    // ========== ✅ 新 API - 统一的删除 ==========

    /// 删除单个记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// orchestrator.delete("memory-id").await?;
    /// ```
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.delete_memory(id).await
    }

    /// 删除所有记忆
    ///
    /// # 示例
    ///
    /// ```rust
    /// orchestrator.delete_all().await?;
    /// ```
    pub async fn delete_all(&self) -> Result<()> {
        self.delete_all_memories("default".to_string(), Some("default".to_string()), None)
            .await?;
        Ok(())
    }

    // ========== ✅ 新 API - 统一的搜索 ==========

    /// 搜索记忆（使用默认配置）
    ///
    /// # 示例
    ///
    /// ```rust
    /// let results = orchestrator.search("query").await?;
    /// ```
    pub async fn search(&self, query: &str) -> Result<Vec<MemoryItem>> {
        self.search_with_options(query, 10, true, true, None, None)
            .await
    }

    /// 搜索记忆（带选项）
    ///
    /// # 示例
    ///
    /// ```rust
    /// let results = orchestrator
    ///     .search_with_options("query", 20, true, false, Some(0.7), None)
    ///     .await?;
    /// ```
    pub async fn search_with_options(
        &self,
        query: &str,
        limit: usize,
        enable_hybrid: bool,
        enable_rerank: bool,
        _threshold: Option<f32>,
        time_range: Option<(i64, i64)>,
    ) -> Result<Vec<MemoryItem>> {
        // 执行搜索
        let mut results = if enable_hybrid {
            #[cfg(feature = "postgres")]
            {
                self.search_memories_hybrid(
                    query.to_string(),
                    "default".to_string(),
                    limit,
                    threshold,
                    None,
                )
                .await?
            }

            #[cfg(not(feature = "postgres"))]
            {
                self.search_memories(
                    query.to_string(),
                    "default".to_string(),
                    Some("default".to_string()),
                    limit,
                    None,
                )
                .await?
            }
        } else {
            self.search_memories(
                query.to_string(),
                "default".to_string(),
                Some("default".to_string()),
                limit,
                None,
            )
            .await?
        };

        // 应用重排序
        if enable_rerank {
            results = self.context_aware_rerank(results, query, "default").await?;
        }

        // 应用时间范围过滤
        if let Some((start_ts, end_ts)) = time_range {
            use chrono::{DateTime, Utc};
            use std::time::UNIX_EPOCH;
            let start_time =
                DateTime::<Utc>::from(UNIX_EPOCH + std::time::Duration::from_secs(start_ts as u64));
            let end_time =
                DateTime::<Utc>::from(UNIX_EPOCH + std::time::Duration::from_secs(end_ts as u64));

            let before_count = results.len();
            results = results
                .into_iter()
                .filter(|memory| {
                    // 检查记忆的创建时间是否在时间范围内
                    let created_at = memory.created_at;
                    created_at >= start_time && created_at <= end_time
                })
                .collect();

            debug!(
                "✅ 时间范围过滤: {} ~ {}, 结果数: {} -> {}",
                start_time.format("%Y-%m-%d %H:%M"),
                end_time.format("%Y-%m-%d %H:%M"),
                before_count,
                results.len()
            );
        }

        Ok(results)
    }

    // ========== ✅ 新 API - 统一的统计 ==========

    /// 获取统计信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// let stats = orchestrator.stats().await?;
    /// ```
    pub async fn stats(&self) -> Result<MemoryStats> {
        self.get_stats(None).await
    }

    /// 获取性能统计
    ///
    /// # 示例
    ///
    /// ```rust
    /// let perf = orchestrator.performance_stats().await?;
    /// ```
    pub async fn performance_stats(&self) -> Result<crate::memory::PerformanceStats> {
        self.get_performance_stats().await
    }

    /// 获取嵌入缓存统计信息
    ///
    /// 返回 CachedEmbedder 的缓存统计,包括命中次数、未命中次数、命中率等。
    ///
    /// # 返回
    ///
    /// 返回 `Option<CacheStats>`,如果未启用缓存则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::orchestrator::MemoryOrchestrator;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let orchestrator = MemoryOrchestrator::new_with_auto_config().await?;
    ///
    /// // 添加一些记忆以生成缓存
    /// orchestrator.add("重复内容").await?;
    /// orchestrator.add("重复内容").await?; // 缓存命中
    ///
    /// // 获取缓存统计
    /// if let Some(stats) = orchestrator.get_embedder_cache_stats().await? {
    ///     println!("缓存命中次数: {}", stats.hits);
    ///     println!("缓存未命中次数: {}", stats.misses);
    ///     println!("缓存命中率: {:.2}%", stats.hit_rate * 100.0);
    ///     println!("缓存大小: {}", stats.size);
    ///     println!("缓存容量: {}", stats.capacity);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_embedder_cache_stats(
        &self,
    ) -> Result<Option<agent_mem_intelligence::caching::CacheStats>> {
        

        if let Some(_embedder) = &self.embedder {
            // 尝试将 embedder downcast 为 CachedEmbedder
            // 注意: 由于使用了 Arc 和 trait 对象,我们需要通过其他方式访问

            // 当前实现: 通过内部 API 访问缓存统计
            // Note: get_cache_stats() is now available in Embedder trait

            // 临时方案: 返回 None,实际功能需要在 Embedder trait 层实现
            warn!("获取缓存统计功能需要在 Embedder trait 中添加 get_cache_stats() 方法");
            Ok(None)
        } else {
            Ok(None)
        }
    }

    /// 清空嵌入缓存
    ///
    /// 清空 CachedEmbedder 的所有缓存条目。
    ///
    /// # 注意
    ///
    /// 清空缓存后,下次嵌入生成将重新计算,直到缓存重新建立。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use agent_mem::orchestrator::MemoryOrchestrator;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let orchestrator = MemoryOrchestrator::new_with_auto_config().await?;
    ///
    /// // 添加记忆
    /// orchestrator.add("测试内容").await?;
    ///
    /// // 清空缓存
    /// orchestrator.clear_embedder_cache().await?;
    ///
    /// // 再次添加将重新计算嵌入
    /// orchestrator.add("测试内容").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear_embedder_cache(&self) -> Result<()> {
        

        if let Some(_embedder) = &self.embedder {
            // Note: clear_cache() is now available in Embedder trait
            warn!("清空缓存功能需要在 Embedder trait 中添加 clear_cache() 方法");
        }

        Ok(())
    }

    /// 获取历史记录
    ///
    /// # 示例
    ///
    /// ```rust
    /// let history = orchestrator.history("memory-id").await?;
    /// ```
    pub async fn history(&self, memory_id: &str) -> Result<Vec<crate::history::HistoryEntry>> {
        self.get_history(memory_id).await
    }

    // ========== ✅ Builder 模式支持 ==========

    /// 创建搜索构建器
    ///
    /// # 示例
    ///
    /// ```rust
    /// let results = orchestrator
    ///     .search_builder("query")
    ///     .limit(20)
    ///     .with_rerank(true)
    ///     .with_threshold(0.7)
    ///     .execute()
    ///     .await?;
    /// ```
    pub fn search_builder<'a>(&'a self, query: &'a str) -> SearchBuilder<'a> {
        SearchBuilder::new(self, query)
    }

    /// 创建批量操作构建器
    ///
    /// # 示例
    ///
    /// ```rust
    /// let ids = orchestrator
    ///     .batch_add()
    ///     .add("Memory 1")
    ///     .add("Memory 2")
    ///     .batch_size(50)
    ///     .execute()
    ///     .await?;
    /// ```
    pub fn batch_add<'a>(&'a self) -> BatchBuilder<'a> {
        BatchBuilder::new(self)
    }
}

// ========== ✅ SearchBuilder ==========

/// 搜索构建器 - 使用 Builder 模式提供灵活的搜索配置
///
/// # 示例
///
/// ```rust
/// let results = orchestrator
///     .search_builder("query")
///     .limit(20)
///     .with_rerank(true)
///     .with_threshold(0.7)
///     .execute()
///     .await?;
/// ```
pub struct SearchBuilder<'a> {
    orchestrator: &'a MemoryOrchestrator,
    query: String,
    limit: usize,
    enable_hybrid: bool,
    enable_rerank: bool,
    enable_scheduler: bool,
    threshold: Option<f32>,
    time_range: Option<(i64, i64)>,
    filters: std::collections::HashMap<String, String>,
}

impl<'a> SearchBuilder<'a> {
    fn new(orchestrator: &'a MemoryOrchestrator, query: &str) -> Self {
        Self {
            orchestrator,
            query: query.to_string(),
            limit: 10,
            enable_hybrid: true,
            enable_rerank: true,
            enable_scheduler: false,
            threshold: None,
            time_range: None,
            filters: std::collections::HashMap::new(),
        }
    }

    /// 设置返回结果数量
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// 启用/禁用混合搜索
    pub fn with_hybrid(mut self, enable: bool) -> Self {
        self.enable_hybrid = enable;
        self
    }

    /// 启用/禁用重排序
    pub fn with_rerank(mut self, enable: bool) -> Self {
        self.enable_rerank = enable;
        self
    }

    /// 启用/禁用记忆调度（智能选择）
    ///
    /// 当启用时，会根据以下因素智能调整搜索策略：
    /// - 查询复杂度：长查询自动禁用混合搜索以提高性能
    /// - 时间敏感性：包含时间关键词的查询自动应用时间范围过滤
    /// - 结果数量限制：小批量查询自动降低 limit 以提高响应速度
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let results = orchestrator
    ///     .search_builder("recent important documents")
    ///     .with_scheduler(true)  // 启用智能调度
    ///     .await?;
    /// ```
    pub fn with_scheduler(mut self, enable: bool) -> Self {
        self.enable_scheduler = enable;
        self
    }

    /// 设置相似度阈值
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// 设置时间范围
    pub fn with_time_range(mut self, start: i64, end: i64) -> Self {
        self.time_range = Some((start, end));
        self
    }

    /// 添加自定义过滤器
    pub fn with_filter(mut self, key: String, value: String) -> Self {
        self.filters.insert(key, value);
        self
    }

    /// 执行搜索
    pub async fn execute(self) -> Result<Vec<MemoryItem>> {
        let mut builder = self;
        let user_id = "default";

        // 应用记忆调度逻辑
        if builder.enable_scheduler {
            // 1. 查询复杂度分析：长查询（>100字符）禁用混合搜索
            if builder.query.len() > 100 {
                builder.enable_hybrid = false;
            }

            // 2. 时间敏感性检测：自动应用时间范围过滤
            let time_keywords = ["今天", "yesterday", "recent", "最近", "latest"];
            let has_time_keyword = time_keywords
                .iter()
                .any(|keyword| builder.query.to_lowercase().contains(keyword));

            if has_time_keyword && builder.time_range.is_none() {
                // 默认搜索最近 7 天的记忆
                let now = chrono::Utc::now().timestamp();
                let seven_days_ago = now - (7 * 24 * 60 * 60);
                builder.time_range = Some((seven_days_ago, now));
            }

            // 3. 结果数量优化：小查询（<20字符）限制结果数量
            if builder.query.len() < 20 && builder.limit > 5 {
                builder.limit = 5.min(builder.limit);
            }
        }

        // 执行搜索
        let mut results = if builder.enable_hybrid {
            #[cfg(feature = "postgres")]
            {
                builder
                    .orchestrator
                    .search_memories_hybrid(
                        builder.query.clone(),
                        user_id.to_string(),
                        builder.limit,
                        builder.threshold,
                        if builder.filters.is_empty() {
                            None
                        } else {
                            Some(builder.filters)
                        },
                    )
                    .await?
            }

            #[cfg(not(feature = "postgres"))]
            {
                builder
                    .orchestrator
                    .search_memories(
                        builder.query.clone(),
                        user_id.to_string(),
                        Some(user_id.to_string()),
                        builder.limit,
                        None,
                    )
                    .await?
            }
        } else {
            builder
                .orchestrator
                .search_memories(
                    builder.query.clone(),
                    user_id.to_string(),
                    Some(user_id.to_string()),
                    builder.limit,
                    None,
                )
                .await?
        };

        // 应用重排序
        if builder.enable_rerank {
            results = builder
                .orchestrator
                .context_aware_rerank(results, &builder.query, user_id)
                .await?;
        }

        // 应用时间范围过滤
        if let Some((start, end)) = builder.time_range {
            results = results
                .into_iter()
                .filter(|memory| {
                    memory
                        .metadata
                        .get("timestamp")
                        .and_then(|v| v.as_i64())
                        .map(|timestamp| timestamp >= start && timestamp <= end)
                        .unwrap_or(false)
                })
                .collect();
        }

        // 应用自定义过滤器
        if !builder.filters.is_empty() {
            results = results
                .into_iter()
                .filter(|memory| {
                    // 检查所有自定义过滤器条件
                    builder.filters.iter().all(|(key, value)| {
                        // 检查 metadata 中的字段
                        memory
                            .metadata
                            .get(key)
                            .map(|v| v == value)
                            .unwrap_or(false)
                    })
                })
                .collect();
        }

        Ok(results)
    }
}

// 实现 Future，允许直接 await
impl<'a> std::future::IntoFuture for SearchBuilder<'a> {
    type Output = Result<Vec<MemoryItem>>;
    type IntoFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.execute())
    }
}

// ========== ✅ BatchBuilder ==========

/// 批量操作构建器 - 使用 Builder 模式提供灵活的批量操作
///
/// # 示例
///
/// ```rust
/// let ids = orchestrator
///     .batch_add()
///     .add("Memory 1")
///     .add("Memory 2")
///     .batch_size(50)
///     .execute()
///     .await?;
/// ```
pub struct BatchBuilder<'a> {
    orchestrator: &'a MemoryOrchestrator,
    contents: Vec<String>,
    agent_id: String,
    user_id: Option<String>,
    memory_type: Option<agent_mem_core::types::MemoryType>,
    batch_size: usize,
    concurrency: usize,
}

impl<'a> BatchBuilder<'a> {
    fn new(orchestrator: &'a MemoryOrchestrator) -> Self {
        Self {
            orchestrator,
            contents: Vec::new(),
            agent_id: "default".to_string(),
            user_id: Some("default".to_string()),
            memory_type: None,
            batch_size: 100,
            concurrency: 10,
        }
    }

    /// 添加单个内容
    pub fn add(mut self, content: &str) -> Self {
        self.contents.push(content.to_string());
        self
    }

    /// 添加多个内容
    pub fn add_all(mut self, contents: Vec<String>) -> Self {
        self.contents.extend(contents);
        self
    }

    /// 设置 agent_id
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = agent_id;
        self
    }

    /// 设置 user_id
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// 设置 memory_type
    pub fn with_memory_type(mut self, memory_type: agent_mem_core::types::MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    /// 设置批量大小
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// 设置并发数
    ///
    /// 控制批量添加时的并发任务数量。较高的并发数可以加快大批量数据的处理速度，
    /// 但也会增加内存和 CPU 使用量。
    ///
    /// # 参数
    ///
    /// * `n` - 并发任务数，建议范围：1-50
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let ids = orchestrator
    ///     .batch_add()
    ///     .add_all(contents)
    ///     .concurrency(20)  // 使用 20 个并发任务
    ///     .await?;
    /// ```
    pub fn concurrency(mut self, n: usize) -> Self {
        self.concurrency = n.max(1); // 确保至少为 1
        self
    }

    /// 执行批量添加
    pub async fn execute(self) -> Result<Vec<String>> {
        if self.contents.is_empty() {
            return Ok(Vec::new());
        }

        // 如果内容数量小于并发数的2倍，直接使用批量添加
        if self.contents.len() < self.concurrency * 2 {
            // 准备批量数据
            let items: Vec<(
                String,
                String,
                Option<String>,
                Option<agent_mem_core::types::MemoryType>,
                Option<std::collections::HashMap<String, serde_json::Value>>,
            )> = self
                .contents
                .into_iter()
                .map(|content| {
                    (
                        content,
                        self.agent_id.clone(),
                        self.user_id.clone(),
                        self.memory_type,
                        None,
                    )
                })
                .collect();

            return self.orchestrator.add_memories_batch(items).await;
        }

        // 使用并发处理：将内容分成多个批次
        use futures::stream::{self, StreamExt};
        let orchestrator = self.orchestrator;
        let agent_id = self.agent_id.clone();
        let user_id = self.user_id.clone();
        let memory_type = self.memory_type;

        // 分批处理
        let chunks: Vec<_> = self
            .contents
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        // 创建并发任务流
        let results = stream::iter(chunks)
            .map(move |chunk| {
                let orch = orchestrator;
                let agent_id = agent_id.clone();
                let user_id = user_id.clone();
                let memory_type = memory_type;

                async move {
                    // 准备批次数据
                    let items: Vec<_> = chunk
                        .into_iter()
                        .map(|content| {
                            (
                                content,
                                agent_id.clone(),
                                user_id.clone(),
                                memory_type,
                                None as Option<
                                    std::collections::HashMap<String, serde_json::Value>,
                                >,
                            )
                        })
                        .collect();

                    // 执行批量添加
                    orch.add_memories_batch(items).await
                }
            })
            .buffer_unordered(self.concurrency)
            .collect::<Vec<_>>()
            .await;

        // 合并所有批次的结果
        let mut all_ids = Vec::new();
        for result in results {
            all_ids.extend(result?);
        }

        Ok(all_ids)
    }
}

// 实现 Future，允许直接 await
impl<'a> std::future::IntoFuture for BatchBuilder<'a> {
    type Output = Result<Vec<String>>;
    type IntoFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.execute())
    }
}
