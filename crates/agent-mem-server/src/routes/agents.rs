//! Agent API routes
//!
//! This module provides REST API endpoints for agent management,
//! inspired by MIRIX's agent routes but implemented in Rust with Axum.
//!
//! Features:
//! - Full CRUD operations for agents
//! - JWT and API Key authentication
//! - Multi-tenant isolation
//! - RBAC authorization
//! - OpenAPI documentation


use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use agent_mem_core::storage::{factory::Repositories, models::{generate_id, Agent}};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use utoipa::ToSchema;

/// Request to create a new agent
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAgentRequest {
    /// Agent name
    pub name: Option<String>,
    /// Agent description
    pub description: Option<String>,
    /// System prompt
    pub system: Option<String>,
    /// Agent type
    pub agent_type: Option<String>,
    /// LLM configuration
    pub llm_config: Option<JsonValue>,
    /// Embedding configuration
    pub embedding_config: Option<JsonValue>,
    /// Tool rules
    pub tool_rules: Option<JsonValue>,
    /// Metadata
    pub metadata: Option<JsonValue>,
}

/// Request to update an agent
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAgentRequest {
    /// Agent name
    pub name: Option<String>,
    /// Agent description
    pub description: Option<String>,
    /// System prompt
    pub system: Option<String>,
    /// Agent type
    pub agent_type: Option<String>,
    /// LLM configuration
    pub llm_config: Option<JsonValue>,
    /// Embedding configuration
    pub embedding_config: Option<JsonValue>,
    /// Tool rules
    pub tool_rules: Option<JsonValue>,
    /// Metadata
    pub metadata: Option<JsonValue>,
}

/// Agent response
#[derive(Debug, Serialize, ToSchema)]
pub struct AgentResponse {
    pub id: String,
    pub organization_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub system: Option<String>,
    pub agent_type: Option<String>,
    pub llm_config: Option<JsonValue>,
    pub embedding_config: Option<JsonValue>,
    pub tool_rules: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub state: Option<String>,
    pub last_active_at: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Agent> for AgentResponse {
    fn from(agent: Agent) -> Self {
        Self {
            id: agent.id,
            organization_id: agent.organization_id,
            name: agent.name,
            description: agent.description,
            system: agent.system,
            agent_type: agent.agent_type,
            llm_config: agent.llm_config,
            embedding_config: agent.embedding_config,
            tool_rules: agent.tool_rules,
            metadata: agent.metadata_,
            state: agent.state,
            last_active_at: agent.last_active_at.map(|dt| dt.to_rfc3339()),
            error_message: agent.error_message,
            created_at: agent.created_at.to_rfc3339(),
            updated_at: agent.updated_at.to_rfc3339(),
        }
    }
}

/// Query parameters for listing agents
#[derive(Debug, Deserialize)]
pub struct ListAgentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Request to send a message to an agent
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    /// Message content
    pub message: String,
    /// User ID (optional, defaults to authenticated user)
    pub user_id: Option<String>,
    /// Whether to memorize the interaction
    #[serde(default)]
    pub memorizing: bool,
    /// Image URIs (optional)
    pub image_uris: Option<Vec<String>>,
    /// Voice files (optional)
    pub voice_files: Option<Vec<String>>,
    /// Additional metadata
    pub metadata: Option<JsonValue>,
}

/// Response from sending a message to an agent
#[derive(Debug, Serialize, ToSchema)]
pub struct SendMessageResponse {
    /// Agent's response
    pub response: String,
    /// Message ID
    pub message_id: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Create a new agent
///
/// Creates a new agent with the specified configuration.
/// Requires authentication and automatically associates the agent with the user's organization.
#[utoipa::path(
    post,
    path = "/api/v1/agents",
    request_body = CreateAgentRequest,
    responses(
        (status = 201, description = "Agent created successfully", body = AgentResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "agents",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn create_agent(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateAgentRequest>,
) -> ServerResult<(StatusCode, Json<ApiResponse<AgentResponse>>)> {
    let repo = repositories.agents.clone();

    // Validate request
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err(ServerError::bad_request("Agent name cannot be empty"));
        }
        if name.len() > 255 {
            return Err(ServerError::bad_request(
                "Agent name too long (max 255 characters)",
            ));
        }
    }

    // Create agent with authenticated user's organization
    let mut agent = Agent::new(auth_user.org_id.clone(), req.name);
    agent.description = req.description;
    agent.system = req.system;
    agent.agent_type = req.agent_type;
    agent.llm_config = req.llm_config;
    agent.embedding_config = req.embedding_config;
    agent.tool_rules = req.tool_rules;
    agent.metadata_ = req.metadata;
    agent.created_by_id = Some(auth_user.user_id.clone());
    agent.last_updated_by_id = Some(auth_user.user_id.clone());

    let created = repo
        .create(&agent)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to create agent: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(AgentResponse::from(created))),
    ))
}

/// Get an agent by ID
///
/// Retrieves a single agent by its ID.
/// Enforces tenant isolation - users can only access agents in their organization.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{id}",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Agent found", body = AgentResponse),
        (status = 404, description = "Agent not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - agent belongs to different organization"),
    ),
    tag = "agents",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_agent(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<Json<ApiResponse<AgentResponse>>> {
    let repo = repositories.agents.clone();

    let agent = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Enforce tenant isolation
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this agent"));
    }

    Ok(Json(ApiResponse::success(AgentResponse::from(agent))))
}

/// Update an agent
///
/// Updates an existing agent's configuration.
/// Enforces tenant isolation and tracks the user who made the update.
#[utoipa::path(
    put,
    path = "/api/v1/agents/{id}",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    request_body = UpdateAgentRequest,
    responses(
        (status = 200, description = "Agent updated successfully", body = AgentResponse),
        (status = 404, description = "Agent not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - agent belongs to different organization"),
    ),
    tag = "agents",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn update_agent(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAgentRequest>,
) -> ServerResult<Json<ApiResponse<AgentResponse>>> {
    let repo = repositories.agents.clone();

    let mut agent = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Enforce tenant isolation
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this agent"));
    }

    // Validate and update fields
    if let Some(name) = req.name {
        if name.trim().is_empty() {
            return Err(ServerError::bad_request("Agent name cannot be empty"));
        }
        if name.len() > 255 {
            return Err(ServerError::bad_request(
                "Agent name too long (max 255 characters)",
            ));
        }
        agent.name = Some(name);
    }
    if let Some(description) = req.description {
        agent.description = Some(description);
    }
    if let Some(system) = req.system {
        agent.system = Some(system);
    }
    if let Some(agent_type) = req.agent_type {
        agent.agent_type = Some(agent_type);
    }
    if let Some(llm_config) = req.llm_config {
        agent.llm_config = Some(llm_config);
    }
    if let Some(embedding_config) = req.embedding_config {
        agent.embedding_config = Some(embedding_config);
    }
    if let Some(tool_rules) = req.tool_rules {
        agent.tool_rules = Some(tool_rules);
    }
    if let Some(metadata) = req.metadata {
        agent.metadata_ = Some(metadata);
    }

    agent.updated_at = Utc::now();
    agent.last_updated_by_id = Some(auth_user.user_id.clone());

    let updated = repo
        .update(&agent)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to update agent: {e}")))?;

    Ok(Json(ApiResponse::success(AgentResponse::from(updated))))
}

/// Delete an agent (soft delete)
///
/// Soft deletes an agent by marking it as deleted.
/// Enforces tenant isolation.
#[utoipa::path(
    delete,
    path = "/api/v1/agents/{id}",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 204, description = "Agent deleted successfully"),
        (status = 404, description = "Agent not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - agent belongs to different organization"),
    ),
    tag = "agents",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn delete_agent(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<StatusCode> {
    let repo = repositories.agents.clone();

    // First check if agent exists and belongs to user's organization
    let agent = repo
        .read(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to read agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Enforce tenant isolation
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied to this agent"));
    }

    repo.delete(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to delete agent: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

/// List agents
///
/// Lists all agents in the user's organization with pagination support.
/// Automatically filters by organization for tenant isolation.
#[utoipa::path(
    get,
    path = "/api/v1/agents",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of agents to return (default: 50, max: 100)"),
        ("offset" = Option<i64>, Query, description = "Number of agents to skip (default: 0)"),
    ),
    responses(
        (status = 200, description = "List of agents", body = Vec<AgentResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "agents",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn list_agents(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListAgentsQuery>,
) -> ServerResult<Json<ApiResponse<Vec<AgentResponse>>>> {
    let repo = repositories.agents.clone();

    // Validate pagination parameters
    let limit = query.limit.unwrap_or(50).min(100); // Max 100 items per page
    let offset = query.offset.unwrap_or(0).max(0); // Ensure non-negative

    let agents = repo
        .list_by_organization(&auth_user.org_id, Some(limit), Some(offset))
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list agents: {e}")))?;

    let responses: Vec<AgentResponse> = agents.into_iter().map(AgentResponse::from).collect();

    Ok(Json(ApiResponse::success(responses)))
}

/// Send a message to an agent
///
/// Sends a message to an agent and receives a response.
/// This endpoint integrates with the LLM system and memory management.
#[utoipa::path(
    post,
    path = "/api/v1/agents/{id}/messages",
    params(
        ("id" = String, Path, description = "Agent ID")
    ),
    request_body = SendMessageRequest,
    responses(
        (status = 200, description = "Message sent successfully", body = SendMessageResponse),
        (status = 404, description = "Agent not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - agent belongs to different organization"),
        (status = 500, description = "Internal server error - LLM processing failed"),
    ),
    tag = "agents",
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn send_message_to_agent(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> ServerResult<Json<ApiResponse<SendMessageResponse>>> {
    use agent_mem_core::storage::message_repository::MessageRepository;
    use agent_mem_core::storage::models::Message;

    let agent_repo = AgentRepository::new(pool.clone());
    let message_repo = MessageRepository::new(pool.clone());

    // Validate agent exists and belongs to user's organization
    let agent = agent_repo
        .read(&id)
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

    let start_time = std::time::Instant::now();

    // Create user message
    let user_message_id = generate_id("msg");
    let now = Utc::now();
    let user_message = Message {
        id: user_message_id.clone(),
        organization_id: auth_user.org_id.clone(),
        user_id: req
            .user_id
            .clone()
            .unwrap_or_else(|| auth_user.user_id.clone()),
        agent_id: id.clone(),
        role: "user".to_string(),
        text: Some(req.message.clone()),
        content: None,
        model: None,
        name: None,
        tool_calls: None,
        tool_call_id: None,
        step_id: None,
        otid: None,
        tool_returns: None,
        group_id: None,
        sender_id: Some(auth_user.user_id.clone()),
        created_at: now,
        updated_at: now,
        is_deleted: false,
        created_by_id: Some(auth_user.user_id.clone()),
        last_updated_by_id: Some(auth_user.user_id.clone()),
    };

    // Note: metadata is ignored as Message model doesn't have this field
    let _ = req.metadata.clone();

    // Save user message
    message_repo
        .create(&user_message)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to save user message: {e}")))?;

    // TODO: Integrate with LLM system to generate response
    // For now, return a placeholder response
    let response_text = format!("Received your message: {}", req.message);

    // Create assistant message
    let assistant_message_id = generate_id("msg");
    let assistant_message = Message {
        id: assistant_message_id.clone(),
        organization_id: auth_user.org_id.clone(),
        user_id: req.user_id.unwrap_or_else(|| auth_user.user_id.clone()),
        agent_id: id.clone(),
        role: "assistant".to_string(),
        text: Some(response_text.clone()),
        content: None,
        model: agent
            .llm_config
            .as_ref()
            .and_then(|c| c.get("model"))
            .and_then(|m| m.as_str())
            .map(|s| s.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        step_id: None,
        otid: None,
        tool_returns: None,
        group_id: None,
        sender_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_deleted: false,
        created_by_id: None,
        last_updated_by_id: None,
    };

    // Save assistant message
    message_repo.create(&assistant_message).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to save assistant message: {e}"))
    })?;

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    let response = SendMessageResponse {
        response: response_text,
        message_id: assistant_message_id,
        processing_time_ms,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get agent state
///
/// Returns the current state of an agent.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{agent_id}/state",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Agent state retrieved successfully", body = AgentStateResponse),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_agent_state(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
) -> ServerResult<Json<ApiResponse<AgentStateResponse>>> {
    let agent_repo = AgentRepository::new(pool.clone());

    // Get agent
    let agent = agent_repo
        .read(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Check authorization
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied"));
    }

    let response = AgentStateResponse {
        agent_id: agent.id,
        state: agent.state.unwrap_or_else(|| "idle".to_string()),
        last_active_at: agent.last_active_at.map(|dt| dt.to_rfc3339()),
        error_message: agent.error_message,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update agent state
///
/// Updates the state of an agent.
#[utoipa::path(
    put,
    path = "/api/v1/agents/{agent_id}/state",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    request_body = UpdateAgentStateRequest,
    responses(
        (status = 200, description = "Agent state updated successfully", body = AgentStateResponse),
        (status = 404, description = "Agent not found"),
        (status = 400, description = "Invalid state transition"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn update_agent_state(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(agent_id): Path<String>,
    Json(req): Json<UpdateAgentStateRequest>,
) -> ServerResult<Json<ApiResponse<AgentStateResponse>>> {
    let agent_repo = AgentRepository::new(pool.clone());

    // Get agent
    let mut agent = agent_repo
        .read(&agent_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get agent: {e}")))?
        .ok_or_else(|| ServerError::not_found("Agent not found"))?;

    // Check authorization
    if agent.organization_id != auth_user.org_id {
        return Err(ServerError::forbidden("Access denied"));
    }

    // Validate state
    let valid_states = vec!["idle", "thinking", "executing", "waiting", "error"];
    if !valid_states.contains(&req.state.as_str()) {
        return Err(ServerError::bad_request(format!(
            "Invalid state: {}. Valid states are: {}",
            req.state,
            valid_states.join(", ")
        )));
    }

    // Update agent state
    agent.state = Some(req.state.clone());
    agent.last_active_at = Some(Utc::now());
    agent.error_message = req.error_message.clone();
    agent.updated_at = Utc::now();

    // Save agent
    agent_repo
        .update(&agent)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to update agent: {e}")))?;

    let response = AgentStateResponse {
        agent_id: agent.id,
        state: agent.state.unwrap_or_else(|| "idle".to_string()),
        last_active_at: agent.last_active_at.map(|dt| dt.to_rfc3339()),
        error_message: agent.error_message,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Agent state response
#[derive(Debug, Serialize, ToSchema)]
pub struct AgentStateResponse {
    pub agent_id: String,
    pub state: String,
    pub last_active_at: Option<String>,
    pub error_message: Option<String>,
}

/// Request to update agent state
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAgentStateRequest {
    /// New state (idle, thinking, executing, waiting, error)
    pub state: String,
    /// Error message (only for error state)
    pub error_message: Option<String>,
}
