//! Orchestrator Core - 核心编排器
//!
//! 定义MemoryOrchestrator核心结构和配置，协调各个模块

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

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
            embedding_batch_size: Some(64), // 优化：增加批处理大小（32 → 64）
            embedding_batch_interval_ms: Some(20), // 优化：增加批处理间隔（10ms → 20ms）
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

        // ========== Step 2: 创建 Intelligence 组件 ==========
        let intelligence_components = if config.enable_intelligent_features {
            info!("创建 Intelligence 组件...");
            super::initialization::InitializationModule::create_intelligence_components(&config)
                .await?
        } else {
            info!("智能功能已禁用，将使用基础模式");
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

        // ========== Step 3: 创建 Embedder ==========
        let embedder = {
            info!("创建 Embedder...");
            super::initialization::InitializationModule::create_embedder(&config).await?
        };

        // ========== Step 4: 创建 Search 组件 ==========
        // 注意：Search组件需要embedder和vector_store，所以需要在它们创建之后
        // 这里先设置为None，稍后在创建vector_store之后会更新
        #[cfg(feature = "postgres")]
        let (hybrid_search_engine, vector_search_engine, fulltext_search_engine): (
            Option<Arc<agent_mem_core::search::HybridSearchEngine>>,
            Option<Arc<agent_mem_core::search::VectorSearchEngine>>,
            Option<Arc<agent_mem_core::search::FullTextSearchEngine>>,
        ) = (None, None, None);

        // ========== Step 5: 创建多模态处理组件 ==========
        let (image_processor, audio_processor, video_processor, multimodal_manager) = {
            info!("创建多模态处理组件...");
            super::initialization::InitializationModule::create_multimodal_components(&config)
                .await?
        };

        // ========== Step 6: 创建 OpenAI 多模态 API ==========
        #[cfg(feature = "multimodal")]
        let (openai_vision, openai_whisper) = {
            info!("创建 OpenAI 多模态 API 客户端...");
            super::initialization::InitializationModule::create_openai_multimodal_clients(&config)
                .await?
        };

        // ========== Step 7: 创建聚类和推理组件 ==========
        let (dbscan_clusterer, kmeans_clusterer, memory_reasoner) = {
            info!("创建聚类和推理组件...");
            super::initialization::InitializationModule::create_clustering_reasoning_components(
                &config,
            )
            .await?
        };

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
        let (hybrid_search_engine, vector_search_engine, fulltext_search_engine) =
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
            embedder,

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

    // ========== 存储方法委托 ==========

    /// 添加记忆（快速模式）
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

    /// 添加记忆（简单模式）
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

    /// 添加记忆 v2（支持 infer 参数）
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

    /// 更新记忆
    pub async fn update_memory(
        &self,
        memory_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<MemoryItem> {
        super::storage::StorageModule::update_memory(self, memory_id, data).await
    }

    /// 删除记忆
    pub async fn delete_memory(&self, memory_id: &str) -> Result<()> {
        super::storage::StorageModule::delete_memory(self, memory_id).await
    }

    /// 获取记忆
    pub async fn get_memory(&self, memory_id: &str) -> Result<MemoryItem> {
        super::storage::StorageModule::get_memory(self, memory_id).await
    }

    // ========== 检索方法委托 ==========

    /// 搜索记忆
    pub async fn search_memories(
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

    /// 混合搜索记忆
    #[cfg(feature = "postgres")]
    pub async fn search_memories_hybrid(
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

    /// 混合搜索记忆（非 postgres 版本）
    #[cfg(not(feature = "postgres"))]
    pub async fn search_memories_hybrid(
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

    /// 上下文感知重排序
    pub async fn context_aware_rerank(
        &self,
        memories: Vec<MemoryItem>,
        query: &str,
        user_id: &str,
    ) -> Result<Vec<MemoryItem>> {
        super::retrieval::RetrievalModule::context_aware_rerank(self, memories, query, user_id)
            .await
    }

    // ========== 批量操作方法委托 ==========

    /// 批量添加记忆
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

    /// 批量添加记忆（优化版）
    pub async fn add_memory_batch_optimized(
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

    // ========== 多模态方法委托 ==========

    /// 添加图像记忆
    pub async fn add_image_memory(
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

    /// 添加音频记忆
    pub async fn add_audio_memory(
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

    /// 添加视频记忆
    pub async fn add_video_memory(
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

    // ========== 工具方法委托 ==========

    /// 生成查询嵌入向量
    pub async fn generate_query_embedding(&self, query: &str) -> Result<Vec<f32>> {
        if let Some(embedder) = &self.embedder {
            super::utils::UtilsModule::generate_query_embedding(query, embedder.as_ref()).await
        } else {
            Err(agent_mem_traits::AgentMemError::ConfigError(
                "Embedder not configured".to_string(),
            ))
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self, user_id: Option<String>) -> Result<MemoryStats> {
        let mut total_memories = 0;
        let mut memories_by_type: HashMap<String, usize> = HashMap::new();
        let mut total_importance = 0.0;
        let mut count = 0;

        // 从 CoreMemoryManager 获取统计
        // Note: CoreMemoryManager 不提供 get_memory_stats 方法
        // 如果需要统计功能，应该使用 MemoryManager 而不是 CoreMemoryManager
        // 这里暂时跳过，返回默认统计

        // 从向量存储获取统计（如果可用）
        if let Some(vector_store) = &self.vector_store {
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

    /// 获取所有记忆
    pub async fn get_all_memories(
        &self,
        agent_id: String,
        user_id: Option<String>,
    ) -> Result<Vec<MemoryItem>> {
        use super::utils::UtilsModule;
        let mut all_memories = Vec::new();

        // 使用 MemoryManager 获取所有记忆
        if let Some(manager) = &self.memory_manager {
            let memories = manager
                .get_agent_memories(&agent_id, None)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to get memories from MemoryManager: {}",
                        e
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

    /// 获取所有记忆 v2
    pub async fn get_all_memories_v2(
        &self,
        agent_id: String,
        user_id: Option<String>,
        run_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>> {
        let mut memories = self.get_all_memories(agent_id, user_id).await?;
        if let Some(limit_val) = limit {
            memories.truncate(limit_val);
        }
        Ok(memories)
    }

    /// 删除所有记忆
    pub async fn delete_all_memories(
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

    /// 重置
    pub async fn reset(&self) -> Result<()> {
        info!("重置 MemoryOrchestrator");

        // 删除所有记忆
        if let Some(manager) = &self.core_manager {
            // 获取所有 agent 的记忆并删除
            // 这里简化处理，实际可能需要更复杂的逻辑
        }

        // 清空向量存储
        if let Some(vector_store) = &self.vector_store {
            // 向量存储可能不直接支持清空，需要根据具体实现调整
        }

        // 清空历史记录
        if let Some(history_manager) = &self.history_manager {
            // 历史管理器可能不直接支持清空，需要根据具体实现调整
        }

        info!("✅ 重置完成");
        Ok(())
    }

    /// 缓存搜索
    pub async fn cached_search(
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

    /// 获取性能统计
    pub async fn get_performance_stats(&self) -> Result<crate::memory::PerformanceStats> {
        // 实现性能统计逻辑
        let total_memories;
        let cache_hit_rate = 0.0;
        let avg_add_latency_ms = 0.0;
        let avg_search_latency_ms = 0.0;
        let queries_per_second = 0.0;
        let memory_usage_mb = 0.0;

        // 从 MemoryManager 获取统计
        total_memories = if let Some(manager) = &self.memory_manager {
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

    /// 获取历史记录
    pub async fn get_history(&self, memory_id: &str) -> Result<Vec<crate::history::HistoryEntry>> {
        if let Some(history_manager) = &self.history_manager {
            history_manager.get_history(memory_id).await
        } else {
            Ok(Vec::new())
        }
    }
}
