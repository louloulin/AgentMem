//! Organization management API routes
//!
//! This module provides REST API endpoints for organization management:
//! - Create organization
//! - Get organization details
//! - Update organization
//! - List organization members
//! - Manage organization settings

use crate::error::{ServerError, ServerResult};
use crate::middleware::AuthUser;
use agent_mem_core::storage::{
    models::Organization,
    repository::{OrganizationRepository, Repository},
};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// Organization response
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub settings: OrganizationSettings,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Organization settings
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrganizationSettings {
    pub max_users: Option<i32>,
    pub max_agents: Option<i32>,
    pub max_memories: Option<i32>,
    pub retention_days: Option<i32>,
}

/// Create organization request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 2))]
    pub name: String,
    pub description: Option<String>,
}

/// Update organization request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOrganizationRequest {
    #[validate(length(min = 2))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub settings: Option<OrganizationSettings>,
}

/// Organization member response
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationMemberResponse {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
    pub joined_at: i64,
}

/// List organizations query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListOrganizationsQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

/// Create a new organization
#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created successfully", body = OrganizationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_organization(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateOrganizationRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Create organization repository
    let org_repo = OrganizationRepository::new(db_pool);

    // Create organization in database
    let org_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let org = Organization {
        id: org_id.clone(),
        name: request.name.clone(),
        created_at: now,
        updated_at: now,
        is_deleted: false,
    };

    let created_org = org_repo
        .create(&org)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to create organization: {e}")))?;

    let response = OrganizationResponse {
        id: created_org.id,
        name: created_org.name,
        description: request.description,
        settings: OrganizationSettings {
            max_users: Some(100),
            max_agents: Some(50),
            max_memories: Some(10000),
            retention_days: Some(365),
        },
        created_at: created_org.created_at.timestamp(),
        updated_at: created_org.updated_at.timestamp(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get organization by ID
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Organization details", body = OrganizationResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_organization(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    // Check if user belongs to the organization
    if auth_user.org_id != org_id && !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden(
            "Access to this organization is forbidden".to_string(),
        ));
    }

    // Create organization repository
    let org_repo = OrganizationRepository::new(db_pool);

    // Fetch organization from database
    let org = org_repo
        .read(&org_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?
        .ok_or_else(|| ServerError::NotFound("Organization not found".to_string()))?;

    let response = OrganizationResponse {
        id: org.id,
        name: org.name,
        description: None, // TODO: Add description field to Organization model
        settings: OrganizationSettings {
            max_users: Some(100),
            max_agents: Some(50),
            max_memories: Some(10000),
            retention_days: Some(365),
        },
        created_at: org.created_at.timestamp(),
        updated_at: org.updated_at.timestamp(),
    };

    Ok(Json(response))
}

/// Update organization
#[utoipa::path(
    put,
    path = "/api/v1/organizations/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID")
    ),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated successfully", body = OrganizationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_organization(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
    Json(request): Json<UpdateOrganizationRequest>,
) -> ServerResult<impl IntoResponse> {
    // Check if user belongs to the organization and has admin role
    if auth_user.org_id != org_id || !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden(
            "Admin role required for this organization".to_string(),
        ));
    }

    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Create organization repository
    let org_repo = OrganizationRepository::new(db_pool);

    // Fetch existing organization
    let mut org = org_repo
        .read(&org_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?
        .ok_or_else(|| ServerError::NotFound("Organization not found".to_string()))?;

    // Update fields
    if let Some(name) = request.name {
        org.name = name;
    }
    org.updated_at = chrono::Utc::now();

    // Update in database
    let updated_org = org_repo
        .update(&org)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to update organization: {e}")))?;

    let response = OrganizationResponse {
        id: updated_org.id,
        name: updated_org.name,
        description: request.description,
        settings: request.settings.unwrap_or(OrganizationSettings {
            max_users: Some(100),
            max_agents: Some(50),
            max_memories: Some(10000),
            retention_days: Some(365),
        }),
        created_at: updated_org.created_at.timestamp(),
        updated_at: updated_org.updated_at.timestamp(),
    };

    Ok(Json(response))
}

/// List organization members
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/members",
    params(
        ("org_id" = String, Path, description = "Organization ID"),
        ListOrganizationsQuery
    ),
    responses(
        (status = 200, description = "List of organization members", body = Vec<OrganizationMemberResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_organization_members(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
    Query(_query): Query<ListOrganizationsQuery>,
) -> ServerResult<impl IntoResponse> {
    // Check if user belongs to the organization
    if auth_user.org_id != org_id && !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden(
            "Access to this organization is forbidden".to_string(),
        ));
    }

    // Create user repository
    let user_repo = agent_mem_core::storage::user_repository::UserRepository::new(db_pool);

    // Fetch members from database
    let users = user_repo
        .list_by_organization(&org_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?;

    let members: Vec<OrganizationMemberResponse> = users
        .into_iter()
        .map(|user| OrganizationMemberResponse {
            user_id: user.id,
            email: user.email,
            name: user.name,
            roles: user.roles,
            joined_at: user.created_at.timestamp(),
        })
        .collect();

    Ok(Json(members))
}

/// Delete organization (admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{org_id}",
    params(
        ("org_id" = String, Path, description = "Organization ID")
    ),
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "organizations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_organization(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(org_id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    // Check if user is admin
    if !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden("Admin role required".to_string()));
    }

    // Create organization repository
    let org_repo = OrganizationRepository::new(db_pool);

    // Delete organization from database (soft delete)
    let deleted = org_repo
        .delete(&org_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to delete organization: {e}")))?;

    if !deleted {
        return Err(ServerError::NotFound("Organization not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
