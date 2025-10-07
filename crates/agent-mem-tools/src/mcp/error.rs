//! MCP 错误类型定义

use std::fmt;

/// MCP 操作结果类型
pub type McpResult<T> = Result<T, McpError>;

/// MCP 错误类型
#[derive(Debug, Clone)]
pub enum McpError {
    /// 连接错误
    ConnectionError(String),
    
    /// 超时错误
    TimeoutError {
        operation: String,
        server_name: String,
        timeout_seconds: u64,
    },
    
    /// 未初始化错误
    NotInitializedError(String),
    
    /// 工具执行错误
    ToolExecutionError {
        tool_name: String,
        message: String,
    },
    
    /// 配置错误
    ConfigError(String),
    
    /// 序列化/反序列化错误
    SerializationError(String),
    
    /// 不支持的服务器类型
    UnsupportedServerType(String),
    
    /// 其他错误
    Other(String),
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::ConnectionError(msg) => write!(f, "MCP connection error: {}", msg),
            McpError::TimeoutError { operation, server_name, timeout_seconds } => {
                write!(
                    f,
                    "MCP timeout error: {} for server '{}' after {} seconds",
                    operation, server_name, timeout_seconds
                )
            }
            McpError::NotInitializedError(server) => {
                write!(f, "MCP server '{}' not initialized", server)
            }
            McpError::ToolExecutionError { tool_name, message } => {
                write!(f, "MCP tool '{}' execution error: {}", tool_name, message)
            }
            McpError::ConfigError(msg) => write!(f, "MCP configuration error: {}", msg),
            McpError::SerializationError(msg) => write!(f, "MCP serialization error: {}", msg),
            McpError::UnsupportedServerType(server_type) => {
                write!(f, "Unsupported MCP server type: {}", server_type)
            }
            McpError::Other(msg) => write!(f, "MCP error: {}", msg),
        }
    }
}

impl std::error::Error for McpError {}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        McpError::ConnectionError(err.to_string())
    }
}

