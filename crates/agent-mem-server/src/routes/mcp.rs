//! MCP 服务端 REST API 路由
//!
//! 提供 MCP 协议的 HTTP 接口

use crate::error::{ServerError, ServerResult};
use crate::models::ApiResponse;
use agent_mem_tools::mcp::{McpServer, ServerInfo};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};
use utoipa::ToSchema;

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
}
