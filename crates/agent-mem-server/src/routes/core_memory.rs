//! Core Memory API Routes
//!
//! Core Memory provides persistent context for agents - Persona and Human blocks.
//! These blocks are always included in the agent's context window.
//!
//! Architecture:
//! - Uses CoreMemoryManager from agent-mem-core (in-process)
//! - For production, data should be synced to the memories table
//! - RESTful API design with OpenAPI documentation
//!
//! 🔴 Phase 1: Core Memory API - AgentMem v2.0

use agent_mem_core::managers::CoreMemoryManager;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};
use utoipa::{IntoParams, ToSchema};

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;

/// Application state holding CoreMemoryManager (for use with Extension extractor)
#[derive(Clone)]
pub struct CoreMemoryState {
    pub manager: Arc<CoreMemoryManager>,
}

impl CoreMemoryState {
    pub fn new(manager: CoreMemoryManager) -> Self {
        Self {
            manager: Arc::new(manager),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert CoreMemoryBlock to CoreMemoryBlockResponse
fn block_to_response(block: agent_mem_core::managers::CoreMemoryBlock) -> CoreMemoryBlockResponse {
    CoreMemoryBlockResponse {
        id: block.id,
        block_type: format!("{:?}", block.block_type),
        content: block.content,
        importance: block.importance,
        max_capacity: block.max_capacity,
        current_size: block.current_size,
        capacity_usage_percent: (block.current_size as f32 / block.max_capacity as f32) * 100.0,
        created_at: block.created_at.to_rfc3339(),
        updated_at: block.updated_at.to_rfc3339(),
        last_accessed: block.last_accessed.to_rfc3339(),
        access_count: block.access_count,
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create a persona block
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreatePersonaRequest {
    /// Agent ID this persona belongs to
    pub agent_id: String,
    /// Content of the persona block
    pub content: String,
    /// Maximum capacity in characters (optional, default: 2000)
    pub max_capacity: Option<usize>,
}

/// Request to create a human block
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateHumanRequest {
    /// User ID this human info belongs to
    pub user_id: String,
    /// Content of the human block
    pub content: String,
    /// Maximum capacity in characters (optional, default: 4000)
    pub max_capacity: Option<usize>,
}

/// Request to update a block's content
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateBlockRequest {
    /// New content for the block
    pub content: String,
}

/// Request to append content to a block
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AppendContentRequest {
    /// Content to append
    pub content: String,
}

/// Query parameters for listing persona blocks
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListPersonaQuery {
    /// Optional agent_id filter
    pub agent_id: Option<String>,
}

/// Query parameters for listing human blocks
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListHumanQuery {
    /// Optional user_id filter
    pub user_id: Option<String>,
}

/// Core memory block response
#[derive(Debug, Serialize, ToSchema)]
pub struct CoreMemoryBlockResponse {
    pub id: String,
    pub block_type: String,
    pub content: String,
    pub importance: f32,
    pub max_capacity: usize,
    pub current_size: usize,
    pub capacity_usage_percent: f32,
    pub created_at: String,
    pub updated_at: String,
    pub last_accessed: String,
    pub access_count: u64,
}

/// Capacity information for a single block
#[derive(Debug, Serialize, ToSchema)]
pub struct CapacityInfo {
    pub id: String,
    pub block_type: String,
    pub current_size: usize,
    pub max_capacity: usize,
    pub usage_percent: f32,
}

/// Capacity response for all blocks
#[derive(Debug, Serialize, ToSchema)]
pub struct CapacityResponse {
    pub persona_blocks: Vec<CapacityInfo>,
    pub human_blocks: Vec<CapacityInfo>,
    pub total_usage_percent: f32,
    pub total_blocks: usize,
}

/// Stats response
#[derive(Debug, Serialize, ToSchema)]
pub struct CoreMemoryStatsResponse {
    pub persona_blocks_count: usize,
    pub human_blocks_count: usize,
    pub total_accesses: u64,
    pub auto_rewrites: u64,
    pub average_capacity_usage: f32,
}

/// Rewrite response
#[derive(Debug, Serialize, ToSchema)]
pub struct RewriteResponse {
    pub block_id: String,
    pub success: bool,
    pub new_size: usize,
    pub message: String,
}

/// Error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// API Endpoints
// ============================================================================

// -------------------- Persona Block Endpoints --------------------

/// POST /api/v1/core-memory/persona
///
/// Create a new persona block for an agent
#[utoipa::path(
    post,
    path = "/api/v1/core-memory/persona",
    tag = "core-memory",
    request_body = CreatePersonaRequest,
    responses(
        (status = 201, description = "Persona block created", body = CoreMemoryBlockResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_persona_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    Json(request): Json<CreatePersonaRequest>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    info!(
        "Creating persona block for agent={}, user={}",
        request.agent_id, auth_user.user_id
    );

    if request.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content cannot be empty"));
    }

    if request.agent_id.trim().is_empty() {
        return Err(ServerError::bad_request("Agent ID cannot be empty"));
    }

    let block_id = state
        .manager
        .create_persona_block(request.content, request.max_capacity)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to create persona block: {}", e)))?;

    let block = state
        .manager
        .get_persona_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get persona block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Created block not found"))?;

    debug!("Persona block created: id={}", block_id);

    Ok(Json(block_to_response(block)))
}

/// GET /api/v1/core-memory/persona/{block_id}
///
/// Get a persona block by ID
#[utoipa::path(
    get,
    path = "/api/v1/core-memory/persona/{block_id}",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Persona block ID")
    ),
    responses(
        (status = 200, description = "Persona block found", body = CoreMemoryBlockResponse),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_persona_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    debug!("Getting persona block: id={}", block_id);

    let block = state
        .manager
        .get_persona_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get persona block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Persona block not found"))?;

    Ok(Json(block_to_response(block)))
}

/// PUT /api/v1/core-memory/persona/{block_id}
///
/// Update a persona block's content
#[utoipa::path(
    put,
    path = "/api/v1/core-memory/persona/{block_id}",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Persona block ID")
    ),
    request_body = UpdateBlockRequest,
    responses(
        (status = 200, description = "Persona block updated", body = CoreMemoryBlockResponse),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_persona_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
    Json(request): Json<UpdateBlockRequest>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    info!("Updating persona block: id={}", block_id);

    if request.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content cannot be empty"));
    }

    state
        .manager
        .update_persona_block(&block_id, request.content)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to update persona block: {}", e)))?;

    let block = state
        .manager
        .get_persona_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get persona block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Persona block not found"))?;

    Ok(Json(block_to_response(block)))
}

/// POST /api/v1/core-memory/persona/{block_id}/append
///
/// Append content to a persona block
#[utoipa::path(
    post,
    path = "/api/v1/core-memory/persona/{block_id}/append",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Persona block ID")
    ),
    request_body = AppendContentRequest,
    responses(
        (status = 200, description = "Content appended", body = CoreMemoryBlockResponse),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn append_to_persona_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
    Json(request): Json<AppendContentRequest>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    info!("Appending to persona block: id={}", block_id);

    if request.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content cannot be empty"));
    }

    state
        .manager
        .append_to_persona_block(&block_id, &request.content)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to append to persona block: {}", e)))?;

    let block = state
        .manager
        .get_persona_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get persona block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Persona block not found"))?;

    Ok(Json(block_to_response(block)))
}

/// GET /api/v1/core-memory/persona
///
/// List all persona blocks (with optional agent_id filter)
#[utoipa::path(
    get,
    path = "/api/v1/core-memory/persona",
    tag = "core-memory",
    params(
        ("agent_id" = Option<String>, Query, description = "Filter by agent ID")
    ),
    responses(
        (status = 200, description = "Persona blocks retrieved", body = Vec<CoreMemoryBlockResponse>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_persona_blocks(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Query(_query): Query<ListPersonaQuery>,
) -> ServerResult<Json<Vec<CoreMemoryBlockResponse>>> {
    debug!("Listing persona blocks");

    let blocks = state
        .manager
        .list_persona_blocks()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list persona blocks: {}", e)))?;

    let response: Vec<CoreMemoryBlockResponse> = blocks
        .into_iter()
        .map(block_to_response)
        .collect();

    Ok(Json(response))
}

// -------------------- Human Block Endpoints --------------------

/// POST /api/v1/core-memory/human
///
/// Create a new human block
#[utoipa::path(
    post,
    path = "/api/v1/core-memory/human",
    tag = "core-memory",
    request_body = CreateHumanRequest,
    responses(
        (status = 201, description = "Human block created", body = CoreMemoryBlockResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_human_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    Json(request): Json<CreateHumanRequest>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    info!(
        "Creating human block for user={}, requested_by={}",
        request.user_id, auth_user.user_id
    );

    if request.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content cannot be empty"));
    }

    if request.user_id.trim().is_empty() {
        return Err(ServerError::bad_request("User ID cannot be empty"));
    }

    let block_id = state
        .manager
        .create_human_block(request.content, request.max_capacity)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to create human block: {}", e)))?;

    let block = state
        .manager
        .get_human_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get human block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Created block not found"))?;

    debug!("Human block created: id={}", block_id);

    Ok(Json(block_to_response(block)))
}

/// GET /api/v1/core-memory/human/{block_id}
///
/// Get a human block by ID
#[utoipa::path(
    get,
    path = "/api/v1/core-memory/human/{block_id}",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Human block ID")
    ),
    responses(
        (status = 200, description = "Human block found", body = CoreMemoryBlockResponse),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_human_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    debug!("Getting human block: id={}", block_id);

    let block = state
        .manager
        .get_human_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get human block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Human block not found"))?;

    Ok(Json(block_to_response(block)))
}

/// PUT /api/v1/core-memory/human/{block_id}
///
/// Update a human block's content
#[utoipa::path(
    put,
    path = "/api/v1/core-memory/human/{block_id}",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Human block ID")
    ),
    request_body = UpdateBlockRequest,
    responses(
        (status = 200, description = "Human block updated", body = CoreMemoryBlockResponse),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_human_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
    Json(request): Json<UpdateBlockRequest>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    info!("Updating human block: id={}", block_id);

    if request.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content cannot be empty"));
    }

    state
        .manager
        .update_human_block(&block_id, request.content)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to update human block: {}", e)))?;

    let block = state
        .manager
        .get_human_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get human block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Human block not found"))?;

    Ok(Json(block_to_response(block)))
}

/// POST /api/v1/core-memory/human/{block_id}/append
///
/// Append content to a human block
#[utoipa::path(
    post,
    path = "/api/v1/core-memory/human/{block_id}/append",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Human block ID")
    ),
    request_body = AppendContentRequest,
    responses(
        (status = 200, description = "Content appended", body = CoreMemoryBlockResponse),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn append_to_human_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
    Json(request): Json<AppendContentRequest>,
) -> ServerResult<Json<CoreMemoryBlockResponse>> {
    info!("Appending to human block: id={}", block_id);

    if request.content.trim().is_empty() {
        return Err(ServerError::bad_request("Content cannot be empty"));
    }

    state
        .manager
        .append_to_human_block(&block_id, &request.content)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to append to human block: {}", e)))?;

    let block = state
        .manager
        .get_human_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get human block: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Human block not found"))?;

    Ok(Json(block_to_response(block)))
}

/// GET /api/v1/core-memory/human
///
/// List all human blocks
#[utoipa::path(
    get,
    path = "/api/v1/core-memory/human",
    tag = "core-memory",
    params(
        ("user_id" = Option<String>, Query, description = "Filter by user ID")
    ),
    responses(
        (status = 200, description = "Human blocks retrieved", body = Vec<CoreMemoryBlockResponse>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_human_blocks(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Query(_query): Query<ListHumanQuery>,
) -> ServerResult<Json<Vec<CoreMemoryBlockResponse>>> {
    debug!("Listing human blocks");

    let blocks = state
        .manager
        .list_human_blocks()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list human blocks: {}", e)))?;

    let response: Vec<CoreMemoryBlockResponse> = blocks
        .into_iter()
        .map(block_to_response)
        .collect();

    Ok(Json(response))
}

// -------------------- Capacity & Stats Endpoints --------------------

/// GET /api/v1/core-memory/capacity
///
/// Get capacity information for all blocks
#[utoipa::path(
    get,
    path = "/api/v1/core-memory/capacity",
    tag = "core-memory",
    responses(
        (status = 200, description = "Capacity info retrieved", body = CapacityResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_capacity(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
) -> ServerResult<Json<CapacityResponse>> {
    debug!("Getting capacity information");

    // Get persona blocks
    let persona_list = state
        .manager
        .list_persona_blocks()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list persona blocks: {}", e)))?;

    let persona_blocks: Vec<CapacityInfo> = persona_list
        .iter()
        .map(|block| CapacityInfo {
            id: block.id.clone(),
            block_type: format!("{:?}", block.block_type),
            current_size: block.current_size,
            max_capacity: block.max_capacity,
            usage_percent: (block.current_size as f32 / block.max_capacity as f32) * 100.0,
        })
        .collect();

    // Get human blocks
    let human_list = state
        .manager
        .list_human_blocks()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list human blocks: {}", e)))?;

    let human_blocks: Vec<CapacityInfo> = human_list
        .iter()
        .map(|block| CapacityInfo {
            id: block.id.clone(),
            block_type: format!("{:?}", block.block_type),
            current_size: block.current_size,
            max_capacity: block.max_capacity,
            usage_percent: (block.current_size as f32 / block.max_capacity as f32) * 100.0,
        })
        .collect();

    // Calculate totals
    let total_size: usize = persona_list.iter().map(|b| b.current_size).sum::<usize>()
        + human_list.iter().map(|b| b.current_size).sum::<usize>();
    let total_capacity: usize = persona_list.iter().map(|b| b.max_capacity).sum::<usize>()
        + human_list.iter().map(|b| b.max_capacity).sum::<usize>();

    let total_usage_percent = if total_capacity > 0 {
        (total_size as f32 / total_capacity as f32) * 100.0
    } else {
        0.0
    };

    Ok(Json(CapacityResponse {
        persona_blocks,
        human_blocks,
        total_usage_percent,
        total_blocks: persona_list.len() + human_list.len(),
    }))
}

/// GET /api/v1/core-memory/stats
///
/// Get CoreMemory statistics
#[utoipa::path(
    get,
    path = "/api/v1/core-memory/stats",
    tag = "core-memory",
    responses(
        (status = 200, description = "Stats retrieved", body = CoreMemoryStatsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_stats(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
) -> ServerResult<Json<CoreMemoryStatsResponse>> {
    debug!("Getting CoreMemory stats");

    let stats = state
        .manager
        .get_stats()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get stats: {}", e)))?;

    Ok(Json(CoreMemoryStatsResponse {
        persona_blocks_count: stats.persona_blocks_count,
        human_blocks_count: stats.human_blocks_count,
        total_accesses: stats.total_accesses,
        auto_rewrites: stats.auto_rewrites,
        average_capacity_usage: stats.average_capacity_usage * 100.0,
    }))
}

/// POST /api/v1/core-memory/rewrite/{block_id}
///
/// Manually trigger rewrite for a block
#[utoipa::path(
    post,
    path = "/api/v1/core-memory/rewrite/{block_id}",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Block ID to rewrite")
    ),
    responses(
        (status = 200, description = "Block rewritten", body = RewriteResponse),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn rewrite_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
) -> ServerResult<Json<RewriteResponse>> {
    info!("Manual rewrite for block: id={}", block_id);

    state
        .manager
        .manual_rewrite_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to rewrite block: {}", e)))?;

    // Try to get the block from persona blocks first, then human blocks
    let persona_result = state
        .manager
        .get_persona_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get persona block: {}", e)))?;

    let human_result = if persona_result.is_none() {
        state
            .manager
            .get_human_block(&block_id)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to get human block: {}", e)))?
    } else {
        None
    };

    let block = persona_result.or(human_result);

    match block {
        Some(b) => Ok(Json(RewriteResponse {
            block_id,
            success: true,
            new_size: b.current_size,
            message: "Block rewritten successfully".to_string(),
        })),
        None => Err(ServerError::not_found("Block not found")),
    }
}

// -------------------- Delete Endpoints --------------------

/// DELETE /api/v1/core-memory/persona/{block_id}
///
/// Delete a persona block
#[utoipa::path(
    delete,
    path = "/api/v1/core-memory/persona/{block_id}",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Persona block ID")
    ),
    responses(
        (status = 204, description = "Block deleted"),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_persona_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
) -> ServerResult<StatusCode> {
    info!("Deleting persona block: id={}", block_id);

    state
        .manager
        .delete_persona_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to delete persona block: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/core-memory/human/{block_id}
///
/// Delete a human block
#[utoipa::path(
    delete,
    path = "/api/v1/core-memory/human/{block_id}",
    tag = "core-memory",
    params(
        ("block_id" = String, Path, description = "Human block ID")
    ),
    responses(
        (status = 204, description = "Block deleted"),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_human_block(
    Extension(state): Extension<Arc<CoreMemoryState>>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Path(block_id): Path<String>,
) -> ServerResult<StatusCode> {
    info!("Deleting human block: id={}", block_id);

    state
        .manager
        .delete_human_block(&block_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to delete human block: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// OpenAPI Schema
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_persona_request_serialization() {
        let request = CreatePersonaRequest {
            agent_id: "agent-1".to_string(),
            content: "You are a helpful assistant".to_string(),
            max_capacity: Some(2000),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("agent-1"));
        assert!(json.contains("helpful assistant"));

        let deserialized: CreatePersonaRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent_id, "agent-1");
    }

    #[test]
    fn test_create_human_request_serialization() {
        let request = CreateHumanRequest {
            user_id: "user-1".to_string(),
            content: "User preferences here".to_string(),
            max_capacity: Some(4000),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("user-1"));

        let deserialized: CreateHumanRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user_id, "user-1");
    }

    #[test]
    fn test_capacity_response_serialization() {
        let response = CapacityResponse {
            persona_blocks: vec![CapacityInfo {
                id: "block-1".to_string(),
                block_type: "Persona".to_string(),
                current_size: 1000,
                max_capacity: 2000,
                usage_percent: 50.0,
            }],
            human_blocks: vec![],
            total_usage_percent: 50.0,
            total_blocks: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("50.0"));
        assert!(json.contains("block-1"));
    }

    #[test]
    fn test_core_memory_block_response_serialization() {
        let response = CoreMemoryBlockResponse {
            id: "test-id".to_string(),
            block_type: "Persona".to_string(),
            content: "Test content".to_string(),
            importance: 0.8,
            max_capacity: 2000,
            current_size: 500,
            capacity_usage_percent: 25.0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            access_count: 10,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Persona"));
        assert!(json.contains("25.0"));
    }
}
