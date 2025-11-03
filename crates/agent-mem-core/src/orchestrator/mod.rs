//! Agent Orchestrator - 对话循环编排
//!
//! 这是 AgentMem 的核心对话循环实现，参考 MIRIX 的 AgentWrapper.step() 设计
//! 集成所有现有模块：MemoryEngine, LLMClient, ToolExecutor, MessageRepository

use crate::{
    engine::MemoryEngine,
    Memory,
    storage::traits::MessageRepositoryTrait,
};

use agent_mem_llm::LLMClient;
use agent_mem_tools::ToolExecutor;
use agent_mem_traits::{
    llm::FunctionDefinition,
    AgentMemError, Message, Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub mod memory_integration;
pub mod tool_integration;
pub mod memory_extraction;

use memory_integration::MemoryIntegrator;
use memory_extraction::MemoryExtractor;
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
            return Err(AgentMemError::ValidationError(
                format!("Message too long: {} bytes (max 100KB)", self.message.len()),
            ));
        }

        // 验证 agent_id 不为空
        if self.agent_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "Agent ID cannot be empty".to_string(),
            ));
        }

        // 验证 agent_id 长度（最大 255 字符）
        if self.agent_id.len() > 255 {
            return Err(AgentMemError::ValidationError(
                format!("Agent ID too long: {} characters (max 255)", self.agent_id.len()),
            ));
        }

        // 验证 user_id 不为空
        if self.user_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "User ID cannot be empty".to_string(),
            ));
        }

        // 验证 user_id 长度（最大 255 字符）
        if self.user_id.len() > 255 {
            return Err(AgentMemError::ValidationError(
                format!("User ID too long: {} characters (max 255)", self.user_id.len()),
            ));
        }

        // 验证 organization_id 不为空
        if self.organization_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "Organization ID cannot be empty".to_string(),
            ));
        }

        // 验证 organization_id 长度（最大 255 字符）
        if self.organization_id.len() > 255 {
            return Err(AgentMemError::ValidationError(
                format!("Organization ID too long: {} characters (max 255)", self.organization_id.len()),
            ));
        }

        // 验证 max_memories 范围（1-1000）
        if self.max_memories == 0 {
            return Err(AgentMemError::ValidationError(
                "max_memories must be at least 1".to_string(),
            ));
        }

        if self.max_memories > 1000 {
            return Err(AgentMemError::ValidationError(
                format!("max_memories too large: {} (max 1000)", self.max_memories),
            ));
        }

        // 验证 session_id 不为空
        if self.session_id.trim().is_empty() {
            return Err(AgentMemError::ValidationError(
                "Session ID cannot be empty".to_string(),
            ));
        }

        // 验证 session_id 长度（最大 255 字符）
        if self.session_id.len() > 255 {
            return Err(AgentMemError::ValidationError(
                format!("Session ID too long: {} characters (max 255)", self.session_id.len()),
            ));
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
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_tool_rounds: 5,
            max_memories: 10,
            auto_extract_memories: true,
            memory_extraction_threshold: 0.5,
            enable_tool_calling: false, // 默认关闭，需要显式启用
        }
    }
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
    memory_extractor: MemoryExtractor,
    tool_integrator: ToolIntegrator,
    /// Working Memory Store - 用于会话级临时上下文（最小改动方案：直接使用Store而非Agent）
    working_store: Option<Arc<dyn agent_mem_traits::WorkingMemoryStore>>,
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
        let memory_extractor = MemoryExtractor::with_default_config(
            llm_client.clone(),
            memory_engine.clone(),
        );

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
                            format!(
                                "[{}] {}",
                                item.created_at.format("%H:%M:%S"),
                                item.content
                            )
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
                    warn!("Failed to get working context for session {}: {}", session_id, e);
                    Ok(String::new()) // 失败时返回空，不影响对话
                }
            }
        } else {
            debug!("Working Memory store not configured, session_id: {}", session_id);
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
            let conversation_pair = format!(
                "User: {}\nAssistant: {}",
                user_message, assistant_response
            );
            
            // 创建工作记忆项
            let item = WorkingMemoryItem {
                id: Uuid::new_v4().to_string(),
                user_id: user_id.to_string(),
                agent_id: agent_id.to_string(),
                session_id: session_id.to_string(),
                content: conversation_pair,
                priority: 1, // 默认优先级
                expires_at: Some(Utc::now() + chrono::Duration::hours(24)), // 24小时后过期
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
            };
            
            match store.add_item(item).await {
                Ok(_) => {
                    debug!("Successfully added working memory item for session: {}", session_id);
                }
                Err(e) => {
                    warn!("Failed to add working memory item for session {}: {}", session_id, e);
                    // 不返回错误，避免影响对话流程
                }
            }
        } else {
            debug!("Working Memory store not configured, skipping update for session: {}", session_id);
        }
        
        Ok(())
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
        // ✅ 验证请求参数
        request.validate()?;

        info!("Starting conversation step for agent_id={}, user_id={}, session_id={}",
              request.agent_id, request.user_id, request.session_id);

        // 0. 获取Working Memory会话上下文
        let working_context = self.get_working_context(&request.session_id).await?;
        if !working_context.is_empty() {
            debug!("Retrieved working context: {} chars", working_context.len());
        }

        // 1. 创建用户消息
        let user_message_id = self.create_user_message(&request).await?;
        debug!("Created user message: {}", user_message_id);

        // 2. 检索相关记忆
        let memories = self.retrieve_memories(&request).await?;
        let memories_retrieved_count = memories.len();
        info!("Retrieved {} memories", memories_retrieved_count);

        // 3. 构建 prompt（注入会话上下文和长期记忆）
        let messages = self.build_messages_with_context(&request, &working_context, &memories).await?;
        debug!("Built {} messages with working context and memories", messages.len());

        // 4. 调用 LLM（可能需要多轮工具调用）
        let (final_response, tool_calls_info) = self.execute_with_tools(
            &messages,
            &request.user_id,
        ).await?;
        debug!("Got final response: {} chars, {} tool calls",
            final_response.len(), tool_calls_info.len());

        // 5. 保存 assistant 消息
        let assistant_message_id = self.create_assistant_message(
            &request.organization_id,
            &request.agent_id,
            &request.user_id,
            &final_response,
        ).await?;
        debug!("Created assistant message: {}", assistant_message_id);

        // 6. 更新Working Memory
        self.update_working_memory(
            &request.session_id,
            &request.user_id,
            &request.agent_id,
            &request.message,
            &final_response,
        ).await?;
        debug!("Updated working memory for session {}", request.session_id);

        // 7. 提取和更新记忆
        let memories_extracted = if self.config.auto_extract_memories {
            self.extract_and_update_memories(&request, &messages).await?
        } else {
            0
        };
        info!("Extracted and updated {} new memories", memories_extracted);

        // 8. 返回响应（✅ memories_count 现在表示检索使用的记忆数量）
        Ok(ChatResponse {
            message_id: assistant_message_id,
            content: final_response,
            memories_updated: memories_extracted > 0,
            memories_count: memories_retrieved_count,  // ✅ 使用检索到的记忆数量
            tool_calls: if tool_calls_info.is_empty() {
                None
            } else {
                Some(tool_calls_info)
            },
        })
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
        let mut messages = self.build_messages_with_memories(&request, &memories).await?;
        debug!("Built {} messages with memories", messages.len());

        let mut tool_calls_info = Vec::new();
        let mut final_response = String::new();
        let mut round = 0;

        // 工具调用循环
        loop {
            round += 1;
            if round > self.config.max_tool_rounds {
                warn!("Reached max tool rounds ({}), stopping", self.config.max_tool_rounds);
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
            .create_assistant_message(&request.organization_id, &request.agent_id, &request.user_id, &final_response)
            .await?;
        debug!("Created assistant message: {}", assistant_message_id);

        // 7. 提取和更新记忆
        let memories_count = if self.config.auto_extract_memories {
            self.extract_and_update_memories(&request, &messages).await?
        } else {
            0
        };
        info!("Extracted and updated {} memories", memories_count);

        // 8. 返回响应
        Ok(ChatResponse {
            message_id: assistant_message_id,
            content: final_response,
            memories_updated: memories_count > 0,
            memories_count,
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
        let created_message = self.message_repo
            .create(&message)
            .await?;

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
        let created_message = self.message_repo
            .create(&message)
            .await?;

        debug!("Created assistant message: {}", created_message.id);
        Ok(created_message.id)
    }

    /// 检索相关记忆
    async fn retrieve_memories(&self, request: &ChatRequest) -> Result<Vec<Memory>> {
        // ✅ 使用 MemoryIntegrator 检索记忆（带session隔离）
        let max_count = self.config.max_memories;
        
        // 使用session_id和user_id进行精确过滤
        let memories = self.memory_integrator
            .retrieve_relevant_memories_with_session(
                &request.message, 
                &request.agent_id,
                Some(&request.user_id),
                Some(&request.session_id),
                max_count
            )
            .await?;

        info!("📋 Retrieved {} memories for session={}, user={}", 
              memories.len(), request.session_id, request.user_id);

        // 过滤和排序
        let memories = self.memory_integrator.filter_by_relevance(memories);
        let memories = self.memory_integrator.sort_memories(memories);

        Ok(memories)
    }

    /// 构建包含会话上下文和记忆的消息列表（新版本，集成Working Memory）
    async fn build_messages_with_context(
        &self,
        request: &ChatRequest,
        working_context: &str,
        memories: &[Memory],
    ) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        // 构建系统消息，优先级：Working Context > 长期记忆
        let mut system_message_parts = Vec::new();

        // 1. 添加会话上下文（优先级最高）
        if !working_context.is_empty() {
            system_message_parts.push(format!(
                "## ⚠️ CURRENT SESSION CONTEXT (HIGHEST PRIORITY)\n\n\
                **IMPORTANT**: The following is the CURRENT conversation in THIS session. \
                This information has the HIGHEST priority and should OVERRIDE any conflicting information from past memories.\n\n\
                **Current Session History:**\n{}",
                working_context
            ));
        }

        // 2. 添加长期记忆（仅供参考）
        if !memories.is_empty() {
            let memory_context = self.memory_integrator.inject_memories_to_prompt(memories);
            system_message_parts.push(format!(
                "## 📚 PAST MEMORIES (For Reference Only)\n\n\
                **Note**: The following are memories from PAST conversations. \
                If there is any conflict between these past memories and the current session context above, \
                ALWAYS prioritize the current session information.\n\n\
                {}",
                memory_context
            ));
        }

        // 如果有任何上下文信息，添加系统消息
        if !system_message_parts.is_empty() {
            let system_content = system_message_parts.join("\n\n");
            messages.push(Message::system(&system_content));
        }

        // 添加用户消息
        messages.push(Message::user(&request.message));

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
            let llm_response = self.llm_client
                .generate_with_functions(&current_messages, &available_tools)
                .await?;

            // 检查是否有工具调用
            if llm_response.function_calls.is_empty() {
                // 没有工具调用，返回文本响应
                let text = llm_response.text.unwrap_or_default();
                info!("LLM response without tool calls, {} total tool calls made", all_tool_calls.len());
                return Ok((text, all_tool_calls));
            }

            // 执行工具调用
            info!("Executing {} tool call(s) in round {}", llm_response.function_calls.len(), round);
            let tool_results = self.tool_integrator
                .execute_tool_calls(&llm_response.function_calls, user_id)
                .await?;

            // 记录工具调用信息
            for result in &tool_results {
                all_tool_calls.push(ToolCallInfo {
                    tool_name: result.tool_name.clone(),
                    arguments: serde_json::from_str(&result.arguments).unwrap_or(serde_json::json!({})),
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
                    format!("Tool '{}' error: {}", result.tool_name, result.error.as_ref().unwrap_or(&"Unknown error".to_string()))
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

    /// 提取和更新记忆
    async fn extract_and_update_memories(
        &self,
        request: &ChatRequest,
        messages: &[Message],
    ) -> Result<usize> {
        // 使用 MemoryExtractor 提取记忆
        let memories = self.memory_extractor
            .extract_from_conversation(messages, &request.agent_id, &request.user_id)
            .await?;

        // 保存记忆
        let count = self.memory_extractor.save_memories(memories).await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_creation() {
        let request = ChatRequest {
            message: "Hello, how are you?".to_string(),
            agent_id: "agent-123".to_string(),
            user_id: "user-456".to_string(),
            organization_id: "org-789".to_string(),
            stream: false,
            max_memories: 10,
        };

        assert_eq!(request.message, "Hello, how are you?");
        assert_eq!(request.agent_id, "agent-123");
        assert_eq!(request.user_id, "user-456");
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
            organization_id: "default".to_string(),
            stream: true,
            max_memories: 5,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.message, deserialized.message);
        assert_eq!(request.stream, deserialized.stream);
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
        assert_eq!(config.max_memories, 10);
        assert!(config.auto_extract_memories);
        assert_eq!(config.memory_extraction_threshold, 0.5);
        assert!(!config.enable_tool_calling);
    }

    #[test]
    fn test_orchestrator_config_custom() {
        let config = OrchestratorConfig {
            max_tool_rounds: 3,
            max_memories: 20,
            auto_extract_memories: false,
            memory_extraction_threshold: 0.7,
            enable_tool_calling: true,
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

