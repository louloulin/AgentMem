//! LumosAI Agent Chat API
//! 
//! 使用LumosAI Agent替代AgentOrchestrator

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use agent_mem_core::storage::factory::Repositories;
use axum::{extract::{Extension, Path}, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

#[cfg(feature = "lumosai")]
use agent_mem_lumosai::agent_factory::LumosAgentFactory;

/// Chat message request
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub message: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: Option<JsonValue>,
}

/// Chat message response
#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
    pub message_id: String,
    pub content: String,
    pub memories_updated: bool,
    pub memories_count: usize,
    pub processing_time_ms: u64,
}

/// Send chat message using LumosAI Agent
#[cfg(feature = "lumosai")]
pub async fn send_chat_message_lumosai(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<ChatMessageRequest>,
) -> ServerResult<Json<ApiResponse<ChatMessageResponse>>> {
    let start_time = std::time::Instant::now();
    info!("💬 Chat request (LumosAI): agent={}, message_len={}", agent_id, req.message.len());
    
    // 1. 验证Agent
    let agent = repositories.agents
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;
    
    debug!("Found agent: {}", agent.name.as_ref().map(|s| s.as_str()).unwrap_or("unnamed"));
    
    // 2. 权限检查
    if agent.organization_id != auth_user.org_id {
        error!("Access denied: agent org {} != user org {}", agent.organization_id, auth_user.org_id);
        return Err(ServerError::forbidden("Access denied"));
    }
    
    // 3. 获取user_id
    let user_id = req.user_id.as_ref().unwrap_or(&auth_user.user_id);
    debug!("Using user_id: {}", user_id);
    
    // 4. 创建LumosAI Agent (使用AgentMem作为记忆后端)
    let factory = LumosAgentFactory::new(repositories.clone());
    let lumos_agent = factory.create_chat_agent(&agent, user_id)
        .await
        .map_err(|e| {
            error!("Failed to create LumosAI agent: {}", e);
            ServerError::internal_error(format!("Failed to create agent: {}", e))
        })?;
    
    info!("✅ Created LumosAI agent with integrated Memory Backend");
    
    // 5. 使用LumosAI的Memory集成API
    use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole};
    use lumosai_core::agent::types::AgentGenerateOptions;
    use lumosai_core::agent::Agent;  // 导入Agent trait
    
    // 构建用户消息
    let user_message = LumosMessage {
        role: LumosRole::User,
        content: req.message.clone(),
        metadata: None,
        name: None,
    };
    
    // 6. 获取Memory Backend并手动调用retrieve
    let mut context_messages = vec![];
    
    if let Some(memory) = lumos_agent.get_memory() {
        debug!("🔍 Retrieving historical memories from AgentMem...");
        
        // 使用MemoryConfig检索历史消息
        use lumosai_core::memory::MemoryConfig;
        let memory_config = MemoryConfig {
            store_id: None,
            namespace: Some(format!("agent_{}", agent.id)),
            enabled: true,
            working_memory: None,
            semantic_recall: None,
            last_messages: Some(10),  // 检索最近10条消息
            query: None,
        };
        
        // 调用memory.retrieve()获取历史
        match memory.retrieve(&memory_config).await {
            Ok(historical_messages) => {
                if !historical_messages.is_empty() {
                    info!("📝 Retrieved {} historical messages from memory", historical_messages.len());
                    context_messages = historical_messages;
                } else {
                    debug!("No historical messages found");
                }
            }
            Err(e) => {
                error!("Failed to retrieve memories: {}", e);
            }
        }
    } else {
        warn!("⚠️  Memory Backend not attached to Agent - get_memory() returned None!");
        // 添加额外的调试信息
        if lumos_agent.has_own_memory() {
            error!("🔴 BUG: Agent.has_own_memory() is true but get_memory() returns None!");
        }
    }
    
    // 7. 构建完整消息列表（历史 + 当前）
    let mut all_messages = context_messages;
    all_messages.push(user_message.clone());
    
    debug!("Calling LumosAI Agent.generate() with {} messages", all_messages.len());
    
    // 8. 调用generate生成响应
    let response = lumos_agent.generate(
        &all_messages,
        &AgentGenerateOptions::default()
    )
        .await
        .map_err(|e| {
            error!("Agent generation failed: {}", e);
            ServerError::internal_error(format!("Agent failed: {}", e))
        })?;
    
    // 9. 保存用户消息和助手响应到Memory
    if let Some(memory) = lumos_agent.get_memory() {
        debug!("💾 Storing conversation to memory...");
        
        // 保存用户消息
        if let Err(e) = memory.store(&user_message).await {
            error!("Failed to store user message: {}", e);
        } else {
            debug!("✅ Stored user message");
        }
        
        // 保存助手响应
        let assistant_message = LumosMessage {
            role: LumosRole::Assistant,
            content: response.response.clone(),
            metadata: None,
            name: None,
        };
        
        if let Err(e) = memory.store(&assistant_message).await {
            error!("Failed to store assistant message: {}", e);
        } else {
            debug!("✅ Stored assistant response");
        }
    }
    
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    info!("✅ Chat response generated in {}ms", processing_time_ms);
    
    // 10. 返回响应
    Ok(Json(ApiResponse::success(ChatMessageResponse {
        message_id: Uuid::new_v4().to_string(),
        content: response.response,
        memories_updated: true,  // 对话已保存到Memory
        memories_count: context_messages.len(),  // 使用的历史记忆数量
        processing_time_ms,
    })))
}

/// Fallback when lumosai feature is not enabled
#[cfg(not(feature = "lumosai"))]
pub async fn send_chat_message_lumosai(
    _repositories: Extension<Arc<Repositories>>,
    _auth_user: Extension<AuthUser>,
    _agent_id: Path<String>,
    _req: Json<ChatMessageRequest>,
) -> ServerResult<Json<ApiResponse<ChatMessageResponse>>> {
    Err(ServerError::internal_error(
        "LumosAI integration not enabled. Compile with --features lumosai"
    ))
}
