//! Stdio 传输层实现
//!
//! 提供基于标准输入输出的 MCP 传输，用于与 Claude Desktop 等本地客户端集成

use super::{Transport, McpError, McpResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// JSON-RPC 2.0 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC 版本（必须是 "2.0"）
    pub jsonrpc: String,
    
    /// 请求 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    
    /// 方法名
    pub method: String,
    
    /// 参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC 版本（必须是 "2.0"）
    pub jsonrpc: String,
    
    /// 请求 ID
    pub id: Value,
    
    /// 结果（成功时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    
    /// 错误（失败时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// 错误代码
    pub code: i32,
    
    /// 错误消息
    pub message: String,
    
    /// 错误数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Stdio 传输
pub struct StdioTransport {
    /// 连接状态
    connected: Arc<RwLock<bool>>,
    
    /// 请求 ID 计数器
    request_id: Arc<RwLock<u64>>,
    
    /// 是否已初始化
    initialized: Arc<RwLock<bool>>,
}

impl StdioTransport {
    /// 创建新的 Stdio 传输
    pub fn new() -> Self {
        Self {
            connected: Arc::new(RwLock::new(false)),
            request_id: Arc::new(RwLock::new(0)),
            initialized: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 生成下一个请求 ID
    async fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.write().await;
        *id += 1;
        *id
    }
    
    /// 发送 JSON-RPC 请求到 stdout
    async fn send_jsonrpc_request(&self, request: &JsonRpcRequest) -> McpResult<()> {
        let json = serde_json::to_string(request)
            .map_err(|e| McpError::SerializationError(e.to_string()))?;
        
        debug!("发送 JSON-RPC 请求: {}", json);
        
        // 写入到 stdout
        let mut stdout = tokio::io::stdout();
        stdout.write_all(json.as_bytes()).await
            .map_err(|e| McpError::TransportError(format!("写入 stdout 失败: {e}")))?;
        stdout.write_all(b"\n").await
            .map_err(|e| McpError::TransportError(format!("写入换行符失败: {e}")))?;
        stdout.flush().await
            .map_err(|e| McpError::TransportError(format!("刷新 stdout 失败: {e}")))?;
        
        Ok(())
    }
    
    /// 从 stdin 读取 JSON-RPC 响应
    async fn read_jsonrpc_response(&self) -> McpResult<JsonRpcResponse> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        
        reader.read_line(&mut line).await
            .map_err(|e| McpError::TransportError(format!("读取 stdin 失败: {e}")))?;
        
        debug!("接收 JSON-RPC 响应: {}", line.trim());
        
        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| McpError::DeserializationError(e.to_string()))?;
        
        // 检查是否有错误
        if let Some(error) = &response.error {
            return Err(McpError::ServerError(format!(
                "服务器错误 [{}]: {}",
                error.code, error.message
            )));
        }
        
        Ok(response)
    }
    
    /// 执行 initialize 握手
    async fn initialize(&self) -> McpResult<()> {
        info!("开始 MCP initialize 握手");
        
        let request_id = self.next_request_id().await;
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(request_id.into())),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "AgentMem MCP Client",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        };
        
        self.send_jsonrpc_request(&request).await?;
        let response = self.read_jsonrpc_response().await?;
        
        if let Some(result) = response.result {
            info!("Initialize 握手成功: {:?}", result);
            *self.initialized.write().await = true;
            Ok(())
        } else {
            Err(McpError::ProtocolError("Initialize 响应缺少 result 字段".to_string()))
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    /// 连接到服务器（对于 stdio，这意味着执行 initialize 握手）
    async fn connect(&mut self) -> McpResult<()> {
        if *self.connected.read().await {
            return Ok(());
        }
        
        info!("连接 Stdio 传输");
        
        // 执行 initialize 握手
        self.initialize().await?;
        
        *self.connected.write().await = true;
        info!("Stdio 传输连接成功");
        
        Ok(())
    }
    
    /// 发送请求
    async fn send_request(&self, method: &str, params: Value) -> McpResult<Value> {
        if !*self.connected.read().await {
            return Err(McpError::ConnectionError("未连接到服务器".to_string()));
        }
        
        let request_id = self.next_request_id().await;
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(request_id.into())),
            method: method.to_string(),
            params: Some(params),
        };
        
        self.send_jsonrpc_request(&request).await?;
        let response = self.read_jsonrpc_response().await?;
        
        response.result.ok_or_else(|| {
            McpError::ProtocolError(format!("响应缺少 result 字段: method={method}"))
        })
    }
    
    /// 断开连接
    async fn disconnect(&mut self) -> McpResult<()> {
        if !*self.connected.read().await {
            return Ok(());
        }
        
        info!("断开 Stdio 传输");
        
        *self.connected.write().await = false;
        *self.initialized.write().await = false;
        
        Ok(())
    }
    
    /// 检查是否已连接
    fn is_connected(&self) -> bool {
        // 注意：这是同步方法，我们不能使用 await
        // 我们需要使用 try_read() 来避免阻塞
        self.connected.try_read()
            .map(|guard| *guard)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(1.into())),
            method: "tools/list".to_string(),
            params: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/list\""));
    }
    
    #[test]
    fn test_jsonrpc_response_deserialization() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
    
    #[test]
    fn test_jsonrpc_error_deserialization() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid Request"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }
}

