//! Agent Orchestrator - 对话循环编排
//!
//! 这是 AgentMem 的核心对话循环实现，参考 MIRIX 的 AgentWrapper.step() 设计
//! 集成所有现有模块：MemoryEngine, LLMClient, ToolExecutor, MessageRepository

use crate::{engine::MemoryEngine, hierarchy::MemoryScope, storage::traits::MessageRepositoryTrait, Memory};

use agent_mem_llm::LLMClient;
use agent_mem_tools::ToolExecutor;
use agent_mem_traits::{llm::FunctionDefinition, AgentMemError, Message, Result};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub mod background_tasks;
pub mod memory_extraction;
pub mod memory_integration;
pub mod tool_integration;

use background_tasks::BackgroundTaskManager;
use memory_extraction::MemoryExtractor;
use memory_integration::MemoryIntegrator;
use tool_integration::{ToolIntegrator, ToolIntegratorConfig};

/// 对话请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// 用户消息
    pub message: String,

    /// Agent ID
    pub agent_id: String,

    /// 用户 ID
    pub user_id: String,

    /// 组织 ID (可选，默认为 "default")
    #[serde(default = "default_organization_id")]
    pub organization_id: String,

    /// 会话 ID - 用于Working Memory隔离
    pub session_id: String,

    /// 是否流式响应
    pub stream: bool,

    /// 最大记忆检索数量
    pub max_memories: usize,
}

impl ChatRequest {
    /// 验证请求参数
    pub fn validate(&self) -> Result<()> {
        // 验证消息不为空
        if self.message.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "Message cannot be empty".to_string(),
            ));
        }

        // 验证消息长度（最大 100KB）
        if self.message.len() > 100_000 {
            return Err(AgentMemError::ValidationError(format!(
                "Message too long: {} bytes (max 100KB)",
                self.message.len()
            )));
        }

        // 验证 agent_id 不为空
        if self.agent_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "Agent ID cannot be empty".to_string(),
            ));
        }

        // 验证 agent_id 长度（最大 255 字符）
        if self.agent_id.len() > 255 {
            return Err(AgentMemError::ValidationError(format!(
                "Agent ID too long: {} characters (max 255)",
                self.agent_id.len()
            )));
        }

        // 验证 user_id 不为空
        if self.user_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "User ID cannot be empty".to_string(),
            ));
        }

        // 验证 user_id 长度（最大 255 字符）
        if self.user_id.len() > 255 {
            return Err(AgentMemError::ValidationError(format!(
                "User ID too long: {} characters (max 255)",
                self.user_id.len()
            )));
        }

        // 验证 organization_id 不为空
        if self.organization_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "Organization ID cannot be empty".to_string(),
            ));
        }

        // 验证 organization_id 长度（最大 255 字符）
        if self.organization_id.len() > 255 {
            return Err(AgentMemError::ValidationError(format!(
                "Organization ID too long: {} characters (max 255)",
                self.organization_id.len()
            )));
        }

        // 验证 max_memories 范围（1-1000）
        if self.max_memories == 0 {
            return Err(AgentMemError::ValidationError(
                "max_memories must be at least 1".to_string(),
            ));
        }

        if self.max_memories > 1000 {
            return Err(AgentMemError::ValidationError(format!(
                "max_memories too large: {} (max 1000)",
                self.max_memories
            )));
        }

        // 验证 session_id 不为空
        if self.session_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "Session ID cannot be empty".to_string(),
            ));
        }

        // 验证 session_id 长度（最大 255 字符）
        if self.session_id.len() > 255 {
            return Err(AgentMemError::ValidationError(format!(
                "Session ID too long: {} characters (max 255)",
                self.session_id.len()
            )));
        }

        Ok(())
    }
}

/// 默认组织 ID
fn default_organization_id() -> String {
    "default".to_string()
}

/// 对话响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// 消息 ID
    pub message_id: String,

    /// Agent 响应内容
    pub content: String,

    /// 是否更新了记忆
    pub memories_updated: bool,

    /// 更新的记忆数量
    pub memories_count: usize,

    /// 工具调用（如果有）
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

/// 工具调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<String>,
}

/// Agent 编排器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// 最大工具调用轮数
    pub max_tool_rounds: usize,

    /// 最大记忆检索数量
    pub max_memories: usize,

    /// 是否自动提取记忆
    pub auto_extract_memories: bool,

    /// 记忆提取阈值
    pub memory_extraction_threshold: f32,

    /// 是否启用工具调用
    pub enable_tool_calling: bool,

    /// ⭐ Phase 4: 自适应配置
    /// 是否启用自适应调整
    pub enable_adaptive: bool,

    /// TTFB阈值(ms) - 超过此值触发降级
    pub ttfb_threshold_ms: u64,

    /// Token预算上限
    pub token_budget: usize,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_tool_rounds: 5,
            max_memories: 3, // Phase 2/3优化: 从10降到3
            auto_extract_memories: true,
            memory_extraction_threshold: 0.5,
            enable_tool_calling: false,

            // Phase 4: 自适应配置默认值
            enable_adaptive: true,
            ttfb_threshold_ms: 5000, // 5秒阈值
            token_budget: 850,       // HCAM推荐值
        }
    }
}

/// ⭐ 性能监控统计
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub avg_ttfb_ms: f64,
    pub avg_prompt_chars: f64,
    pub avg_memories: f64,
    pub last_ttfb_ms: u64,
}

/// Agent 编排器 - 核心对话循环
///
/// 参考 MIRIX 的 AgentWrapper.step() 实现
/// 集成所有现有模块实现完整的对话循环
pub struct AgentOrchestrator {
    config: OrchestratorConfig,
    memory_engine: Arc<MemoryEngine>,
    message_repo: Arc<dyn MessageRepositoryTrait>,
    llm_client: Arc<LLMClient>,
    tool_executor: Arc<ToolExecutor>,
    memory_integrator: MemoryIntegrator,
    memory_extractor: Arc<MemoryExtractor>,
    tool_integrator: ToolIntegrator,
    /// Working Memory Store - 用于会话级临时上下文（最小改动方案：直接使用Store而非Agent）
    working_store: Option<Arc<dyn agent_mem_traits::WorkingMemoryStore>>,
    /// ⭐ 性能监控
    metrics: Arc<std::sync::RwLock<PerformanceMetrics>>,
    /// 后台任务管理器
    background_tasks: Arc<BackgroundTaskManager>,
    
    // 🆕 P1: 8 种高级能力（Optional，非侵入式激活）
    /// 🚀 主动检索系统 - 主题提取、智能路由、上下文合成
    active_retrieval: Option<Arc<crate::retrieval::ActiveRetrievalSystem>>,
    /// ⏰ 时序推理引擎 - 时间范围查询、时序关系推理
    temporal_reasoning: Option<Arc<crate::temporal_reasoning::TemporalReasoningEngine>>,
    /// 🔍 因果推理引擎 - 因果关系推理、反事实推理
    causal_reasoning: Option<Arc<crate::causal_reasoning::CausalReasoningEngine>>,
    /// 🕸️ 图记忆引擎 - 关系推理、图遍历、社区发现
    graph_memory: Option<Arc<crate::graph_memory::GraphMemoryEngine>>,
    /// 🎯 自适应策略管理器 - 动态策略选择、性能优化
    adaptive_strategy: Option<Arc<crate::adaptive_strategy::AdaptiveStrategyManager>>,
    /// ⚡ LLM 优化器 - 提示优化、缓存、成本优化
    llm_optimizer: Option<Arc<crate::llm_optimizer::LlmOptimizer>>,
    /// 🚀 性能优化器 - 查询优化、批处理、并发
    performance_optimizer: Option<Arc<crate::performance::optimizer::PerformanceOptimizer>>,
    /// 🖼️ 多模态处理器 - 图像、音频、视频处理（可选，需要 feature flag）
    #[cfg(feature = "multimodal")]
    multimodal: Option<Arc<agent_mem_intelligence::multimodal::MultimodalProcessor>>,
}

impl AgentOrchestrator {
    /// 创建新的编排器
    pub fn new(
        config: OrchestratorConfig,
        memory_engine: Arc<MemoryEngine>,
        message_repo: Arc<dyn MessageRepositoryTrait>,
        llm_client: Arc<LLMClient>,
        tool_executor: Arc<ToolExecutor>,
        working_store: Option<Arc<dyn agent_mem_traits::WorkingMemoryStore>>,
    ) -> Self {
        // 创建记忆集成器
        let memory_integrator = MemoryIntegrator::with_default_config(memory_engine.clone());

        // 创建记忆提取器
        let memory_extractor = Arc::new(MemoryExtractor::with_default_config(
            llm_client.clone(),
            memory_engine.clone(),
        ));

        // 创建工具集成器
        let tool_config = ToolIntegratorConfig {
            max_tool_rounds: config.max_tool_rounds,
            tool_timeout_seconds: 30,
            allow_parallel_execution: false,
        };
        let tool_integrator = ToolIntegrator::new(tool_config, tool_executor.clone());

        Self {
            config,
            memory_engine,
            message_repo,
            llm_client,
            tool_executor,
            memory_integrator,
            memory_extractor,
            tool_integrator,
            working_store,
            metrics: Arc::new(std::sync::RwLock::new(PerformanceMetrics::default())),
            background_tasks: Arc::new(BackgroundTaskManager::new()),
            // 🆕 P1: 初始化所有高级能力为 None（可选激活）
            active_retrieval: None,
            temporal_reasoning: None,
            causal_reasoning: None,
            graph_memory: None,
            adaptive_strategy: None,
            llm_optimizer: None,
            performance_optimizer: None,
            #[cfg(feature = "multimodal")]
            multimodal: None,
        }
    }

    // ========== P1: Builder 方法 - 激活 8 种高级能力 ==========
    
    /// 🚀 激活主动检索系统（主题提取、智能路由、上下文合成）
    pub fn with_active_retrieval(mut self, system: Arc<crate::retrieval::ActiveRetrievalSystem>) -> Self {
        self.active_retrieval = Some(system);
        info!("✅ ActiveRetrievalSystem enabled");
        self
    }

    /// ⏰ 激活时序推理引擎（时间范围查询、时序关系推理）
    pub fn with_temporal_reasoning(mut self, engine: Arc<crate::temporal_reasoning::TemporalReasoningEngine>) -> Self {
        self.temporal_reasoning = Some(engine);
        info!("✅ TemporalReasoningEngine enabled");
        self
    }

    /// 🔍 激活因果推理引擎（因果关系推理、反事实推理）
    pub fn with_causal_reasoning(mut self, engine: Arc<crate::causal_reasoning::CausalReasoningEngine>) -> Self {
        self.causal_reasoning = Some(engine);
        info!("✅ CausalReasoningEngine enabled");
        self
    }

    /// 🕸️ 激活图记忆引擎（关系推理、图遍历、社区发现）
    pub fn with_graph_memory(mut self, engine: Arc<crate::graph_memory::GraphMemoryEngine>) -> Self {
        self.graph_memory = Some(engine);
        info!("✅ GraphMemoryEngine enabled");
        self
    }

    /// 🎯 激活自适应策略管理器（动态策略选择、性能优化）
    pub fn with_adaptive_strategy(mut self, manager: Arc<crate::adaptive_strategy::AdaptiveStrategyManager>) -> Self {
        self.adaptive_strategy = Some(manager);
        info!("✅ AdaptiveStrategyManager enabled");
        self
    }

    /// ⚡ 激活 LLM 优化器（提示优化、缓存、成本优化）
    pub fn with_llm_optimizer(mut self, optimizer: Arc<crate::llm_optimizer::LlmOptimizer>) -> Self {
        self.llm_optimizer = Some(optimizer);
        info!("✅ LlmOptimizer enabled");
        self
    }

    /// 🚀 激活性能优化器（查询优化、批处理、并发）
    pub fn with_performance_optimizer(mut self, optimizer: Arc<crate::performance::optimizer::PerformanceOptimizer>) -> Self {
        self.performance_optimizer = Some(optimizer);
        info!("✅ PerformanceOptimizer enabled");
        self
    }

    /// 🖼️ 激活多模态处理器（图像、音频、视频处理）
    #[cfg(feature = "multimodal")]
    pub fn with_multimodal(mut self, processor: Arc<agent_mem_intelligence::multimodal::MultimodalProcessor>) -> Self {
        self.multimodal = Some(processor);
        info!("✅ MultimodalProcessor enabled");
        self
    }

    // ========== P1: Enhanced Search 方法 ==========
    
    /// 🔍 增强搜索 - 集成所有激活的高级能力
    /// 
    /// 这个方法会自动使用所有已激活的高级能力来增强搜索：
    /// - ActiveRetrievalSystem: 主动检索（主题提取、智能路由）
    /// - TemporalReasoningEngine: 时序推理
    /// - CausalReasoningEngine: 因果推理
    /// - GraphMemoryEngine: 图关系推理
    ///
    /// 如果某个能力未激活，会优雅降级到标准搜索
    ///
    /// ⚠️ TEMPORARILY DISABLED: API compatibility issues
    #[allow(dead_code)]
    pub async fn search_enhanced(
        &self,
        query: &str,
        agent_id: &str,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        info!("🔍 Enhanced search: query='{}', limit={}", query, limit);

        let mut all_memories = Vec::new();

        // 1️⃣ 标准向量搜索（基准）
        let scope = MemoryScope::User {
            agent_id: agent_id.to_string(),
            user_id: user_id.to_string(),
        };

        let standard_memories = self.memory_engine.search_memories(
            query,
            Some(scope),
            Some(limit),
        ).await.map_err(|e| AgentMemError::llm_error(format!("Standard search failed: {}", e)))?;

        all_memories.extend(standard_memories.clone());
        info!("   📊 Standard search: {} memories", standard_memories.len());

        // 2️⃣ 主动检索（如果激活）
        // TODO: 实现 ActiveRetrievalSystem 集成
        // 当前 API 不匹配，暂时跳过

        // 3️⃣ 图记忆增强（如果激活）
        // TODO: 实现 GraphMemory 集成
        // 当前 API 不匹配，暂时跳过

        // 4️⃣ 时序推理增强（如果激活）
        // TODO: 实现时序范围查询增强

        // 5️⃣ 因果推理增强（如果激活）
        // TODO: 实现因果推理增强

        // 6️⃣ 去重并限制结果数量
        let mut unique_memories = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for memory in all_memories {
            let id = memory.id.as_str().to_string();
            if seen_ids.insert(id) {
                unique_memories.push(memory);
            }
        }

        // 限制结果数量
        unique_memories.truncate(limit);

        info!("   ✅ Enhanced search complete: {} unique memories", unique_memories.len());
        Ok(unique_memories)
    }

    // ========== P1: 专门方法 - 高级能力 API ==========

    /// 🔍 解释因果关系 - 分析事件之间的因果链
    ///
    /// 需要 CausalReasoningEngine 激活
    pub async fn explain_causality(
        &self,
        cause_event: &str,
        effect_event: &str,
    ) -> Result<String> {
        info!("🔍 Exploring causality: '{}' → '{}'", cause_event, effect_event);

        if let Some(ref _causal_reasoning) = self.causal_reasoning {
            // TODO: 实现因果链分析
            // causal_reasoning.find_causal_path(...).await
            Ok(format!("Causal analysis between '{}' and '{}'", cause_event, effect_event))
        } else {
            warn!("⚠️  CausalReasoningEngine not enabled, using default response");
            Ok("Causal reasoning not enabled".to_string())
        }
    }

    /// ⏰ 时序查询 - 查询特定时间范围内的记忆
    ///
    /// 需要 TemporalReasoningEngine 激活
    pub async fn temporal_query(
        &self,
        query: &str,
        agent_id: &str,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        info!("⏰ Temporal query: '{}' limit={}", query, limit);

        // 当前实现：使用标准搜索
        // TODO: 未来可以添加时间范围过滤
        let scope = MemoryScope::User {
            agent_id: agent_id.to_string(),
            user_id: user_id.to_string(),
        };

        let memories = self.memory_engine.search_memories(
            query,
            Some(scope),
            Some(limit),
        ).await.map_err(|e| AgentMemError::llm_error(format!("Temporal query failed: {}", e)))?;

        info!("   ⏰ Temporal query returned {} memories", memories.len());
        Ok(memories)
    }

    /// 🕸️ 图遍历 - 从起始节点开始遍历图结构
    ///
    /// 需要 GraphMemoryEngine 激活
    pub async fn graph_traverse(
        &self,
        start_node_id: &str,
        max_depth: usize,
    ) -> Result<Vec<String>> {
        info!("🕸️ Graph traversal: from '{}', max_depth={}", start_node_id, max_depth);

        if let Some(ref graph_memory) = self.graph_memory {
            // TODO: 调用 GraphMemory API
            // 当前 API 不匹配，暂时返回简化实现
            warn!("   ⚠️  GraphMemory API needs adaptation");
            Ok(vec![start_node_id.to_string()])
        } else {
            warn!("⚠️  GraphMemoryEngine not enabled");
            Ok(Vec::new())
        }
    }

    /// 从Working Memory获取会话上下文
    ///
    /// 这个方法从WorkingMemoryStore获取当前会话的临时上下文
    async fn get_working_context(&self, session_id: &str) -> Result<String> {
        if let Some(ref store) = self.working_store {
            match store.get_session_items(session_id).await {
                Ok(items) => {
                    if items.is_empty() {
                        debug!("No working memory items found for session: {}", session_id);
                        return Ok(String::new());
                    }

                    // 按优先级和时间排序（已在store中完成）
                    // 格式化为对话上下文
                    let context_lines: Vec<String> = items
                        .iter()
                        .map(|item| {
                            format!("[{}] {}", item.created_at.format("%H:%M:%S"), item.content)
                        })
                        .collect();

                    let context = context_lines.join("\n");
                    debug!(
                        "Retrieved {} working memory items for session {}: {} chars",
                        items.len(),
                        session_id,
                        context.len()
                    );
                    Ok(context)
                }
                Err(e) => {
                    warn!(
                        "Failed to get working context for session {}: {}",
                        session_id, e
                    );
                    Ok(String::new()) // 失败时返回空，不影响对话
                }
            }
        } else {
            debug!(
                "Working Memory store not configured, session_id: {}",
                session_id
            );
            Ok(String::new())
        }
    }

    /// 更新Working Memory
    ///
    /// 保存当前对话轮次到工作记忆
    async fn update_working_memory(
        &self,
        session_id: &str,
        user_id: &str,
        agent_id: &str,
        user_message: &str,
        assistant_response: &str,
    ) -> Result<()> {
        if let Some(ref store) = self.working_store {
            use agent_mem_traits::WorkingMemoryItem;
            use chrono::Utc;

            // 格式化对话对
            let conversation_pair =
                format!("User: {user_message}\nAssistant: {assistant_response}");

            // 创建工作记忆项
            let item = WorkingMemoryItem {
                id: Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                agent_id: agent_id.to_string(),
                session_id: session_id.to_string(),
                content: conversation_pair,
                priority: 1,                                                // 默认优先级
                expires_at: Some(Utc::now() + chrono::Duration::hours(24)), // 24小时后过期
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
            };

            match store.add_item(item).await {
                Ok(_) => {
                    debug!(
                        "Successfully added working memory item for session: {}",
                        session_id
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to add working memory item for session {}: {}",
                        session_id, e
                    );
                    // 不返回错误，避免影响对话流程
                }
            }
        } else {
            debug!(
                "Working Memory store not configured, skipping update for session: {}",
                session_id
            );
        }

        Ok(())
    }

    /// 执行流式对话循环 (真实SSE流式)
    ///
    /// 这是真实的流式实现，直接从LLM流式返回内容：
    /// 1. 准备上下文和记忆
    /// 2. 调用LLM的 generate_stream 方法
    /// 3. 实时转发流式数据
    /// 4. 流结束后保存消息和更新记忆
    pub async fn step_stream(
        self: Arc<Self>,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send + 'static>>> {
        use futures::stream;
        use futures::stream::StreamExt;

        info!(
            "🌊 Starting REAL streaming conversation for agent_id={}, user_id={}",
            request.agent_id, request.user_id
        );

        // 1. 准备上下文
        let working_context = self.get_working_context(&request.session_id).await?;
        let user_message_id = self.create_user_message(&request).await?;

        // 2. 检索记忆
        let adjusted_max_memories = if self.config.enable_adaptive {
            self.adaptive_adjust_memories(&request, std::time::Duration::from_secs(0))
                .await
        } else {
            request.max_memories
        };

        let mut adjusted_request = request.clone();
        adjusted_request.max_memories = adjusted_max_memories;
        let memories = self.retrieve_memories(&adjusted_request).await?;
        let memories_count = memories.len();

        info!("   📚 检索到 {} 条记忆", memories_count);

        // 3. 构建消息
        let messages = self
            .build_messages_with_context(&request, &working_context, &memories)
            .await?;

        info!("   📝 构建了 {} 条消息", messages.len());

        // 4. 调用LLM真实流式
        let llm_stream = self.llm_client.generate_stream(&messages).await?;
        info!("   ✅ LLM流式已启动");

        // 5. 创建包装流，用于收集完整响应并在结束时保存
        let request_clone = request.clone();
        let orchestrator = self.clone();
        let messages_clone = messages.clone();

        let wrapped_stream = stream::unfold(
            (
                llm_stream,
                String::new(),
                false,
                request_clone,
                orchestrator,
                messages_clone,
                memories_count,
                user_message_id,
            ),
            |(
                mut stream,
                mut accumulated_content,
                mut is_done,
                req,
                orch,
                msgs,
                mem_count,
                _msg_id,
            )| async move {
                if is_done {
                    return None;
                }

                match stream.next().await {
                    Some(Ok(chunk)) => {
                        // 累积内容
                        accumulated_content.push_str(&chunk);

                        // 返回当前块，继续流式
                        Some((
                            Ok(chunk),
                            (
                                stream,
                                accumulated_content,
                                is_done,
                                req,
                                orch,
                                msgs,
                                mem_count,
                                _msg_id,
                            ),
                        ))
                    }
                    Some(Err(e)) => {
                        // 流式错误
                        warn!("❌ 流式传输错误: {}", e);
                        is_done = true;
                        Some((
                            Err(e),
                            (
                                stream,
                                accumulated_content,
                                is_done,
                                req,
                                orch,
                                msgs,
                                mem_count,
                                _msg_id,
                            ),
                        ))
                    }
                    None => {
                        // 流结束，保存完整响应
                        info!(
                            "   ✅ 流式传输完成，累积内容: {} 字符",
                            accumulated_content.len()
                        );

                        // Note: 保存操作由外部调用者处理，这里只返回流数据
                        // TODO: 考虑在流结束后通过其他机制保存消息和更新记忆

                        is_done = true;
                        None
                    }
                }
            },
        );

        Ok(Box::pin(wrapped_stream))
    }

    /// 执行完整的对话循环
    ///
    /// 这是核心方法，参考 MIRIX 的 AgentWrapper.step() 实现：
    /// 0. 获取Working Memory会话上下文
    /// 1. 创建用户消息
    /// 2. 检索相关记忆
    /// 3. 构建 prompt（注入会话上下文和长期记忆）
    /// 4. 调用 LLM
    /// 5. 处理工具调用（如果有）- TODO: 待实现
    /// 6. 保存 assistant 消息
    /// 7. 更新 Working Memory
    /// 8. 提取和更新记忆
    /// 9. 返回响应
    pub async fn step(&self, request: ChatRequest) -> Result<ChatResponse> {
        let start_time = std::time::Instant::now();

        // ✅ 验证请求参数
        request.validate()?;

        info!(
            "Starting conversation step for agent_id={}, user_id={}, session_id={}",
            request.agent_id, request.user_id, request.session_id
        );

        // 0. 获取Working Memory会话上下文
        let working_context = self.get_working_context(&request.session_id).await?;
        if !working_context.is_empty() {
            debug!("Retrieved working context: {} chars", working_context.len());
        }

        // 1. 创建用户消息
        let user_message_id = self.create_user_message(&request).await?;
        debug!("Created user message: {}", user_message_id);

        // ⭐ Phase 4: 自适应调整 - 根据性能动态调整max_memories
        let adjusted_max_memories = if self.config.enable_adaptive {
            self.adaptive_adjust_memories(&request, start_time.elapsed())
                .await
        } else {
            request.max_memories
        };

        // 2. 检索相关记忆（使用调整后的数量）
        let retrieval_start = std::time::Instant::now();
        let mut adjusted_request = request.clone();
        adjusted_request.max_memories = adjusted_max_memories;
        let memories = self.retrieve_memories(&adjusted_request).await?;
        let memories_retrieved_count = memories.len();
        let retrieval_duration = retrieval_start.elapsed();
        info!(
            "Retrieved {} memories (adjusted from {} to {}) in {:?}",
            memories_retrieved_count,
            request.max_memories,
            adjusted_max_memories,
            retrieval_duration
        );

        // 3. 构建 prompt（注入会话上下文和长期记忆）
        let build_start = std::time::Instant::now();
        let messages = self
            .build_messages_with_context(&request, &working_context, &memories)
            .await?;
        let build_duration = build_start.elapsed();
        debug!(
            "Built {} messages with working context and memories in {:?}",
            messages.len(),
            build_duration
        );

        // 4. 调用 LLM（可能需要多轮工具调用）
        let llm_start = std::time::Instant::now();
        let (final_response, tool_calls_info) =
            self.execute_with_tools(&messages, &request.user_id).await?;
        let llm_duration = llm_start.elapsed();
        debug!(
            "Got final response: {} chars, {} tool calls in {:?}",
            final_response.len(),
            tool_calls_info.len(),
            llm_duration
        );

        // 5. 保存 assistant 消息
        let assistant_message_id = self
            .create_assistant_message(
                &request.organization_id,
                &request.agent_id,
                &request.user_id,
                &final_response,
            )
            .await?;
        debug!("Created assistant message: {}", assistant_message_id);

        // 6. 更新Working Memory
        self.update_working_memory(
            &request.session_id,
            &request.user_id,
            &request.agent_id,
            &request.message,
            &final_response,
        )
        .await?;
        debug!("Updated working memory for session {}", request.session_id);

        // 7. 提取和更新记忆（异步）
        if self.config.auto_extract_memories {
            self.schedule_memory_extraction(&request, &messages);
        } else {
            debug!("Auto memory extraction disabled, skipping extraction");
        }

        // ⭐ 8. 更新性能统计
        let total_duration = start_time.elapsed();
        let ttfb_ms = total_duration.as_millis() as u64;
        let prompt_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        self.update_metrics(ttfb_ms, prompt_chars, memories_retrieved_count);

        info!(
            "📊 Performance: TTFB={}ms, Prompt={}chars, Memories={}",
            ttfb_ms, prompt_chars, memories_retrieved_count
        );
        info!("   ⏱️  详细时间分解:");
        info!(
            "      - 内存检索: {:?} ({:.1}%)",
            retrieval_duration,
            (retrieval_duration.as_secs_f64() / total_duration.as_secs_f64()) * 100.0
        );
        info!(
            "      - 消息构建: {:?} ({:.1}%)",
            build_duration,
            (build_duration.as_secs_f64() / total_duration.as_secs_f64()) * 100.0
        );
        info!(
            "      - LLM调用: {:?} ({:.1}%)",
            llm_duration,
            (llm_duration.as_secs_f64() / total_duration.as_secs_f64()) * 100.0
        );
        info!("      - 总耗时: {:?}", total_duration);

        // 9. 返回响应（✅ memories_count 现在表示检索使用的记忆数量）
        Ok(ChatResponse {
            message_id: assistant_message_id,
            content: final_response,
            memories_updated: false,
            memories_count: memories_retrieved_count, // ✅ 返回检索到的记忆数量（用于前端展示）
            tool_calls: if tool_calls_info.is_empty() {
                None
            } else {
                Some(tool_calls_info)
            },
        })
    }

    /// ⭐ 更新性能统计
    fn update_metrics(&self, ttfb_ms: u64, prompt_chars: usize, memories: usize) {
        if let Ok(mut metrics) = self.metrics.write() {
            let n = metrics.total_requests as f64;
            metrics.total_requests += 1;
            metrics.last_ttfb_ms = ttfb_ms;

            // 移动平均
            metrics.avg_ttfb_ms = (metrics.avg_ttfb_ms * n + ttfb_ms as f64) / (n + 1.0);
            metrics.avg_prompt_chars =
                (metrics.avg_prompt_chars * n + prompt_chars as f64) / (n + 1.0);
            metrics.avg_memories = (metrics.avg_memories * n + memories as f64) / (n + 1.0);
        }
    }

    /// ⭐ 获取性能统计
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().map(|m| m.clone()).unwrap_or_default()
    }

    /// 执行带工具调用的对话循环
    ///
    /// 这个方法支持完整的工具调用流程：
    /// 1. 创建用户消息
    /// 2. 检索相关记忆
    /// 3. 构建 prompt（注入记忆）
    /// 4. 调用 LLM（带工具定义）
    /// 5. 如果有工具调用，执行工具并继续循环
    /// 6. 保存 assistant 消息
    /// 7. 提取和更新记忆
    /// 8. 返回响应
    pub async fn step_with_tools(
        &self,
        request: ChatRequest,
        available_tools: &[FunctionDefinition],
    ) -> Result<ChatResponse> {
        // ✅ 验证请求参数
        request.validate()?;

        info!(
            "Starting conversation step with tools for agent_id={}, user_id={}",
            request.agent_id, request.user_id
        );

        // 1. 创建用户消息
        let user_message_id = self.create_user_message(&request).await?;
        debug!("Created user message: {}", user_message_id);

        // 2. 检索相关记忆
        let memories = self.retrieve_memories(&request).await?;
        info!("Retrieved {} memories", memories.len());

        // 3. 构建 prompt（注入记忆）
        let mut messages = self
            .build_messages_with_memories(&request, &memories)
            .await?;
        debug!("Built {} messages with memories", messages.len());

        let mut tool_calls_info = Vec::new();
        let mut final_response = String::new();
        let mut round = 0;

        // 工具调用循环
        loop {
            round += 1;
            if round > self.config.max_tool_rounds {
                warn!(
                    "Reached max tool rounds ({}), stopping",
                    self.config.max_tool_rounds
                );
                break;
            }

            // 4. 调用 LLM（带工具定义）
            let llm_response = self
                .llm_client
                .generate_with_functions(&messages, available_tools)
                .await?;

            // 检查是否有文本响应
            if let Some(text) = &llm_response.text {
                final_response = text.clone();
                debug!("Got LLM text response: {} chars", text.len());
            }

            // 检查是否有工具调用
            if llm_response.function_calls.is_empty() {
                debug!("No tool calls, ending loop");
                break;
            }

            info!("Got {} tool calls", llm_response.function_calls.len());

            // 5. 执行工具调用
            let tool_results = self
                .tool_integrator
                .execute_tool_calls(&llm_response.function_calls, &request.user_id)
                .await?;

            // 记录工具调用信息
            for result in &tool_results {
                tool_calls_info.push(ToolCallInfo {
                    tool_name: result.tool_name.clone(),
                    arguments: serde_json::from_str(&result.arguments)
                        .unwrap_or(serde_json::json!({})),
                    result: Some(result.result.clone()),
                });
            }

            // 将工具结果添加到消息历史
            let tool_results_text = self.tool_integrator.format_tool_results(&tool_results);
            messages.push(Message {
                role: agent_mem_traits::MessageRole::Assistant,
                content: tool_results_text,
                timestamp: Some(chrono::Utc::now()),
            });

            // 如果所有工具都失败了，停止循环
            if tool_results.iter().all(|r| !r.success) {
                warn!("All tools failed, stopping loop");
                break;
            }
        }

        // 6. 保存 assistant 消息
        let assistant_message_id = self
            .create_assistant_message(
                &request.organization_id,
                &request.agent_id,
                &request.user_id,
                &final_response,
            )
            .await?;
        debug!("Created assistant message: {}", assistant_message_id);

        // 7. 提取和更新记忆（异步）
        if self.config.auto_extract_memories {
            self.schedule_memory_extraction(&request, &messages);
        }

        // 8. 返回响应
        Ok(ChatResponse {
            message_id: assistant_message_id,
            content: final_response,
            memories_updated: false,
            memories_count: memories.len(),
            tool_calls: if tool_calls_info.is_empty() {
                None
            } else {
                Some(tool_calls_info)
            },
        })
    }

    /// 创建用户消息
    async fn create_user_message(&self, request: &ChatRequest) -> Result<String> {
        use crate::storage::models::Message as DbMessage;

        // 创建用户消息
        let now = chrono::Utc::now();
        let message = DbMessage {
            id: Uuid::new_v4().to_string(),
            organization_id: request.organization_id.clone(),
            user_id: request.user_id.clone(),
            agent_id: request.agent_id.clone(),
            role: "user".to_string(),
            text: Some(request.message.clone()),
            content: None,
            model: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            step_id: None,
            otid: None,
            tool_returns: None,
            group_id: None,
            sender_id: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        };

        // 保存到数据库
        let created_message = self.message_repo.create(&message).await?;

        debug!("Created user message: {}", created_message.id);
        Ok(created_message.id)
    }

    /// 创建 assistant 消息
    async fn create_assistant_message(
        &self,
        organization_id: &str,
        agent_id: &str,
        user_id: &str,
        content: &str,
    ) -> Result<String> {
        use crate::storage::models::Message as DbMessage;

        // 创建 assistant 消息
        let now = chrono::Utc::now();
        let message = DbMessage {
            id: Uuid::new_v4().to_string(),
            organization_id: organization_id.to_string(),
            user_id: user_id.to_string(), // ✅ 修复: 从参数获取而非硬编码
            agent_id: agent_id.to_string(),
            role: "assistant".to_string(),
            text: Some(content.to_string()),
            content: None,
            model: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            step_id: None,
            otid: None,
            tool_returns: None,
            group_id: None,
            sender_id: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        };

        // 保存到数据库
        let created_message = self.message_repo.create(&message).await?;

        debug!("Created assistant message: {}", created_message.id);
        Ok(created_message.id)
    }

    /// 检索相关记忆
    /// ⭐ Phase 4: 自适应调整记忆数量
    /// 根据历史性能动态调整
    async fn adaptive_adjust_memories(
        &self,
        _request: &ChatRequest,
        elapsed: std::time::Duration,
    ) -> usize {
        let base_max = self.config.max_memories;
        let elapsed_ms = elapsed.as_millis() as u64;

        // 如果已经超过阈值，减少记忆数量
        if elapsed_ms > self.config.ttfb_threshold_ms {
            let reduced = base_max.saturating_sub(1).max(1);
            warn!(
                "⚠️  Adaptive: High latency {}ms > {}ms, reducing memories {} → {}",
                elapsed_ms, self.config.ttfb_threshold_ms, base_max, reduced
            );
            reduced
        } else if elapsed_ms < 1000 && base_max < 5 {
            // 如果性能很好，适度增加
            let increased = (base_max + 1).min(5);
            info!(
                "✅ Adaptive: Low latency {}ms, increasing memories {} → {}",
                elapsed_ms, base_max, increased
            );
            increased
        } else {
            base_max
        }
    }

    async fn retrieve_memories(&self, request: &ChatRequest) -> Result<Vec<Memory>> {
        // 🆕 Phase 1: 使用 Episodic-first检索（基于认知理论）
        // 理论依据: Atkinson-Shiffrin模型 + HCAM分层检索
        let max_count = request.max_memories;

        // 使用新的 retrieve_episodic_first 方法
        // Priority 1: Episodic Memory (Agent/User) - 主要来源
        // Priority 2: Working Memory (Session) - 补充上下文
        // Priority 3: Semantic Memory (Agent global) - 备选
        let memories = self
            .memory_integrator
            .retrieve_episodic_first(
                &request.message,
                &request.agent_id,
                Some(&request.user_id),
                Some(&request.session_id),
                max_count,
            )
            .await?;

        info!(
            "📋 Retrieved {} memories (Episodic-first) for user={}, agent={}",
            memories.len(),
            request.user_id,
            request.agent_id
        );

        // 🆕 认知架构验证: 日志已在 retrieve_episodic_first 中记录
        debug!("Memory sources: Episodic (主要) + Working (补充) + Semantic (备选)");

        // Phase 2/3: 过滤和排序
        let memories = self.memory_integrator.filter_by_relevance(memories);
        let memories = self.memory_integrator.sort_memories(memories);

        // Phase 5: 去重和压缩
        let memories = self.memory_integrator.deduplicate_memories(memories);
        let memories = self.memory_integrator.compress_memories(memories);

        Ok(memories)
    }

    /// ⭐ Phase 3: HCAM分层Prompt构建（极简风格）
    ///
    /// 优化目标：从4606字符降至<500字符（-89%）
    /// 理论依据：HCAM模型 - 简洁优先原则
    async fn build_messages_with_context(
        &self,
        request: &ChatRequest,
        working_context: &str,
        memories: &[Memory],
    ) -> Result<Vec<Message>> {
        use crate::prompt::MemorySummarizer;

        let mut messages = Vec::new();

        // ✅ Task 1.1: 使用智能摘要压缩记忆内容
        // 创建摘要器：每条记忆最大200字符
        let summarizer = MemorySummarizer::new(200);

        // ✅ 限制记忆数量为3条（减少90% Prompt大小）
        let limited_memories = memories.iter().take(3);

        let mut memory_text = String::new();
        for (i, mem) in limited_memories.enumerate() {
            let content = match &mem.content {
                agent_mem_traits::Content::Text(t) => t.as_str(),
                _ => "[data]",
            };

            // ✅ 智能摘要化每条记忆（保留头尾信息）
            let summary = summarizer.summarize(content);

            // ✅ 极简格式：移除类型标签，节省空间
            memory_text.push_str(&format!("{}. {}\n", i + 1, summary));
        }

        // ✅ 极简Prompt模板
        let system_message = if memory_text.is_empty() {
            // 无记忆时：仅30字符
            "You are a helpful assistant.".to_string()
        } else {
            // 有记忆时：约600-800字符
            format!("Context:\n{memory_text}\n\nUse context when relevant.")
        };

        // 构建消息列表
        messages.push(Message::system(&system_message));
        messages.push(Message::user(&request.message));

        // 记录Prompt大小（用于监控）
        let total_chars = system_message.len() + request.message.len();
        debug!(
            "📏 Prompt size: {} chars (system: {}, user: {}), memories: {}/{}",
            total_chars,
            system_message.len(),
            request.message.len(),
            memories.iter().take(3).count(),
            memories.len()
        );

        Ok(messages)
    }

    /// 构建包含记忆的消息列表（保留旧版本以兼容）
    async fn build_messages_with_memories(
        &self,
        request: &ChatRequest,
        memories: &[Memory],
    ) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        // 添加系统消息（包含记忆）
        if !memories.is_empty() {
            let memory_context = self.memory_integrator.inject_memories_to_prompt(memories);
            messages.push(Message::system(&memory_context));
        }

        // 添加用户消息
        messages.push(Message::user(&request.message));

        Ok(messages)
    }

    /// 执行带工具调用的 LLM 对话
    ///
    /// 参考 MIRIX 的实现，支持多轮工具调用
    async fn execute_with_tools(
        &self,
        messages: &[Message],
        user_id: &str,
    ) -> Result<(String, Vec<ToolCallInfo>)> {
        let mut current_messages = messages.to_vec();
        let mut all_tool_calls = Vec::new();
        let mut round = 0;
        let max_rounds = 5; // 最大工具调用轮数

        loop {
            round += 1;
            if round > max_rounds {
                warn!("Reached maximum tool call rounds ({})", max_rounds);
                break;
            }

            debug!("Tool call round {}/{}", round, max_rounds);

            // 获取可用工具
            let available_tools = self.get_available_tools().await;

            // 调用 LLM（支持工具调用）
            let llm_response = self
                .llm_client
                .generate_with_functions(&current_messages, &available_tools)
                .await?;

            // 检查是否有工具调用
            if llm_response.function_calls.is_empty() {
                // 没有工具调用，返回文本响应
                let text = llm_response.text.unwrap_or_default();
                info!(
                    "LLM response without tool calls, {} total tool calls made",
                    all_tool_calls.len()
                );
                return Ok((text, all_tool_calls));
            }

            // 执行工具调用
            info!(
                "Executing {} tool call(s) in round {}",
                llm_response.function_calls.len(),
                round
            );
            let tool_results = self
                .tool_integrator
                .execute_tool_calls(&llm_response.function_calls, user_id)
                .await?;

            // 记录工具调用信息
            for result in &tool_results {
                all_tool_calls.push(ToolCallInfo {
                    tool_name: result.tool_name.clone(),
                    arguments: serde_json::from_str(&result.arguments)
                        .unwrap_or(serde_json::json!({})),
                    result: if result.success {
                        Some(result.result.clone())
                    } else {
                        result.error.clone()
                    },
                });
            }

            // 将工具结果添加到消息历史
            if let Some(assistant_text) = llm_response.text {
                current_messages.push(Message::assistant(&assistant_text));
            }

            // 添加工具结果消息
            for result in &tool_results {
                let tool_message = if result.success {
                    format!("Tool '{}' result: {}", result.tool_name, result.result)
                } else {
                    format!(
                        "Tool '{}' error: {}",
                        result.tool_name,
                        result
                            .error
                            .as_ref()
                            .unwrap_or(&"Unknown error".to_string())
                    )
                };
                current_messages.push(Message::system(&tool_message));
            }

            // 继续下一轮（让 LLM 处理工具结果）
        }

        // 如果达到最大轮数，返回最后的消息
        let final_text = "Maximum tool call rounds reached. Please try again.".to_string();
        Ok((final_text, all_tool_calls))
    }

    /// 获取可用的工具定义
    async fn get_available_tools(&self) -> Vec<FunctionDefinition> {
        // 从 ToolIntegrator 获取工具定义
        match self.tool_integrator.get_tool_definitions().await {
            Ok(tools) => tools,
            Err(e) => {
                warn!("Failed to get tool definitions: {}", e);
                Vec::new()
            }
        }
    }

    /// 将记忆提取任务丢到后台执行
    fn schedule_memory_extraction(&self, request: &ChatRequest, messages: &[Message]) {
        let extractor = self.memory_extractor.clone();
        let background = self.background_tasks.clone();
        let request_clone = request.clone();
        let messages_clone: Vec<Message> = messages.to_vec();

        let task_id = background.spawn_memory_extraction(extractor, request_clone, messages_clone);

        info!(
            "📤 Memory extraction scheduled as task {} for session {}",
            task_id, request.session_id
        );
    }

    /// 提取和更新记忆
    async fn extract_and_update_memories(
        &self,
        request: &ChatRequest,
        messages: &[Message],
    ) -> Result<usize> {
        // 使用 MemoryExtractor 提取记忆
        let memories = self
            .memory_extractor
            .extract_from_conversation(messages, &request.agent_id, &request.user_id)
            .await?;

        // 保存记忆
        let count = self.memory_extractor.save_memories(memories).await?;

        Ok(count)
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_creation() {
        let request = ChatRequest {
            message: "Hello, how are you?".to_string(),
            agent_id: "agent-123".to_string(),
            user_id: "user-456".to_string(),
            session_id: "session-abc".to_string(),
            organization_id: "org-789".to_string(),
            stream: false,
            max_memories: 10,
        };

        assert_eq!(request.message, "Hello, how are you?");
        assert_eq!(request.agent_id, "agent-123");
        assert_eq!(request.user_id, "user-456");
        assert_eq!(request.session_id, "session-abc");
        assert_eq!(request.organization_id, "org-789");
        assert!(!request.stream);
        assert_eq!(request.max_memories, 10);
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            message: "Test message".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            session_id: "session-1".to_string(),
            organization_id: "default".to_string(),
            stream: true,
            max_memories: 5,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.message, deserialized.message);
        assert_eq!(request.stream, deserialized.stream);
        assert_eq!(request.session_id, deserialized.session_id);
    }

    #[test]
    fn test_chat_response_creation() {
        let response = ChatResponse {
            message_id: "msg-123".to_string(),
            content: "I'm doing well, thank you!".to_string(),
            memories_updated: true,
            memories_count: 3,
            tool_calls: None,
        };

        assert_eq!(response.message_id, "msg-123");
        assert!(response.memories_updated);
        assert_eq!(response.memories_count, 3);
        assert!(response.tool_calls.is_none());
    }

    #[test]
    fn test_chat_response_with_tool_calls() {
        let tool_call = ToolCallInfo {
            tool_name: "search".to_string(),
            arguments: serde_json::json!({"query": "weather"}),
            result: Some("Sunny, 25°C".to_string()),
        };

        let response = ChatResponse {
            message_id: "msg-456".to_string(),
            content: "The weather is sunny".to_string(),
            memories_updated: false,
            memories_count: 0,
            tool_calls: Some(vec![tool_call]),
        };

        assert!(response.tool_calls.is_some());
        assert_eq!(response.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(response.tool_calls.as_ref().unwrap()[0].tool_name, "search");
    }

    #[test]
    fn test_tool_call_info_creation() {
        let tool_call = ToolCallInfo {
            tool_name: "calculator".to_string(),
            arguments: serde_json::json!({"operation": "add", "a": 5, "b": 3}),
            result: Some("8".to_string()),
        };

        assert_eq!(tool_call.tool_name, "calculator");
        assert!(tool_call.result.is_some());
        assert_eq!(tool_call.arguments["operation"], "add");
    }

    #[test]
    fn test_orchestrator_config_default() {
        let config = OrchestratorConfig::default();

        assert_eq!(config.max_tool_rounds, 5);
        assert_eq!(config.max_memories, 3); // Phase 2/3优化: 从10降到3
        assert!(config.auto_extract_memories);
        assert_eq!(config.memory_extraction_threshold, 0.5);
        assert!(!config.enable_tool_calling);
        assert!(config.enable_adaptive); // Phase 4: 自适应默认启用
    }

    #[test]
    fn test_orchestrator_config_custom() {
        let config = OrchestratorConfig {
            max_tool_rounds: 3,
            max_memories: 20,
            auto_extract_memories: false,
            memory_extraction_threshold: 0.7,
            enable_tool_calling: true,
            enable_adaptive: false,
            token_budget: 8000,
            ttfb_threshold_ms: 500,
        };

        assert_eq!(config.max_tool_rounds, 3);
        assert_eq!(config.max_memories, 20);
        assert!(!config.auto_extract_memories);
        assert_eq!(config.memory_extraction_threshold, 0.7);
        assert!(config.enable_tool_calling);
    }

    #[test]
    fn test_orchestrator_config_serialization() {
        let config = OrchestratorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OrchestratorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.max_tool_rounds, deserialized.max_tool_rounds);
        assert_eq!(config.max_memories, deserialized.max_memories);
    }

    #[test]
    fn test_chat_request_with_empty_message() {
        let request = ChatRequest {
            message: "".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            session_id: "session-empty".to_string(),
            organization_id: "default".to_string(),
            stream: false,
            max_memories: 5,
        };

        assert!(request.message.is_empty());
    }

    #[test]
    fn test_chat_request_with_long_message() {
        let long_message = "A".repeat(10000);
        let request = ChatRequest {
            message: long_message.clone(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            session_id: "session-long".to_string(),
            organization_id: "default".to_string(),
            stream: false,
            max_memories: 5,
        };

        assert_eq!(request.message.len(), 10000);
    }

    #[test]
    fn test_chat_response_serialization() {
        let response = ChatResponse {
            message_id: "msg-1".to_string(),
            content: "Response content".to_string(),
            memories_updated: true,
            memories_count: 2,
            tool_calls: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ChatResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.message_id, deserialized.message_id);
        assert_eq!(response.memories_updated, deserialized.memories_updated);
    }

    #[test]
    fn test_tool_call_info_serialization() {
        let tool_call = ToolCallInfo {
            tool_name: "test_tool".to_string(),
            arguments: serde_json::json!({"param": "value"}),
            result: Some("success".to_string()),
        };

        let json = serde_json::to_string(&tool_call).unwrap();
        let deserialized: ToolCallInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(tool_call.tool_name, deserialized.tool_name);
        assert_eq!(tool_call.result, deserialized.result);
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        // TODO: 添加完整的集成测试
        // 需要 mock LLMClient, MemoryEngine, MessageRepository, ToolExecutor
    }
}
