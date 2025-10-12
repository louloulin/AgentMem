//! MCP (Model Context Protocol) 支持
//!
//! 提供 MCP 协议的客户端和服务端实现，用于与外部 MCP 服务器通信
//! 以及将 AgentMem 工具暴露为 MCP 服务。

pub mod types;
pub mod client;
pub mod server;
pub mod manager;
pub mod marketplace;
pub mod error;
pub mod resources;
pub mod prompts;
pub mod transport;
pub mod discovery;
pub mod auth;

pub use types::{McpServerConfig, McpServerType, McpTool};
pub use client::McpClient;
pub use server::{McpServer, ServerInfo, ServerCapabilities};
pub use manager::McpClientManager;
pub use marketplace::{McpMarketplace, McpServerListing};
pub use error::{McpError, McpResult};
pub use resources::{
    McpResource, ResourceContent, ResourceTemplate, TemplateParameter,
    McpListResourcesResponse, McpReadResourceRequest, McpReadResourceResponse,
    McpSubscribeResourceRequest, McpSubscribeResourceResponse,
    ResourceChangeEvent, ResourceChangeType, ResourceSubscription,
    ResourceManager, CacheStats,
};
pub use prompts::{
    McpPrompt, PromptArgument, PromptContent,
    McpListPromptsResponse, McpGetPromptRequest, McpGetPromptResponse,
    PromptManager,
};
pub use transport::{
    Transport, HttpTransport, SseTransport,
};
pub use discovery::{
    ToolDiscovery, ToolMetadata, ToolType, ToolLoader, HttpToolLoader,
};
pub use auth::{
    AuthManager, AuthContext, AuthMethod, Permission, Role, Credentials,
    AuditLogger, AuditEvent, AuditEventType, JwtConfig, OAuth2Config,
};
