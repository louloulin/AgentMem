//! Memory Orchestrator - 记忆编排器
//!
//! 负责协调多个 Agent，智能路由请求，管理智能组件

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use agent_mem_core::{
    CoreAgent, EpisodicAgent, SemanticAgent, ProceduralAgent,
    ResourceAgent, WorkingAgent, KnowledgeAgent, ContextualAgent,
    MemoryAgent,  // 导入 MemoryAgent trait
};
use agent_mem_traits::{
    MemoryItem, Result,
    // TODO: 在任务 1.2 中使用这些类型
    // FactExtractor, DecisionEngine, LLMProvider,
};

// 使用 agent_mem_core 的 MemoryType（与 TaskRequest 兼容）
use agent_mem_core::types::MemoryType;

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
/// 核心职责：
/// 1. 智能路由: 根据内容类型路由到对应 Agent
/// 2. Agent 协调: 协调多个 Agent 完成复杂任务
/// 3. 智能组件管理: 管理 FactExtractor, DecisionEngine 等
/// 4. 降级处理: 无智能组件时降级到基础模式
pub struct MemoryOrchestrator {
    // Agents
    core_agent: Option<Arc<RwLock<CoreAgent>>>,
    episodic_agent: Option<Arc<RwLock<EpisodicAgent>>>,
    semantic_agent: Option<Arc<RwLock<SemanticAgent>>>,
    procedural_agent: Option<Arc<RwLock<ProceduralAgent>>>,
    resource_agent: Option<Arc<RwLock<ResourceAgent>>>,
    working_agent: Option<Arc<RwLock<WorkingAgent>>>,
    knowledge_agent: Option<Arc<RwLock<KnowledgeAgent>>>,
    contextual_agent: Option<Arc<RwLock<ContextualAgent>>>,
    
    // 智能组件 (TODO: 在任务 1.2 中实现)
    // fact_extractor: Option<Arc<dyn FactExtractor>>,
    // decision_engine: Option<Arc<dyn DecisionEngine>>,
    // llm_provider: Option<Arc<dyn LLMProvider>>,
    
    // 配置
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
        info!("使用配置初始化 MemoryOrchestrator: {:?}", config);
        
        // 创建所有 Agents
        // 注意：这里使用 from_env() 会自动从环境变量创建存储后端
        let core_agent = match CoreAgent::from_env("default".to_string()).await {
            Ok(mut agent) => {
                // 调用 initialize() 方法
                if let Err(e) = agent.initialize().await {
                    warn!("CoreAgent 初始化失败: {}, 将禁用核心记忆功能", e);
                    None
                } else {
                    info!("CoreAgent 初始化成功");
                    Some(Arc::new(RwLock::new(agent)))
                }
            }
            Err(e) => {
                warn!("CoreAgent 创建失败: {}, 将禁用核心记忆功能", e);
                None
            }
        };

        let episodic_agent = match EpisodicAgent::from_env("default".to_string()).await {
            Ok(mut agent) => {
                // 调用 initialize() 方法
                if let Err(e) = agent.initialize().await {
                    warn!("EpisodicAgent 初始化失败: {}, 将禁用情景记忆功能", e);
                    None
                } else {
                    info!("EpisodicAgent 初始化成功");
                    Some(Arc::new(RwLock::new(agent)))
                }
            }
            Err(e) => {
                warn!("EpisodicAgent 创建失败: {}, 将禁用情景记忆功能", e);
                None
            }
        };

        let semantic_agent = match SemanticAgent::from_env("default".to_string()).await {
            Ok(mut agent) => {
                // 调用 initialize() 方法
                if let Err(e) = agent.initialize().await {
                    warn!("SemanticAgent 初始化失败: {}, 将禁用语义记忆功能", e);
                    None
                } else {
                    info!("SemanticAgent 初始化成功");
                    Some(Arc::new(RwLock::new(agent)))
                }
            }
            Err(e) => {
                warn!("SemanticAgent 创建失败: {}, 将禁用语义记忆功能", e);
                None
            }
        };

        // ProceduralAgent 暂时不支持 from_env，使用 new() 创建
        let procedural_agent = {
            let mut agent = ProceduralAgent::new("default".to_string());
            // 调用 initialize() 方法
            if let Err(e) = agent.initialize().await {
                warn!("ProceduralAgent 初始化失败: {}", e);
                None
            } else {
                info!("ProceduralAgent 初始化（基础模式）");
                Some(Arc::new(RwLock::new(agent)))
            }
        };
        
        // 其他 Agents 暂时设为 None（在后续任务中实现）
        let resource_agent = None;
        let working_agent = None;
        let knowledge_agent = None;
        let contextual_agent = None;
        
        // TODO: 在任务 1.2 中创建智能组件
        // let (fact_extractor, decision_engine, llm_provider) = if config.enable_intelligent_features {
        //     Self::create_intelligent_components(&config).await?
        // } else {
        //     (None, None, None)
        // };
        
        Ok(Self {
            core_agent,
            episodic_agent,
            semantic_agent,
            procedural_agent,
            resource_agent,
            working_agent,
            knowledge_agent,
            contextual_agent,
            config,
        })
    }
    
    /// 添加记忆 (智能路由)
    pub async fn add_memory(
        &self,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        memory_type: Option<MemoryType>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        debug!("添加记忆: content={}, agent_id={}, user_id={:?}, memory_type={:?}",
               content, agent_id, user_id, memory_type);

        // 1. 推断记忆类型 (如果未指定)
        let memory_type = if let Some(mt) = memory_type {
            mt
        } else {
            self.infer_memory_type(&content).await?
        };

        debug!("推断的记忆类型: {:?}", memory_type);

        // 2. 路由到对应 Agent 并添加记忆
        let memory_id = self.route_add_to_agent(
            memory_type.clone(),
            content,
            agent_id,
            user_id,
            metadata,
        ).await?;

        info!("记忆已添加: id={}, type={:?}", memory_id, memory_type);

        Ok(memory_id)
    }
    
    /// 搜索记忆 (跨所有 Agents)
    ///
    /// 真正调用 core 模块的 Agent 执行搜索任务
    pub async fn search_memories(
        &self,
        query: String,
        agent_id: String,
        user_id: Option<String>,
        limit: usize,
        memory_type: Option<MemoryType>,
    ) -> Result<Vec<MemoryItem>> {
        use agent_mem_core::coordination::meta_manager::TaskRequest;

        debug!("搜索记忆: query={}, memory_type={:?}", query, memory_type);

        let user_id_str = user_id.unwrap_or_else(|| "default".to_string());
        let mut all_results = Vec::new();

        // 准备搜索参数
        let params = serde_json::json!({
            "query": query,
            "agent_id": agent_id,
            "user_id": user_id_str,
            "limit": limit,
        });

        // 根据 memory_type 决定搜索哪些 Agents
        let search_all = memory_type.is_none();

        // 搜索 CoreAgent
        if search_all || memory_type == Some(MemoryType::Core) {
            if let Some(agent) = &self.core_agent {
                debug!("搜索 CoreAgent");
                let task = TaskRequest::new(
                    MemoryType::Core,
                    "search".to_string(),
                    params.clone(),
                );

                let mut agent_lock = agent.write().await;
                if let Ok(response) = agent_lock.execute_task(task).await {
                    if response.success {
                        if let Some(data) = response.data {
                            if let Some(blocks) = data.get("blocks").and_then(|v| v.as_array()) {
                                for block in blocks {
                                    if let Ok(item) = serde_json::from_value::<MemoryItem>(block.clone()) {
                                        all_results.push(item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 搜索 EpisodicAgent
        if search_all || memory_type == Some(MemoryType::Episodic) {
            if let Some(agent) = &self.episodic_agent {
                debug!("搜索 EpisodicAgent");
                let task = TaskRequest::new(
                    MemoryType::Episodic,
                    "search".to_string(),
                    params.clone(),
                );

                let mut agent_lock = agent.write().await;
                if let Ok(response) = agent_lock.execute_task(task).await {
                    if response.success {
                        if let Some(data) = response.data {
                            if let Some(events) = data.get("events").and_then(|v| v.as_array()) {
                                for event in events {
                                    if let Ok(item) = serde_json::from_value::<MemoryItem>(event.clone()) {
                                        all_results.push(item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 搜索 SemanticAgent
        if search_all || memory_type == Some(MemoryType::Semantic) {
            if let Some(agent) = &self.semantic_agent {
                debug!("搜索 SemanticAgent");
                let task = TaskRequest::new(
                    MemoryType::Semantic,
                    "search".to_string(),
                    params.clone(),
                );

                let mut agent_lock = agent.write().await;
                if let Ok(response) = agent_lock.execute_task(task).await {
                    if response.success {
                        if let Some(data) = response.data {
                            // SemanticAgent 返回 "results" 字段，包含 SemanticMemoryItem 数组
                            if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
                                for item in results {
                                    if let Ok(semantic_item) = serde_json::from_value::<agent_mem_traits::SemanticMemoryItem>(item.clone()) {
                                        // 转换 SemanticMemoryItem 到 MemoryItem
                                        let mem_item = Self::semantic_to_memory_item(semantic_item);
                                        all_results.push(mem_item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 按重要性和时间排序
        all_results.sort_by(|a, b| {
            b.importance.partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });

        // 限制结果数量
        all_results.truncate(limit);

        debug!("搜索完成，返回 {} 条结果", all_results.len());
        Ok(all_results)
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
            String::new(),  // 空查询
            agent_id,
            user_id,
            1000,  // 默认最多返回 1000 条
            None,  // 所有类型
        ).await
    }

    /// 获取统计信息
    pub async fn get_stats(
        &self,
        user_id: Option<String>,
    ) -> Result<MemoryStats> {
        debug!("获取统计信息: user_id={:?}", user_id);

        // 获取所有记忆
        let all_memories = self.get_all_memories("default".to_string(), user_id).await?;

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
            storage_size_bytes: 0,  // TODO: 实现实际的存储大小计算
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
            || content_lower.contains("我叫") {
            return Ok(MemoryType::Core);
        }

        // 情景记忆关键词
        if content_lower.contains("happened")
            || content_lower.contains("did")
            || content_lower.contains("went to")
            || content_lower.contains("发生")
            || content_lower.contains("去了") {
            return Ok(MemoryType::Episodic);
        }

        // 程序记忆关键词
        if content_lower.contains("how to")
            || content_lower.contains("步骤")
            || content_lower.contains("方法") {
            return Ok(MemoryType::Procedural);
        }

        // 默认为语义记忆
        Ok(MemoryType::Semantic)
    }

    /// 路由添加操作到对应的 Agent
    ///
    /// 真正调用 core 模块的 Agent 执行任务
    async fn route_add_to_agent(
        &self,
        memory_type: MemoryType,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        use agent_mem_core::coordination::meta_manager::TaskRequest;

        debug!("路由到 Agent: type={:?}", memory_type);

        // 准备任务参数
        let mut params = serde_json::json!({
            "content": content,
            "agent_id": agent_id,
            "user_id": user_id.unwrap_or_else(|| "default".to_string()),
        });

        if let Some(meta) = metadata {
            params["metadata"] = serde_json::to_value(meta)?;
        }

        // 根据记忆类型路由到对应的 Agent
        match memory_type {
            MemoryType::Core => {
                if let Some(agent) = &self.core_agent {
                    debug!("路由到 CoreAgent");
                    let task = TaskRequest::new(
                        MemoryType::Core,
                        "create_block".to_string(),
                        params,
                    );

                    let mut agent_lock = agent.write().await;
                    let response = agent_lock.execute_task(task).await.map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))?;

                    if response.success {
                        if let Some(data) = response.data {
                            if let Some(block_id) = data.get("block_id").and_then(|v| v.as_str()) {
                                return Ok(block_id.to_string());
                            }
                        }
                    }

                    return Err(agent_mem_traits::AgentMemError::MemoryError(
                        "Failed to create core memory block".to_string()
                    ));
                } else {
                    warn!("CoreAgent 未初始化，降级到 SemanticAgent");
                }
            }
            MemoryType::Episodic => {
                if let Some(agent) = &self.episodic_agent {
                    debug!("路由到 EpisodicAgent");
                    let task = TaskRequest::new(
                        MemoryType::Episodic,
                        "create_event".to_string(),
                        params,
                    );

                    let mut agent_lock = agent.write().await;
                    let response = agent_lock.execute_task(task).await.map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))?;

                    if response.success {
                        if let Some(data) = response.data {
                            if let Some(event_id) = data.get("event_id").and_then(|v| v.as_str()) {
                                return Ok(event_id.to_string());
                            }
                        }
                    }

                    return Err(agent_mem_traits::AgentMemError::MemoryError(
                        "Failed to create episodic event".to_string()
                    ));
                } else {
                    warn!("EpisodicAgent 未初始化，降级到 SemanticAgent");
                }
            }
            MemoryType::Procedural => {
                if let Some(agent) = &self.procedural_agent {
                    debug!("路由到 ProceduralAgent");
                    let task = TaskRequest::new(
                        MemoryType::Procedural,
                        "create_skill".to_string(),
                        params,
                    );

                    let mut agent_lock = agent.write().await;
                    let response = agent_lock.execute_task(task).await.map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))?;

                    if response.success {
                        if let Some(data) = response.data {
                            if let Some(skill_id) = data.get("skill_id").and_then(|v| v.as_str()) {
                                return Ok(skill_id.to_string());
                            }
                        }
                    }

                    return Err(agent_mem_traits::AgentMemError::MemoryError(
                        "Failed to create procedural skill".to_string()
                    ));
                } else {
                    warn!("ProceduralAgent 未初始化，降级到 SemanticAgent");
                }
            }
            _ => {
                debug!("记忆类型 {:?} 使用 SemanticAgent", memory_type);
            }
        }

        // 默认使用 SemanticAgent（最通用的记忆类型）
        if let Some(agent) = &self.semantic_agent {
            use chrono::Utc;

            debug!("使用 SemanticAgent 处理记忆");

            // 构造完整的 SemanticMemoryItem
            let user_id_str = params.get("user_id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
            let item = agent_mem_traits::SemanticMemoryItem {
                id: uuid::Uuid::new_v4().to_string(),
                organization_id: "default".to_string(),
                user_id: user_id_str,
                agent_id: agent_id.clone(),
                name: content.chars().take(100).collect(),  // 使用前100个字符作为名称
                summary: content.clone(),
                details: Some(content.clone()),
                source: None,
                tree_path: vec![],
                metadata: params.get("metadata").cloned().unwrap_or(serde_json::json!({})),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let item_params = serde_json::to_value(&item)?;

            let task = TaskRequest::new(
                MemoryType::Semantic,
                "insert".to_string(),
                item_params,
            );

            let mut agent_lock = agent.write().await;
            let response = agent_lock.execute_task(task).await.map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))?;

            if response.success {
                if let Some(data) = response.data {
                    // SemanticAgent 返回 "item_id" 而不是 "knowledge_id"
                    if let Some(item_id) = data.get("item_id").and_then(|v| v.as_str()) {
                        return Ok(item_id.to_string());
                    }
                }
            }

            return Err(agent_mem_traits::AgentMemError::MemoryError(
                format!("Failed to insert semantic knowledge: {:?}", response.error)
            ));
        }

        // 如果所有 Agent 都不可用，返回错误
        Err(agent_mem_traits::AgentMemError::MemoryError(
            "No available agent to handle memory".to_string()
        ))
    }

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
        let memory_id = self.add_memory(
            content.clone(),
            agent_id.clone(),
            user_id.clone(),
            mem_type,
            metadata,
        ).await?;

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
    pub async fn get_memory(&self, memory_id: &str) -> Result<MemoryItem> {
        use agent_mem_core::coordination::meta_manager::TaskRequest;

        debug!("获取记忆: {}", memory_id);

        // 准备参数
        let params = serde_json::json!({
            "id": memory_id,
        });

        // 尝试从各个 Agent 获取（因为我们不知道记忆存储在哪个 Agent）
        // 优先尝试 SemanticAgent（最常用）
        if let Some(agent) = &self.semantic_agent {
            let task = TaskRequest::new(
                MemoryType::Semantic,
                "read_knowledge".to_string(),
                params.clone(),
            );

            let mut agent_lock = agent.write().await;
            if let Ok(response) = agent_lock.execute_task(task).await {
                if response.success {
                    if let Some(data) = response.data {
                        if let Ok(item) = serde_json::from_value::<MemoryItem>(data) {
                            return Ok(item);
                        }
                    }
                }
            }
        }

        // 尝试 EpisodicAgent
        if let Some(agent) = &self.episodic_agent {
            let task = TaskRequest::new(
                MemoryType::Episodic,
                "read_event".to_string(),
                params.clone(),
            );

            let mut agent_lock = agent.write().await;
            if let Ok(response) = agent_lock.execute_task(task).await {
                if response.success {
                    if let Some(data) = response.data {
                        if let Ok(item) = serde_json::from_value::<MemoryItem>(data) {
                            return Ok(item);
                        }
                    }
                }
            }
        }

        // 尝试 CoreAgent
        if let Some(agent) = &self.core_agent {
            let task = TaskRequest::new(
                MemoryType::Core,
                "read_block".to_string(),
                params,
            );

            let mut agent_lock = agent.write().await;
            if let Ok(response) = agent_lock.execute_task(task).await {
                if response.success {
                    if let Some(data) = response.data {
                        if let Ok(item) = serde_json::from_value::<MemoryItem>(data) {
                            return Ok(item);
                        }
                    }
                }
            }
        }

        // 如果所有 Agent 都找不到，返回错误
        Err(agent_mem_traits::AgentMemError::NotFound(format!(
            "Memory {} not found",
            memory_id
        )))
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
    pub async fn update_memory(
        &self,
        memory_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<MemoryItem> {
        use agent_mem_core::coordination::meta_manager::TaskRequest;

        debug!("更新记忆: {}", memory_id);

        // 准备参数
        let mut params = serde_json::json!({
            "id": memory_id,
        });

        // 合并更新数据
        if let Some(obj) = params.as_object_mut() {
            for (key, value) in data {
                obj.insert(key, value);
            }
        }

        // 尝试从各个 Agent 更新
        if let Some(agent) = &self.semantic_agent {
            let task = TaskRequest::new(
                MemoryType::Semantic,
                "update_knowledge".to_string(),
                params.clone(),
            );

            let mut agent_lock = agent.write().await;
            if let Ok(response) = agent_lock.execute_task(task).await {
                if response.success {
                    if let Some(data) = response.data {
                        if let Ok(item) = serde_json::from_value::<MemoryItem>(data) {
                            return Ok(item);
                        }
                    }
                }
            }
        }

        if let Some(agent) = &self.episodic_agent {
            let task = TaskRequest::new(
                MemoryType::Episodic,
                "update_event".to_string(),
                params.clone(),
            );

            let mut agent_lock = agent.write().await;
            if let Ok(response) = agent_lock.execute_task(task).await {
                if response.success {
                    if let Some(data) = response.data {
                        if let Ok(item) = serde_json::from_value::<MemoryItem>(data) {
                            return Ok(item);
                        }
                    }
                }
            }
        }

        if let Some(agent) = &self.core_agent {
            let task = TaskRequest::new(
                MemoryType::Core,
                "update_block".to_string(),
                params,
            );

            let mut agent_lock = agent.write().await;
            if let Ok(response) = agent_lock.execute_task(task).await {
                if response.success {
                    if let Some(data) = response.data {
                        if let Ok(item) = serde_json::from_value::<MemoryItem>(data) {
                            return Ok(item);
                        }
                    }
                }
            }
        }

        Err(agent_mem_traits::AgentMemError::NotFound(format!(
            "Memory {} not found",
            memory_id
        )))
    }

    /// 删除记忆
    pub async fn delete_memory(&self, memory_id: &str) -> Result<()> {
        use agent_mem_core::coordination::meta_manager::TaskRequest;

        debug!("删除记忆: {}", memory_id);

        let params = serde_json::json!({
            "id": memory_id,
        });

        // 尝试从各个 Agent 删除
        let mut deleted = false;

        if let Some(agent) = &self.semantic_agent {
            let task = TaskRequest::new(
                MemoryType::Semantic,
                "delete_knowledge".to_string(),
                params.clone(),
            );

            let mut agent_lock = agent.write().await;
            if let Ok(response) = agent_lock.execute_task(task).await {
                if response.success {
                    deleted = true;
                }
            }
        }

        if !deleted {
            if let Some(agent) = &self.episodic_agent {
                let task = TaskRequest::new(
                    MemoryType::Episodic,
                    "delete_event".to_string(),
                    params.clone(),
                );

                let mut agent_lock = agent.write().await;
                if let Ok(response) = agent_lock.execute_task(task).await {
                    if response.success {
                        deleted = true;
                    }
                }
            }
        }

        if !deleted {
            if let Some(agent) = &self.core_agent {
                let task = TaskRequest::new(
                    MemoryType::Core,
                    "delete_block".to_string(),
                    params,
                );

                let mut agent_lock = agent.write().await;
                if let Ok(response) = agent_lock.execute_task(task).await {
                    if response.success {
                        deleted = true;
                    }
                }
            }
        }

        if deleted {
            Ok(())
        } else {
            Err(agent_mem_traits::AgentMemError::NotFound(format!(
                "Memory {} not found",
                memory_id
            )))
        }
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
        use agent_mem_traits::{Session, Entity, Relation};
        use std::collections::HashMap;

        let mut metadata = HashMap::new();
        if let Some(details) = &item.details {
            metadata.insert("details".to_string(), serde_json::json!(details));
        }
        if let Some(source) = &item.source {
            metadata.insert("source".to_string(), serde_json::json!(source));
        }
        metadata.insert("tree_path".to_string(), serde_json::json!(item.tree_path));
        metadata.insert("organization_id".to_string(), serde_json::json!(item.organization_id));

        // 合并原有的 metadata
        if let Ok(meta_map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(item.metadata.clone()) {
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
            importance: 0.5,  // 默认重要性
            embedding: None,
            last_accessed_at: item.updated_at,
            access_count: 0,
            expires_at: None,
            version: 1,
        }
    }
}

