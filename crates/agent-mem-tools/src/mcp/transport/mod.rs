//! MCP 传输层
//!
//! 提供多种传输协议的实现：
//! - HTTP: 标准 HTTP/HTTPS 传输
//! - SSE: Server-Sent Events 流式传输
//! - Stdio: 标准输入输出传输

pub mod http;
pub mod sse;

pub use http::HttpTransport;
pub use sse::SseTransport;

use super::error::{McpError, McpResult};
use async_trait::async_trait;
use serde_json::Value;

/// 传输层 trait
#[async_trait]
pub trait Transport: Send + Sync {
    /// 连接到服务器
    async fn connect(&mut self) -> McpResult<()>;
    
    /// 发送请求
    async fn send_request(&self, method: &str, params: Value) -> McpResult<Value>;
    
    /// 断开连接
    async fn disconnect(&mut self) -> McpResult<()>;
    
    /// 检查是否已连接
    fn is_connected(&self) -> bool;
}

