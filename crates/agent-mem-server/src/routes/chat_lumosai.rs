//! LumosAI Agent Chat API
//! 
//! 使用LumosAI Agent替代AgentOrchestrator

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use agent_mem_core::storage::factory::Repositories;
use axum::{
    extract::{Extension, Path},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::Stream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[cfg(feature = "lumosai")]
use agent_mem_lumosai::agent_factory::LumosAgentFactory;
#[cfg(feature = "lumosai")]
use crate::routes::memory::MemoryManager;

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
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
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
    // ✅ 使用memory_manager中的memory API（统一接口）
    let factory = LumosAgentFactory::new(memory_manager.memory.clone());
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
    
    // 6. LumosAI会自动处理memory，这里不需要手动操作
    // generate()方法内部会自动调用memory.retrieve()和memory.store()
    let context_messages = vec![];
    let memories_count = 0;  // LumosAI自动管理，这里设为0
    
    // 7. 构建完整消息列表（只有当前消息，历史由LumosAI自动加载）
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
    
    // 9. Memory存储由LumosAI的generate()方法自动完成
    // 不需要手动调用store()
    
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    info!("✅ Chat response generated in {}ms", processing_time_ms);
    
    // 10. 返回响应
    Ok(Json(ApiResponse::success(ChatMessageResponse {
        message_id: Uuid::new_v4().to_string(),
        content: response.response,
        memories_updated: true,  // 对话已保存到Memory
        memories_count,  // 使用的历史记忆数量
        processing_time_ms,
    })))
}

/// Send chat message using LumosAI Agent with streaming (SSE)
#[cfg(feature = "lumosai")]
pub async fn send_chat_message_lumosai_stream(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<ChatMessageRequest>,
) -> ServerResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    use lumosai_core::agent::streaming::{AgentEvent, StreamingAgentExt};
    use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole};
    use lumosai_core::agent::types::AgentGenerateOptions;
    use lumosai_core::agent::Agent;
    
    info!("💬 Chat request (LumosAI Streaming): agent={}, message_len={}", agent_id, req.message.len());
    
    // 1. 验证Agent
    let agent = repositories.agents
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;
    
    // 2. 权限检查
    if agent.organization_id != auth_user.org_id {
        error!("Access denied: agent org {} != user org {}", agent.organization_id, auth_user.org_id);
        return Err(ServerError::forbidden("Access denied"));
    }
    
    // 3. 获取user_id
    let user_id = req.user_id.as_ref().unwrap_or(&auth_user.user_id).clone();
    debug!("Using user_id: {}", user_id);
    
    // 4. 创建LumosAI Agent
    let factory = LumosAgentFactory::new(memory_manager.memory.clone());
    let lumos_agent = factory.create_chat_agent(&agent, &user_id)
        .await
        .map_err(|e| {
            error!("Failed to create LumosAI agent: {}", e);
            ServerError::internal_error(format!("Failed to create agent: {}", e))
        })?;
    
    info!("✅ Created LumosAI agent with streaming support");
    
    // 5. 构建用户消息
    let user_message = LumosMessage {
        role: LumosRole::User,
        content: req.message.clone(),
        metadata: None,
        name: None,
    };
    
    let messages = vec![user_message];
    let options = AgentGenerateOptions::default();
    
    // 6. 创建事件流
    let event_stream = lumos_agent.generate_stream_events(&messages, &options);
    
    // 7. 转换为 SSE 格式
    let sse_stream = event_stream.map(|event_result| {
        match event_result {
            Ok(event) => {
                let sse_data = match event {
                    AgentEvent::AgentStarted { .. } => {
                        serde_json::json!({
                            "chunk_type": "start",
                            "message": "Agent started"
                        })
                    },
                    AgentEvent::TextDelta { delta, .. } => {
                        serde_json::json!({
                            "chunk_type": "content",
                            "content": delta
                        })
                    },
                    AgentEvent::GenerationComplete { final_response, total_steps } => {
                        serde_json::json!({
                            "chunk_type": "done",
                            "final_response": final_response,
                            "total_steps": total_steps,
                            "memories_updated": true,
                            "memories_count": 0
                        })
                    },
                    AgentEvent::ToolCalled { tool_name, arguments, .. } => {
                        serde_json::json!({
                            "chunk_type": "tool_call",
                            "tool_name": tool_name,
                            "arguments": arguments
                        })
                    },
                    AgentEvent::Error { error, .. } => {
                        serde_json::json!({
                            "chunk_type": "error",
                            "content": error
                        })
                    },
                    _ => {
                        serde_json::json!({
                            "chunk_type": "metadata",
                            "data": format!("{:?}", event)
                        })
                    }
                };
                
                Event::default()
                    .json_data(sse_data)
                    .map_err(|e| axum::Error::new(e))
            },
            Err(e) => {
                let error_data = serde_json::json!({
                    "chunk_type": "error",
                    "content": format!("Stream error: {}", e)
                });
                
                Event::default()
                    .json_data(error_data)
                    .map_err(|e| axum::Error::new(e))
            }
        }
    });
    
    // 8. 返回 SSE 响应
    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
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

#[cfg(not(feature = "lumosai"))]
pub async fn send_chat_message_lumosai_stream(
    _repositories: Extension<Arc<Repositories>>,
    _auth_user: Extension<AuthUser>,
    _agent_id: Path<String>,
    _req: Json<ChatMessageRequest>,
) -> ServerResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    use futures::stream;
    
    // 返回一个包含错误的单元素流
    let error_stream = stream::once(async {
        Event::default()
            .json_data(serde_json::json!({
                "chunk_type": "error",
                "content": "LumosAI integration not enabled. Compile with --features lumosai"
            }))
            .map_err(|e| axum::Error::new(e))
    });
    
    Ok(Sse::new(error_stream).keep_alive(KeepAlive::default()))
}
