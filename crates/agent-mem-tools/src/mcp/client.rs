//! MCP 客户端实现
//!
//! 提供与 MCP 服务器通信的客户端

use super::error::{McpError, McpResult};
use super::types::{
    McpContent, McpListToolsResponse, McpServerConfig, McpServerType, McpTool,
    McpToolCallRequest, McpToolCallResponse,
};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// MCP 客户端
pub struct McpClient {
    /// 服务器配置
    config: McpServerConfig,
    
    /// 是否已初始化
    initialized: Arc<RwLock<bool>>,
    
    /// 子进程（用于 Stdio 类型）
    #[allow(dead_code)]
    process: Arc<RwLock<Option<Child>>>,
    
    /// 工具缓存
    tools_cache: Arc<RwLock<Option<Vec<McpTool>>>>,
}

impl McpClient {
    /// 创建新的 MCP 客户端
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            initialized: Arc::new(RwLock::new(false)),
            process: Arc::new(RwLock::new(None)),
            tools_cache: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 连接到 MCP 服务器
    pub async fn connect(&self) -> McpResult<()> {
        match self.config.server_type {
            McpServerType::Stdio => self.connect_stdio().await,
            McpServerType::Sse => self.connect_sse().await,
            McpServerType::Http => self.connect_http().await,
        }
    }
    
    /// 连接到 Stdio 类型的服务器
    async fn connect_stdio(&self) -> McpResult<()> {
        let command = self.config.command.as_ref()
            .ok_or_else(|| McpError::ConfigError("Missing command for stdio server".to_string()))?;
        
        let args = self.config.args.as_ref()
            .map(|a| a.as_slice())
            .unwrap_or(&[]);
        
        // 启动子进程
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // 设置环境变量
        if let Some(env) = &self.config.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }
        
        let child = cmd.spawn()
            .map_err(|e| McpError::ConnectionError(format!("Failed to spawn process: {}", e)))?;
        
        *self.process.write().await = Some(child);
        *self.initialized.write().await = true;
        
        Ok(())
    }
    
    /// 连接到 SSE 类型的服务器
    async fn connect_sse(&self) -> McpResult<()> {
        let _url = self.config.url.as_ref()
            .ok_or_else(|| McpError::ConfigError("Missing URL for SSE server".to_string()))?;
        
        // TODO: 实现 SSE 连接逻辑
        // 这里需要使用 reqwest 或其他 HTTP 客户端库
        
        *self.initialized.write().await = true;
        Ok(())
    }
    
    /// 连接到 HTTP 类型的服务器
    async fn connect_http(&self) -> McpResult<()> {
        let _url = self.config.url.as_ref()
            .ok_or_else(|| McpError::ConfigError("Missing URL for HTTP server".to_string()))?;
        
        // TODO: 实现 HTTP 连接逻辑
        
        *self.initialized.write().await = true;
        Ok(())
    }
    
    /// 检查是否已初始化
    async fn check_initialized(&self) -> McpResult<()> {
        if !*self.initialized.read().await {
            return Err(McpError::NotInitializedError(
                self.config.server_name.clone()
            ));
        }
        Ok(())
    }
    
    /// 列出可用的工具
    pub async fn list_tools(&self) -> McpResult<Vec<McpTool>> {
        self.check_initialized().await?;
        
        // 检查缓存
        if let Some(cached_tools) = self.tools_cache.read().await.as_ref() {
            return Ok(cached_tools.clone());
        }
        
        // 模拟工具发现（实际实现需要与 MCP 服务器通信）
        let tools = self.discover_tools().await?;
        
        // 更新缓存
        *self.tools_cache.write().await = Some(tools.clone());
        
        Ok(tools)
    }
    
    /// 发现工具（模拟实现）
    async fn discover_tools(&self) -> McpResult<Vec<McpTool>> {
        // TODO: 实现真实的工具发现逻辑
        // 这里返回一个示例工具列表
        Ok(vec![
            McpTool {
                name: format!("{}_example_tool", self.config.server_name),
                description: "An example MCP tool".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "Input parameter"
                        }
                    },
                    "required": ["input"]
                }),
            }
        ])
    }
    
    /// 执行工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> McpResult<McpToolCallResponse> {
        self.check_initialized().await?;
        
        // 创建工具调用请求
        let request = McpToolCallRequest {
            name: tool_name.to_string(),
            arguments,
        };
        
        // 执行工具（模拟实现）
        self.call_tool(request).await
    }
    
    /// 调用工具（模拟实现）
    async fn call_tool(&self, request: McpToolCallRequest) -> McpResult<McpToolCallResponse> {
        // TODO: 实现真实的工具调用逻辑
        // 这里返回一个模拟响应
        
        Ok(McpToolCallResponse {
            content: vec![McpContent::Text {
                text: format!(
                    "Mock response from tool '{}' with args: {}",
                    request.name,
                    request.arguments
                ),
            }],
            is_error: false,
        })
    }
    
    /// 断开连接
    pub async fn disconnect(&self) -> McpResult<()> {
        if let Some(mut child) = self.process.write().await.take() {
            child.kill()
                .map_err(|e| McpError::ConnectionError(format!("Failed to kill process: {}", e)))?;
        }
        
        *self.initialized.write().await = false;
        *self.tools_cache.write().await = None;
        
        Ok(())
    }
    
    /// 获取服务器名称
    pub fn server_name(&self) -> &str {
        &self.config.server_name
    }
    
    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        *self.initialized.read().await
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // 尝试清理子进程
        if let Ok(mut process) = self.process.try_write() {
            if let Some(mut child) = process.take() {
                let _ = child.kill();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client() {
        let config = McpServerConfig::stdio(
            "test-server".to_string(),
            "echo".to_string(),
            vec!["hello".to_string()],
            None,
        );
        
        let client = McpClient::new(config);
        assert_eq!(client.server_name(), "test-server");
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_list_tools_not_initialized() {
        let config = McpServerConfig::stdio(
            "test-server".to_string(),
            "echo".to_string(),
            vec![],
            None,
        );
        
        let client = McpClient::new(config);
        let result = client.list_tools().await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::NotInitializedError(_)));
    }

    #[tokio::test]
    async fn test_execute_tool_not_initialized() {
        let config = McpServerConfig::stdio(
            "test-server".to_string(),
            "echo".to_string(),
            vec![],
            None,
        );
        
        let client = McpClient::new(config);
        let result = client.execute_tool("test_tool", serde_json::json!({})).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::NotInitializedError(_)));
    }
}

