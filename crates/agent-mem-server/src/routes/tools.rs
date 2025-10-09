//! Tool API routes
//!
//! This module provides REST API endpoints for tool management and execution.
//!
//! Features:
//! - Full CRUD operations for tools
//! - Tool execution in sandboxed environment
//! - JWT and API Key authentication
//! - Multi-tenant isolation
//! - OpenAPI documentation

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use agent_mem_core::storage::{
    models::{generate_id, Tool},
    repository::Repository,
    tool_repository::ToolRepository,
};
use agent_mem_tools::sandbox::SandboxManager;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use utoipa::ToSchema;

/// Request to register a new tool
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterToolRequest {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// JSON schema for tool parameters
    pub json_schema: Option<JsonValue>,
    /// Source type (e.g., "python", "bash", "javascript")
    pub source_type: Option<String>,
    /// Source code for the tool
    pub source_code: Option<String>,
    /// Tags for categorization
    pub tags: Option<Vec<String>>,
    /// Additional metadata
    pub metadata: Option<JsonValue>,
}

/// Request to update a tool
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateToolRequest {
    /// Tool name
    pub name: Option<String>,
    /// Tool description
    pub description: Option<String>,
    /// JSON schema for tool parameters
    pub json_schema: Option<JsonValue>,
    /// Source type
    pub source_type: Option<String>,
    /// Source code
    pub source_code: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Metadata
    pub metadata: Option<JsonValue>,
}

/// Tool response
#[derive(Debug, Serialize, ToSchema)]
pub struct ToolResponse {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub json_schema: Option<JsonValue>,
    pub source_type: Option<String>,
    pub source_code: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<JsonValue>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Tool> for ToolResponse {
    fn from(tool: Tool) -> Self {
        Self {
            id: tool.id,
            organization_id: tool.organization_id,
            name: tool.name,
            description: tool.description,
            json_schema: tool.json_schema,
            source_type: tool.source_type,
            source_code: tool.source_code,
            tags: tool.tags,
            metadata: tool.metadata_,
            created_at: tool.created_at.to_rfc3339(),
            updated_at: tool.updated_at.to_rfc3339(),
        }
    }
}

/// Request to execute a tool
#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteToolRequest {
    /// Tool arguments as key-value pairs
    pub arguments: HashMap<String, String>,
    /// Timeout in seconds (optional, default: 30)
    pub timeout_seconds: Option<u64>,
}

/// Tool execution response
#[derive(Debug, Serialize, ToSchema)]
pub struct ToolExecutionResponse {
    pub tool_id: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
    pub execution_time_ms: u64,
}

/// Query parameters for listing tools
#[derive(Debug, Deserialize)]
pub struct ListToolsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub tags: Option<String>, // Comma-separated tags
}

/// Register a new tool
///
/// Creates a new tool with the specified configuration.
/// Requires authentication and automatically associates with user's organization.
#[utoipa::path(
    post,
    path = "/api/v1/tools",
    request_body = RegisterToolRequest,
    responses(
        (status = 201, description = "Tool registered successfully", body = ToolResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "tools",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn register_tool(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<RegisterToolRequest>,
) -> ServerResult<(StatusCode, Json<ApiResponse<ToolResponse>>)> {
    let repo = ToolRepository::new(pool);

    // Validate request
    if req.name.trim().is_empty() {
        return Err(ServerError::bad_request("Tool name cannot be empty"));
    }
    if req.name.len() > 255 {
        return Err(ServerError::bad_request(
            "Tool name too long (max 255 characters)",
        ));
    }

    let now = Utc::now();
    let tool = Tool {
        id: generate_id("tool"),
        organization_id: auth_user.org_id.clone(),
        name: req.name,
        description: req.description,
        json_schema: req.json_schema,
        source_type: req.source_type,
        source_code: req.source_code,
        tags: req.tags,
        metadata_: req.metadata,
        created_at: now,
        updated_at: now,
        is_deleted: false,
        created_by_id: Some(auth_user.user_id.clone()),
        last_updated_by_id: Some(auth_user.user_id.clone()),
    };

    let created = repo
        .create(&tool)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to create tool: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(ToolResponse::from(created))),
    ))
}

/// Get a tool by ID
///
/// Retrieves a single tool by its ID.
/// Enforces tenant isolation.
#[utoipa::path(
    get,
    path = "/api/v1/tools/{id}",
    params(
        ("id" = String, Path, description = "Tool ID")
    ),
    responses(
        (status = 200, description = "Tool found", body = ToolResponse),
        (status = 404, description = "Tool not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - tool belongs to different organization"),
    ),
    tag = "tools",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_tool(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<Json<ApiResponse<ToolResponse>>> {
    let repo = ToolRepository::new(pool);

    let tool = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read tool: {e}")))?
        .ok_or_else(|| ServerError::not_found("Tool not found"))?;

    // Enforce tenant isolation
    if tool.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this tool"));
    }

    Ok(Json(ApiResponse::success(ToolResponse::from(tool))))
}

/// List tools
///
/// Lists all tools in the user's organization with optional filtering by tags.
/// Automatically filters by organization for tenant isolation.
#[utoipa::path(
    get,
    path = "/api/v1/tools",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of tools to return (default: 50, max: 100)"),
        ("offset" = Option<i64>, Query, description = "Number of tools to skip (default: 0)"),
        ("tags" = Option<String>, Query, description = "Comma-separated tags to filter by"),
    ),
    responses(
        (status = 200, description = "List of tools", body = Vec<ToolResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "tools",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn list_tools(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListToolsQuery>,
) -> ServerResult<Json<ApiResponse<Vec<ToolResponse>>>> {
    let repo = ToolRepository::new(pool);

    // Validate pagination parameters
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0).max(0);

    let tools = if let Some(tags_str) = query.tags {
        let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        // Default to match_any (false) for tag filtering
        repo.list_by_tags(&auth_user.org_id, &tags, false).await
    } else {
        repo.list_by_organization(&auth_user.org_id, Some(limit), Some(offset))
            .await
    }
    .map_err(|e| ServerError::internal_error(format!("Failed to list tools: {e}")))?;

    let responses: Vec<ToolResponse> = tools.into_iter().map(ToolResponse::from).collect();

    Ok(Json(ApiResponse::success(responses)))
}

/// Update a tool
///
/// Updates an existing tool's configuration.
/// Enforces tenant isolation and tracks the user who made the update.
#[utoipa::path(
    put,
    path = "/api/v1/tools/{id}",
    params(
        ("id" = String, Path, description = "Tool ID")
    ),
    request_body = UpdateToolRequest,
    responses(
        (status = 200, description = "Tool updated successfully", body = ToolResponse),
        (status = 404, description = "Tool not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - tool belongs to different organization"),
    ),
    tag = "tools",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn update_tool(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<UpdateToolRequest>,
) -> ServerResult<Json<ApiResponse<ToolResponse>>> {
    let repo = ToolRepository::new(pool);

    let mut tool = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read tool: {e}")))?
        .ok_or_else(|| ServerError::not_found("Tool not found"))?;

    // Enforce tenant isolation
    if tool.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this tool"));
    }

    // Validate and update fields
    if let Some(name) = req.name {
        if name.trim().is_empty() {
            return Err(ServerError::bad_request("Tool name cannot be empty"));
        }
        if name.len() > 255 {
            return Err(ServerError::bad_request(
                "Tool name too long (max 255 characters)",
            ));
        }
        tool.name = name;
    }
    if let Some(description) = req.description {
        tool.description = Some(description);
    }
    if let Some(json_schema) = req.json_schema {
        tool.json_schema = Some(json_schema);
    }
    if let Some(source_type) = req.source_type {
        tool.source_type = Some(source_type);
    }
    if let Some(source_code) = req.source_code {
        tool.source_code = Some(source_code);
    }
    if let Some(tags) = req.tags {
        tool.tags = Some(tags);
    }
    if let Some(metadata) = req.metadata {
        tool.metadata_ = Some(metadata);
    }

    tool.updated_at = Utc::now();
    tool.last_updated_by_id = Some(auth_user.user_id.clone());

    let updated = repo
        .update(&tool)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to update tool: {e}")))?;

    Ok(Json(ApiResponse::success(ToolResponse::from(updated))))
}

/// Delete a tool (soft delete)
///
/// Soft deletes a tool by marking it as deleted.
/// Enforces tenant isolation.
#[utoipa::path(
    delete,
    path = "/api/v1/tools/{id}",
    params(
        ("id" = String, Path, description = "Tool ID")
    ),
    responses(
        (status = 204, description = "Tool deleted successfully"),
        (status = 404, description = "Tool not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - tool belongs to different organization"),
    ),
    tag = "tools",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn delete_tool(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<StatusCode> {
    let repo = ToolRepository::new(pool);

    // First check if tool exists and belongs to user's organization
    let tool = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read tool: {e}")))?
        .ok_or_else(|| ServerError::not_found("Tool not found"))?;

    // Enforce tenant isolation
    if tool.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this tool"));
    }

    repo.delete(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to delete tool: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Execute a tool
///
/// Executes a tool in a sandboxed environment with the provided arguments.
/// This is a critical security feature that ensures tools run safely.
#[utoipa::path(
    post,
    path = "/api/v1/tools/{id}/execute",
    params(
        ("id" = String, Path, description = "Tool ID")
    ),
    request_body = ExecuteToolRequest,
    responses(
        (status = 200, description = "Tool executed successfully", body = ToolExecutionResponse),
        (status = 404, description = "Tool not found"),
        (status = 500, description = "Tool execution failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - tool belongs to different organization"),
    ),
    tag = "tools",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn execute_tool(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteToolRequest>,
) -> ServerResult<Json<ApiResponse<ToolExecutionResponse>>> {
    let repo = ToolRepository::new(pool);

    // Validate tool exists and belongs to user's organization
    let tool = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read tool: {e}")))?
        .ok_or_else(|| ServerError::not_found("Tool not found"))?;

    // Enforce tenant isolation
    if tool.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this tool"));
    }

    // Validate tool has source code
    let source_code = tool
        .source_code
        .ok_or_else(|| ServerError::bad_request("Tool has no source code to execute"))?;

    let source_type = tool.source_type.unwrap_or_else(|| "bash".to_string());

    // Create sandbox with default configuration
    let sandbox = SandboxManager::default();

    // Determine command based on source type
    let (command, args) = match source_type.as_str() {
        "bash" | "sh" => {
            // For bash scripts, write to temp file and execute
            ("bash", vec!["-c".to_string(), source_code.clone()])
        }
        "python" => ("python3", vec!["-c".to_string(), source_code.clone()]),
        "javascript" | "js" => ("node", vec!["-e".to_string(), source_code.clone()]),
        _ => {
            return Err(ServerError::bad_request(format!(
                "Unsupported source type: {source_type}"
            )));
        }
    };

    // Add user arguments as environment variables
    let mut env_vars = HashMap::new();
    for (key, value) in req.arguments {
        env_vars.insert(format!("ARG_{}", key.to_uppercase()), value);
    }

    // Execute in sandbox with timeout
    let timeout = Duration::from_secs(req.timeout_seconds.unwrap_or(30));
    let start_time = std::time::Instant::now();

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = sandbox
        .execute_command(command, &args_refs, timeout)
        .await
        .map_err(|e| ServerError::internal_error(format!("Tool execution failed: {e}")))?;

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    let response = ToolExecutionResponse {
        tool_id: tool.id,
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.exit_code,
        success: output.success,
        execution_time_ms,
    };

    Ok(Json(ApiResponse::success(response)))
}
