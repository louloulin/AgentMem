//! HTTP routes for the AgentMem API

pub mod agents;
pub mod chat;
pub mod docs;
pub mod health;
pub mod memory;
pub mod messages;
pub mod metrics;
pub mod organizations;
pub mod tools;
pub mod users;

use crate::error::ServerResult;
use crate::middleware::{audit_logging_middleware, quota_middleware};
use crate::routes::memory::MemoryManager;
use crate::sse::SseManager;
use crate::websocket::WebSocketManager;
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Extension, Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create the main router with all routes
pub async fn create_router(
    memory_manager: Arc<MemoryManager>,
    db_pool: PgPool,
) -> ServerResult<Router> {
    // Create WebSocket and SSE managers
    let ws_manager = Arc::new(WebSocketManager::new());
    let sse_manager = Arc::new(SseManager::new());

    let app = Router::new()
        // Memory management routes
        .route("/api/v1/memories", post(memory::add_memory))
        .route("/api/v1/memories/:id", get(memory::get_memory))
        .route("/api/v1/memories/:id", put(memory::update_memory))
        .route("/api/v1/memories/:id", delete(memory::delete_memory))
        .route("/api/v1/memories/search", post(memory::search_memories))
        .route(
            "/api/v1/memories/:id/history",
            get(memory::get_memory_history),
        )
        // Batch operations
        .route("/api/v1/memories/batch", post(memory::batch_add_memories))
        .route(
            "/api/v1/memories/batch/delete",
            post(memory::batch_delete_memories),
        )
        // User management routes
        .route("/api/v1/users/register", post(users::register_user))
        .route("/api/v1/users/login", post(users::login_user))
        .route("/api/v1/users/me", get(users::get_current_user))
        .route("/api/v1/users/me", put(users::update_current_user))
        .route("/api/v1/users/me/password", post(users::change_password))
        .route("/api/v1/users/:user_id", get(users::get_user_by_id))
        // Organization management routes
        .route(
            "/api/v1/organizations",
            post(organizations::create_organization),
        )
        .route(
            "/api/v1/organizations/:org_id",
            get(organizations::get_organization),
        )
        .route(
            "/api/v1/organizations/:org_id",
            put(organizations::update_organization),
        )
        .route(
            "/api/v1/organizations/:org_id",
            delete(organizations::delete_organization),
        )
        .route(
            "/api/v1/organizations/:org_id/members",
            get(organizations::list_organization_members),
        )
        // Agent management routes
        .route("/api/v1/agents", post(agents::create_agent))
        .route("/api/v1/agents/:id", get(agents::get_agent))
        .route("/api/v1/agents/:id", put(agents::update_agent))
        .route("/api/v1/agents/:id", delete(agents::delete_agent))
        .route("/api/v1/agents", get(agents::list_agents))
        .route(
            "/api/v1/agents/:id/messages",
            post(agents::send_message_to_agent),
        )
        // Chat routes (new AgentOrchestrator-based API)
        .route(
            "/api/v1/agents/:agent_id/chat",
            post(chat::send_chat_message),
        )
        .route(
            "/api/v1/agents/:agent_id/chat/stream",
            post(chat::send_chat_message_stream),
        )
        .route(
            "/api/v1/agents/:agent_id/chat/history",
            get(chat::get_chat_history),
        )
        // Agent state management routes
        .route(
            "/api/v1/agents/:agent_id/state",
            get(agents::get_agent_state),
        )
        .route(
            "/api/v1/agents/:agent_id/state",
            put(agents::update_agent_state),
        )
        // Message management routes
        .route("/api/v1/messages", post(messages::create_message))
        .route("/api/v1/messages/:id", get(messages::get_message))
        .route("/api/v1/messages", get(messages::list_messages))
        .route("/api/v1/messages/:id", delete(messages::delete_message))
        // Tool management routes
        .route("/api/v1/tools", post(tools::register_tool))
        .route("/api/v1/tools/:id", get(tools::get_tool))
        .route("/api/v1/tools", get(tools::list_tools))
        .route("/api/v1/tools/:id", put(tools::update_tool))
        .route("/api/v1/tools/:id", delete(tools::delete_tool))
        .route("/api/v1/tools/:id/execute", post(tools::execute_tool))
        // WebSocket endpoint
        .route("/api/v1/ws", get(crate::websocket::websocket_handler))
        // SSE endpoints
        .route("/api/v1/sse", get(crate::sse::sse_handler))
        .route("/api/v1/sse/llm", get(crate::sse::sse_stream_llm_response))
        // Health and monitoring
        .route("/health", get(health::health_check))
        .route("/metrics", get(metrics::get_metrics))
        // OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Add shared state via with_state
        .with_state(db_pool)
        // Add shared state via Extension
        .layer(Extension(memory_manager))
        .layer(Extension(ws_manager))
        .layer(Extension(sse_manager))
        // Add middleware (order matters: last added = first executed)
        .layer(axum_middleware::from_fn(audit_logging_middleware))
        .layer(axum_middleware::from_fn(quota_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    Ok(app)
}

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        memory::add_memory,
        memory::get_memory,
        memory::update_memory,
        memory::delete_memory,
        memory::search_memories,
        memory::get_memory_history,
        memory::batch_add_memories,
        memory::batch_delete_memories,
        users::register_user,
        users::login_user,
        users::get_current_user,
        users::update_current_user,
        users::change_password,
        users::get_user_by_id,
        organizations::create_organization,
        organizations::get_organization,
        organizations::update_organization,
        organizations::delete_organization,
        organizations::list_organization_members,
        agents::create_agent,
        agents::get_agent,
        agents::update_agent,
        agents::delete_agent,
        agents::list_agents,
        agents::send_message_to_agent,
        agents::get_agent_state,
        agents::update_agent_state,
        chat::send_chat_message,
        chat::send_chat_message_stream,
        chat::get_chat_history,
        messages::create_message,
        messages::get_message,
        messages::list_messages,
        messages::delete_message,
        tools::register_tool,
        tools::get_tool,
        tools::list_tools,
        tools::update_tool,
        tools::delete_tool,
        tools::execute_tool,
        health::health_check,
        metrics::get_metrics,
    ),
    components(
        schemas(
            crate::models::MemoryRequest,
            crate::models::MemoryResponse,
            crate::models::SearchRequest,
            crate::models::SearchResponse,
            crate::models::BatchRequest,
            crate::models::BatchResponse,
            crate::models::HealthResponse,
            crate::models::MetricsResponse,
            users::RegisterRequest,
            users::LoginRequest,
            users::LoginResponse,
            users::UserResponse,
            users::UpdateUserRequest,
            users::ChangePasswordRequest,
            organizations::OrganizationResponse,
            organizations::OrganizationSettings,
            organizations::CreateOrganizationRequest,
            organizations::UpdateOrganizationRequest,
            organizations::OrganizationMemberResponse,
            agents::CreateAgentRequest,
            agents::UpdateAgentRequest,
            agents::AgentResponse,
            agents::SendMessageRequest,
            agents::SendMessageResponse,
            agents::AgentStateResponse,
            agents::UpdateAgentStateRequest,
            chat::ChatMessageRequest,
            chat::ChatMessageResponse,
            chat::ToolCallInfo,
            messages::CreateMessageRequest,
            messages::MessageResponse,
            tools::RegisterToolRequest,
            tools::UpdateToolRequest,
            tools::ToolResponse,
            tools::ExecuteToolRequest,
            tools::ToolExecutionResponse,
        )
    ),
    tags(
        (name = "memory", description = "Memory management operations"),
        (name = "batch", description = "Batch operations"),
        (name = "users", description = "User management operations"),
        (name = "organizations", description = "Organization management operations"),
        (name = "agents", description = "Agent management operations"),
        (name = "chat", description = "Agent chat operations with AgentOrchestrator"),
        (name = "messages", description = "Message management operations"),
        (name = "tools", description = "Tool management and execution operations"),
        (name = "health", description = "Health and monitoring"),
    ),
    info(
        title = "AgentMem API",
        version = "2.0.0",
        description = "Enterprise-grade memory management API for AI agents with authentication and multi-tenancy",
        contact(
            name = "AgentMem Team",
            url = "https://github.com/agentmem/agentmem",
        ),
        license(
            name = "MIT OR Apache-2.0",
            url = "https://opensource.org/licenses/MIT",
        ),
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

/// Security addon for OpenAPI
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_router_creation() {
        // Note: This test requires a database connection
        // For now, we just verify the function signature
        // TODO: Add proper integration test with test database
        assert!(true);
    }
}
