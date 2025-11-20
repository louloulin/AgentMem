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
    let request_start = std::time::Instant::now();
    let request_id = Uuid::new_v4();
    
    info!("🚀 [REQUEST-{}] Chat request started", request_id);
    info!("   Agent: {}, Message length: {}, User: {}", 
          agent_id, req.message.len(), req.user_id.as_deref().unwrap_or("default"));
    
    // 1. 验证Agent
    let step1_start = std::time::Instant::now();
    let agent = repositories.agents
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;
    info!("   ⏱️  [STEP1] Agent query: {:?}", step1_start.elapsed());
    
    debug!("   Found agent: {}", agent.name.as_ref().map(|s| s.as_str()).unwrap_or("unnamed"));
    
    // 2. 权限检查
    let step2_start = std::time::Instant::now();
    if agent.organization_id != auth_user.org_id {
        error!("   ❌ Access denied: agent org {} != user org {}", agent.organization_id, auth_user.org_id);
        return Err(ServerError::forbidden("Access denied"));
    }
    info!("   ⏱️  [STEP2] Permission check: {:?}", step2_start.elapsed());
    
    // 3. 获取user_id
    let user_id = req.user_id.as_ref().unwrap_or(&auth_user.user_id);
    debug!("   Using user_id: {}", user_id);
    
    // 4. 创建LumosAI Agent (使用AgentMem作为记忆后端)
    let step3_start = std::time::Instant::now();
    let factory = LumosAgentFactory::new(memory_manager.memory.clone());
    let lumos_agent = factory.create_chat_agent(&agent, user_id)
        .await
        .map_err(|e| {
            error!("   ❌ Failed to create LumosAI agent: {}", e);
            ServerError::internal_error(format!("Failed to create agent: {}", e))
        })?;
    info!("   ⏱️  [STEP3] Agent creation: {:?}", step3_start.elapsed());
    
    info!("   ✅ LumosAI agent created with memory backend");
    
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
    
    // 8. 调用generate生成响应
    let step4_start = std::time::Instant::now();
    info!("   📤 Calling Agent.generate() with {} messages", all_messages.len());
    
    let response = lumos_agent.generate(
        &all_messages,
        &AgentGenerateOptions::default()
    )
        .await
        .map_err(|e| {
            error!("   ❌ Agent generation failed: {}", e);
            ServerError::internal_error(format!("Agent failed: {}", e))
        })?;
    
    let step4_duration = step4_start.elapsed();
    info!("   ⏱️  [STEP4] Agent.generate(): {:?}", step4_duration);
    
    if step4_duration.as_secs() > 30 {
        warn!("   ⚠️  Generation took > 30s! Check performance");
    }
    
    // 9. Memory存储由LumosAI的generate()方法自动完成
    // 不需要手动调用store()
    
    let total_duration = request_start.elapsed();
    let processing_time_ms = total_duration.as_millis() as u64;
    
    info!("✅ [REQUEST-{}] Completed in {:?}", request_id, total_duration);
    info!("   Response length: {}, Steps: {}", 
          response.response.len(), response.steps.len());
    
    if total_duration.as_secs() > 60 {
        warn!("   ⚠️  Total time > 60s! Performance issue detected");
    }
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

/// Helper function to create streaming events from a complete response
#[cfg(feature = "lumosai")]
fn create_streaming_events(
    response_text: String,
    total_steps: usize,
) -> Vec<Result<lumosai_core::agent::streaming::AgentEvent, Box<dyn std::error::Error + Send + Sync>>> {
    use lumosai_core::agent::streaming::AgentEvent;
    use chrono::Utc;
    
    let mut events = Vec::new();
    let agent_id = uuid::Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();
    
    // 1. Agent started event
    events.push(Ok(AgentEvent::AgentStarted {
        agent_id: agent_id.clone(),
        timestamp: timestamp.clone(),
    }));
    
    // 2. Split response into chunks and create TextDelta events
    const CHUNK_SIZE: usize = 10; // 每次发送10个字符
    for chunk in response_text.as_bytes().chunks(CHUNK_SIZE) {
        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
            events.push(Ok(AgentEvent::TextDelta {
                delta: text,
                step_id: Some("0".to_string()),
            }));
        }
    }
    
    // 3. Generation complete event
    events.push(Ok(AgentEvent::GenerationComplete {
        final_response: response_text,
        total_steps,
    }));
    
    events
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
    use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole};
    use lumosai_core::agent::types::AgentGenerateOptions;
    use lumosai_core::agent::streaming::AgentEvent;
    
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
    
    // 6. 调用generate获取完整响应
    let generate_result = lumos_agent.generate(&messages, &options).await
        .map_err(|e| ServerError::internal_error(e.to_string()))?;
    
    // 7. 将完整响应转换为streaming events
    let response_text = generate_result.response;
    let total_steps = generate_result.steps.len();
    
    // 创建模拟的streaming events
    let events = create_streaming_events(response_text, total_steps);
    let event_stream = futures::stream::iter(events);
    
    // 8. 转换为 SSE 格式
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
    
    // 9. 返回 SSE 响应
    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

/// Fallback when lumosai feature is not enabled - non-streaming
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

/// Fallback when lumosai feature is not enabled - streaming
#[cfg(not(feature = "lumosai"))]
pub async fn send_chat_message_lumosai_stream(
    _repositories: Extension<Arc<Repositories>>,
    _auth_user: Extension<AuthUser>,
    _agent_id: Path<String>,
    _req: Json<ChatMessageRequest>,
) -> ServerResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    use futures::stream;
    use axum::response::sse::{Event, KeepAlive};
    
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
