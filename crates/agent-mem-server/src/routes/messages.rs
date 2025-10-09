//! Message API routes
//!
//! This module provides REST API endpoints for message management.
//!
//! Features:
//! - Full CRUD operations for messages
//! - JWT and API Key authentication
//! - Multi-tenant isolation
//! - Filtering by agent and user
//! - OpenAPI documentation

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use agent_mem_core::storage::{
    message_repository::MessageRepository,
    models::{generate_id, Message},
    repository::Repository,
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use utoipa::ToSchema;

/// Request to create a new message
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    /// User ID
    pub user_id: String,
    /// Agent ID
    pub agent_id: String,
    /// Message role (user, assistant, system, tool)
    pub role: String,
    /// Message text
    pub text: Option<String>,
    /// Message content (structured)
    pub content: Option<JsonValue>,
    /// Model name
    pub model: Option<String>,
    /// Tool calls
    pub tool_calls: Option<JsonValue>,
    /// Metadata
    pub metadata: Option<JsonValue>,
}

/// Message response
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub role: String,
    pub text: Option<String>,
    pub content: Option<JsonValue>,
    pub model: Option<String>,
    pub tool_calls: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub created_at: String,
}

impl From<Message> for MessageResponse {
    fn from(msg: Message) -> Self {
        Self {
            id: msg.id,
            organization_id: msg.organization_id,
            user_id: msg.user_id,
            agent_id: msg.agent_id,
            role: msg.role,
            text: msg.text,
            content: msg.content,
            model: msg.model,
            tool_calls: msg.tool_calls,
            metadata: None, // Message model doesn't have metadata field
            created_at: msg.created_at.to_rfc3339(),
        }
    }
}

/// Query parameters for listing messages
#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub agent_id: Option<String>,
    pub user_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Create a new message
///
/// Creates a new message in the system.
/// Requires authentication and automatically associates with user's organization.
#[utoipa::path(
    post,
    path = "/api/v1/messages",
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "Message created successfully", body = MessageResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "messages",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn create_message(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateMessageRequest>,
) -> ServerResult<(StatusCode, Json<ApiResponse<MessageResponse>>)> {
    let repo = MessageRepository::new(pool);

    // Validate request
    if req.role.is_empty() {
        return Err(ServerError::bad_request("Message role cannot be empty"));
    }
    if !["user", "assistant", "system", "tool"].contains(&req.role.as_str()) {
        return Err(ServerError::bad_request("Invalid message role"));
    }

    let organization_id = auth_user.org_id.clone();

    let now = Utc::now();
    let message = Message {
        id: generate_id("msg"),
        organization_id,
        user_id: req.user_id,
        agent_id: req.agent_id,
        role: req.role,
        text: req.text,
        content: req.content,
        model: req.model,
        name: None,
        tool_calls: req.tool_calls,
        tool_call_id: None,
        step_id: None,
        otid: None,
        tool_returns: None,
        group_id: None,
        sender_id: None,
        created_at: now,
        updated_at: now,
        is_deleted: false,
        created_by_id: Some(auth_user.user_id.clone()),
        last_updated_by_id: Some(auth_user.user_id.clone()),
    };

    // Note: metadata is ignored as Message model doesn't have this field
    let _ = req.metadata;

    let created = repo
        .create(&message)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to create message: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(MessageResponse::from(created))),
    ))
}

/// Get a message by ID
///
/// Retrieves a single message by its ID.
/// Enforces tenant isolation.
#[utoipa::path(
    get,
    path = "/api/v1/messages/{id}",
    params(
        ("id" = String, Path, description = "Message ID")
    ),
    responses(
        (status = 200, description = "Message found", body = MessageResponse),
        (status = 404, description = "Message not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - message belongs to different organization"),
    ),
    tag = "messages",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_message(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<Json<ApiResponse<MessageResponse>>> {
    let repo = MessageRepository::new(pool);

    let message = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read message: {e}")))?
        .ok_or_else(|| ServerError::not_found("Message not found"))?;

    // Enforce tenant isolation
    if message.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this message"));
    }

    Ok(Json(ApiResponse::success(MessageResponse::from(message))))
}

/// List messages
///
/// Lists messages with optional filtering by agent or user.
/// Automatically filters by organization for tenant isolation.
#[utoipa::path(
    get,
    path = "/api/v1/messages",
    params(
        ("agent_id" = Option<String>, Query, description = "Filter by agent ID"),
        ("user_id" = Option<String>, Query, description = "Filter by user ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of messages to return (default: 50, max: 100)"),
        ("offset" = Option<i64>, Query, description = "Number of messages to skip (default: 0)"),
    ),
    responses(
        (status = 200, description = "List of messages", body = Vec<MessageResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "messages",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn list_messages(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListMessagesQuery>,
) -> ServerResult<Json<ApiResponse<Vec<MessageResponse>>>> {
    let repo = MessageRepository::new(pool);

    // Validate pagination parameters
    let limit = query.limit.map(|l| l.min(100)).or(Some(50));
    let offset = query.offset.map(|o| o.max(0)).or(Some(0));

    let messages = if let Some(agent_id) = query.agent_id {
        repo.list_by_agent(&agent_id, limit, offset)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to list messages: {e}")))?
    } else {
        // For user_id or organization-wide listing, use list() and filter
        let all_messages = repo
            .list(limit, offset)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to list messages: {e}")))?;
        all_messages
            .into_iter()
            .filter(|m| {
                m.organization_id == auth_user.org_id
                    && (query.user_id.is_none() || Some(&m.user_id) == query.user_id.as_ref())
            })
            .collect()
    };

    // Filter by organization for tenant isolation
    let filtered_messages: Vec<Message> = messages
        .into_iter()
        .filter(|m| m.organization_id == auth_user.org_id)
        .collect();

    let responses: Vec<MessageResponse> = filtered_messages
        .into_iter()
        .map(MessageResponse::from)
        .collect();

    Ok(Json(ApiResponse::success(responses)))
}

/// Delete a message (soft delete)
///
/// Soft deletes a message by marking it as deleted.
/// Enforces tenant isolation.
#[utoipa::path(
    delete,
    path = "/api/v1/messages/{id}",
    params(
        ("id" = String, Path, description = "Message ID")
    ),
    responses(
        (status = 204, description = "Message deleted successfully"),
        (status = 404, description = "Message not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - message belongs to different organization"),
    ),
    tag = "messages",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn delete_message(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<StatusCode> {
    let repo = MessageRepository::new(pool);

    // First check if message exists and belongs to user's organization
    let message = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read message: {e}")))?
        .ok_or_else(|| ServerError::not_found("Message not found"))?;

    // Enforce tenant isolation
    if message.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this message"));
    }

    repo.delete(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to delete message: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}
