//! Working Memory API Routes
//!
//! Working Memory provides session-based temporary context for conversations.
//! Uses the unified memories table with memory_type='working'.
//!
//! Architecture:
//! - Directly uses WorkingMemoryStore from Repositories (high cohesion)
//! - No wrapper needed (low coupling)
//! - RESTful API design

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};
use utoipa::{IntoParams, ToSchema};

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use agent_mem_core::storage::factory::Repositories;
use agent_mem_traits::WorkingMemoryItem;

/// Request to add a working memory item
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddWorkingMemoryRequest {
    /// Session ID for isolation
    pub session_id: String,
    /// Content of the memory item
    pub content: String,
    /// Priority (1-10, default: 1)
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Optional expiration time (seconds from now)
    pub expires_in_seconds: Option<i64>,
    /// Optional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_priority() -> i32 {
    1
}

/// Response for added working memory item
#[derive(Debug, Serialize, ToSchema)]
pub struct AddWorkingMemoryResponse {
    pub id: String,
    pub session_id: String,
    pub content: String,
    pub created_at: String,
}

/// Query parameters for getting working memory
#[derive(Debug, Deserialize, IntoParams)]
pub struct GetWorkingMemoryQuery {
    /// Session ID to filter by
    pub session_id: String,
    /// Minimum priority to filter by
    pub min_priority: Option<i32>,
}

/// Query parameters for clearing working memory
#[derive(Debug, Deserialize, ToSchema)]
pub struct ClearWorkingMemoryQuery {
    /// Session ID to clear
    pub session_id: String,
}

/// Response for clearing working memory
#[derive(Debug, Serialize, ToSchema)]
pub struct ClearWorkingMemoryResponse {
    pub deleted_count: i64,
    pub session_id: String,
}

/// POST /api/v1/working-memory
///
/// Add a working memory item for a session
#[utoipa::path(
    post,
    path = "/api/v1/working-memory",
    tag = "working-memory",
    request_body = AddWorkingMemoryRequest,
    responses(
        (status = 200, description = "Item added successfully", body = AddWorkingMemoryResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_working_memory(
    axum::Extension(repositories): axum::Extension<Arc<Repositories>>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    Json(request): Json<AddWorkingMemoryRequest>,
) -> ServerResult<Json<AddWorkingMemoryResponse>> {
    info!(
        "Adding working memory item for user={}, session={}",
        auth_user.user_id, request.session_id
    );

    // Validate request
    if request.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content cannot be empty"));
    }

    if request.session_id.trim().is_empty() {
        return Err(ServerError::bad_request("Session ID cannot be empty"));
    }

    // Create working memory item
    let expires_at = request.expires_in_seconds.map(|seconds| {
        chrono::Utc::now() + chrono::Duration::seconds(seconds)
    });

    let item = WorkingMemoryItem {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: auth_user.user_id.clone(),
        agent_id: "default".to_string(), // Can be enhanced to accept agent_id
        session_id: request.session_id.clone(),
        content: request.content.clone(),
        priority: request.priority.clamp(1, 10), // Ensure priority is 1-10
        expires_at,
        metadata: request.metadata.clone(),
        created_at: chrono::Utc::now(),
    };

    // ✅ Use WorkingMemoryStore from Repositories (high cohesion, low coupling)
    let added_item = repositories
        .working_memory
        .add_item(item)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to add working memory item: {}", e)))?;

    debug!("Working memory item added: id={}", added_item.id);

    Ok(Json(AddWorkingMemoryResponse {
        id: added_item.id,
        session_id: added_item.session_id,
        content: added_item.content,
        created_at: added_item.created_at.to_rfc3339(),
    }))
}

/// GET /api/v1/working-memory
///
/// Get working memory items for a session
#[utoipa::path(
    get,
    path = "/api/v1/working-memory",
    tag = "working-memory",
    params(GetWorkingMemoryQuery),
    responses(
        (status = 200, description = "Items retrieved successfully", body = Vec<WorkingMemoryItem>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_working_memory(
    axum::Extension(repositories): axum::Extension<Arc<Repositories>>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    Query(query): Query<GetWorkingMemoryQuery>,
) -> ServerResult<Json<Vec<WorkingMemoryItem>>> {
    info!(
        "Getting working memory for user={}, session={}",
        auth_user.user_id, query.session_id
    );

    if query.session_id.trim().is_empty() {
        return Err(ServerError::bad_request("Session ID cannot be empty"));
    }

    // ✅ Use WorkingMemoryStore from Repositories
    let items = if let Some(min_priority) = query.min_priority {
        repositories
            .working_memory
            .get_by_priority(&query.session_id, min_priority)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to get working memory items: {}", e)))?
    } else {
        repositories
            .working_memory
            .get_session_items(&query.session_id)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to get working memory items: {}", e)))?
    };

    debug!("Retrieved {} working memory items", items.len());

    Ok(Json(items))
}

/// DELETE /api/v1/working-memory/:item_id
///
/// Delete a specific working memory item
#[utoipa::path(
    delete,
    path = "/api/v1/working-memory/{item_id}",
    tag = "working-memory",
    params(
        ("item_id" = String, Path, description = "Working memory item ID")
    ),
    responses(
        (status = 204, description = "Item deleted successfully"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_working_memory_item(
    axum::Extension(repositories): axum::Extension<Arc<Repositories>>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    Path(item_id): Path<String>,
) -> ServerResult<StatusCode> {
    info!(
        "Deleting working memory item: user={}, item_id={}",
        auth_user.user_id, item_id
    );

    // ✅ Use WorkingMemoryStore from Repositories
    let deleted = repositories
        .working_memory
        .remove_item(&item_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to delete working memory item: {}", e)))?;

    if deleted {
        debug!("Working memory item deleted: id={}", item_id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ServerError::not_found("Working memory item not found"))
    }
}

/// DELETE /api/v1/working-memory/sessions/:session_id
///
/// Clear all working memory items for a session
#[utoipa::path(
    delete,
    path = "/api/v1/working-memory/sessions/{session_id}",
    tag = "working-memory",
    params(
        ("session_id" = String, Path, description = "Session ID to clear")
    ),
    responses(
        (status = 200, description = "Session cleared successfully", body = ClearWorkingMemoryResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn clear_working_memory(
    axum::Extension(repositories): axum::Extension<Arc<Repositories>>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    Path(session_id): Path<String>,
) -> ServerResult<Json<ClearWorkingMemoryResponse>> {
    info!(
        "Clearing working memory for user={}, session={}",
        auth_user.user_id, session_id
    );

    if session_id.trim().is_empty() {
        return Err(ServerError::bad_request("Session ID cannot be empty"));
    }

    // ✅ Use WorkingMemoryStore from Repositories
    let deleted_count = repositories
        .working_memory
        .clear_session(&session_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to clear working memory: {}", e)))?;

    info!("Cleared {} working memory items for session {}", deleted_count, session_id);

    Ok(Json(ClearWorkingMemoryResponse {
        deleted_count,
        session_id,
    }))
}

/// POST /api/v1/working-memory/cleanup
///
/// Clean up expired working memory items (admin endpoint)
#[utoipa::path(
    post,
    path = "/api/v1/working-memory/cleanup",
    tag = "working-memory",
    responses(
        (status = 200, description = "Cleanup completed", body = CleanupResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cleanup_expired(
    axum::Extension(repositories): axum::Extension<Arc<Repositories>>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
) -> ServerResult<Json<CleanupResponse>> {
    info!("Cleaning up expired working memory items (user={})", auth_user.user_id);

    // ✅ Use WorkingMemoryStore from Repositories
    let cleaned_count = repositories
        .working_memory
        .clear_expired()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to cleanup expired items: {}", e)))?;

    info!("Cleaned up {} expired working memory items", cleaned_count);

    Ok(Json(CleanupResponse {
        cleaned_count,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CleanupResponse {
    pub cleaned_count: i64,
}

