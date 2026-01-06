//! MCP 工具市场
//!
//! 提供 MCP 服务器的注册表和发现功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP 服务器列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerListing {
    /// 服务器 ID
    pub id: String,

    /// 服务器名称
    pub name: String,

    /// 服务器描述
    pub description: String,

    /// 服务器分类
    pub category: String,

    /// 启动命令
    pub command: String,

    /// 命令参数
    pub args: Vec<String>,

    /// 环境变量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    /// 安装命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_command: Option<String>,

    /// 文档 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,

    /// GitHub URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_url: Option<String>,

    /// 作者
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// 标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// 系统要求
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<Vec<String>>,
}

impl McpServerListing {
    /// 创建新的服务器列表项
    pub fn new(
        id: String,
        name: String,
        description: String,
        category: String,
        command: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            category,
            command,
            args,
            env: None,
            install_command: None,
            documentation_url: None,
            github_url: None,
            author: None,
            tags: None,
            requirements: None,
        }
    }

    /// 设置环境变量
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = Some(env);
        self
    }

    /// 设置安装命令
    pub fn with_install_command(mut self, install_command: String) -> Self {
        self.install_command = Some(install_command);
        self
    }

    /// 设置文档 URL
    pub fn with_documentation_url(mut self, documentation_url: String) -> Self {
        self.documentation_url = Some(documentation_url);
        self
    }

    /// 设置 GitHub URL
    pub fn with_github_url(mut self, github_url: String) -> Self {
        self.github_url = Some(github_url);
        self
    }

    /// 设置作者
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// 设置标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// 设置系统要求
    pub fn with_requirements(mut self, requirements: Vec<String>) -> Self {
        self.requirements = Some(requirements);
        self
    }
}

/// MCP 工具市场
pub struct McpMarketplace {
    /// 服务器映射 (id -> listing)
    servers: HashMap<String, McpServerListing>,
}

impl McpMarketplace {
    /// 创建新的工具市场
    pub fn new() -> Self {
        let mut marketplace = Self {
            servers: HashMap::new(),
        };

        // 初始化内置服务器
        marketplace.initialize_builtin_servers();

        marketplace
    }

    /// 初始化内置服务器
    fn initialize_builtin_servers(&mut self) {
        // 示例：文件系统服务器
        let filesystem_server = McpServerListing::new(
            "filesystem".to_string(),
            "Filesystem MCP Server".to_string(),
            "Access and manipulate files and directories".to_string(),
            "System".to_string(),
            "npx".to_string(),
            vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
            ],
        )
        .with_author("Anthropic".to_string())
        .with_tags(vec![
            "filesystem".to_string(),
            "files".to_string(),
            "system".to_string(),
        ])
        .with_documentation_url("https://github.com/modelcontextprotocol/servers".to_string())
        .with_github_url("https://github.com/modelcontextprotocol/servers".to_string());

        self.servers
            .insert(filesystem_server.id.clone(), filesystem_server);

        // 示例：GitHub 服务器
        let github_server = McpServerListing::new(
            "github".to_string(),
            "GitHub MCP Server".to_string(),
            "Interact with GitHub repositories, issues, and pull requests".to_string(),
            "Development".to_string(),
            "npx".to_string(),
            vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
        )
        .with_author("Anthropic".to_string())
        .with_tags(vec![
            "github".to_string(),
            "git".to_string(),
            "development".to_string(),
        ])
        .with_documentation_url("https://github.com/modelcontextprotocol/servers".to_string())
        .with_github_url("https://github.com/modelcontextprotocol/servers".to_string())
        .with_requirements(vec!["GitHub API token".to_string()]);

        self.servers.insert(github_server.id.clone(), github_server);

        // 示例：Brave Search 服务器
        let brave_search_server = McpServerListing::new(
            "brave-search".to_string(),
            "Brave Search MCP Server".to_string(),
            "Web search using Brave Search API".to_string(),
            "Search".to_string(),
            "npx".to_string(),
            vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-brave-search".to_string(),
            ],
        )
        .with_author("Anthropic".to_string())
        .with_tags(vec![
            "search".to_string(),
            "web".to_string(),
            "brave".to_string(),
        ])
        .with_documentation_url("https://github.com/modelcontextprotocol/servers".to_string())
        .with_github_url("https://github.com/modelcontextprotocol/servers".to_string())
        .with_requirements(vec!["Brave Search API key".to_string()]);

        self.servers
            .insert(brave_search_server.id.clone(), brave_search_server);
    }

    /// 获取所有服务器
    pub fn get_all_servers(&self) -> Vec<McpServerListing> {
        self.servers.values().cloned().collect()
    }

    /// 根据 ID 获取服务器
    pub fn get_server(&self, server_id: &str) -> Option<McpServerListing> {
        self.servers.get(server_id).cloned()
    }

    /// 根据分类获取服务器
    pub fn get_by_category(&self, category: &str) -> Vec<McpServerListing> {
        self.servers
            .values()
            .filter(|s| s.category == category)
            .cloned()
            .collect()
    }

    /// 获取所有分类
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .servers
            .values()
            .map(|s| s.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        categories.sort();
        categories
    }

    /// 搜索服务器
    pub fn search(&self, query: &str) -> Vec<McpServerListing> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for server in self.servers.values() {
            // 在名称中搜索
            if server.name.to_lowercase().contains(&query_lower) {
                results.push(server.clone());
                continue;
            }

            // 在描述中搜索
            if server.description.to_lowercase().contains(&query_lower) {
                results.push(server.clone());
                continue;
            }

            // 在标签中搜索
            if let Some(tags) = &server.tags {
                if tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower))
                {
                    results.push(server.clone());
                    continue;
                }
            }
        }

        results
    }

    /// 添加自定义服务器
    pub fn add_server(&mut self, listing: McpServerListing) -> Result<(), String> {
        if self.servers.contains_key(&listing.id) {
            return Err(format!("Server with ID '{}' already exists", listing.id));
        }

        self.servers.insert(listing.id.clone(), listing);
        Ok(())
    }

    /// 移除服务器
    pub fn remove_server(&mut self, server_id: &str) -> Result<McpServerListing, String> {
        self.servers
            .remove(server_id)
            .ok_or_else(|| format!("Server with ID '{server_id}' not found"))
    }

    /// 获取服务器数量
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }
}

impl Default for McpMarketplace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_marketplace() {
        let marketplace = McpMarketplace::new();
        assert!(marketplace.server_count() > 0);
    }

    #[test]
    fn test_get_all_servers() {
        let marketplace = McpMarketplace::new();
        let servers = marketplace.get_all_servers();
        assert!(!servers.is_empty());
    }

    #[test]
    fn test_get_server() {
        let marketplace = McpMarketplace::new();
        let server = marketplace.get_server("filesystem");
        assert!(server.is_some());
        assert_eq!(server.unwrap().id, "filesystem");
    }

    #[test]
    fn test_get_server_not_found() {
        let marketplace = McpMarketplace::new();
        let server = marketplace.get_server("nonexistent");
        assert!(server.is_none());
    }

    #[test]
    fn test_get_by_category() {
        let marketplace = McpMarketplace::new();
        let servers = marketplace.get_by_category("System");
        assert!(!servers.is_empty());
        assert!(servers.iter().all(|s| s.category == "System"));
    }

    #[test]
    fn test_get_categories() {
        let marketplace = McpMarketplace::new();
        let categories = marketplace.get_categories();
        assert!(!categories.is_empty());
        assert!(categories.contains(&"System".to_string()));
    }

    #[test]
    fn test_search_by_name() {
        let marketplace = McpMarketplace::new();
        let results = marketplace.search("filesystem");
        assert!(!results.is_empty());
        assert!(results.iter().any(|s| s.id == "filesystem"));
    }

    #[test]
    fn test_search_by_description() {
        let marketplace = McpMarketplace::new();
        let results = marketplace.search("files");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_by_tag() {
        let marketplace = McpMarketplace::new();
        let results = marketplace.search("github");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_no_results() {
        let marketplace = McpMarketplace::new();
        let results = marketplace.search("nonexistent_xyz_123");
        assert!(results.is_empty());
    }

    #[test]
    fn test_add_custom_server() {
        let mut marketplace = McpMarketplace::new();
        let initial_count = marketplace.server_count();

        let custom_server = McpServerListing::new(
            "custom-server".to_string(),
            "Custom Server".to_string(),
            "A custom MCP server".to_string(),
            "Custom".to_string(),
            "node".to_string(),
            vec!["server.js".to_string()],
        );

        let result = marketplace.add_server(custom_server);
        assert!(result.is_ok());
        assert_eq!(marketplace.server_count(), initial_count + 1);
    }

    #[test]
    fn test_add_duplicate_server() {
        let mut marketplace = McpMarketplace::new();

        let server = McpServerListing::new(
            "filesystem".to_string(),
            "Duplicate".to_string(),
            "Duplicate server".to_string(),
            "Test".to_string(),
            "test".to_string(),
            vec![],
        );

        let result = marketplace.add_server(server);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_server() {
        let mut marketplace = McpMarketplace::new();
        let initial_count = marketplace.server_count();

        let result = marketplace.remove_server("filesystem");
        assert!(result.is_ok());
        assert_eq!(marketplace.server_count(), initial_count - 1);
    }

    #[test]
    fn test_remove_nonexistent_server() {
        let mut marketplace = McpMarketplace::new();
        let result = marketplace.remove_server("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_server_listing_builder() {
        let server = McpServerListing::new(
            "test".to_string(),
            "Test Server".to_string(),
            "A test server".to_string(),
            "Test".to_string(),
            "node".to_string(),
            vec!["server.js".to_string()],
        )
        .with_author("Test Author".to_string())
        .with_tags(vec!["test".to_string(), "example".to_string()])
        .with_documentation_url("https://example.com".to_string());

        assert_eq!(server.author, Some("Test Author".to_string()));
        assert_eq!(
            server.tags,
            Some(vec!["test".to_string(), "example".to_string()])
        );
        assert_eq!(
            server.documentation_url,
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_server_listing_serialization() {
        let server = McpServerListing::new(
            "test".to_string(),
            "Test Server".to_string(),
            "A test server".to_string(),
            "Test".to_string(),
            "node".to_string(),
            vec!["server.js".to_string()],
        );

        let json = serde_json::to_string(&server).unwrap();
        let deserialized: McpServerListing = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "test");
        assert_eq!(deserialized.name, "Test Server");
    }
}
