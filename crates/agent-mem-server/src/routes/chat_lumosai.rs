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
    
    debug!("Found agent: {}", agent.name);
    
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
    
    info!("✅ Created LumosAI agent: {}", agent.name);
    
    // 5. 构建消息
    use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole};
    let messages = vec![
        LumosMessage {
            role: LumosRole::User,
            content: req.message.clone(),
            metadata: req.metadata.clone(),
            name: None,
        },
    ];
    
    // 6. 调用LumosAI Agent生成响应
    use lumosai_core::agent::AgentGenerateOptions;
    let response = lumos_agent.generate(
        &messages,
        &AgentGenerateOptions::default(),
    ).await
        .map_err(|e| {
            error!("Agent generation failed: {}", e);
            ServerError::internal_error(format!("Agent failed: {}", e))
        })?;
    
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    info!("✅ Chat response generated in {}ms", processing_time_ms);
    
    // 7. 返回响应
    Ok(Json(ApiResponse::success(ChatMessageResponse {
        message_id: Uuid::new_v4().to_string(),
        content: response,
        memories_updated: true,
        memories_count: 1,
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
