//! HTTP routes for the AgentMem API

// All routes now use Repository Traits and work with both LibSQL and PostgreSQL
pub mod agents;
pub mod chat;
pub mod chat_lumosai;  // LumosAI集成
pub mod docs;
// Graph routes require PostgreSQL-specific managers (temporarily disabled for LibSQL)
#[cfg(feature = "postgres")]
pub mod graph;
pub mod health;
pub mod mcp;
pub mod memory; // ✅ 统一API实现：基于agent-mem Memory API
pub mod messages;
pub mod metrics;
pub mod organizations;
pub mod plugins; // 🆕 Plugin management API
pub mod stats;
pub mod tools;
pub mod users;
pub mod working_memory; // ✅ Working Memory API：基于 WorkingMemoryStore trait

use crate::error::{ServerError, ServerResult};
use crate::middleware::rbac::rbac_middleware;
use crate::middleware::{
    audit_logging_middleware, default_auth_middleware, metrics_middleware, quota_middleware,
};
use crate::rbac::RbacChecker;
use tracing::info;
// ✅ 使用memory::MemoryManager（基于agent-mem统一API）
use crate::routes::memory::MemoryManager;
use crate::sse::SseManager;
use crate::websocket::WebSocketManager;
use agent_mem_core::storage::factory::Repositories;
use agent_mem_observability::metrics::MetricsRegistry;
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Extension, Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create the main router with all routes
pub async fn create_router(
    memory_manager: Arc<MemoryManager>,
    metrics_registry: Arc<MetricsRegistry>,
    repositories: Repositories,
) -> ServerResult<Router<()>> {
    // Create WebSocket and SSE managers
    let ws_manager = Arc::new(WebSocketManager::new());
    let sse_manager = Arc::new(SseManager::new());

    // Create RBAC checker for role-based access control
    let rbac_checker = Arc::new(RbacChecker);

    // 🆕 Initialize MCP Server with ToolExecutor
    use agent_mem_tools::executor::ToolExecutor;
    use agent_mem_tools::mcp::server::McpServerConfig;
    use agent_mem_tools::mcp::McpServer;

    let tool_executor = Arc::new(ToolExecutor::new());
    let mcp_config = McpServerConfig::default();
    let mcp_server = Arc::new(McpServer::new(mcp_config, tool_executor));

    // Initialize MCP server
    mcp_server
        .initialize()
        .await
        .map_err(|e| ServerError::ServerError(format!("Failed to initialize MCP server: {e}")))?;

    info!("MCP server initialized successfully");

    let mut app = Router::new()
        // Memory management routes (✅ 使用Memory统一API)
        // 🆕 Fix 1: 添加GET方法支持全局列表查询
        .route(
            "/api/v1/memories",
            get(memory::list_all_memories).post(memory::add_memory),
        )
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
        // Health and monitoring
        .route("/health", get(health::health_check))
        .route("/health/live", get(health::liveness_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/metrics", get(metrics::get_metrics))
        .route("/metrics/prometheus", get(metrics::get_prometheus_metrics))
        // Dashboard statistics
        .route("/api/v1/stats/dashboard", get(stats::get_dashboard_stats))
        .route(
            "/api/v1/stats/memories/growth",
            get(stats::get_memory_growth),
        )
        .route(
            "/api/v1/stats/agents/activity",
            get(stats::get_agent_activity_stats),
        )
        .route(
            "/api/v1/stats/memory/quality",
            get(stats::get_memory_quality_stats),
        );

    // Add all routes (now database-agnostic via Repository Traits)
    app = app
        // User management routes
        .route("/api/v1/users", get(users::get_users_list))
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
        // ===== Chat routes (v1 - 推荐使用) =====
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
        // ===== ✅ Task 1.5: 兼容路由（向后兼容，解决404错误）=====
        .route(
            "/api/agents/:agent_id/chat",
            post(chat::send_chat_message),
        )
        .route(
            "/api/agents/:agent_id/chat/stream",
            post(chat::send_chat_message_stream),
        )
        .route(
            "/api/agents/:agent_id/chat/history",
            get(chat::get_chat_history),
        )
        // ===== LumosAI集成路由 (experimental) =====
        // 注意：更具体的路径必须在前面，避免被通用路径匹配
        .route(
            "/api/v1/agents/:agent_id/chat/lumosai/stream",
            post(chat_lumosai::send_chat_message_lumosai_stream),
        )
        .route(
            "/api/v1/agents/:agent_id/chat/lumosai",
            post(chat_lumosai::send_chat_message_lumosai),
        )
        // ===== ✅ Task 1.5: LumosAI 兼容路由 =====
        .route(
            "/api/agents/:agent_id/chat/lumosai/stream",
            post(chat_lumosai::send_chat_message_lumosai_stream),
        )
        .route(
            "/api/agents/:agent_id/chat/lumosai",
            post(chat_lumosai::send_chat_message_lumosai),
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
        // Agent memories route
        .route(
            "/api/v1/agents/:agent_id/memories",
            get(memory::get_agent_memories),
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
        // MCP server routes
        .route("/api/v1/mcp/info", get(mcp::get_server_info))
        .route("/api/v1/mcp/tools", get(mcp::list_tools))
        .route("/api/v1/mcp/tools/call", post(mcp::call_tool))
        .route("/api/v1/mcp/tools/:tool_name", get(mcp::get_tool))
        .route("/api/v1/mcp/health", get(mcp::health_check))
        // Working Memory routes (session-based temporary context)
        .route(
            "/api/v1/working-memory",
            post(working_memory::add_working_memory),
        )
        .route(
            "/api/v1/working-memory",
            get(working_memory::get_working_memory),
        )
        .route(
            "/api/v1/working-memory/:item_id",
            delete(working_memory::delete_working_memory_item),
        )
        .route(
            "/api/v1/working-memory/sessions/:session_id",
            delete(working_memory::clear_working_memory),
        )
        .route(
            "/api/v1/working-memory/cleanup",
            post(working_memory::cleanup_expired),
        )
        // 🆕 Plugin management routes
        .route("/api/v1/plugins", get(plugins::list_plugins))
        .route("/api/v1/plugins", post(plugins::register_plugin))
        .route("/api/v1/plugins/:id", get(plugins::get_plugin));

    // Graph visualization routes (PostgreSQL only)
    #[cfg(feature = "postgres")]
    let app = {
        app.route("/api/v1/graph/data", get(graph::get_graph_data))
            .route(
                "/api/v1/graph/associations",
                post(graph::create_association),
            )
            .route(
                "/api/v1/graph/memories/:memory_id/associations",
                get(graph::get_memory_associations),
            )
            .route("/api/v1/graph/stats", get(graph::get_graph_stats))
    };

    let app = app
        // WebSocket endpoint
        .route("/api/v1/ws", get(crate::websocket::websocket_handler))
        // SSE endpoints
        .route("/api/v1/sse", get(crate::sse::sse_handler))
        .route("/api/v1/sse/llm", get(crate::sse::sse_stream_llm_response))
        // Add OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Add state for plugin routes
        .with_state(memory_manager.clone());

    // Add middleware and shared state (order matters: last added = first executed)
    let app = app
        // Add middleware (these middleware layers execute BEFORE the Extension layers below)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(axum_middleware::from_fn(quota_middleware))
        .layer(axum_middleware::from_fn(audit_logging_middleware))
        .layer(axum_middleware::from_fn(rbac_middleware)) // ✅ RBAC权限检查
        .layer(axum_middleware::from_fn(metrics_middleware))
        // Add default auth middleware (injects default AuthUser when auth is disabled)
        .layer(axum_middleware::from_fn(default_auth_middleware))
        // Add shared state via Extension (must be after middleware that uses them)
        .layer(Extension(rbac_checker)) // ✅ RBAC检查器
        .layer(Extension(sse_manager))
        .layer(Extension(ws_manager))
        .layer(Extension(mcp_server)) // 🆕 Add MCP server extension
        .layer(Extension(metrics_registry))
        .layer(Extension(memory_manager))
        .layer(Extension(Arc::new(repositories)));

    Ok(app)
}

/// OpenAPI documentation structure (all routes - database-agnostic)
#[derive(OpenApi)]
#[openapi(
    paths(
        memory::add_memory,
        memory::list_all_memories,
        memory::get_memory,
        memory::update_memory,
        memory::delete_memory,
        memory::search_memories,
        memory::get_memory_history,
        memory::batch_add_memories,
        memory::batch_delete_memories,
        memory::get_agent_memories,
        users::register_user,
        users::login_user,
        users::get_current_user,
        users::update_current_user,
        users::change_password,
        users::get_user_by_id,
        users::get_users_list,
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
        mcp::get_server_info,
        mcp::list_tools,
        mcp::call_tool,
        mcp::get_tool,
        mcp::health_check,
        working_memory::add_working_memory,
        working_memory::get_working_memory,
        working_memory::delete_working_memory_item,
        working_memory::clear_working_memory,
        working_memory::cleanup_expired,
        // Note: graph routes are only available with postgres feature
        health::health_check,
        health::liveness_check,
        health::readiness_check,
        metrics::get_metrics,
        metrics::get_prometheus_metrics,
        stats::get_dashboard_stats,
        stats::get_memory_growth,
        stats::get_agent_activity_stats,
        stats::get_memory_quality_stats,
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
            crate::models::ComponentStatus,
            crate::models::MetricsResponse,
            stats::DashboardStats,
            stats::ActivityLog,
            stats::MemoryGrowthPoint,
            stats::MemoryGrowthResponse,
            stats::AgentActivityStats,
            stats::AgentActivityResponse,
            users::RegisterRequest,
            users::LoginRequest,
            users::LoginResponse,
            users::UserResponse,
            users::UpdateUserRequest,
            users::ChangePasswordRequest,
            users::UsersListResponse,
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
            working_memory::AddWorkingMemoryRequest,
            working_memory::AddWorkingMemoryResponse,
            working_memory::ClearWorkingMemoryResponse,
            working_memory::CleanupResponse,
            // Note: graph schemas are only available with postgres feature
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
        (name = "mcp", description = "MCP (Model Context Protocol) server operations"),
        (name = "working-memory", description = "Working Memory operations for session-based temporary context"),
        (name = "graph", description = "Knowledge graph visualization and querying operations"),
        (name = "health", description = "Health and monitoring"),
        (name = "statistics", description = "Dashboard statistics and analytics"),
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
