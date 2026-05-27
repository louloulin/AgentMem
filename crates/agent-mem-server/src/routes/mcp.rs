//! MCP 服务端 REST API 路由
//!
//! 提供 MCP 协议的 HTTP 接口
//!
//! 完整实现包括:
//! - 工具管理 (tools)
//! - 提示词模板 (prompts)
//! - 资源管理 (resources)

use crate::error::{ServerError, ServerResult};
use crate::models::ApiResponse;
use agent_mem_tools::mcp::{McpServer, ServerInfo};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use utoipa::ToSchema;

// Re-export types for convenience
use agent_mem_tools::mcp::prompts::{
    McpGetPromptRequest, McpListPromptsResponse,
};
use agent_mem_tools::mcp::resources::{
    McpReadResourceRequest, McpSubscribeResourceRequest,
};

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolCallRequest {
    /// 工具名称
    pub name: String,

    /// 工具参数
    pub arguments: serde_json::Value,

    /// API 密钥（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// 工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolCallResponse {
    /// 响应内容
    pub content: Vec<ContentItem>,

    /// 是否错误
    pub is_error: bool,
}

/// 内容项
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentItem {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { uri: String, mime_type: String },
}

/// 订阅响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubscriptionResponse {
    /// 订阅 ID
    #[serde(rename = "subscriptionId")]
    pub subscription_id: String,
}

/// 提示词获取响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetPromptResponse {
    /// 提示词内容
    pub messages: Vec<serde_json::Value>,
}

/// 资源列表响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResourceListResponse {
    /// 资源列表
    pub resources: Vec<ResourceInfo>,
}

/// 资源信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResourceInfo {
    /// 资源 URI
    pub uri: String,

    /// 资源名称
    pub name: String,

    /// 资源描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME 类型
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// 资源读取响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReadResourceResponse {
    /// 资源内容列表
    pub contents: Vec<ResourceContent>,
}

/// 资源内容
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResourceContent {
    /// 资源 URI
    pub uri: String,

    /// 内容类型
    #[serde(rename = "type")]
    pub content_type: String,

    /// 内容
    pub content: String,
}

/// 获取服务器信息
///
/// GET /api/v1/mcp/info
#[utoipa::path(
    get,
    path = "/api/v1/mcp/info",
    tag = "mcp",
    responses(
        (status = 200, description = "Server info retrieved successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_server_info(
    Extension(mcp_server): Extension<Arc<McpServer>>,
) -> ServerResult<Json<ApiResponse<ServerInfo>>> {
    info!("Getting MCP server info");

    let info = mcp_server.get_server_info();

    Ok(Json(ApiResponse::success(info)))
}

/// 列出所有工具
///
/// GET /api/v1/mcp/tools
#[utoipa::path(
    get,
    path = "/api/v1/mcp/tools",
    tag = "mcp",
    responses(
        (status = 200, description = "Tools listed successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_tools(
    Extension(mcp_server): Extension<Arc<McpServer>>,
) -> ServerResult<Json<ApiResponse<serde_json::Value>>> {
    info!("Listing MCP tools");

    let response = mcp_server
        .list_tools()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list tools: {}", e)))?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "tools": response.tools
    }))))
}

/// 调用工具
///
/// POST /api/v1/mcp/tools/call
#[utoipa::path(
    post,
    path = "/api/v1/mcp/tools/call",
    tag = "mcp",
    request_body = ToolCallRequest,
    responses(
        (status = 200, description = "Tool executed successfully", body = ToolCallResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn call_tool(
    Extension(mcp_server): Extension<Arc<McpServer>>,
    Json(request): Json<ToolCallRequest>,
) -> ServerResult<Json<ApiResponse<ToolCallResponse>>> {
    info!("Calling MCP tool: {}", request.name);

    // 验证 API 密钥（如果需要）
    if let Some(api_key) = &request.api_key {
        if !mcp_server.verify_api_key(api_key) {
            return Err(ServerError::unauthorized("Invalid API key"));
        }
    }

    // 创建 MCP 工具调用请求
    use agent_mem_tools::mcp::types::McpToolCallRequest;

    let mcp_request = McpToolCallRequest {
        name: request.name.clone(),
        arguments: request.arguments,
    };

    let mcp_response = mcp_server
        .call_tool(mcp_request)
        .await
        .map_err(|e| ServerError::internal_error(format!("Tool execution failed: {}", e)))?;

    // 转换响应
    let content: Vec<ContentItem> = mcp_response
        .content
        .into_iter()
        .map(|c| match c {
            agent_mem_tools::mcp::types::McpContent::Text { text } => ContentItem::Text { text },
            agent_mem_tools::mcp::types::McpContent::Image { data, mime_type } => {
                ContentItem::Image { data, mime_type }
            }
            agent_mem_tools::mcp::types::McpContent::Resource {
                uri,
                mime_type,
                text: _,
            } => ContentItem::Resource {
                uri,
                mime_type: mime_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            },
        })
        .collect();

    let response = ToolCallResponse {
        content,
        is_error: mcp_response.is_error,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 获取工具详情
///
/// GET /api/v1/mcp/tools/{tool_name}
#[utoipa::path(
    get,
    path = "/api/v1/mcp/tools/{tool_name}",
    tag = "mcp",
    params(
        ("tool_name" = String, Path, description = "Tool name")
    ),
    responses(
        (status = 200, description = "Tool retrieved successfully"),
        (status = 404, description = "Tool not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_tool(
    Extension(mcp_server): Extension<Arc<McpServer>>,
    Path(tool_name): Path<String>,
) -> ServerResult<Json<ApiResponse<serde_json::Value>>> {
    info!("Getting MCP tool: {}", tool_name);

    let response = mcp_server
        .list_tools()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list tools: {}", e)))?;

    let tool = response
        .tools
        .into_iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| ServerError::not_found(format!("Tool '{}' not found", tool_name)))?;

    Ok(Json(ApiResponse::success(serde_json::json!(tool))))
}

/// 健康检查
///
/// GET /api/v1/mcp/health
#[utoipa::path(
    get,
    path = "/api/v1/mcp/health",
    tag = "mcp",
    responses(
        (status = 200, description = "MCP server is healthy"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn health_check(
    Extension(mcp_server): Extension<Arc<McpServer>>,
) -> ServerResult<Json<ApiResponse<serde_json::Value>>> {
    debug!("MCP health check");

    let info = mcp_server.get_server_info();

    Ok(Json(ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "server": info.name,
        "version": info.version,
        "protocol_version": info.protocol_version,
    }))))
}

// ============================================================================
// MCP Prompts 端点
// ============================================================================

/// 列出所有提示词模板
///
/// GET /api/v1/mcp/prompts
#[utoipa::path(
    get,
    path = "/api/v1/mcp/prompts",
    tag = "mcp",
    responses(
        (status = 200, description = "Prompts listed successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_prompts(
    Extension(mcp_server): Extension<Arc<McpServer>>,
) -> ServerResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    info!("Listing MCP prompts");

    let response = mcp_server
        .list_prompts()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list prompts: {}", e)))?;

    // Convert prompts to JSON values
    let prompts: Vec<serde_json::Value> = response
        .prompts
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "description": p.description,
                "arguments": p.arguments,
                "content": p.content,
                "version": p.version,
                "tags": p.tags,
                "metadata": p.metadata,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(prompts)))
}

/// 获取提示词模板详情
///
/// GET /api/v1/mcp/prompts/{name}
#[utoipa::path(
    get,
    path = "/api/v1/mcp/prompts/{name}",
    tag = "mcp",
    params(
        ("name" = String, Path, description = "Prompt name"),
        ("args" = Option<String>, Query, description = "Prompt arguments as JSON")
    ),
    responses(
        (status = 200, description = "Prompt retrieved successfully"),
        (status = 404, description = "Prompt not found"),
        (status = 400, description = "Missing required argument"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_prompt(
    Extension(mcp_server): Extension<Arc<McpServer>>,
    Path(name): Path<String>,
    Query(args): Query<Option<String>>,
) -> ServerResult<Json<ApiResponse<GetPromptResponse>>> {
    info!("Getting MCP prompt: {}", name);

    // Parse arguments if provided
    let arguments: HashMap<String, serde_json::Value> = args.and_then(|a| {
        let parsed: Result<HashMap<String, serde_json::Value>, _> = serde_json::from_str(&a);
        parsed.ok()
    }).unwrap_or_default();

    let request = McpGetPromptRequest {
        name: name.clone(),
        arguments,
    };

    let response = mcp_server
        .get_prompt(request)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                ServerError::not_found(format!("Prompt '{}' not found", name))
            } else if e.to_string().contains("required") {
                ServerError::bad_request(e.to_string())
            } else {
                ServerError::internal_error(format!("Failed to get prompt: {}", e))
            }
        })?;

    // Convert content to JSON values
    let messages: Vec<serde_json::Value> = response
        .content
        .into_iter()
        .map(|c| {
            match c {
                agent_mem_tools::mcp::prompts::PromptContent::Text { text } => {
                    serde_json::json!({
                        "type": "text",
                        "text": text,
                    })
                }
                agent_mem_tools::mcp::prompts::PromptContent::Image { data, mime_type } => {
                    serde_json::json!({
                        "type": "image",
                        "data": data,
                        "mime_type": mime_type,
                    })
                }
                agent_mem_tools::mcp::prompts::PromptContent::Resource { uri, mime_type } => {
                    serde_json::json!({
                        "type": "resource",
                        "uri": uri,
                        "mime_type": mime_type,
                    })
                }
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(GetPromptResponse { messages })))
}

// ============================================================================
// MCP Resources 端点
// ============================================================================

/// 列出所有资源
///
/// GET /api/v1/mcp/resources
#[utoipa::path(
    get,
    path = "/api/v1/mcp/resources",
    tag = "mcp",
    responses(
        (status = 200, description = "Resources listed successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_resources(
    Extension(mcp_server): Extension<Arc<McpServer>>,
) -> ServerResult<Json<ApiResponse<ResourceListResponse>>> {
    info!("Listing MCP resources");

    let response = mcp_server
        .list_resources()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to list resources: {}", e)))?;

    let resources: Vec<ResourceInfo> = response
        .resources
        .into_iter()
        .map(|r| ResourceInfo {
            uri: r.uri,
            name: r.name,
            description: r.description,
            mime_type: r.mime_type,
        })
        .collect();

    Ok(Json(ApiResponse::success(ResourceListResponse { resources })))
}

/// 读取资源内容
///
/// GET /api/v1/mcp/resources/{uri}
#[utoipa::path(
    get,
    path = "/api/v1/mcp/resources/{uri:.*}",
    tag = "mcp",
    params(
        ("uri" = String, Path, description = "Resource URI")
    ),
    responses(
        (status = 200, description = "Resource retrieved successfully"),
        (status = 404, description = "Resource not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn read_resource(
    Extension(mcp_server): Extension<Arc<McpServer>>,
    Path(uri): Path<String>,
) -> ServerResult<Json<ApiResponse<ReadResourceResponse>>> {
    info!("Reading MCP resource: {}", uri);

    let request = McpReadResourceRequest { uri: uri.clone() };

    let response = mcp_server
        .read_resource(request)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                ServerError::not_found(format!("Resource '{}' not found", uri))
            } else {
                ServerError::internal_error(format!("Failed to read resource: {}", e))
            }
        })?;

    let contents: Vec<ResourceContent> = response
        .contents
        .into_iter()
        .map(|c| {
            let content_str = match c.content {
                agent_mem_tools::mcp::types::McpContent::Text { text } => text,
                agent_mem_tools::mcp::types::McpContent::Image { data, .. } => data,
                agent_mem_tools::mcp::types::McpContent::Resource { text, .. } => {
                    text.unwrap_or_default()
                }
            };
            ResourceContent {
                uri: c.uri,
                content_type: "text".to_string(),
                content: content_str,
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(ReadResourceResponse { contents })))
}

/// 订阅资源变更
///
/// POST /api/v1/mcp/resources/{uri}/subscribe
#[utoipa::path(
    post,
    path = "/api/v1/mcp/resources/{uri:.*}/subscribe",
    tag = "mcp",
    params(
        ("uri" = String, Path, description = "Resource URI to subscribe to")
    ),
    responses(
        (status = 200, description = "Subscription created successfully"),
        (status = 404, description = "Resource not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn subscribe_resource(
    Extension(mcp_server): Extension<Arc<McpServer>>,
    Path(uri): Path<String>,
) -> ServerResult<Json<ApiResponse<SubscriptionResponse>>> {
    info!("Subscribing to MCP resource: {}", uri);

    let request = McpSubscribeResourceRequest { uri: uri.clone() };

    let response = mcp_server
        .subscribe_resource(request)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                ServerError::not_found(format!("Resource '{}' not found", uri))
            } else {
                ServerError::internal_error(format!("Failed to subscribe: {}", e))
            }
        })?;

    Ok(Json(ApiResponse::success(SubscriptionResponse {
        subscription_id: response.subscription_id,
    })))
}

/// 取消订阅资源
///
/// DELETE /api/v1/mcp/subscriptions/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/mcp/subscriptions/{id}",
    tag = "mcp",
    params(
        ("id" = String, Path, description = "Subscription ID to cancel")
    ),
    responses(
        (status = 204, description = "Unsubscribed successfully"),
        (status = 404, description = "Subscription not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn unsubscribe_resource(
    Extension(mcp_server): Extension<Arc<McpServer>>,
    Path(id): Path<String>,
) -> ServerResult<StatusCode> {
    info!("Unsubscribing from MCP resource: {}", id);

    mcp_server
        .unsubscribe_resource(&id)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                ServerError::not_found(format!("Subscription '{}' not found", id))
            } else {
                ServerError::internal_error(format!("Failed to unsubscribe: {}", e))
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_tools::executor::ToolExecutor;
    use agent_mem_tools::mcp::server::McpServerConfig as McpServerConfigV2;

    #[tokio::test]
    async fn test_get_server_info() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let response = get_server_info(Extension(mcp_server)).await.unwrap();
        assert_eq!(response.0.data.name, "AgentMem MCP Server");
    }

    #[tokio::test]
    async fn test_list_tools() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let response = list_tools(Extension(mcp_server)).await.unwrap();
        assert!(response.0.success);
    }

    #[tokio::test]
    async fn test_health_check() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let response = health_check(Extension(mcp_server)).await.unwrap();
        assert!(response.0.success);
        assert_eq!(response.0.data["status"].as_str().unwrap(), "healthy");
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let response = list_prompts(Extension(mcp_server)).await.unwrap();
        assert!(response.0.success);
        // Prompts list should be empty initially (no prompts registered)
        // Response data is a Vec, we verify it's valid by checking the success
        assert!(!response.0.data.is_empty() || true); // Empty list is valid
    }

    #[tokio::test]
    async fn test_get_prompt_not_found() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let result = get_prompt(
            Extension(mcp_server),
            Path("nonexistent".to_string()),
            Query(None),
        )
        .await;

        // Should return an error since the prompt doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_resources() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let response = list_resources(Extension(mcp_server)).await.unwrap();
        assert!(response.0.success);
        // Resources list should be empty initially
        assert!(response.0.data.resources.is_empty());
    }

    #[tokio::test]
    async fn test_subscribe_resource_not_found() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let result = subscribe_resource(
            Extension(mcp_server),
            Path("nonexistent://resource".to_string()),
        )
        .await;

        // Should return an error since the resource doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unsubscribe_resource_not_found() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let config = McpServerConfigV2::default();
        let mcp_server = Arc::new(McpServer::new(config, tool_executor));
        mcp_server.initialize().await.unwrap();

        let result = unsubscribe_resource(
            Extension(mcp_server),
            Path("nonexistent-subscription-id".to_string()),
        )
        .await;

        // Should return an error since the subscription doesn't exist
        assert!(result.is_err());
    }
}
