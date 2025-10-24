//! MCP 客户端管理器
//!
//! 管理多个 MCP 服务器连接

use super::client::McpClient;
use super::error::{McpError, McpResult};
use super::types::{McpServerConfig, McpServerInfo, McpTool, McpToolCallResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP 客户端管理器
pub struct McpClientManager {
    /// 客户端映射 (server_name -> client)
    clients: Arc<RwLock<HashMap<String, Arc<McpClient>>>>,
    
    /// 服务器配置映射
    configs: Arc<RwLock<HashMap<String, McpServerConfig>>>,
}

impl McpClientManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 添加服务器
    pub async fn add_server(&self, config: McpServerConfig) -> McpResult<()> {
        let server_name = config.server_name.clone();
        
        // 检查是否已存在
        if self.clients.read().await.contains_key(&server_name) {
            return Err(McpError::ConfigError(
                format!("Server '{server_name}' already exists")
            ));
        }
        
        // 创建客户端
        let client = Arc::new(McpClient::new(config.clone()));
        
        // 连接到服务器
        client.connect().await?;
        
        // 存储客户端和配置
        self.clients.write().await.insert(server_name.clone(), client);
        self.configs.write().await.insert(server_name.clone(), config);
        
        Ok(())
    }
    
    /// 移除服务器
    pub async fn remove_server(&self, server_name: &str) -> McpResult<()> {
        let client = self.clients.write().await.remove(server_name)
            .ok_or_else(|| McpError::ConfigError(
                format!("Server '{server_name}' not found")
            ))?;
        
        // 断开连接
        client.disconnect().await?;
        
        // 移除配置
        self.configs.write().await.remove(server_name);
        
        Ok(())
    }
    
    /// 列出所有服务器
    pub async fn list_servers(&self) -> Vec<String> {
        self.clients.read().await.keys().cloned().collect()
    }
    
    /// 获取服务器信息
    pub async fn get_server_info(&self, server_name: &str) -> McpResult<McpServerInfo> {
        let clients = self.clients.read().await;
        let configs = self.configs.read().await;
        
        let client = clients.get(server_name)
            .ok_or_else(|| McpError::ConfigError(
                format!("Server '{server_name}' not found")
            ))?;
        
        let config = configs.get(server_name)
            .ok_or_else(|| McpError::ConfigError(
                format!("Config for server '{server_name}' not found")
            ))?;
        
        Ok(McpServerInfo {
            name: server_name.to_string(),
            server_type: format!("{:?}", config.server_type),
            initialized: client.is_connected().await,
            connected: client.is_connected().await,
            config: serde_json::to_value(config)?,
        })
    }
    
    /// 列出所有工具
    pub async fn list_all_tools(&self) -> McpResult<HashMap<String, Vec<McpTool>>> {
        let clients = self.clients.read().await;
        let mut all_tools = HashMap::new();
        
        for (server_name, client) in clients.iter() {
            match client.list_tools().await {
                Ok(tools) => {
                    all_tools.insert(server_name.clone(), tools);
                }
                Err(e) => {
                    eprintln!("Failed to list tools for server '{server_name}': {e}");
                    all_tools.insert(server_name.clone(), vec![]);
                }
            }
        }
        
        Ok(all_tools)
    }
    
    /// 列出指定服务器的工具
    pub async fn list_tools(&self, server_name: &str) -> McpResult<Vec<McpTool>> {
        let clients = self.clients.read().await;
        let client = clients.get(server_name)
            .ok_or_else(|| McpError::ConfigError(
                format!("Server '{server_name}' not found")
            ))?;
        
        client.list_tools().await
    }
    
    /// 执行工具
    pub async fn execute_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> McpResult<McpToolCallResponse> {
        let clients = self.clients.read().await;
        let client = clients.get(server_name)
            .ok_or_else(|| McpError::ConfigError(
                format!("Server '{server_name}' not found")
            ))?;
        
        client.execute_tool(tool_name, arguments).await
    }
    
    /// 查找工具（在所有服务器中搜索）
    pub async fn find_tool(&self, tool_name: &str) -> McpResult<Option<(String, McpTool)>> {
        let all_tools = self.list_all_tools().await?;
        
        for (server_name, tools) in all_tools {
            for tool in tools {
                if tool.name == tool_name {
                    return Ok(Some((server_name, tool)));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 断开所有连接
    pub async fn disconnect_all(&self) -> McpResult<()> {
        let clients = self.clients.write().await;
        
        for (server_name, client) in clients.iter() {
            if let Err(e) = client.disconnect().await {
                eprintln!("Failed to disconnect server '{server_name}': {e}");
            }
        }
        
        Ok(())
    }
    
    /// 获取客户端数量
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }
}

impl Default for McpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for McpClientManager {
    fn drop(&mut self) {
        // 尝试断开所有连接
        // 注意：这里不能使用 async，所以只是尽力而为
        if let Ok(clients) = self.clients.try_read() {
            for (server_name, _client) in clients.iter() {
                eprintln!("Warning: MCP client '{server_name}' not properly disconnected");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[tokio::test]
    async fn test_create_manager() {
        let manager = McpClientManager::new();
        assert_eq!(manager.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_list_servers_empty() {
        let manager = McpClientManager::new();
        let servers = manager.list_servers().await;
        assert_eq!(servers.len(), 0);
    }

    #[tokio::test]
    async fn test_add_duplicate_server() {
        let manager = McpClientManager::new();
        
        let config1 = McpServerConfig::stdio(
            "test-server".to_string(),
            "echo".to_string(),
            vec![],
            None,
        );
        
        let config2 = McpServerConfig::stdio(
            "test-server".to_string(),
            "echo".to_string(),
            vec![],
            None,
        );
        
        // 第一次添加应该成功（如果 echo 命令可用）
        let _ = manager.add_server(config1).await;
        
        // 第二次添加应该失败
        let result = manager.add_server(config2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_server() {
        let manager = McpClientManager::new();
        let result = manager.remove_server("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_server_info_not_found() {
        let manager = McpClientManager::new();
        let result = manager.get_server_info("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tools_not_found() {
        let manager = McpClientManager::new();
        let result = manager.list_tools("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_tool_not_found() {
        let manager = McpClientManager::new();
        let result = manager.execute_tool(
            "nonexistent",
            "tool",
            serde_json::json!({}),
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_tool_empty() {
        let manager = McpClientManager::new();
        let result = manager.find_tool("test_tool").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

