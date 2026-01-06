//! MCP 类型定义
//!
//! 定义 MCP 协议相关的数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP 服务器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    /// 标准输入输出传输
    Stdio,
    /// Server-Sent Events 传输
    Sse,
    /// HTTP 传输
    Http,
}

/// MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// 服务器名称
    pub server_name: String,

    /// 服务器类型
    #[serde(rename = "type")]
    pub server_type: McpServerType,

    /// Stdio 配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    /// SSE/HTTP 配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

impl McpServerConfig {
    /// 创建 Stdio 类型的服务器配置
    pub fn stdio(
        server_name: String,
        command: String,
        args: Vec<String>,
        env: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            server_name,
            server_type: McpServerType::Stdio,
            command: Some(command),
            args: Some(args),
            env,
            url: None,
            headers: None,
        }
    }

    /// 创建 SSE 类型的服务器配置
    pub fn sse(server_name: String, url: String, headers: Option<HashMap<String, String>>) -> Self {
        Self {
            server_name,
            server_type: McpServerType::Sse,
            command: None,
            args: None,
            env: None,
            url: Some(url),
            headers,
        }
    }

    /// 创建 HTTP 类型的服务器配置
    pub fn http(
        server_name: String,
        url: String,
        headers: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            server_name,
            server_type: McpServerType::Http,
            command: None,
            args: None,
            env: None,
            url: Some(url),
            headers,
        }
    }
}

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// 工具名称
    pub name: String,

    /// 工具描述
    pub description: String,

    /// 输入参数 schema (JSON Schema 格式)
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallRequest {
    /// 工具名称
    pub name: String,

    /// 工具参数
    pub arguments: serde_json::Value,
}

/// MCP 工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResponse {
    /// 响应内容
    pub content: Vec<McpContent>,

    /// 是否有错误
    #[serde(default)]
    pub is_error: bool,
}

/// MCP 内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    /// 文本内容
    #[serde(rename = "text")]
    Text { text: String },

    /// 图片内容
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },

    /// 资源内容
    #[serde(rename = "resource")]
    Resource {
        uri: String,
        mime_type: Option<String>,
        text: Option<String>,
    },
}

/// MCP 工具列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpListToolsResponse {
    /// 工具列表
    pub tools: Vec<McpTool>,
}

/// MCP 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    /// 服务器名称
    pub name: String,

    /// 服务器类型
    #[serde(rename = "type")]
    pub server_type: String,

    /// 是否已初始化
    pub initialized: bool,

    /// 是否已连接
    pub connected: bool,

    /// 配置信息
    pub config: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_config() {
        let config = McpServerConfig::stdio(
            "test-server".to_string(),
            "python".to_string(),
            vec!["-m".to_string(), "server".to_string()],
            None,
        );

        assert_eq!(config.server_name, "test-server");
        assert_eq!(config.server_type, McpServerType::Stdio);
        assert_eq!(config.command, Some("python".to_string()));
    }

    #[test]
    fn test_sse_config() {
        let config = McpServerConfig::sse(
            "test-sse".to_string(),
            "http://localhost:8080/sse".to_string(),
            None,
        );

        assert_eq!(config.server_name, "test-sse");
        assert_eq!(config.server_type, McpServerType::Sse);
        assert_eq!(config.url, Some("http://localhost:8080/sse".to_string()));
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: McpTool = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_tool");
        assert_eq!(deserialized.description, "A test tool");
    }

    #[test]
    fn test_mcp_content_text() {
        let content = McpContent::Text {
            text: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("Hello, world!"));
    }
}
