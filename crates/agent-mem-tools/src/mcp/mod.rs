//! MCP (Model Context Protocol) 支持
//!
//! 提供 MCP 协议的客户端和服务端实现，用于与外部 MCP 服务器通信
//! 以及将 AgentMem 工具暴露为 MCP 服务。

pub mod auth;
pub mod client;
pub mod discovery;
pub mod error;
pub mod logging;
pub mod manager;
pub mod marketplace;
pub mod prompts;
pub mod resources;
pub mod sampling;
pub mod server;
pub mod transport;
pub mod types;

pub use auth::{
    AuditEvent, AuditEventType, AuditLogger, AuthContext, AuthManager, AuthMethod, Credentials,
    JwtConfig, OAuth2Config, Permission, Role,
};
pub use client::McpClient;
pub use discovery::{HttpToolLoader, ToolDiscovery, ToolLoader, ToolMetadata, ToolType};
pub use error::{McpError, McpResult};
pub use logging::{LogEntry, LogFilter, LogLevel, LogStats, LoggingConfig, LoggingManager};
pub use manager::McpClientManager;
pub use marketplace::{McpMarketplace, McpServerListing};
pub use prompts::{
    McpGetPromptRequest, McpGetPromptResponse, McpListPromptsResponse, McpPrompt, PromptArgument,
    PromptContent, PromptManager,
};
pub use resources::{
    CacheStats, McpListResourcesResponse, McpReadResourceRequest, McpReadResourceResponse,
    McpResource, McpSubscribeResourceRequest, McpSubscribeResourceResponse, ResourceChangeEvent,
    ResourceChangeType, ResourceContent, ResourceManager, ResourceSubscription, ResourceTemplate,
    TemplateParameter,
};
pub use sampling::{
    CreateMessageRequest, CreateMessageResponse, SamplingManager, SamplingMessage, SamplingParams,
    SamplingRole, StopReason, StreamChunk, TokenUsage,
};
pub use server::{McpServer, ServerCapabilities, ServerInfo};
pub use transport::{HttpTransport, SseTransport, Transport};
pub use types::{McpServerConfig, McpServerType, McpTool};

#[cfg(test)]
mod server_tests;
