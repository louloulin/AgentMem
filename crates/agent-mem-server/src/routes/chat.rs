//! Chat API routes
//!
//! This module provides REST API endpoints for agent chat functionality.
//! It integrates the AgentOrchestrator to provide complete conversation loops
//! with memory retrieval, LLM calls, and memory extraction.
//!
//! Features:
//! - Complete conversation loop with AgentOrchestrator
//! - Memory retrieval and injection
//! - LLM integration (14+ providers)
//! - Automatic memory extraction
//! - Tool calling support (TODO)
//! - Streaming responses via SSE (TODO)

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use crate::orchestrator_factory::create_orchestrator;
use agent_mem_core::orchestrator::{AgentOrchestrator, ChatRequest as OrchestratorChatRequest};
use agent_mem_core::storage::factory::Repositories;
use axum::{
    extract::{Extension, Path},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request to send a chat message
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatMessageRequest {
    /// Message content
    pub message: String,
    /// User ID (optional, defaults to authenticated user)
    pub user_id: Option<String>,
    /// Session ID (optional, auto-generated if not provided)
    pub session_id: Option<String>,
    /// Whether to stream the response (TODO)
    #[serde(default)]
    pub stream: bool,
    /// Additional metadata
    pub metadata: Option<JsonValue>,
}

/// Chat message response
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatMessageResponse {
    /// Response message ID
    pub message_id: String,
    /// Response content
    pub content: String,
    /// Whether memories were updated
    pub memories_updated: bool,
    /// Number of memories updated
    pub memories_count: usize,
    /// Tool calls made (if any)
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Streaming chunk for SSE
#[derive(Debug, Serialize, ToSchema)]
pub struct StreamChunk {
    /// Chunk type: "content", "tool_call", "memory_update", "done"
    pub chunk_type: String,
    /// Content for this chunk
    pub content: Option<String>,
    /// Tool call information (if chunk_type is "tool_call")
    pub tool_call: Option<ToolCallInfo>,
    /// Memory update count (if chunk_type is "memory_update")
    pub memories_count: Option<usize>,
}

/// Tool call information
#[derive(Debug, Serialize, ToSchema)]
pub struct ToolCallInfo {
    /// Tool name
    pub tool_name: String,
    /// Tool arguments (JSON)
    pub arguments: JsonValue,
    /// Tool result
    pub result: Option<String>,
}

impl From<agent_mem_core::orchestrator::ToolCallInfo> for ToolCallInfo {
    fn from(info: agent_mem_core::orchestrator::ToolCallInfo) -> Self {
        Self {
            tool_name: info.tool_name,
            arguments: info.arguments,
            result: info.result,
        }
    }
}

/// Send a chat message to an agent
///
/// Sends a message to an agent and receives a response.
/// This endpoint uses the AgentOrchestrator to provide a complete conversation loop:
/// 1. Retrieve relevant memories
/// 2. Inject memories into prompt
/// 3. Call LLM
/// 4. Extract and update memories
/// 5. Return response
///
/// # Example
///
/// ```json
/// {
///   "message": "Hello! Tell me about yourself.",
///   "user_id": "user-123",
///   "stream": false
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/agents/{agent_id}/chat",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    request_body = ChatMessageRequest,
    responses(
        (status = 200, description = "Chat response generated successfully", body = ChatMessageResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    ),
    tag = "chat"
)]
pub async fn send_chat_message(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<ChatMessageRequest>,
) -> ServerResult<Json<ApiResponse<ChatMessageResponse>>> {
    let start_time = std::time::Instant::now();

    // Validate agent exists and belongs to user's organization
    let agent_repo = repositories.agents.clone();
    let agent = agent_repo
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Enforce tenant isolation
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this agent"));
    }

    // Validate message content
    if req.message.trim().is_empty() {
        return Err(ServerError::bad_request("Message content cannot be empty"));
    }

    // ✅ 创建 AgentOrchestrator
    info!("Creating AgentOrchestrator for agent: {}", agent_id);
    let orchestrator = create_orchestrator(&agent, &repositories).await?;

    // ✅ 构建 OrchestratorChatRequest
    let user_id = req.user_id.unwrap_or_else(|| auth_user.user_id.clone());

    // ✅ 生成或使用提供的session_id
    let session_id = req
        .session_id
        .unwrap_or_else(|| format!("{}_{}", user_id, Uuid::new_v4()));
    debug!("Using session_id: {}", session_id);

    let orchestrator_request = OrchestratorChatRequest {
        message: req.message.clone(),
        agent_id: agent_id.clone(),
        user_id: user_id.clone(),
        organization_id: auth_user.org_id.clone(),
        session_id,
        stream: req.stream,
        max_memories: 10,
    };

    debug!(
        "Calling orchestrator.step() with request: {:?}",
        orchestrator_request
    );

    // ✅ 调用 orchestrator.step()
    let orchestrator_response = orchestrator
        .step(orchestrator_request)
        .await
        .map_err(|e| ServerError::internal_error(format!("Orchestrator failed: {}", e)))?;

    info!(
        "Orchestrator completed: message_id={}, memories_count={}",
        orchestrator_response.message_id, orchestrator_response.memories_count
    );

    // ✅ 转换响应
    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    let response = ChatMessageResponse {
        message_id: orchestrator_response.message_id,
        content: orchestrator_response.content,
        memories_updated: orchestrator_response.memories_updated,
        memories_count: orchestrator_response.memories_count,
        tool_calls: orchestrator_response
            .tool_calls
            .map(|calls| calls.into_iter().map(|c| c.into()).collect()),
        processing_time_ms,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Send a chat message with streaming response
///
/// Sends a message to an agent and receives a streaming response using Server-Sent Events (SSE).
/// This endpoint provides real-time updates as the agent processes the message.
///
/// # Example
///
/// ```json
/// {
///   "message": "Hello! Tell me about yourself.",
///   "user_id": "user-123",
///   "stream": true
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/agents/{agent_id}/chat/stream",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    request_body = ChatMessageRequest,
    responses(
        (status = 200, description = "Streaming chat response", content_type = "text/event-stream"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    ),
    tag = "chat"
)]
pub async fn send_chat_message_stream(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<ChatMessageRequest>,
) -> ServerResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    use agent_mem_core::orchestrator::ChatRequest as OrchestratorChatRequest;
    use futures::stream;

    // Validate agent exists and belongs to user's organization
    let agent_repo = repositories.agents.clone();
    let agent = agent_repo
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Enforce tenant isolation
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this agent"));
    }

    // Validate message content
    if req.message.trim().is_empty() {
        return Err(ServerError::bad_request("Message cannot be empty"));
    }

    // Determine user_id
    let user_id = req
        .user_id
        .clone()
        .unwrap_or_else(|| auth_user.user_id.clone());

    info!(
        "Starting streaming chat for agent_id={}, user_id={}, message_len={}",
        agent_id,
        user_id,
        req.message.len()
    );

    // Create AgentOrchestrator
    let orchestrator = create_orchestrator(&agent, &repositories)
        .await
        .map_err(|e| {
            error!("Failed to create orchestrator: {}", e);
            ServerError::internal_error(format!("Failed to create orchestrator: {}", e))
        })?;

    // Create orchestrator request
    let session_id_for_stream = req
        .session_id
        .clone()
        .unwrap_or_else(|| format!("{}_{}", user_id, Uuid::new_v4()));
    let orchestrator_request = OrchestratorChatRequest {
        message: req.message.clone(),
        agent_id: agent_id.clone(),
        user_id: user_id.clone(),
        organization_id: auth_user.org_id.clone(),
        session_id: session_id_for_stream,
        stream: true,
        max_memories: 10,
    };

    info!("🌊 启动真实SSE流式传输");

    // ✅ 使用Orchestrator的真实流式方法（需要Arc）
    let orchestrator_arc = Arc::new(orchestrator);
    let llm_stream = match orchestrator_arc.step_stream(orchestrator_request).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to start streaming: {}", e);
            // 返回错误流
            let error_chunk = StreamChunk {
                chunk_type: "error".to_string(),
                content: Some(format!("Failed to start stream: {}", e)),
                tool_call: None,
                memories_count: None,
            };
            let error_json = serde_json::to_string(&error_chunk).unwrap_or_else(|_| {
                "{\"chunk_type\":\"error\",\"content\":\"Stream initialization failed\"}".to_string()
            });
            let error_stream = stream::once(async move { Ok::<Event, Infallible>(Event::default().data(error_json)) });
            return Ok(Sse::new(error_stream).keep_alive(
                KeepAlive::new()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive-text"),
            ));
        }
    };

    // ✅ 转换LLM流为SSE Event流
    let response_stream = stream::unfold(
        (llm_stream, true, false),
        |(mut llm_stream, is_first, mut is_done)| async move {
            if is_done {
                return None;
            }

            // 发送start事件（仅第一次）
            if is_first {
                let start_chunk = StreamChunk {
                    chunk_type: "start".to_string(),
                    content: None,
                    tool_call: None,
                    memories_count: None,
                };
                let start_json = serde_json::to_string(&start_chunk)
                    .unwrap_or_else(|_| "{\"chunk_type\":\"start\"}".to_string());
                let start_event = Ok::<Event, Infallible>(Event::default().data(start_json));
                return Some((start_event, (llm_stream, false, is_done)));
            }

            // 从LLM流读取下一块
            match llm_stream.next().await {
                Some(Ok(content_chunk)) => {
                    // 发送内容块
                    let content_event_chunk = StreamChunk {
                        chunk_type: "content".to_string(),
                        content: Some(content_chunk),
                        tool_call: None,
                        memories_count: None,
                    };
                    let content_json = serde_json::to_string(&content_event_chunk)
                        .unwrap_or_else(|_| "{\"chunk_type\":\"content\",\"content\":\"\"}".to_string());
                    let event = Ok::<Event, Infallible>(Event::default().data(content_json));
                    Some((event, (llm_stream, false, is_done)))
                }
                Some(Err(e)) => {
                    // LLM流错误
                    error!("LLM stream error: {}", e);
                    let error_chunk = StreamChunk {
                        chunk_type: "error".to_string(),
                        content: Some(format!("LLM error: {}", e)),
                        tool_call: None,
                        memories_count: None,
                    };
                    let error_json = serde_json::to_string(&error_chunk)
                        .unwrap_or_else(|_| "{\"chunk_type\":\"error\"}".to_string());
                    is_done = true;
                    let event = Ok::<Event, Infallible>(Event::default().data(error_json));
                    Some((event, (llm_stream, false, is_done)))
                }
                None => {
                    // 流结束
                    info!("✅ LLM流式传输完成");
                    let done_chunk = StreamChunk {
                        chunk_type: "done".to_string(),
                        content: None,
                        tool_call: None,
                        memories_count: Some(0), // TODO: 从context传递memories_count
                    };
                    let done_json = serde_json::to_string(&done_chunk)
                        .unwrap_or_else(|_| "{\"chunk_type\":\"done\"}".to_string());
                    is_done = true;
                    let event = Ok::<Event, Infallible>(Event::default().data(done_json));
                    Some((event, (llm_stream, false, is_done)))
                }
            }
        },
    );

    Ok(Sse::new(response_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

/// Get chat history for an agent
///
/// Returns the recent chat messages for an agent.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{agent_id}/chat/history",
    params(
        ("agent_id" = String, Path, description = "Agent ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of messages to return (default: 50)")
    ),
    responses(
        (status = 200, description = "Chat history retrieved successfully"),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    ),
    tag = "chat"
)]
pub async fn get_chat_history(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
) -> ServerResult<Json<ApiResponse<Vec<JsonValue>>>> {
    // Validate agent exists and belongs to user's organization
    let agent_repo = repositories.agents.clone();
    let agent = agent_repo
        .find_by_id(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Enforce tenant isolation
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this agent"));
    }

    // Get recent messages
    let message_repo = repositories.messages.clone();
    let messages = message_repo
        .find_by_agent_id(&agent_id, 50)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get messages: {e}")))?;

    // Convert to JSON
    let history: Vec<JsonValue> = messages
        .into_iter()
        .map(|msg| {
            serde_json::json!({
                "id": msg.id,
                "role": msg.role,
                "content": msg.text,
                "created_at": msg.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(history)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_deserialization() {
        let json = r#"{"message": "Hello", "stream": false}"#;
        let req: ChatMessageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message, "Hello");
        assert_eq!(req.stream, false);
    }
}

