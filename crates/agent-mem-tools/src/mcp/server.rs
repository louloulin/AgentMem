//! MCP 服务端实现
//!
//! 将 AgentMem 工具暴露为 MCP 服务，允许其他 MCP 客户端调用

use super::error::{McpError, McpResult};
use super::types::{McpContent, McpListToolsResponse, McpTool, McpToolCallRequest, McpToolCallResponse};
use super::resources::{
    ResourceManager, McpListResourcesResponse,
    McpReadResourceRequest, McpReadResourceResponse, McpSubscribeResourceRequest,
    McpSubscribeResourceResponse, ResourceChangeType,
};
use crate::executor::{Tool, ToolExecutor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// MCP 服务端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// 服务器名称
    pub name: String,
    
    /// 服务器版本
    pub version: String,
    
    /// 服务器描述
    pub description: String,
    
    /// 是否需要认证
    pub require_auth: bool,
    
    /// API 密钥（如果需要认证）
    pub api_keys: Vec<String>,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "AgentMem MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "AgentMem tools exposed via MCP protocol".to_string(),
            require_auth: false,
            api_keys: vec![],
        }
    }
}

/// MCP 服务端
pub struct McpServer {
    /// 配置
    config: McpServerConfig,

    /// 工具执行器
    tool_executor: Arc<ToolExecutor>,

    /// 已注册的工具
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,

    /// 资源管理器
    resource_manager: Arc<ResourceManager>,

    /// 是否已初始化
    initialized: Arc<RwLock<bool>>,
}

impl McpServer {
    /// 创建新的 MCP 服务端
    pub fn new(config: McpServerConfig, tool_executor: Arc<ToolExecutor>) -> Self {
        Self {
            config,
            tool_executor,
            tools: Arc::new(RwLock::new(HashMap::new())),
            resource_manager: Arc::new(ResourceManager::new(300)), // 5 分钟缓存
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config(tool_executor: Arc<ToolExecutor>) -> Self {
        Self::new(McpServerConfig::default(), tool_executor)
    }

    /// 使用自定义资源管理器创建
    pub fn with_resource_manager(
        config: McpServerConfig,
        tool_executor: Arc<ToolExecutor>,
        resource_manager: Arc<ResourceManager>,
    ) -> Self {
        Self {
            config,
            tool_executor,
            tools: Arc::new(RwLock::new(HashMap::new())),
            resource_manager,
            initialized: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 初始化服务器
    pub async fn initialize(&self) -> McpResult<()> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }

        info!("Initializing MCP server: {}", self.config.name);

        // 从 ToolExecutor 加载所有工具
        let tool_names = self.tool_executor.list_tools().await;
        let mut tools = self.tools.write().await;

        for tool_name in tool_names {
            if let Some(tool) = self.tool_executor.get_tool(&tool_name).await {
                tools.insert(tool_name.clone(), tool);
                debug!("Registered tool: {}", tool_name);
            }
        }

        *initialized = true;
        info!("MCP server initialized with {} tools", tools.len());

        Ok(())
    }
    
    /// 注册工具
    pub async fn register_tool(&self, tool: Arc<dyn Tool>) -> McpResult<()> {
        let tool_name = tool.name().to_string();
        let mut tools = self.tools.write().await;

        if tools.contains_key(&tool_name) {
            return Err(McpError::ConfigError(
                format!("Tool '{}' already registered", tool_name)
            ));
        }

        tools.insert(tool_name.clone(), tool);
        info!("Registered tool: {}", tool_name);

        Ok(())
    }

    /// 取消注册工具
    pub async fn unregister_tool(&self, tool_name: &str) -> McpResult<()> {
        let mut tools = self.tools.write().await;

        if tools.remove(tool_name).is_none() {
            return Err(McpError::ConfigError(
                format!("Tool '{}' not found", tool_name)
            ));
        }

        info!("Unregistered tool: {}", tool_name);
        Ok(())
    }
    
    /// 列出所有工具
    pub async fn list_tools(&self) -> McpResult<McpListToolsResponse> {
        self.check_initialized().await?;

        let tools = self.tools.read().await;
        let mcp_tools: Vec<McpTool> = tools
            .values()
            .map(|tool| {
                let schema = tool.schema();
                // 将 ParameterSchema 转换为 JSON Value
                let input_schema = serde_json::to_value(&schema.parameters)
                    .unwrap_or_else(|_| serde_json::json!({}));

                McpTool {
                    name: tool.name().to_string(),
                    description: tool.description().to_string(),
                    input_schema,
                }
            })
            .collect();

        Ok(McpListToolsResponse { tools: mcp_tools })
    }
    
    /// 调用工具
    pub async fn call_tool(&self, request: McpToolCallRequest) -> McpResult<McpToolCallResponse> {
        self.check_initialized().await?;

        info!("Calling tool: {} with args: {:?}", request.name, request.arguments);

        // 创建执行上下文
        use crate::executor::ExecutionContext;
        use std::time::Duration;

        let context = ExecutionContext {
            user: "mcp-server".to_string(),
            timeout: Duration::from_secs(30),
        };

        // 执行工具
        let result = self.tool_executor
            .execute_tool(&request.name, request.arguments, &context)
            .await
            .map_err(|e| McpError::ToolExecutionError {
                tool_name: request.name.clone(),
                message: e.to_string(),
            })?;

        // 转换结果为 MCP 格式
        let content = vec![McpContent::Text {
            text: serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| format!("{:?}", result)),
        }];

        Ok(McpToolCallResponse {
            content,
            is_error: false,
        })
    }
    
    /// 验证 API 密钥
    pub fn verify_api_key(&self, api_key: &str) -> bool {
        if !self.config.require_auth {
            return true;
        }
        
        self.config.api_keys.contains(&api_key.to_string())
    }
    
    /// 获取服务器信息
    pub fn get_server_info(&self) -> ServerInfo {
        ServerInfo {
            name: self.config.name.clone(),
            version: self.config.version.clone(),
            description: self.config.description.clone(),
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: true,
                resources: true,  // 现在支持 Resources
                prompts: false,
            },
        }
    }

    /// 列出所有资源
    pub async fn list_resources(&self) -> McpResult<McpListResourcesResponse> {
        self.check_initialized().await?;

        info!("Listing MCP resources");
        let resources = self.resource_manager.list_resources().await?;

        Ok(McpListResourcesResponse { resources })
    }

    /// 读取资源内容
    pub async fn read_resource(&self, request: McpReadResourceRequest) -> McpResult<McpReadResourceResponse> {
        self.check_initialized().await?;

        info!("Reading MCP resource: {}", request.uri);
        let content = self.resource_manager.read_resource(&request.uri).await?;

        Ok(McpReadResourceResponse {
            contents: vec![content],
        })
    }

    /// 订阅资源变更
    pub async fn subscribe_resource(&self, request: McpSubscribeResourceRequest) -> McpResult<McpSubscribeResourceResponse> {
        self.check_initialized().await?;

        info!("Subscribing to MCP resource: {}", request.uri);
        let subscription = self.resource_manager.subscribe_resource(&request.uri).await?;

        Ok(McpSubscribeResourceResponse {
            subscription_id: subscription.id,
        })
    }

    /// 取消订阅资源
    pub async fn unsubscribe_resource(&self, subscription_id: &str) -> McpResult<()> {
        self.check_initialized().await?;

        info!("Unsubscribing from MCP resource: {}", subscription_id);
        self.resource_manager.unsubscribe_resource(subscription_id).await
    }

    /// 通知资源变更
    pub async fn notify_resource_change(&self, uri: &str, change_type: ResourceChangeType) -> McpResult<()> {
        self.check_initialized().await?;

        info!("Notifying resource change: {} ({:?})", uri, change_type);
        self.resource_manager.notify_resource_change(uri, change_type).await
    }

    /// 获取资源管理器
    pub fn resource_manager(&self) -> &Arc<ResourceManager> {
        &self.resource_manager
    }
    
    /// 检查是否已初始化
    async fn check_initialized(&self) -> McpResult<()> {
        let initialized = self.initialized.read().await;
        if !*initialized {
            return Err(McpError::NotInitializedError("MCP Server".to_string()));
        }
        Ok(())
    }
}

/// 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
}

/// 服务器能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: bool,
    pub resources: bool,
    pub prompts: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::ExecutionContext;
    use crate::schema::ToolSchema;
    use async_trait::async_trait;
    use serde_json::json;

    // Mock 工具
    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock_tool"
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn schema(&self) -> ToolSchema {
            use crate::schema::{ParameterSchema, PropertySchema};
            use std::collections::HashMap;

            let mut properties = HashMap::new();
            properties.insert(
                "input".to_string(),
                PropertySchema {
                    prop_type: "string".to_string(),
                    description: "Input parameter".to_string(),
                    enum_values: None,
                    default: None,
                    minimum: None,
                    maximum: None,
                    items: None,
                },
            );

            ToolSchema {
                name: "mock_tool".to_string(),
                description: "A mock tool for testing".to_string(),
                parameters: ParameterSchema {
                    param_type: "object".to_string(),
                    properties,
                    required: vec!["input".to_string()],
                },
            }
        }

        async fn execute(
            &self,
            args: serde_json::Value,
            _context: &ExecutionContext,
        ) -> crate::ToolResult<serde_json::Value> {
            Ok(json!({
                "result": format!("Processed: {}", args["input"].as_str().unwrap_or(""))
            }))
        }
    }
    
    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let server = McpServer::with_default_config(tool_executor);
        
        assert!(server.initialize().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_register_and_list_tools() {
        let tool_executor = Arc::new(ToolExecutor::new());
        let server = McpServer::with_default_config(tool_executor);
        
        server.initialize().await.unwrap();
        
        let mock_tool = Arc::new(MockTool);
        server.register_tool(mock_tool).await.unwrap();
        
        let response = server.list_tools().await.unwrap();
        assert_eq!(response.tools.len(), 1);
        assert_eq!(response.tools[0].name, "mock_tool");
    }
}

