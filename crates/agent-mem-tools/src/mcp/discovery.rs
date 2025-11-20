//! MCP 工具发现和注册
//!
//! 提供动态工具发现、加载和注册功能

use super::error::{McpError, McpResult};
use super::transport::{HttpTransport, Transport};
use super::types::{McpListToolsResponse, McpTool};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 工具元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// 工具名称
    pub name: String,

    /// 工具版本
    pub version: String,

    /// 工具描述
    pub description: String,

    /// 工具来源（URL 或路径）
    pub source: String,

    /// 工具类型
    pub tool_type: ToolType,

    /// 依赖列表
    pub dependencies: Vec<String>,

    /// 标签
    pub tags: Vec<String>,

    /// 额外元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 工具类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolType {
    /// HTTP 工具
    Http,

    /// Stdio 工具
    Stdio,

    /// 本地工具
    Local,

    /// 插件工具
    Plugin,
}

/// 工具发现器
pub struct ToolDiscovery {
    /// 工具注册表
    registry: Arc<RwLock<HashMap<String, ToolMetadata>>>,

    /// 已发现的工具
    discovered_tools: Arc<RwLock<HashMap<String, McpTool>>>,
}

impl ToolDiscovery {
    /// 创建新的工具发现器
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            discovered_tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 发现工具
    pub async fn discover_tools(&self, server_url: &str) -> McpResult<Vec<McpTool>> {
        info!("Discovering tools from server: {}", server_url);

        // 1. 通过 HTTP 发现工具
        let http_tools = self.discover_http_tools(server_url).await?;

        info!("Discovered {} tools from HTTP", http_tools.len());

        // 2. 注册到本地
        for tool in &http_tools {
            self.register_discovered_tool(tool.clone()).await?;
        }

        Ok(http_tools)
    }

    /// 通过 HTTP 发现工具
    async fn discover_http_tools(&self, url: &str) -> McpResult<Vec<McpTool>> {
        debug!("Discovering HTTP tools from: {}", url);

        // 创建 HTTP 传输
        let mut transport = HttpTransport::new(url.to_string(), None);

        // 连接到服务器
        transport.connect().await?;

        // 发送 list_tools 请求
        let response = transport
            .send_request("tools/list", serde_json::json!({}))
            .await?;

        // 解析响应
        let tools_response: McpListToolsResponse =
            serde_json::from_value(response).map_err(|e| {
                McpError::SerializationError(format!("Failed to parse tools response: {e}"))
            })?;

        // 断开连接
        transport.disconnect().await?;

        Ok(tools_response.tools)
    }

    /// 注册已发现的工具
    async fn register_discovered_tool(&self, tool: McpTool) -> McpResult<()> {
        debug!("Registering discovered tool: {}", tool.name);

        let mut discovered = self.discovered_tools.write().await;
        discovered.insert(tool.name.clone(), tool);

        Ok(())
    }

    /// 注册工具元数据
    pub async fn register_metadata(&self, metadata: ToolMetadata) -> McpResult<()> {
        info!(
            "Registering tool metadata: {} v{}",
            metadata.name, metadata.version
        );

        // 检查依赖
        self.check_dependencies(&metadata).await?;

        let mut registry = self.registry.write().await;
        registry.insert(metadata.name.clone(), metadata);

        Ok(())
    }

    /// 检查工具依赖
    async fn check_dependencies(&self, metadata: &ToolMetadata) -> McpResult<()> {
        let registry = self.registry.read().await;

        for dep in &metadata.dependencies {
            if !registry.contains_key(dep) {
                warn!("Missing dependency: {} for tool {}", dep, metadata.name);
                // 不阻止注册，只是警告
            }
        }

        Ok(())
    }

    /// 获取工具元数据
    pub async fn get_metadata(&self, name: &str) -> Option<ToolMetadata> {
        let registry = self.registry.read().await;
        registry.get(name).cloned()
    }

    /// 列出所有已注册的工具元数据
    pub async fn list_metadata(&self) -> Vec<ToolMetadata> {
        let registry = self.registry.read().await;
        registry.values().cloned().collect()
    }

    /// 列出所有已发现的工具
    pub async fn list_discovered_tools(&self) -> Vec<McpTool> {
        let discovered = self.discovered_tools.read().await;
        discovered.values().cloned().collect()
    }

    /// 搜索工具
    pub async fn search_tools(&self, query: &str) -> Vec<ToolMetadata> {
        let registry = self.registry.read().await;

        registry
            .values()
            .filter(|metadata| {
                metadata.name.contains(query)
                    || metadata.description.contains(query)
                    || metadata.tags.iter().any(|tag| tag.contains(query))
            })
            .cloned()
            .collect()
    }

    /// 按标签过滤工具
    pub async fn filter_by_tags(&self, tags: &[String]) -> Vec<ToolMetadata> {
        let registry = self.registry.read().await;

        registry
            .values()
            .filter(|metadata| tags.iter().any(|tag| metadata.tags.contains(tag)))
            .cloned()
            .collect()
    }

    /// 去重工具列表
    fn deduplicate_tools(&self, tools: Vec<McpTool>) -> Vec<McpTool> {
        let mut seen = HashMap::new();
        let mut unique_tools = Vec::new();

        for tool in tools {
            if !seen.contains_key(&tool.name) {
                seen.insert(tool.name.clone(), true);
                unique_tools.push(tool);
            }
        }

        unique_tools
    }
}

impl Default for ToolDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具加载器 trait
#[async_trait]
pub trait ToolLoader: Send + Sync {
    /// 加载工具
    async fn load(&self, metadata: &ToolMetadata) -> McpResult<bool>;

    /// 卸载工具
    async fn unload(&self, name: &str) -> McpResult<()>;

    /// 检查工具是否已加载
    async fn is_loaded(&self, name: &str) -> bool;
}

/// HTTP 工具加载器
pub struct HttpToolLoader {
    /// 已加载的工具
    loaded_tools: Arc<RwLock<HashMap<String, String>>>,
}

impl HttpToolLoader {
    /// 创建新的 HTTP 工具加载器
    pub fn new() -> Self {
        Self {
            loaded_tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for HttpToolLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolLoader for HttpToolLoader {
    async fn load(&self, metadata: &ToolMetadata) -> McpResult<bool> {
        if metadata.tool_type != ToolType::Http {
            return Ok(false);
        }

        info!(
            "Loading HTTP tool: {} from {}",
            metadata.name, metadata.source
        );

        let mut loaded = self.loaded_tools.write().await;
        loaded.insert(metadata.name.clone(), metadata.source.clone());

        Ok(true)
    }

    async fn unload(&self, name: &str) -> McpResult<()> {
        info!("Unloading HTTP tool: {}", name);

        let mut loaded = self.loaded_tools.write().await;
        loaded.remove(name);

        Ok(())
    }

    async fn is_loaded(&self, name: &str) -> bool {
        let loaded = self.loaded_tools.read().await;
        loaded.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_discovery_creation() {
        let discovery = ToolDiscovery::new();
        let tools = discovery.list_discovered_tools().await;
        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_register_metadata() {
        let discovery = ToolDiscovery::new();

        let metadata = ToolMetadata {
            name: "test_tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            source: "http://localhost:8080".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
        };

        discovery.register_metadata(metadata.clone()).await.unwrap();

        let retrieved = discovery.get_metadata("test_tool").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_tool");
    }

    #[tokio::test]
    async fn test_list_metadata() {
        let discovery = ToolDiscovery::new();

        let metadata1 = ToolMetadata {
            name: "tool1".to_string(),
            version: "1.0.0".to_string(),
            description: "Tool 1".to_string(),
            source: "http://localhost:8080".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
        };

        let metadata2 = ToolMetadata {
            name: "tool2".to_string(),
            version: "2.0.0".to_string(),
            description: "Tool 2".to_string(),
            source: "http://localhost:8081".to_string(),
            tool_type: ToolType::Stdio,
            dependencies: vec![],
            tags: vec!["production".to_string()],
            metadata: HashMap::new(),
        };

        discovery.register_metadata(metadata1).await.unwrap();
        discovery.register_metadata(metadata2).await.unwrap();

        let all_metadata = discovery.list_metadata().await;
        assert_eq!(all_metadata.len(), 2);
    }

    #[tokio::test]
    async fn test_search_tools() {
        let discovery = ToolDiscovery::new();

        let metadata = ToolMetadata {
            name: "search_tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A tool for searching".to_string(),
            source: "http://localhost:8080".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec!["search".to_string(), "utility".to_string()],
            metadata: HashMap::new(),
        };

        discovery.register_metadata(metadata).await.unwrap();

        let results = discovery.search_tools("search").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "search_tool");
    }

    #[tokio::test]
    async fn test_filter_by_tags() {
        let discovery = ToolDiscovery::new();

        let metadata1 = ToolMetadata {
            name: "tool1".to_string(),
            version: "1.0.0".to_string(),
            description: "Tool 1".to_string(),
            source: "http://localhost:8080".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec!["test".to_string(), "utility".to_string()],
            metadata: HashMap::new(),
        };

        let metadata2 = ToolMetadata {
            name: "tool2".to_string(),
            version: "2.0.0".to_string(),
            description: "Tool 2".to_string(),
            source: "http://localhost:8081".to_string(),
            tool_type: ToolType::Stdio,
            dependencies: vec![],
            tags: vec!["production".to_string()],
            metadata: HashMap::new(),
        };

        discovery.register_metadata(metadata1).await.unwrap();
        discovery.register_metadata(metadata2).await.unwrap();

        let results = discovery.filter_by_tags(&["test".to_string()]).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "tool1");
    }

    #[tokio::test]
    async fn test_http_tool_loader() {
        let loader = HttpToolLoader::new();

        let metadata = ToolMetadata {
            name: "http_tool".to_string(),
            version: "1.0.0".to_string(),
            description: "An HTTP tool".to_string(),
            source: "http://localhost:8080".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec![],
            metadata: HashMap::new(),
        };

        let loaded = loader.load(&metadata).await.unwrap();
        assert!(loaded);

        let is_loaded = loader.is_loaded("http_tool").await;
        assert!(is_loaded);

        loader.unload("http_tool").await.unwrap();

        let is_loaded = loader.is_loaded("http_tool").await;
        assert!(!is_loaded);
    }

    #[tokio::test]
    async fn test_check_dependencies() {
        let discovery = ToolDiscovery::new();

        // 注册依赖工具
        let dep_metadata = ToolMetadata {
            name: "dependency_tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A dependency".to_string(),
            source: "http://localhost:8080".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec![],
            metadata: HashMap::new(),
        };

        discovery.register_metadata(dep_metadata).await.unwrap();

        // 注册依赖于上面工具的工具
        let metadata = ToolMetadata {
            name: "main_tool".to_string(),
            version: "1.0.0".to_string(),
            description: "Main tool".to_string(),
            source: "http://localhost:8080".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec!["dependency_tool".to_string()],
            tags: vec![],
            metadata: HashMap::new(),
        };

        let result = discovery.register_metadata(metadata).await;
        assert!(result.is_ok());
    }
}
