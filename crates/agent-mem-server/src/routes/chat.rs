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
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::convert::Infallible;
use std::sync::Arc;
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

    // Clone for the stream
    let orchestrator = Arc::new(orchestrator);
    let orchestrator_clone = orchestrator.clone();

    // ✅ Create real streaming response - 真正的流式响应
    // 使用状态机模式：0=start, 1=streaming content, 2=done
    enum StreamState {
        Start(Arc<AgentOrchestrator>, OrchestratorChatRequest),
        Streaming(String, usize, usize), // (content, memories_count, char_index)
        Done,
    }

    let initial_state = StreamState::Start(orchestrator_clone, orchestrator_request);

    let response_stream = stream::unfold(initial_state, |state| async move {
        match state {
            StreamState::Start(orch, req) => {
                // Send start chunk
                let start_chunk = StreamChunk {
                    chunk_type: "start".to_string(),
                    content: None,
                    tool_call: None,
                    memories_count: None,
                };

                let start_event = match serde_json::to_string(&start_chunk) {
                    Ok(json) => Ok(Event::default().data(json)),
                    Err(e) => {
                        error!("Failed to serialize start chunk: {}", e);
                        // Even if serialization fails, send a fallback error message
                        let fallback_error = StreamChunk {
                            chunk_type: "error".to_string(),
                            content: Some("Failed to initialize stream".to_string()),
                            tool_call: None,
                            memories_count: None,
                        };
                        match serde_json::to_string(&fallback_error) {
                            Ok(json) => Ok(Event::default().data(json)),
                            Err(_) => {
                                // Last resort: send plain text error
                                Ok(Event::default().data("{\"chunk_type\":\"error\",\"content\":\"Stream initialization failed\"}"))
                            }
                        }
                    }
                };

                // Execute orchestrator and get full response
                match orch.step(req).await {
                    Ok(response) => {
                        let content = response.content;
                        let memories_count = response.memories_count;

                        info!(
                            "Orchestrator step completed: content_len={}, memories_count={}",
                            content.len(),
                            memories_count
                        );

                        // Transition to streaming state
                        Some((
                            start_event,
                            StreamState::Streaming(content, memories_count, 0),
                        ))
                    }
                    Err(e) => {
                        error!("Orchestrator step failed: {}", e);
                        // Send error and end - ensure we always send something
                        let error_chunk = StreamChunk {
                            chunk_type: "error".to_string(),
                            content: Some(format!("Orchestrator error: {}", e)),
                            tool_call: None,
                            memories_count: None,
                        };

                        let error_event = match serde_json::to_string(&error_chunk) {
                            Ok(json) => Ok(Event::default().data(json)),
                            Err(ser_err) => {
                                error!("Failed to serialize error chunk: {}", ser_err);
                                // Fallback: send plain text error
                                Ok(Event::default().data(format!(
                                    "{{\"chunk_type\":\"error\",\"content\":\"{}\"}}",
                                    e.to_string().replace('"', "\\\"")
                                )))
                            }
                        };
                        Some((error_event, StreamState::Done))
                    }
                }
            }
            StreamState::Streaming(content, memories_count, char_index) => {
                // Stream content in small chunks for typewriter effect
                const CHUNK_SIZE: usize = 5; // Send 5 chars at a time

                let char_count = content.chars().count();
                if char_index >= char_count {
                    // All content sent, send done chunk
                    let done_chunk = StreamChunk {
                        chunk_type: "done".to_string(),
                        content: None,
                        tool_call: None,
                        memories_count: Some(memories_count),
                    };

                    let done_event = match serde_json::to_string(&done_chunk) {
                        Ok(json) => Ok(Event::default().data(json)),
                        Err(e) => {
                            error!("Failed to serialize done chunk: {}", e);
                            // Fallback: send plain text done message
                            Ok(Event::default().data(format!(
                                "{{\"chunk_type\":\"done\",\"memories_count\":{}}}",
                                memories_count
                            )))
                        }
                    };
                    Some((done_event, StreamState::Done))
                } else {
                    // Send next chunk of content (using character-based indexing for UTF-8 safety)
                    let chars: Vec<char> = content.chars().collect();
                    let end_index = std::cmp::min(char_index + CHUNK_SIZE, chars.len());
                    let chunk_content: String = chars[char_index..end_index].iter().collect();

                    let content_chunk = StreamChunk {
                        chunk_type: "content".to_string(),
                        content: Some(chunk_content.clone()),
                        tool_call: None,
                        memories_count: None,
                    };

                    let content_event = match serde_json::to_string(&content_chunk) {
                        Ok(json) => Ok(Event::default().data(json)),
                        Err(e) => {
                            error!("Failed to serialize content chunk: {}", e);
                            // Fallback: send plain text content
                            Ok(Event::default().data(format!(
                                "{{\"chunk_type\":\"content\",\"content\":\"{}\"}}",
                                chunk_content.replace('"', "\\\"").replace('\n', "\\n")
                            )))
                        }
                    };

                    // Add small delay for typewriter effect (optional, can be removed for faster streaming)
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

                    Some((
                        content_event,
                        StreamState::Streaming(content, memories_count, end_index),
                    ))
                }
            }
            StreamState::Done => None,
        }
    });

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
