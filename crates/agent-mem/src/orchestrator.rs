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
};
use agent_mem_traits::{
    MemoryItem, MemoryType, Result,
    // TODO: 在任务 1.2 中使用这些类型
    // FactExtractor, DecisionEngine, LLMProvider,
};

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
            Ok(agent) => {
                info!("CoreAgent 初始化成功");
                Some(Arc::new(RwLock::new(agent)))
            }
            Err(e) => {
                warn!("CoreAgent 初始化失败: {}, 将禁用核心记忆功能", e);
                None
            }
        };
        
        let episodic_agent = match EpisodicAgent::from_env("default".to_string()).await {
            Ok(agent) => {
                info!("EpisodicAgent 初始化成功");
                Some(Arc::new(RwLock::new(agent)))
            }
            Err(e) => {
                warn!("EpisodicAgent 初始化失败: {}, 将禁用情景记忆功能", e);
                None
            }
        };
        
        let semantic_agent = match SemanticAgent::from_env("default".to_string()).await {
            Ok(agent) => {
                info!("SemanticAgent 初始化成功");
                Some(Arc::new(RwLock::new(agent)))
            }
            Err(e) => {
                warn!("SemanticAgent 初始化失败: {}, 将禁用语义记忆功能", e);
                None
            }
        };
        
        // ProceduralAgent 暂时不支持 from_env，使用 new() 创建
        let procedural_agent = {
            info!("ProceduralAgent 初始化（基础模式）");
            Some(Arc::new(RwLock::new(ProceduralAgent::new("default".to_string()))))
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
    /// 注意：当前实现为基础版本，返回空结果
    /// 完整的跨 Agent 搜索将在后续任务中实现
    pub async fn search_memories(
        &self,
        query: String,
        _agent_id: String,
        _user_id: Option<String>,
        _limit: usize,
        memory_type: Option<MemoryType>,
    ) -> Result<Vec<MemoryItem>> {
        debug!("搜索记忆: query={}, memory_type={:?}", query, memory_type);

        // 根据 memory_type 决定搜索哪些 Agents
        let search_all = memory_type.is_none();

        if search_all || memory_type == Some(MemoryType::Core) {
            if self.core_agent.is_some() {
                debug!("将搜索 CoreAgent");
            }
        }

        if search_all || memory_type == Some(MemoryType::Episodic) {
            if self.episodic_agent.is_some() {
                debug!("将搜索 EpisodicAgent");
            }
        }

        if search_all || memory_type == Some(MemoryType::Semantic) {
            if self.semantic_agent.is_some() {
                debug!("将搜索 SemanticAgent");
            }
        }

        // TODO: 实现实际的跨 Agent 搜索
        // 当前返回空结果
        debug!("搜索完成（基础实现），返回空结果");
        Ok(vec![])
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
    /// 注意：当前实现为基础版本，返回占位符 ID
    /// 完整的 Agent 集成将在后续任务中实现
    async fn route_add_to_agent(
        &self,
        memory_type: MemoryType,
        _content: String,
        _agent_id: String,
        _user_id: Option<String>,
        _metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        // 根据记忆类型记录日志
        match memory_type {
            MemoryType::Core => {
                if self.core_agent.is_some() {
                    debug!("路由到 CoreAgent");
                } else {
                    warn!("CoreAgent 未初始化");
                }
            }
            MemoryType::Episodic => {
                if self.episodic_agent.is_some() {
                    debug!("路由到 EpisodicAgent");
                } else {
                    warn!("EpisodicAgent 未初始化");
                }
            }
            MemoryType::Semantic => {
                if self.semantic_agent.is_some() {
                    debug!("路由到 SemanticAgent");
                } else {
                    warn!("SemanticAgent 未初始化");
                }
            }
            MemoryType::Procedural => {
                if self.procedural_agent.is_some() {
                    debug!("路由到 ProceduralAgent");
                } else {
                    warn!("ProceduralAgent 未初始化");
                }
            }
            _ => {
                debug!("记忆类型 {:?} 使用默认处理", memory_type);
            }
        }

        // TODO: 实现实际的 Agent 调用
        // 当前返回占位符 ID
        Ok(uuid::Uuid::new_v4().to_string())
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
        debug!("获取记忆: {}", memory_id);

        // TODO: 实现从存储后端获取记忆
        // 当前返回错误
        Err(agent_mem_traits::AgentMemError::NotFound(format!(
            "Memory {} not found (get_memory not yet implemented)",
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
        _data: HashMap<String, serde_json::Value>,
    ) -> Result<MemoryItem> {
        debug!("更新记忆: {}", memory_id);

        // TODO: 实现记忆更新
        // 当前返回错误
        Err(agent_mem_traits::AgentMemError::NotFound(format!(
            "Memory {} not found (update_memory not yet implemented)",
            memory_id
        )))
    }

    /// 删除记忆
    pub async fn delete_memory(&self, memory_id: &str) -> Result<()> {
        debug!("删除记忆: {}", memory_id);

        // TODO: 实现记忆删除
        // 当前返回错误
        Err(agent_mem_traits::AgentMemError::NotFound(format!(
            "Memory {} not found (delete_memory not yet implemented)",
            memory_id
        )))
    }

    /// 删除所有记忆
    pub async fn delete_all_memories(
        &self,
        _agent_id: String,
        _user_id: Option<String>,
        _run_id: Option<String>,
    ) -> Result<usize> {
        debug!("删除所有记忆");

        // TODO: 实现批量删除
        // 当前返回 0
        Ok(0)
    }
}

