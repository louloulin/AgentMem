//! LumosAI Agent Chat API
//!
//! 使用LumosAI Agent替代AgentOrchestrator

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use agent_mem_core::storage::factory::Repositories;
use axum::{
    extract::{Extension, Path},
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;

#[cfg(feature = "lumosai")]
use crate::routes::memory::MemoryManager;
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
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<ChatMessageRequest>,
) -> ServerResult<Json<ApiResponse<ChatMessageResponse>>> {
    let request_start = std::time::Instant::now();
    let request_id = Uuid::new_v4();

    info!("🚀 [REQUEST-{}] Chat request started", request_id);
    info!(
        "   Agent: {}, Message length: {}, User: {}",
        agent_id,
        req.message.len(),
        req.user_id.as_deref().unwrap_or("default")
    );

    // 1. 验证Agent
    let step1_start = std::time::Instant::now();
    let agent = repositories
        .agents
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;
    info!("   ⏱️  [STEP1] Agent query: {:?}", step1_start.elapsed());

    debug!(
        "   Found agent: {}",
        agent.name.as_ref().map(|s| s.as_str()).unwrap_or("unnamed")
    );

    // 2. 权限检查
    let step2_start = std::time::Instant::now();
    if agent.organization_id != auth_user.org_id {
        error!(
            "   ❌ Access denied: agent org {} != user org {}",
            agent.organization_id, auth_user.org_id
        );
        return Err(ServerError::forbidden("Access denied"));
    }
    info!(
        "   ⏱️  [STEP2] Permission check: {:?}",
        step2_start.elapsed()
    );

    // 3. 获取user_id
    let user_id = req.user_id.as_ref().unwrap_or(&auth_user.user_id);
    debug!("   Using user_id: {}", user_id);

    // 4. 创建LumosAI Agent (使用AgentMem作为记忆后端)
    let step3_start = std::time::Instant::now();
    let factory = LumosAgentFactory::new(memory_manager.memory.clone());
    let lumos_agent = factory
        .create_chat_agent(&agent, user_id)
        .await
        .map_err(|e| {
            error!("   ❌ Failed to create LumosAI agent: {}", e);
            ServerError::internal_error(format!("Failed to create agent: {}", e))
        })?;
    info!("   ⏱️  [STEP3] Agent creation: {:?}", step3_start.elapsed());

    info!("   ✅ LumosAI agent created with memory backend");

    // 5. 使用LumosAI的Memory集成API
    use lumosai_core::agent::types::AgentGenerateOptions;
    use lumosai_core::agent::Agent;
    use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole}; // 导入Agent trait

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
    let memories_count = 0; // LumosAI自动管理，这里设为0

    // 7. 构建完整消息列表（只有当前消息，历史由LumosAI自动加载）
    let mut all_messages = context_messages;
    all_messages.push(user_message.clone());

    // 8. 调用generate生成响应
    let step4_start = std::time::Instant::now();
    info!(
        "   📤 Calling Agent.generate() with {} messages",
        all_messages.len()
    );

    let response = lumos_agent
        .generate(&all_messages, &AgentGenerateOptions::default())
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

    info!(
        "✅ [REQUEST-{}] Completed in {:?}",
        request_id, total_duration
    );
    info!(
        "   Response length: {}, Steps: {}",
        response.response.len(),
        response.steps.len()
    );

    if total_duration.as_secs() > 60 {
        warn!("   ⚠️  Total time > 60s! Performance issue detected");
    }
    info!("✅ Chat response generated in {}ms", processing_time_ms);

    // 10. 返回响应
    Ok(Json(ApiResponse::success(ChatMessageResponse {
        message_id: Uuid::new_v4().to_string(),
        content: response.response,
        memories_updated: true, // 对话已保存到Memory
        memories_count,         // 使用的历史记忆数量
        processing_time_ms,
    })))
}

/// Helper function to create streaming events from a complete response
#[cfg(feature = "lumosai")]
fn create_streaming_events(
    response_text: String,
    total_steps: usize,
) -> Vec<Result<lumosai_core::agent::streaming::AgentEvent, Box<dyn std::error::Error + Send + Sync>>>
{
    use chrono::Utc;
    use lumosai_core::agent::streaming::AgentEvent;

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

/// Send chat message using LumosAI Agent with streaming (SSE) - REAL STREAMING
#[cfg(feature = "lumosai")]
pub async fn send_chat_message_lumosai_stream(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<ChatMessageRequest>,
) -> ServerResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    use futures::StreamExt;
    use lumosai_core::agent::streaming::{AgentEvent, StreamingAgent, StreamingConfig};
    use lumosai_core::agent::types::AgentGenerateOptions;
    use lumosai_core::llm::{Message as LumosMessage, Role as LumosRole};
    use tokio_stream::wrappers::ReceiverStream;

    let start_time = std::time::Instant::now();
    info!(
        "🚀 [REAL-STREAMING] Chat request: agent={}, message_len={}",
        agent_id,
        req.message.len()
    );
    info!("⏱️  [+0ms] Request received");

    // 1. 验证Agent
    let agent = repositories
        .agents
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    info!(
        "⏱️  [+{}ms] Agent verified",
        start_time.elapsed().as_millis()
    );

    // 2. 权限检查
    if agent.organization_id != auth_user.org_id {
        error!(
            "Access denied: agent org {} != user org {}",
            agent.organization_id, auth_user.org_id
        );
        return Err(ServerError::forbidden("Access denied"));
    }

    info!(
        "⏱️  [+{}ms] Permission checked",
        start_time.elapsed().as_millis()
    );

    // 3. 获取user_id
    let user_id = req.user_id.as_ref().unwrap_or(&auth_user.user_id).clone();
    debug!("Using user_id: {}", user_id);

    // 4. 创建LumosAI Agent
    info!(
        "⏱️  [+{}ms] Starting Agent Factory",
        start_time.elapsed().as_millis()
    );
    let factory = LumosAgentFactory::new(memory_manager.memory.clone());
    let lumos_agent = factory
        .create_chat_agent(&agent, &user_id)
        .await
        .map_err(|e| {
            error!("Failed to create LumosAI agent: {}", e);
            ServerError::internal_error(format!("Failed to create agent: {}", e))
        })?;

    info!(
        "⏱️  [+{}ms] BasicAgent created",
        start_time.elapsed().as_millis()
    );
    info!("✅ Created BasicAgent, converting to StreamingAgent...");

    // 5. ⭐ 转换为StreamingAgent以支持真实token-by-token streaming
    let streaming_config = StreamingConfig {
        text_buffer_size: 1,  // 每1个字符发送一次，最快响应
        emit_metadata: false, // 禁用metadata减少开销
        emit_memory_updates: false,
        text_delta_delay_ms: None, // 无延迟，实时发送
    };

    let streaming_agent = StreamingAgent::with_config(lumos_agent, streaming_config);
    info!(
        "⏱️  [+{}ms] StreamingAgent created",
        start_time.elapsed().as_millis()
    );
    info!("✅ StreamingAgent created with real-time token streaming");

    // 6. 构建用户消息
    let user_message = LumosMessage {
        role: LumosRole::User,
        content: req.message.clone(),
        metadata: None,
        name: None,
    };

    let messages = vec![user_message];
    let options = AgentGenerateOptions::default();

    // 7. ⭐ 使用channel解决生命周期问题
    info!(
        "⏱️  [+{}ms] Creating streaming channel",
        start_time.elapsed().as_millis()
    );
    info!("📤 Setting up REAL TOKEN STREAMING with channel");

    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // 在独立任务中执行streaming，避免生命周期问题
    tokio::spawn(async move {
        let mut event_stream = streaming_agent.execute_streaming(&messages, &options);
        while let Some(event_result) = event_stream.next().await {
            if tx.send(event_result).await.is_err() {
                break; // 接收端已关闭
            }
        }
    });

    // 8. 转换为 SSE 格式
    let sse_stream = ReceiverStream::new(rx).map(move |event_result| {
        match event_result {
            Ok(event) => {
                let sse_data = match event {
                    AgentEvent::AgentStarted {
                        agent_id: aid,
                        timestamp,
                    } => {
                        info!("🎬 [SSE] Agent started: {}", aid);
                        serde_json::json!({
                            "chunk_type": "start",
                            "agent_id": aid,
                            "timestamp": timestamp
                        })
                    }
                    AgentEvent::TextDelta { delta, step_id } => {
                        // ⭐ 真实的token增量，实时发送
                        serde_json::json!({
                            "chunk_type": "content",
                            "content": delta,
                            "step_id": step_id
                        })
                    }
                    AgentEvent::ToolCallStart { tool_call, step_id } => {
                        info!("🔧 [SSE] Tool call start: {}", tool_call.name);
                        serde_json::json!({
                            "chunk_type": "tool_call_start",
                            "tool_name": tool_call.name,
                            "step_id": step_id
                        })
                    }
                    AgentEvent::ToolCallComplete {
                        tool_result,
                        step_id,
                    } => {
                        info!("✅ [SSE] Tool call complete: {:?}", tool_result.status);
                        serde_json::json!({
                            "chunk_type": "tool_call_complete",
                            "status": format!("{:?}", tool_result.status),
                            "step_id": step_id
                        })
                    }
                    AgentEvent::GenerationComplete {
                        final_response,
                        total_steps,
                    } => {
                        let elapsed = start_time.elapsed();
                        info!(
                            "🏁 [SSE] Generation complete in {:?}, steps: {}",
                            elapsed, total_steps
                        );
                        serde_json::json!({
                            "chunk_type": "done",
                            "response": final_response,
                            "total_steps": total_steps,
                            "elapsed_ms": elapsed.as_millis() as u64
                        })
                    }
                    AgentEvent::MessageSent { message, timestamp } => {
                        serde_json::json!({
                            "chunk_type": "message",
                            "message": message,
                            "timestamp": timestamp
                        })
                    }
                    AgentEvent::AgentStopped {
                        agent_id: aid,
                        timestamp,
                    } => {
                        info!("🛑 [SSE] Agent stopped: {}", aid);
                        serde_json::json!({
                            "chunk_type": "stop",
                            "agent_id": aid,
                            "timestamp": timestamp
                        })
                    }
                    AgentEvent::MemoryUpdate { key, operation } => {
                        serde_json::json!({
                            "chunk_type": "memory_update",
                            "key": key,
                            "operation": format!("{:?}", operation)
                        })
                    }
                    AgentEvent::Error { error, step_id } => {
                        error!("❌ [SSE] Error: {}", error);
                        serde_json::json!({
                            "chunk_type": "error",
                            "content": error,
                            "step_id": step_id
                        })
                    }
                    AgentEvent::Metadata { key, value } => {
                        serde_json::json!({
                            "chunk_type": "metadata",
                            "key": key,
                            "value": value
                        })
                    }
                    AgentEvent::StepComplete { step, step_id } => {
                        info!("✨ [SSE] Step complete: {}", step_id);
                        serde_json::json!({
                            "chunk_type": "step_complete",
                            "step_type": format!("{:?}", step.step_type),
                            "step_id": step_id
                        })
                    }
                    _ => {
                        serde_json::json!({
                            "chunk_type": "unknown",
                            "data": "event"
                        })
                    }
                };

                Event::default()
                    .json_data(sse_data)
                    .map_err(|e| axum::Error::new(e))
            }
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
        "LumosAI integration not enabled. Compile with --features lumosai",
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
    use axum::response::sse::{Event, KeepAlive};
    use futures::stream;

    let error_stream = stream::once(async {
        Event::default()
            .json_data(serde_json::json!({
                "chunk_type": "error",
                "content": "LumosAI integration not enabled. Compile with --features lumosai"
            }))
            .map_err(axum::Error::new)
    });

    Ok(Sse::new(error_stream).keep_alive(KeepAlive::default()))
}
