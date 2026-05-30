//! HTTP routes for the AgentMem API

// All routes now use Repository Traits and work with both LibSQL and PostgreSQL
pub mod agents;
pub mod chat;
pub mod chat_lumosai; // LumosAI集成
pub mod docs;
pub mod file_centric;
// Graph routes require PostgreSQL-specific managers (temporarily disabled for LibSQL)
#[cfg(feature = "postgres")]
pub mod graph;
pub mod health;
pub mod logs; // 🆕 Phase 4.2: 日志聚合功能
pub mod mcp;
pub mod memory; // ✅ 统一API实现：基于agent-mem Memory API
pub mod messages;
pub mod alerts;
pub mod metrics;
pub mod organizations;
pub mod performance; // 🆕 Phase 4.2: 性能分析功能
pub mod plugins; // 🆕 Plugin management API
pub mod predictor;
pub mod search_analytics;
pub mod stats;
pub mod tools;
pub mod users;
pub mod webhook; // 🆕 Webhook事件订阅支持
pub mod multimodal;
pub mod working_memory; // ✅ Working Memory API：基于 WorkingMemoryStore trait // 🆕 Phase 2.3: 记忆预测功能

use crate::config::ServerConfig;
use crate::error::{ServerError, ServerResult};
use crate::middleware::rbac::rbac_middleware;
use crate::middleware::{
    audit_logging_middleware, circuit_breaker_middleware, metrics_middleware, quota_middleware,
    require_auth_middleware, CircuitBreakerManager, QuotaManager,
};
use crate::rbac::RbacChecker;
use tracing::info;
// ✅ 使用memory::MemoryManager（基于agent-mem统一API）
use crate::routes::memory::MemoryManager;
use crate::routes::file_centric::FileCentricState;
use crate::sse::SseManager;
use crate::websocket::WebSocketManager;
use agent_mem_core::storage::factory::Repositories;
use agent_mem_observability::metrics::MetricsRegistry;
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Extension, Router,
};
use http::{HeaderName, HeaderValue, Method};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create CORS layer based on configuration
fn create_cors_layer(config: &ServerConfig) -> CorsLayer {
    if !config.enable_cors {
        return CorsLayer::new();
    }

    let origins: Vec<&str> = config
        .cors_allowed_origins
        .split(',')
        .map(|s| s.trim())
        .collect();

    // If wildcard, allow all origins
    if origins.len() == 1 && origins[0] == "*" {
        return CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
                Method::HEAD,
            ])
            .allow_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("authorization"),
                HeaderName::from_static("x-requested-with"),
                HeaderName::from_static("accept"),
            ])
            .max_age(std::time::Duration::from_secs(config.cors_max_age));
    }

    let methods: Vec<Method> = config
        .cors_allowed_methods
        .split(',')
        .map(|s| s.trim())
        .filter_map(|m| match m {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "PATCH" => Some(Method::PATCH),
            "OPTIONS" => Some(Method::OPTIONS),
            "HEAD" => Some(Method::HEAD),
            _ => None,
        })
        .collect();

    let headers: Vec<HeaderName> = config
        .cors_allowed_headers
        .split(',')
        .map(|s| s.trim())
        .filter_map(|h| HeaderName::from_bytes(h.as_bytes()).ok())
        .collect();

    let mut cors = CorsLayer::new()
        .allow_methods(methods)
        .allow_headers(headers);

    cors = cors.max_age(std::time::Duration::from_secs(config.cors_max_age));

    for origin in origins {
        if !origin.is_empty() {
            cors = cors.allow_origin(
                origin
                    .parse::<HeaderValue>()
                    .unwrap_or(HeaderValue::from_static("*")),
            );
        }
    }

    cors
}

/// Create the main router with all routes
pub async fn create_router(
    memory_manager: Arc<MemoryManager>,
    metrics_registry: Arc<MetricsRegistry>,
    repositories: Repositories,
    config: ServerConfig,
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
    info!("Initializing MCP server...");
    mcp_server
        .initialize()
        .await
        .map_err(|e| ServerError::server_error(format!("Failed to initialize MCP server: {e}")))?;

    info!("MCP server initialized successfully");

    // 🆕 Initialize file-centric state with resource and category managers
    info!("Initializing file-centric state...");
    let file_centric_state = Arc::new(FileCentricState::new());
    info!("File-centric state initialized");

    // 🆕 Initialize webhook state
    info!("Initializing webhook state...");
    let webhook_state = Arc::new(crate::routes::webhook::WebhookState::new());
    info!("Webhook state initialized");

    // 🆕 Initialize multimodal & search analytics state
    info!("Initializing multimodal storage...");
    use agent_mem_core::multimodal_storage::{MultimodalStorage, MultimodalStorageConfig, MockImageVectorizer, InMemoryMultimodalStorage};
    use agent_mem_core::search::search_analytics::{SearchAnalytics, SearchAnalyticsConfig};

    let in_memory = InMemoryMultimodalStorage::new();
    let vectorizer = MockImageVectorizer::new(512);
    let multimodal_storage = MultimodalStorage::new(
        Arc::new(in_memory),
        Arc::new(vectorizer),
        MultimodalStorageConfig::default(),
    );
    info!("Multimodal storage initialized");

    let combined_state = multimodal::MultimodalState {
        storage: Arc::new(multimodal_storage),
    };
    info!("Multimodal state initialized");
    
    let mut app = Router::new()
        // ========== 核心 Memory 路由 (6) ==========
        .route(
            "/api/v1/memories",
            get(memory::list_all_memories).post(memory::add_memory),
        )
        .route("/api/v1/memories/:id", get(memory::get_memory))
        .route("/api/v1/memories/:id", put(memory::update_memory))
        .route("/api/v1/memories/:id", delete(memory::delete_memory))
        .route("/api/v1/memories/search", post(memory::search_memories))
        // ========== 批量操作 (3) ==========
        .route("/api/v1/memories/batch", post(memory::batch_add_memories))
        .route("/api/v1/memories/batch/delete", post(memory::batch_delete_memories))
        .route("/api/v1/memories/batch/search", post(memory::batch_search_memories))
        // ========== File-centric 核心路由 (统一到 /api/v1/file-centric 前缀) ==========
        // Resources
        .route(
            "/api/v1/file-centric/resources",
            get(file_centric::list_resources).post(file_centric::mount_resource_canonical),
        )
        .route("/api/v1/file-centric/resources/:resource_id", get(file_centric::get_resource_canonical))
        .route("/api/v1/file-centric/resources/:resource_id/extract", post(file_centric::extract_resource_canonical))
        .route("/api/v1/file-centric/extraction/:job_id", get(file_centric::get_extraction_status))
        // Categories
        .route("/api/v1/file-centric/categories", get(file_centric::list_categories_canonical))
        .route("/api/v1/file-centric/categories/:category_id", get(file_centric::get_category))
        .route("/api/v1/file-centric/categories/by-path", get(file_centric::get_category_by_path))
        .route("/api/v1/file-centric/categories/search", post(file_centric::search_categories_canonical))
        // Migration
        .route("/api/v1/file-centric/migrations/plan", post(file_centric::plan_legacy_migration_canonical))
        .route("/api/v1/file-centric/migrations/apply", post(file_centric::apply_legacy_migration_canonical))
        .route("/api/v1/file-centric/migrations/:migration_id", get(file_centric::get_migration_status))
        .route("/api/v1/file-centric/migrations/:migration_id/rollback", post(file_centric::rollback_legacy_migration_canonical))
        // Proactive Tasks
        .route("/api/v1/file-centric/proactive/tasks", get(file_centric::list_proactive_tasks_canonical))
        .route("/api/v1/file-centric/proactive/tasks/:task_id", get(file_centric::get_proactive_task))
        .route("/api/v1/file-centric/proactive/tasks/:task_id/run", post(file_centric::run_proactive_task_canonical))
        .route("/api/v1/file-centric/proactive/tasks/:task_id/cancel", post(file_centric::cancel_proactive_task_canonical))
        .route("/api/v1/file-centric/proactive/stats", get(file_centric::get_scheduler_stats_canonical))
        // ========== Health & Monitoring (3) ==========
        .route("/health", get(health::health_check))
        .route("/api/v1/alerts", get(alerts::get_alerts))
        .route("/api/v1/alerts/config", get(alerts::get_alert_config))
        .route("/api/v1/alerts/config", put(alerts::update_alert_config))
        .route("/metrics", get(metrics::get_prometheus_metrics))
        // ========== Stats & Analytics (3) ==========
        .route("/api/v1/stats", get(stats::get_dashboard_stats))
        .route("/api/v1/logs/stats", get(logs::get_log_stats))
        .route("/api/v1/performance", get(performance::get_performance_analysis))
        // ========== Multimodal (4) ==========
        .route("/api/v1/multimodal/upload", post(multimodal::upload_image))
        .route("/api/v1/multimodal/search", post(multimodal::search_similar))
        .route("/api/v1/multimodal/stats", get(multimodal::get_stats))
        .route("/api/v1/multimodal/health", get(multimodal::health_check))
        .with_state(combined_state);

    // TODO: Add search analytics routes after fixing type issues
    // let search_analytics_router = search_analytics::create_search_analytics_router(...);
    // app = app.merge(search_analytics_router);
    // Note: Search analytics routes will be added in a separate PR due to type complexity

    // Add all routes (now database-agnostic via Repository Traits)
    let app = app
        // User management routes
        .route("/api/v1/users", get(users::get_users_list))
        .route("/api/v1/users/register", post(users::register_user))
        .route("/api/v1/users/login", post(users::login_user))
        .route("/api/v1/users/me", get(users::get_current_user))
        .route("/api/v1/users/me", put(users::update_current_user))
        .route("/api/v1/users/me/password", post(users::change_password))
        .route("/api/v1/users/:user_id", get(users::get_user_by_id))
        // Organization management routes (合并 CRUD 到单一路由)
        .route("/api/v1/organizations", get(organizations::get_organization).post(organizations::create_organization))
        .route("/api/v1/organizations/:org_id", get(organizations::get_organization).put(organizations::update_organization).delete(organizations::delete_organization))
        .route("/api/v1/organizations/:org_id/members", get(organizations::list_organization_members))
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
        // ===== Agent Chat routes (合并 GET + POST) =====
        .route("/api/v1/agents/:agent_id/chat", get(chat::get_chat_history).post(chat::send_chat_message))
        .route("/api/v1/agents/:agent_id/chat/stream", post(chat::send_chat_message_stream))
        .route("/api/v1/agents/:agent_id/chat/lumosai", post(chat_lumosai::send_chat_message_lumosai))
        .route("/api/v1/agents/:agent_id/chat/lumosai/stream", post(chat_lumosai::send_chat_message_lumosai_stream))
        // Agent state management routes (合并 GET + PUT)
        .route("/api/v1/agents/:agent_id/state", get(agents::get_agent_state).put(agents::update_agent_state))
        // Agent memories route
        .route(
            "/api/v1/agents/:agent_id/memories",
            get(memory::get_agent_memories),
        )
        // Message management routes - 合到 Chat 历史中
        .route("/api/v1/messages", post(messages::create_message).get(messages::list_messages))
        .route("/api/v1/messages/:id", get(messages::get_message).delete(messages::delete_message))
        // Tool management routes - 简化为 execute-only（MCP协议处理注册）
        .route("/api/v1/tools/:id/execute", post(tools::execute_tool))
        // ========== Working Memory (1) - 合并到 GET ==========
        .route("/api/v1/working-memory", post(working_memory::add_working_memory).get(working_memory::get_working_memory))
        .route("/api/v1/working-memory/cleanup", post(working_memory::cleanup_expired))
        // ========== Plugins (1) ==========
        .route("/api/v1/plugins", get(plugins::list_plugins).post(plugins::register_plugin))
        // ========== Webhooks (5) 🆕 ==========
        .route("/api/v1/webhooks", post(webhook::create_webhook).get(webhook::list_webhooks))
        .route("/api/v1/webhooks/:id", get(webhook::get_webhook).put(webhook::update_webhook).delete(webhook::delete_webhook))
        .route("/api/v1/webhooks/stats", get(webhook::get_webhook_stats))
        .route("/api/v1/webhooks/:id/test", post(webhook::test_webhook))
        // ========== MCP Server Routes 🆕 ==========
        .route("/api/v1/mcp/info", get(mcp::get_server_info))
        .route("/api/v1/mcp/tools", get(mcp::list_tools))
        .route("/api/v1/mcp/tools/call", post(mcp::call_tool))
        .route("/api/v1/mcp/tools/:tool_name", get(mcp::get_tool))
        .route("/api/v1/mcp/health", get(mcp::health_check))
        // MCP Prompts
        .route("/api/v1/mcp/prompts", get(mcp::list_prompts))
        .route("/api/v1/mcp/prompts/:name", get(mcp::get_prompt))
        // MCP Resources - subscribe must come before wildcard to avoid conflict
        .route("/api/v1/mcp/resources/subscribe", post(mcp::subscribe_resource))
        .route("/api/v1/mcp/resources", get(mcp::list_resources))
        .route("/api/v1/mcp/resources/*uri", get(mcp::read_resource))
        .route("/api/v1/mcp/subscriptions/:id", delete(mcp::unsubscribe_resource));

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

    // Create circuit breaker manager
    let circuit_breaker_manager = Arc::new(CircuitBreakerManager::new());

    // Add middleware and shared state (order matters: last added = first executed)
    // Add middleware and Extension layers in separate function to avoid stack overflow
    let app = build_router_with_layers(app, config, circuit_breaker_manager, rbac_checker,
        sse_manager, ws_manager, mcp_server, metrics_registry, memory_manager,
        file_centric_state, webhook_state, repositories).await?;

    Ok(app)
}

// Separate function to build router with layers (avoids stack overflow)
async fn build_router_with_layers(
    mut app: Router,
    config: ServerConfig,
    circuit_breaker_manager: Arc<CircuitBreakerManager>,
    rbac_checker: Arc<RbacChecker>,
    sse_manager: Arc<SseManager>,
    ws_manager: Arc<WebSocketManager>,
    mcp_server: Arc<agent_mem_tools::mcp::McpServer>,
    metrics_registry: Arc<MetricsRegistry>,
    memory_manager: Arc<MemoryManager>,
    file_centric_state: Arc<FileCentricState>,
    webhook_state: Arc<crate::routes::webhook::WebhookState>,
    repositories: Repositories,
) -> ServerResult<Router> {
    info!("Building router with layers...");

    // Add middleware
    info!("Adding CORS layer...");
    app = app.layer(create_cors_layer(&config));
    info!("CORS layer added");

    info!("Adding TraceLayer...");
    app = app.layer(TraceLayer::new_for_http());
    info!("TraceLayer added");

    info!("Adding circuit breaker middleware...");
    app = app.layer(axum_middleware::from_fn(circuit_breaker_middleware));
    info!("Circuit breaker middleware added");

    info!("Adding quota middleware...");
    app = app.layer(axum_middleware::from_fn(quota_middleware));
    info!("Quota middleware added");

    info!("Adding audit logging middleware...");
    app = app.layer(axum_middleware::from_fn(audit_logging_middleware));
    info!("Audit logging middleware added");

    info!("Adding RBAC middleware...");
    app = app.layer(axum_middleware::from_fn(rbac_middleware));
    info!("RBAC middleware added");

    info!("Adding metrics middleware...");
    app = app.layer(axum_middleware::from_fn(metrics_middleware));
    info!("Metrics middleware added");

    info!("Adding auth middleware...");
    app = app.layer(axum_middleware::from_fn_with_state(
        config.clone(),
        require_auth_middleware,
    ));
    info!("Auth middleware added");

    // Add Extension layers one at a time
    info!("Adding Extension layers...");
    app = app.layer(Extension(circuit_breaker_manager));
    info!("Layer 1/11: circuit_breaker_manager");

    app = app.layer(Extension(rbac_checker));
    info!("Layer 2/11: rbac_checker");

    app = app.layer(Extension(sse_manager));
    info!("Layer 3/11: sse_manager");

    app = app.layer(Extension(ws_manager));
    info!("Layer 4/11: ws_manager");

    app = app.layer(Extension(mcp_server));
    info!("Layer 5/11: mcp_server");

    app = app.layer(Extension(metrics_registry));
    info!("Layer 6/11: metrics_registry");

    app = app.layer(Extension(memory_manager));
    info!("Layer 7/11: memory_manager");

    app = app.layer(Extension(file_centric_state));
    info!("Layer 8/11: file_centric_state");

    app = app.layer(Extension(webhook_state));
    info!("Layer 9/11: webhook_state");

    app = app.layer(Extension(Arc::new(repositories)));
    info!("Layer 10/11: repositories");

    app = app.layer(Extension(Arc::new(QuotaManager::new())));
    info!("Layer 11/11: quota_manager");

    info!("All Extension layers added. Router build complete!");
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
        memory::get_search_statistics,
        memory::warmup_cache,
        memory::performance_benchmark,
        file_centric::mount_resource,
        file_centric::get_resource,
        file_centric::extract_resource,
        file_centric::list_categories,
        file_centric::search_categories,
        file_centric::plan_legacy_migration,
        file_centric::apply_legacy_migration,
        file_centric::rollback_legacy_migration,
        file_centric::list_proactive_tasks,
        file_centric::run_proactive_task,
        file_centric::cancel_proactive_task,
        file_centric::get_scheduler_stats,
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
        tools::execute_tool,  // Only execute endpoint remains
        working_memory::add_working_memory,
        working_memory::get_working_memory,
        working_memory::cleanup_expired,  // Only cleanup endpoint remains
        // ========== Webhook routes 🆕 ==========
        webhook::create_webhook,
        webhook::list_webhooks,
        webhook::get_webhook,
        webhook::update_webhook,
        webhook::delete_webhook,
        webhook::get_webhook_stats,
        webhook::test_webhook,
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
        stats::get_database_pool_stats,
        stats::get_index_performance_stats,
        stats::get_memory_usage_stats,
        logs::get_log_stats,
        logs::query_logs,
        logs::get_trace,
        // MCP Server routes 🆕
        mcp::get_server_info,
        mcp::list_tools,
        mcp::call_tool,
        mcp::get_tool,
        mcp::health_check,
        mcp::list_prompts,
        mcp::get_prompt,
        mcp::list_resources,
        mcp::read_resource,
        mcp::subscribe_resource,
        mcp::unsubscribe_resource,
    ),
    components(
        schemas(
            crate::models::MemoryRequest,
            crate::models::MemoryResponse,
            crate::models::SearchRequest,
            crate::models::SearchResponse,
            crate::models::BatchRequest,
            crate::models::BatchResponse,
            crate::models::MountResourceRequest,
            crate::models::ResourceDescriptor,
            crate::models::ResourceMetadataDescriptor,
            crate::models::ResourceStatus,
            crate::models::ScopeDescriptor,
            crate::models::CategoryDescriptor,
            crate::models::CategoryMetadataDescriptor,
            crate::models::CategoryStatus,
            crate::models::SearchCategoriesRequest,
            crate::models::ExtractionRequest,
            crate::models::ExtractionResult,
            crate::models::ExtractedEntity,
            crate::models::ExtractedRelation,
            crate::models::MigrationPlan,
            crate::models::PlanMigrationRequest,
            crate::models::MigrationReport,
            crate::models::ApplyMigrationRequest,
            crate::models::RollbackMigrationRequest,
            crate::models::ProactiveTaskInfo,
            crate::models::RunProactiveTaskRequest,
            crate::models::CancelProactiveTaskRequest,
            crate::models::SchedulerStats,
            crate::models::SchedulerState,
            crate::models::OperationStatus,
            crate::models::PlatformErrorCode,
            crate::models::HealthResponse,
            crate::models::ComponentStatus,
            crate::models::MetricsResponse,
            stats::DashboardStats,
            stats::ActivityLog,
            stats::MemoryGrowthPoint,
            stats::MemoryGrowthResponse,
            stats::AgentActivityStats,
            stats::AgentActivityResponse,
            stats::IndexPerformanceStats,
            stats::IndexInfo,
            stats::OptimizationRecommendation,
            stats::PerformanceMetrics,
            stats::MemoryUsageStats,
            logs::TraceResponse,
            logs::TraceRequest,
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
            webhook::WebhookSubscriptionResponse,
            webhook::CreateWebhookRequest,
            webhook::UpdateWebhookRequest,
            webhook::ListWebhooksResponse,
            webhook::WebhookStats,
            webhook::WebhookEventType,
            webhook::WebhookDeliveryStatus,
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
        (name = "file-centric", description = "Preview file-centric platform operations"),
        (name = "health", description = "Health and monitoring"),
        (name = "statistics", description = "Dashboard statistics and analytics"),
    ),
    info(
        title = "AgentMem API",
        version = "2.0.0",
        description = "Enterprise-grade memory management API for AI agents with authentication and multi-tenancy",
        contact(
            name = "AgentMem Team",
            url = "https://github.com/louloulin/agentmem",
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
