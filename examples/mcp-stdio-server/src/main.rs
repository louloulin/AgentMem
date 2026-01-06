//! AgentMem MCP Stdio 服务器
//!
//! 这是一个通过标准输入输出与 Claude Desktop 集成的 MCP 服务器示例

use agent_mem_core::client::{AgentMemClient, AgentMemClientConfig};
use agent_mem_tools::executor::ToolExecutor;
use agent_mem_tools::mcp::server::{McpServer, McpServerConfig};
use agent_mem_tools::mcp::transport::stdio::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use agent_mem_tools::register_agentmem_tools;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志（输出到 stderr，避免干扰 stdio 通信）
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("AgentMem MCP Stdio 服务器启动");

    // 创建 AgentMem 客户端（使用默认配置）
    let config = AgentMemClientConfig::default();
    let client = Arc::new(AgentMemClient::new(config));

    // 创建工具执行器
    let executor = Arc::new(ToolExecutor::new());

    // 注册 AgentMem 工具
    register_agentmem_tools(&executor).await?;
    info!(
        "已注册 {} 个 AgentMem 工具",
        executor.list_tools().await.len()
    );

    // 创建 MCP 服务器配置
    let config = McpServerConfig {
        name: "AgentMem MCP Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "AgentMem 记忆管理工具，通过 MCP 协议提供服务".to_string(),
        require_auth: false,
        api_keys: vec![],
    };

    // 创建 MCP 服务器
    let server = Arc::new(McpServer::new(config, executor));

    // 初始化服务器
    server.initialize().await?;

    info!("MCP 服务器初始化完成，等待客户端连接...");

    // 主循环：读取 stdin，处理请求，写入 stdout
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    loop {
        let mut line = String::new();

        // 读取一行 JSON-RPC 请求
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF，客户端断开连接
                info!("客户端断开连接");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                debug!("接收到请求: {}", line);

                // 解析 JSON-RPC 请求
                let request: JsonRpcRequest = match serde_json::from_str(line) {
                    Ok(req) => req,
                    Err(e) => {
                        error!("解析请求失败: {}", e);

                        // 返回错误响应
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: Value::Null,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32700,
                                message: format!("Parse error: {}", e),
                                data: None,
                            }),
                        };

                        let response_json = serde_json::to_string(&error_response)?;
                        stdout.write_all(response_json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                        continue;
                    }
                };

                // 处理请求
                let response = handle_request(&server, &client, request).await;

                // 发送响应
                let response_json = serde_json::to_string(&response)?;
                debug!("发送响应: {}", response_json);

                stdout.write_all(response_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
            Err(e) => {
                error!("读取 stdin 失败: {}", e);
                break;
            }
        }
    }

    info!("MCP 服务器关闭");
    Ok(())
}

/// 处理 JSON-RPC 请求
async fn handle_request(
    server: &Arc<McpServer>,
    client: &Arc<AgentMemClient>,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let request_id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => {
            // 返回服务器信息
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "AgentMem MCP Server",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })),
                error: None,
            }
        }
        "ping" | "health" | "healthcheck" => {
            // 健康检查支持（Claude Code需要）
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: Some(serde_json::json!({
                    "status": "ok"
                })),
                error: None,
            }
        }
        "tools/list" => {
            // 返回工具列表
            match server.list_tools().await {
                Ok(tools_response) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request_id,
                    result: Some(serde_json::to_value(tools_response).unwrap()),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request_id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: format!("Internal error: {}", e),
                        data: None,
                    }),
                },
            }
        }
        "tools/call" => {
            // 调用工具
            let params = request.params.unwrap_or(Value::Null);

            match server
                .call_tool(serde_json::from_value(params).unwrap())
                .await
            {
                Ok(tool_response) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request_id,
                    result: Some(serde_json::to_value(tool_response).unwrap()),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request_id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: format!("Tool execution error: {}", e),
                        data: None,
                    }),
                },
            }
        }
        _ => {
            // 不支持的方法
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            }
        }
    }
}
