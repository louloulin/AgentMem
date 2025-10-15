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
    
    /// 序列化错误
    SerializationError(String),

    /// 反序列化错误
    DeserializationError(String),

    /// 协议错误
    ProtocolError(String),

    /// 服务器错误
    ServerError(String),

    /// 不支持的服务器类型
    UnsupportedServerType(String),
    
    /// 资源未找到
    ResourceNotFound(String),

    /// 订阅未找到
    SubscriptionNotFound(String),

    /// 工具未找到
    ToolNotFound(String),

    /// 工具加载错误
    ToolLoadError(String),

    /// 传输错误
    TransportError(String),

    /// 未连接
    NotConnected,

    /// 认证失败
    AuthenticationFailed(String),

    /// 权限拒绝
    PermissionDenied(String),

    /// 内部错误
    Internal(String),

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
            McpError::DeserializationError(msg) => write!(f, "MCP deserialization error: {}", msg),
            McpError::ProtocolError(msg) => write!(f, "MCP protocol error: {}", msg),
            McpError::ServerError(msg) => write!(f, "MCP server error: {}", msg),
            McpError::UnsupportedServerType(server_type) => {
                write!(f, "Unsupported MCP server type: {}", server_type)
            }
            McpError::ResourceNotFound(uri) => {
                write!(f, "MCP resource not found: {}", uri)
            }
            McpError::SubscriptionNotFound(id) => {
                write!(f, "MCP subscription not found: {}", id)
            }
            McpError::ToolNotFound(name) => {
                write!(f, "MCP tool not found: {}", name)
            }
            McpError::ToolLoadError(name) => {
                write!(f, "MCP tool load error: {}", name)
            }
            McpError::TransportError(msg) => {
                write!(f, "MCP transport error: {}", msg)
            }
            McpError::NotConnected => {
                write!(f, "MCP not connected")
            }
            McpError::AuthenticationFailed(msg) => {
                write!(f, "MCP authentication failed: {}", msg)
            }
            McpError::PermissionDenied(msg) => {
                write!(f, "MCP permission denied: {}", msg)
            }
            McpError::Internal(msg) => {
                write!(f, "MCP internal error: {}", msg)
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

