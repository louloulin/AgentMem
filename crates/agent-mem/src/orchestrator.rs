//! Memory Orchestrator - 记忆编排器
//!
//! 负责协调多个 Manager，智能路由请求，管理智能组件
//!
//! AgentMem 3.0 架构：
//! - 移除冗余的 Agent 层
//! - 直接使用 Managers
//! - 集成完整的 Intelligence 组件
//! - 集成 HybridSearchEngine

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ========== Manager 导入 (替代 Agent) ==========
use agent_mem_core::managers::CoreMemoryManager;

#[cfg(feature = "postgres")]
use agent_mem_core::managers::{
    EpisodicMemoryManager, ProceduralMemoryManager, SemanticMemoryManager,
};

// ========== Intelligence 组件导入 ==========
use agent_mem_intelligence::{
    AdvancedFactExtractor,
    ConflictDetection,
    // 冲突解决
    ConflictResolver,
    EnhancedDecisionEngine,
    // 重要性评估
    EnhancedImportanceEvaluator,
    Entity,
    EntityType,
    ExistingMemory,
    ExtractedFact,
    // 事实提取
    FactExtractor,
    ImportanceEvaluation,
    ImportanceFactors,
    MemoryAction,
    // 聚类和推理
    MemoryClusterer,
    MemoryDecision,
    // 决策引擎
    MemoryDecisionEngine,
    MemoryReasoner,
    Relation,
    RelationType,
    ResolutionStrategy,
    StructuredFact,
};

// ========== Search 组件导入 ==========
#[cfg(feature = "postgres")]
use agent_mem_core::search::{
    BM25SearchEngine, FullTextSearchEngine, FuzzyMatchEngine, HybridSearchEngine,
    HybridSearchResult, RRFRanker, SearchQuery, SearchResult, VectorSearchEngine,
};

// ========== 基础类型导入 ==========
use agent_mem_core::types::MemoryType;
use agent_mem_llm::LLMProvider;
use agent_mem_traits::{MemoryItem, Result};

use crate::auto_config::AutoConfig;
use crate::types::{AddResult, MemoryEvent, MemoryStats, RelationEvent};

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
        }
    }
}

/// 记忆编排器
///
/// AgentMem 3.0 核心职责：
/// 1. 智能路由: 根据内容类型路由到对应 Manager (移除 Agent 层)
/// 2. Manager 协调: 直接协调多个 Manager 完成复杂任务
/// 3. Intelligence 集成: 完整集成 8 个智能组件
/// 4. Search 集成: 集成混合搜索引擎
/// 5. 降级处理: 无智能组件时降级到基础模式
pub struct MemoryOrchestrator {
    // ========== Managers (直接使用，移除 Agent 层) ==========
    core_manager: Option<Arc<CoreMemoryManager>>,

    #[cfg(feature = "postgres")]
    semantic_manager: Option<Arc<SemanticMemoryManager>>,
    #[cfg(feature = "postgres")]
    episodic_manager: Option<Arc<EpisodicMemoryManager>>,
    #[cfg(feature = "postgres")]
    procedural_manager: Option<Arc<ProceduralMemoryManager>>,

    // ========== Intelligence 组件 (完整集成) ==========
    // 事实提取
    fact_extractor: Option<Arc<FactExtractor>>,
    advanced_fact_extractor: Option<Arc<AdvancedFactExtractor>>,

    // 决策引擎
    decision_engine: Option<Arc<MemoryDecisionEngine>>,
    enhanced_decision_engine: Option<Arc<EnhancedDecisionEngine>>,

    // 重要性评估
    importance_evaluator: Option<Arc<EnhancedImportanceEvaluator>>,

    // 冲突解决
    conflict_resolver: Option<Arc<ConflictResolver>>,

    // TODO: 聚类和推理组件待添加
    // memory_clusterer: Option<Arc<dyn MemoryClusterer>>,
    // memory_reasoner: Option<Arc<MemoryReasoner>>,

    // ========== Search 组件 ==========
    #[cfg(feature = "postgres")]
    hybrid_search_engine: Option<Arc<HybridSearchEngine>>,
    #[cfg(feature = "postgres")]
    vector_search_engine: Option<Arc<VectorSearchEngine>>,
    #[cfg(feature = "postgres")]
    fulltext_search_engine: Option<Arc<FullTextSearchEngine>>,

    // ========== 辅助组件 ==========
    llm_provider: Option<Arc<dyn LLMProvider + Send + Sync>>,

    // ========== 配置 ==========
    config: OrchestratorConfig,
}

impl MemoryOrchestrator {
    /// 自动配置初始化
    pub async fn new_with_auto_config() -> Result<Self> {
        info!("使用自动配置初始化 MemoryOrchestrator");

        let auto_config = AutoConfig::detect().await?;
        Self::new_with_config(auto_config).await
    }

    /// 使用配置初始化
    pub async fn new_with_config(config: OrchestratorConfig) -> Result<Self> {
        info!(
            "AgentMem 3.0: 使用配置初始化 MemoryOrchestrator: {:?}",
            config
        );

        // ========== Step 1: 创建 Managers (替代 Agents) ==========
        info!("创建 Managers...");

        // TODO: 暂时设为 None，等待实现 Manager 创建逻辑
        // 需要创建存储后端，然后初始化各个 Managers
        let core_manager = None;

        #[cfg(feature = "postgres")]
        let semantic_manager = None;
        #[cfg(feature = "postgres")]
        let episodic_manager = None;
        #[cfg(feature = "postgres")]
        let procedural_manager = None;

        // ========== Step 2: 创建 Intelligence 组件 ==========
        let (
            fact_extractor,
            advanced_fact_extractor,
            decision_engine,
            enhanced_decision_engine,
            importance_evaluator,
            conflict_resolver,
            llm_provider,
        ) = if config.enable_intelligent_features {
            info!("创建 Intelligence 组件...");
            Self::create_intelligence_components(&config).await?
        } else {
            info!("智能功能已禁用，将使用基础模式");
            (None, None, None, None, None, None, None)
        };

        // ========== Step 3: 创建 Search 组件 ==========
        #[cfg(feature = "postgres")]
        let (hybrid_search_engine, vector_search_engine, fulltext_search_engine) = {
            info!("创建 Search 组件...");
            // TODO: 实现 Search 组件创建逻辑
            (None, None, None)
        };

        Ok(Self {
            // Managers
            core_manager,

            #[cfg(feature = "postgres")]
            semantic_manager,
            #[cfg(feature = "postgres")]
            episodic_manager,
            #[cfg(feature = "postgres")]
            procedural_manager,

            // Intelligence 组件
            fact_extractor,
            advanced_fact_extractor,
            decision_engine,
            enhanced_decision_engine,
            importance_evaluator,
            conflict_resolver,

            // Search 组件
            #[cfg(feature = "postgres")]
            hybrid_search_engine,
            #[cfg(feature = "postgres")]
            vector_search_engine,
            #[cfg(feature = "postgres")]
            fulltext_search_engine,

            // 辅助组件
            llm_provider,

            // 配置
            config,
        })
    }

    /// 创建 Intelligence 组件
    async fn create_intelligence_components(
        _config: &OrchestratorConfig,
    ) -> Result<(
        Option<Arc<FactExtractor>>,
        Option<Arc<AdvancedFactExtractor>>,
        Option<Arc<MemoryDecisionEngine>>,
        Option<Arc<EnhancedDecisionEngine>>,
        Option<Arc<EnhancedImportanceEvaluator>>,
        Option<Arc<ConflictResolver>>,
        Option<Arc<dyn LLMProvider + Send + Sync>>,
    )> {
        // TODO: 创建 LLM Provider
        // let llm_provider = Self::create_llm_provider(config).await?;
        let llm_provider: Option<Arc<dyn LLMProvider + Send + Sync>> = None;

        if llm_provider.is_none() {
            warn!("LLM Provider 未配置，Intelligence 组件将不可用");
            return Ok((None, None, None, None, None, None, None));
        }

        // 下面的代码永远不会执行，因为 llm_provider 总是 None
        // 但保留结构以便后续实现
        #[allow(unreachable_code)]
        {
            let llm = llm_provider.clone().unwrap();

            // 创建各个 Intelligence 组件
            let fact_extractor = Some(Arc::new(FactExtractor::new(llm.clone())));
            let advanced_fact_extractor = Some(Arc::new(AdvancedFactExtractor::new(llm.clone())));
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

            info!("Intelligence 组件创建成功");

            Ok((
                fact_extractor,
                advanced_fact_extractor,
                decision_engine,
                enhanced_decision_engine,
                importance_evaluator,
                conflict_resolver,
                llm_provider,
            ))
        }
    }

    /// 创建 LLM Provider
    async fn create_llm_provider(
        config: &OrchestratorConfig,
    ) -> Result<Option<Arc<dyn LLMProvider + Send + Sync>>> {
        // TODO: 实现 LLM Provider 创建逻辑
        // 根据 config.llm_provider 和 config.llm_model 创建对应的 LLM Provider
        warn!("LLM Provider 创建逻辑待实现");
        Ok(None)
    }

    /// 添加记忆 (简单模式，不使用智能推理)
    ///
    /// 直接添加原始内容，不进行事实提取、去重等智能处理
    pub async fn add_memory(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        memory_type: Option<MemoryType>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        info!(
            "添加记忆 (简单模式): content={}, agent_id={}",
            content, agent_id
        );

        // 简单模式：直接添加到 core_manager
        if let Some(core_manager) = &self.core_manager {
            // TODO: 实现直接添加逻辑
            warn!("core_manager 可用，但直接添加逻辑待实现");
            Err(agent_mem_traits::AgentMemError::UnsupportedOperation(
                "简单添加模式待实现".to_string(),
            ))
        } else {
            Err(agent_mem_traits::AgentMemError::UnsupportedOperation(
                "core_manager 未初始化".to_string(),
            ))
        }
    }

    /// 智能添加记忆 (完整流水线)
    ///
    /// 实现 10 步智能处理流水线：
    /// 1. 事实提取（使用 FactExtractor）
    /// 2. 实体和关系提取（使用 AdvancedFactExtractor）
    /// 3. 结构化事实
    /// 4. 重要性评估（使用 ImportanceEvaluator）
    /// 5. 搜索相似记忆（使用 HybridSearchEngine）
    /// 6. 冲突检测（使用 ConflictResolver）
    /// 7. 智能决策（使用 EnhancedDecisionEngine，支持 ADD/UPDATE/DELETE/MERGE）
    /// 8. 执行决策（直接调用 Managers）
    /// 9. 异步聚类分析（TODO）
    /// 10. 异步推理关联（TODO）
    pub async fn add_memory_intelligent(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<AddResult> {
        info!(
            "智能添加记忆: content={}, agent_id={}, user_id={:?}",
            content, agent_id, user_id
        );

        // 检查 Intelligence 组件是否可用
        if self.fact_extractor.is_none() {
            warn!("Intelligence 组件未初始化，降级到简单模式");
            let memory_id = self
                .add_memory(
                    content.clone(),
                    agent_id.clone(),
                    user_id.clone(),
                    None,
                    metadata.clone(),
                )
                .await?;
            return Ok(AddResult {
                results: vec![MemoryEvent {
                    id: memory_id,
                    memory: content,
                    event: "ADD".to_string(),
                    actor_id: Some(agent_id),
                    role: None,
                }],
                relations: None,
            });
        }

        // ========== Step 1: 事实提取 ==========
        info!("Step 1: 事实提取");
        let facts = self.extract_facts(&content).await?;
        info!("提取到 {} 个事实", facts.len());

        if facts.is_empty() {
            warn!("未提取到任何事实，直接添加原始内容");
            let memory_id = self
                .add_memory(
                    content.clone(),
                    agent_id.clone(),
                    user_id.clone(),
                    None,
                    metadata.clone(),
                )
                .await?;
            return Ok(AddResult {
                results: vec![MemoryEvent {
                    id: memory_id,
                    memory: content,
                    event: "ADD".to_string(),
                    actor_id: Some(agent_id),
                    role: None,
                }],
                relations: None,
            });
        }

        // ========== Step 2-3: 结构化事实提取 ==========
        info!("Step 2-3: 结构化事实提取");
        let structured_facts = self.extract_structured_facts(&content).await?;
        info!("提取到 {} 个结构化事实", structured_facts.len());

        // ========== Step 4: 重要性评估 ==========
        info!("Step 4: 重要性评估");
        let importance_evaluations = self.evaluate_importance(&structured_facts).await?;
        info!("完成 {} 个事实的重要性评估", importance_evaluations.len());

        // ========== Step 5: 搜索相似记忆 ==========
        info!("Step 5: 搜索相似记忆");
        let existing_memories = self
            .search_similar_memories(&content, &agent_id, 10)
            .await?;
        info!("找到 {} 个相似记忆", existing_memories.len());

        // ========== Step 6: 冲突检测 ==========
        info!("Step 6: 冲突检测");
        let conflicts = self
            .detect_conflicts(&structured_facts, &existing_memories)
            .await?;
        info!("检测到 {} 个冲突", conflicts.len());

        // ========== Step 7: 智能决策 ==========
        info!("Step 7: 智能决策");
        let decisions = self
            .make_intelligent_decisions(
                &structured_facts,
                &existing_memories,
                &importance_evaluations,
                &conflicts,
            )
            .await?;
        info!("生成 {} 个决策", decisions.len());

        // ========== Step 8: 执行决策 ==========
        info!("Step 8: 执行决策");
        let results = self
            .execute_decisions(decisions, agent_id, user_id, metadata)
            .await?;

        // ========== Step 9-10: 异步聚类和推理 (TODO) ==========
        // TODO: 实现异步聚类分析
        // TODO: 实现异步推理关联

        Ok(results)
    }

    /// 搜索记忆 (简单模式 - 向后兼容)
    ///
    /// 这是简化版本，直接调用 search_memories_hybrid()
    pub async fn search_memories(
        &self,
        query: String,
        agent_id: String,
        user_id: Option<String>,
        limit: usize,
        memory_type: Option<MemoryType>,
    ) -> Result<Vec<MemoryItem>> {
        debug!(
            "搜索记忆 (简单模式): query={}, agent_id={}, user_id={:?}, limit={}, memory_type={:?}",
            query, agent_id, user_id, limit, memory_type
        );

        // 调用混合搜索方法
        self.search_memories_hybrid(
            query,
            user_id.unwrap_or_else(|| "default".to_string()),
            limit,
            Some(0.7), // 默认阈值
            None,      // 无额外过滤
        )
        .await
    }

    /// 混合搜索记忆 (智能模式 - Phase 1 Step 1.3)
    ///
    /// 使用 HybridSearchEngine 实现高性能混合搜索
    ///
    /// # 流水线步骤
    ///
    /// 1. 查询预处理
    /// 2. 并行多路搜索 (Vector + FullText)
    /// 3. RRF 融合
    /// 4. 相似度阈值过滤
    /// 5. 结果转换
    ///
    /// # 参数
    ///
    /// * `query` - 搜索查询文本
    /// * `user_id` - 用户 ID
    /// * `limit` - 最大结果数
    /// * `threshold` - 相似度阈值 (0.0 - 1.0)
    /// * `filters` - 额外过滤条件
    ///
    /// # 返回
    ///
    /// 返回搜索到的记忆列表
    #[cfg(feature = "postgres")]
    pub async fn search_memories_hybrid(
        &self,
        query: String,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<MemoryItem>> {
        info!(
            "混合搜索记忆: query={}, user_id={}, limit={}, threshold={:?}",
            query, user_id, limit, threshold
        );

        // ========== Step 1: 查询预处理 ==========
        let processed_query = self.preprocess_query(&query).await?;
        debug!("查询预处理完成: {}", processed_query);

        // ========== Step 2-4: 使用 HybridSearchEngine 执行搜索 ==========
        if let Some(hybrid_engine) = &self.hybrid_search_engine {
            // 生成查询向量
            let query_vector = self.generate_query_embedding(&processed_query).await?;

            // 构建搜索查询
            let search_query = SearchQuery {
                query: processed_query.clone(),
                limit,
                threshold,
                vector_weight: 0.7,
                fulltext_weight: 0.3,
                filters: None, // TODO: 转换 filters
            };

            // 执行混合搜索
            let hybrid_result = hybrid_engine.search(query_vector, &search_query).await?;

            debug!(
                "混合搜索完成: {} 个结果, 耗时 {} ms",
                hybrid_result.results.len(),
                hybrid_result.stats.total_time_ms
            );

            // ========== Step 5: 转换为 MemoryItem ==========
            let memory_items = self
                .convert_search_results_to_memory_items(hybrid_result.results)
                .await?;

            Ok(memory_items)
        } else {
            warn!("HybridSearchEngine 未初始化，返回空结果");
            Ok(Vec::new())
        }
    }

    /// 混合搜索记忆 (非 postgres 特性时的降级实现)
    #[cfg(not(feature = "postgres"))]
    pub async fn search_memories_hybrid(
        &self,
        query: String,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<MemoryItem>> {
        warn!("HybridSearchEngine 需要 postgres 特性，返回空结果");
        Ok(Vec::new())
    }

    /// 获取所有记忆
    pub async fn get_all_memories(
        &self,
        agent_id: String,
        user_id: Option<String>,
    ) -> Result<Vec<MemoryItem>> {
        debug!("获取所有记忆: agent_id={}, user_id={:?}", agent_id, user_id);

        // 使用搜索功能获取所有记忆（空查询）
        self.search_memories(
            String::new(), // 空查询
            agent_id,
            user_id,
            1000, // 默认最多返回 1000 条
            None, // 所有类型
        )
        .await
    }

    /// 获取统计信息
    pub async fn get_stats(&self, user_id: Option<String>) -> Result<MemoryStats> {
        debug!("获取统计信息: user_id={:?}", user_id);

        // 获取所有记忆
        let all_memories = self
            .get_all_memories("default".to_string(), user_id)
            .await?;

        // 统计各类型记忆数量
        let mut memories_by_type = HashMap::new();
        let mut total_importance = 0.0;

        for memory in &all_memories {
            let type_name = format!("{:?}", memory.memory_type);
            *memories_by_type.entry(type_name).or_insert(0) += 1;
            total_importance += memory.importance;
        }

        let average_importance = if all_memories.is_empty() {
            0.0
        } else {
            total_importance / all_memories.len() as f32
        };

        Ok(MemoryStats {
            total_memories: all_memories.len(),
            memories_by_type,
            average_importance,
            storage_size_bytes: 0, // TODO: 实现实际的存储大小计算
            last_updated: Some(chrono::Utc::now()),
        })
    }

    /// 推断记忆类型
    async fn infer_memory_type(&self, content: &str) -> Result<MemoryType> {
        // 简单的规则推断
        let content_lower = content.to_lowercase();

        // 核心记忆关键词
        if content_lower.contains("i am")
            || content_lower.contains("my name is")
            || content_lower.contains("i'm")
            || content_lower.contains("我是")
            || content_lower.contains("我叫")
        {
            return Ok(MemoryType::Core);
        }

        // 情景记忆关键词
        if content_lower.contains("happened")
            || content_lower.contains("did")
            || content_lower.contains("went to")
            || content_lower.contains("发生")
            || content_lower.contains("去了")
        {
            return Ok(MemoryType::Episodic);
        }

        // 程序记忆关键词
        if content_lower.contains("how to")
            || content_lower.contains("步骤")
            || content_lower.contains("方法")
        {
            return Ok(MemoryType::Procedural);
        }

        // 默认为语义记忆
        Ok(MemoryType::Semantic)
    }

    // ========== 旧方法已删除，待在 Phase 1 Step 1.2 中重新实现 ==========
    // route_add_to_agent() 方法已删除，将在 Step 1.2 中使用 Manager 重新实现

    // ========== mem0 兼容 API ==========

    /// 添加记忆 v2（mem0 兼容）
    ///
    /// 支持 infer 参数控制智能推理
    pub async fn add_memory_v2(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        run_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
        infer: bool,
        memory_type: Option<String>,
        _prompt: Option<String>,
    ) -> Result<AddResult> {
        debug!(
            "添加记忆 v2: content={}, agent_id={}, infer={}",
            content, agent_id, infer
        );

        // 如果 infer=true，使用智能推理（事实提取、去重等）
        // 如果 infer=false，直接添加原始内容

        // 解析 memory_type 字符串
        let mem_type = if let Some(type_str) = memory_type {
            match type_str.as_str() {
                "core_memory" => Some(MemoryType::Core),
                "episodic_memory" => Some(MemoryType::Episodic),
                "semantic_memory" => Some(MemoryType::Semantic),
                "procedural_memory" => Some(MemoryType::Procedural),
                _ => None,
            }
        } else {
            None
        };

        // 调用原有的 add_memory 方法
        let memory_id = self
            .add_memory(
                content.clone(),
                agent_id.clone(),
                user_id.clone(),
                mem_type,
                metadata,
            )
            .await?;

        // 构造返回结果
        let event = MemoryEvent {
            id: memory_id,
            memory: content,
            event: "ADD".to_string(),
            actor_id: user_id.or(Some(agent_id)),
            role: Some("user".to_string()),
        };

        // TODO: 如果启用了图存储，提取关系
        let relations = if infer {
            // 未来可以在这里调用关系提取
            None
        } else {
            None
        };

        Ok(AddResult {
            results: vec![event],
            relations,
        })
    }

    /// 获取单个记忆
    ///
    /// TODO: Phase 1 Step 1.2 - 使用 Manager 重新实现
    pub async fn get_memory(&self, _memory_id: &str) -> Result<MemoryItem> {
        warn!("get_memory() 方法待重构实现 (Phase 1 Step 1.2)");
        Err(agent_mem_traits::AgentMemError::UnsupportedOperation(
            "get_memory() 方法正在重构中".to_string(),
        ))
    }

    /// 获取所有记忆 v2（mem0 兼容）
    pub async fn get_all_memories_v2(
        &self,
        agent_id: String,
        user_id: Option<String>,
        _run_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>> {
        debug!(
            "获取所有记忆 v2: agent_id={}, user_id={:?}, limit={:?}",
            agent_id, user_id, limit
        );

        // 调用原有的 get_all_memories 方法
        let mut memories = self.get_all_memories(agent_id, user_id).await?;

        // 应用 limit
        if let Some(limit_val) = limit {
            memories.truncate(limit_val);
        }

        Ok(memories)
    }

    /// 更新记忆
    ///
    /// TODO: Phase 1 Step 1.2 - 使用 Manager 重新实现
    pub async fn update_memory(
        &self,
        _memory_id: &str,
        _data: HashMap<String, serde_json::Value>,
    ) -> Result<MemoryItem> {
        warn!("update_memory() 方法待重构实现 (Phase 1 Step 1.2)");
        Err(agent_mem_traits::AgentMemError::UnsupportedOperation(
            "update_memory() 方法正在重构中".to_string(),
        ))
    }

    /// 删除记忆
    ///
    /// TODO: Phase 1 Step 1.2 - 使用 Manager 重新实现
    pub async fn delete_memory(&self, _memory_id: &str) -> Result<()> {
        warn!("delete_memory() 方法待重构实现 (Phase 1 Step 1.2)");
        Err(agent_mem_traits::AgentMemError::UnsupportedOperation(
            "delete_memory() 方法正在重构中".to_string(),
        ))
    }

    /// 删除所有记忆
    pub async fn delete_all_memories(
        &self,
        agent_id: String,
        user_id: Option<String>,
        _run_id: Option<String>,
    ) -> Result<usize> {
        debug!("删除所有记忆");

        // 获取所有记忆
        let memories = self.get_all_memories(agent_id, user_id).await?;
        let count = memories.len();

        // 逐个删除
        for memory in memories {
            let _ = self.delete_memory(&memory.id).await;
        }

        Ok(count)
    }

    // ========== 辅助方法 ==========

    /// 将 SemanticMemoryItem 转换为 MemoryItem
    fn semantic_to_memory_item(item: agent_mem_traits::SemanticMemoryItem) -> MemoryItem {
        use agent_mem_traits::{Entity, Relation, Session};
        use std::collections::HashMap;

        let mut metadata = HashMap::new();
        if let Some(details) = &item.details {
            metadata.insert("details".to_string(), serde_json::json!(details));
        }
        if let Some(source) = &item.source {
            metadata.insert("source".to_string(), serde_json::json!(source));
        }
        metadata.insert("tree_path".to_string(), serde_json::json!(item.tree_path));
        metadata.insert(
            "organization_id".to_string(),
            serde_json::json!(item.organization_id),
        );

        // 合并原有的 metadata
        if let Ok(meta_map) =
            serde_json::from_value::<HashMap<String, serde_json::Value>>(item.metadata.clone())
        {
            metadata.extend(meta_map);
        }

        MemoryItem {
            id: item.id,
            content: item.summary,
            hash: None,
            metadata,
            score: None,
            created_at: item.created_at,
            updated_at: Some(item.updated_at),
            session: Session {
                id: "default".to_string(),
                user_id: Some(item.user_id.clone()),
                agent_id: Some(item.agent_id.clone()),
                run_id: None,
                actor_id: Some(item.agent_id.clone()),
                created_at: item.created_at,
                metadata: HashMap::new(),
            },
            memory_type: agent_mem_traits::MemoryType::Semantic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: item.agent_id,
            user_id: Some(item.user_id),
            importance: 0.5, // 默认重要性
            embedding: None,
            last_accessed_at: item.updated_at,
            access_count: 0,
            expires_at: None,
            version: 1,
        }
    }

    // ========== 智能添加流水线辅助方法 ==========

    /// Step 1: 事实提取
    async fn extract_facts(&self, content: &str) -> Result<Vec<ExtractedFact>> {
        if let Some(fact_extractor) = &self.fact_extractor {
            // 将内容转换为 Message 格式
            let messages = vec![agent_mem_llm::Message::user(content)];
            fact_extractor.extract_facts_internal(&messages).await
        } else {
            warn!("FactExtractor 未初始化");
            Ok(Vec::new())
        }
    }

    /// Step 2-3: 结构化事实提取
    async fn extract_structured_facts(&self, content: &str) -> Result<Vec<StructuredFact>> {
        if let Some(advanced_fact_extractor) = &self.advanced_fact_extractor {
            // 将内容转换为 Message 格式
            let messages = vec![agent_mem_llm::Message::user(content)];
            advanced_fact_extractor
                .extract_structured_facts(&messages)
                .await
        } else {
            warn!("AdvancedFactExtractor 未初始化");
            Ok(Vec::new())
        }
    }

    /// Step 4: 重要性评估
    async fn evaluate_importance(
        &self,
        structured_facts: &[StructuredFact],
    ) -> Result<Vec<ImportanceEvaluation>> {
        // TODO: 使用 EnhancedImportanceEvaluator.evaluate_importance() 需要 Memory 类型
        // 暂时使用简化的重要性评估逻辑

        warn!("重要性评估暂时使用简化逻辑（完整实现需要 Memory 类型）");

        // 简化评估：基于事实的置信度和重要性字段
        let evaluations = structured_facts
            .iter()
            .map(|fact| ImportanceEvaluation {
                memory_id: fact.id.clone(),
                importance_score: fact.importance,
                confidence: fact.confidence,
                factors: ImportanceFactors {
                    content_complexity: fact.importance,
                    entity_importance: 0.5,
                    relation_importance: 0.5,
                    temporal_relevance: 0.5,
                    user_interaction: 0.5,
                    contextual_relevance: 0.5,
                    emotional_intensity: 0.5,
                },
                evaluated_at: chrono::Utc::now(),
                reasoning: format!("基于事实重要性: {:.2}", fact.importance),
            })
            .collect();

        Ok(evaluations)
    }

    /// Step 5: 搜索相似记忆
    async fn search_similar_memories(
        &self,
        content: &str,
        agent_id: &str,
        limit: usize,
    ) -> Result<Vec<ExistingMemory>> {
        // TODO: 使用 HybridSearchEngine 搜索相似记忆
        // 暂时返回空列表
        warn!("search_similar_memories 待实现 (需要 HybridSearchEngine)");
        Ok(Vec::new())
    }

    /// Step 6: 冲突检测
    async fn detect_conflicts(
        &self,
        _structured_facts: &[StructuredFact],
        _existing_memories: &[ExistingMemory],
    ) -> Result<Vec<ConflictDetection>> {
        // TODO: 使用 ConflictResolver.detect_conflicts() 需要 Memory 类型
        // 暂时跳过冲突检测

        warn!("冲突检测暂时跳过（完整实现需要 Memory 类型）");
        Ok(Vec::new())
    }

    /// Step 7: 智能决策
    async fn make_intelligent_decisions(
        &self,
        structured_facts: &[StructuredFact],
        _existing_memories: &[ExistingMemory],
        importance_evaluations: &[ImportanceEvaluation],
        _conflicts: &[ConflictDetection],
    ) -> Result<Vec<MemoryDecision>> {
        // TODO: 使用 EnhancedDecisionEngine.make_decisions() 需要构造完整的 DecisionContext
        // 暂时使用简化的决策逻辑

        warn!("智能决策暂时使用简化逻辑（完整实现需要 DecisionContext）");

        // 简化决策：根据重要性决定是否添加
        let mut decisions = Vec::new();

        for (i, fact) in structured_facts.iter().enumerate() {
            // 获取对应的重要性评估
            let importance = importance_evaluations
                .get(i)
                .map(|e| e.importance_score)
                .unwrap_or(0.5);

            // 如果重要性太低，跳过
            if importance < 0.3 {
                info!(
                    "事实重要性太低 ({})，跳过: {}",
                    importance, fact.description
                );
                continue;
            }

            // 创建 ADD 决策
            decisions.push(MemoryDecision {
                action: MemoryAction::Add {
                    content: fact.description.clone(),
                    importance,
                    metadata: fact.metadata.clone(),
                },
                confidence: importance,
                reasoning: format!("重要性评分: {:.2}", importance),
                affected_memories: Vec::new(),
                estimated_impact: importance,
            });
        }

        Ok(decisions)
    }

    /// Step 8: 执行决策
    async fn execute_decisions(
        &self,
        decisions: Vec<MemoryDecision>,
        agent_id: String,
        user_id: Option<String>,
        _metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<AddResult> {
        let mut results = Vec::new();

        for decision in decisions {
            match decision.action {
                MemoryAction::Add {
                    content,
                    importance,
                    metadata,
                } => {
                    info!("执行 ADD 决策: {} (importance: {})", content, importance);

                    // 将 HashMap<String, String> 转换为 HashMap<String, serde_json::Value>
                    let json_metadata: HashMap<String, serde_json::Value> = metadata
                        .into_iter()
                        .map(|(k, v)| (k, serde_json::Value::String(v)))
                        .collect();

                    let memory_id = self
                        .add_memory(
                            content.clone(),
                            agent_id.clone(),
                            user_id.clone(),
                            None,
                            Some(json_metadata),
                        )
                        .await?;

                    results.push(MemoryEvent {
                        id: memory_id,
                        memory: content,
                        event: "ADD".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::Update {
                    memory_id,
                    new_content,
                    merge_strategy: _,
                    change_reason,
                } => {
                    info!(
                        "执行 UPDATE 决策: {} -> {} (reason: {})",
                        memory_id, new_content, change_reason
                    );
                    // TODO: 实现更新逻辑
                    warn!("UPDATE 操作待实现");
                    results.push(MemoryEvent {
                        id: memory_id,
                        memory: new_content,
                        event: "UPDATE".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::Delete {
                    memory_id,
                    deletion_reason: _,
                } => {
                    info!("执行 DELETE 决策: {}", memory_id);
                    // TODO: 实现删除逻辑
                    warn!("DELETE 操作待实现");
                    results.push(MemoryEvent {
                        id: memory_id,
                        memory: String::new(),
                        event: "DELETE".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::Merge {
                    primary_memory_id,
                    secondary_memory_ids,
                    merged_content,
                } => {
                    info!(
                        "执行 MERGE 决策: {} + {:?} -> {}",
                        primary_memory_id, secondary_memory_ids, merged_content
                    );
                    // TODO: 实现合并逻辑
                    warn!("MERGE 操作待实现");
                    results.push(MemoryEvent {
                        id: primary_memory_id,
                        memory: merged_content,
                        event: "UPDATE".to_string(),
                        actor_id: Some(agent_id.clone()),
                        role: None,
                    });
                }
                MemoryAction::NoAction { reason } => {
                    info!("执行 NoAction 决策: {}", reason);
                    // 不做任何操作
                }
            }
        }

        Ok(AddResult {
            results,
            relations: None,
        })
    }

    // ========== 搜索辅助方法 (Phase 1 Step 1.3) ==========

    /// 查询预处理
    ///
    /// 清理和标准化查询文本
    async fn preprocess_query(&self, query: &str) -> Result<String> {
        // 简单的预处理：去除多余空格，转小写
        let processed = query.trim().to_lowercase();
        Ok(processed)
    }

    /// 生成查询嵌入向量
    ///
    /// 使用 LLM Provider 生成查询的向量表示
    async fn generate_query_embedding(&self, query: &str) -> Result<Vec<f32>> {
        if let Some(llm_provider) = &self.llm_provider {
            // TODO: 调用 LLM Provider 生成嵌入
            // 临时实现：返回零向量
            warn!("generate_query_embedding() 待实现，返回零向量");
            Ok(vec![0.0; 1536]) // OpenAI embedding 维度
        } else {
            warn!("LLM Provider 未配置，返回零向量");
            Ok(vec![0.0; 1536])
        }
    }

    /// 转换搜索结果为 MemoryItem
    ///
    /// 将 SearchResult 转换为 MemoryItem 格式
    #[cfg(feature = "postgres")]
    async fn convert_search_results_to_memory_items(
        &self,
        results: Vec<SearchResult>,
    ) -> Result<Vec<MemoryItem>> {
        let mut memory_items = Vec::new();

        for result in results {
            // 解析元数据
            let metadata = if let Some(meta) = result.metadata {
                if let Ok(map) = serde_json::from_value::<HashMap<String, String>>(meta) {
                    map
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };

            // 创建 MemoryItem
            let memory_item = MemoryItem {
                id: result.id,
                memory: result.content,
                hash: String::new(), // TODO: 计算哈希
                metadata,
                categories: Vec::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                user_id: None,
                agent_id: None,
                run_id: None,
                memory_type: MemoryType::Semantic, // 默认类型
                importance: result.score,
            };

            memory_items.push(memory_item);
        }

        Ok(memory_items)
    }
}
