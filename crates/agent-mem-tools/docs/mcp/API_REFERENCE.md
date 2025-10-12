# MCP API 参考

本文档提供 AgentMem MCP API 的完整参考。

## 目录

- [服务端 API](#服务端-api)
- [客户端 API](#客户端-api)
- [数据类型](#数据类型)
- [错误类型](#错误类型)
- [协议规范](#协议规范)

## 服务端 API

### McpServer

MCP 服务端主类。

#### 构造函数

```rust
pub fn new(config: McpServerConfig) -> Self
```

创建新的 MCP 服务端实例。

**参数**:
- `config`: 服务端配置

**返回**: `McpServer` 实例

**示例**:
```rust
let config = McpServerConfig {
    name: "my-server".to_string(),
    version: "1.0.0".to_string(),
    ..Default::default()
};
let server = McpServer::new(config);
```

#### 方法

##### register_tool

```rust
pub async fn register_tool(&mut self, tool: McpTool) -> Result<()>
```

注册一个工具。

**参数**:
- `tool`: 要注册的工具

**返回**: `Result<()>`

**示例**:
```rust
server.register_tool(my_tool).await?;
```

##### register_tools

```rust
pub async fn register_tools(&mut self, tools: Vec<McpTool>) -> Result<()>
```

批量注册工具。

**参数**:
- `tools`: 要注册的工具列表

**返回**: `Result<()>`

##### register_resource_provider

```rust
pub async fn register_resource_provider(
    &mut self,
    provider: Box<dyn ResourceProvider>
) -> Result<()>
```

注册资源提供者。

**参数**:
- `provider`: 资源提供者实现

**返回**: `Result<()>`

##### register_prompt_provider

```rust
pub async fn register_prompt_provider(
    &mut self,
    provider: Box<dyn PromptProvider>
) -> Result<()>
```

注册提示词提供者。

**参数**:
- `provider`: 提示词提供者实现

**返回**: `Result<()>`

##### start_stdio

```rust
pub async fn start_stdio(&mut self) -> Result<()>
```

使用 Stdio 传输启动服务端。

**返回**: `Result<()>`

##### start_sse

```rust
pub async fn start_sse(&mut self, config: SseConfig) -> Result<()>
```

使用 SSE 传输启动服务端。

**参数**:
- `config`: SSE 配置

**返回**: `Result<()>`

##### start_http

```rust
pub async fn start_http(&mut self, config: HttpConfig) -> Result<()>
```

使用 HTTP 传输启动服务端。

**参数**:
- `config`: HTTP 配置

**返回**: `Result<()>`

### McpServerConfig

服务端配置结构。

```rust
pub struct McpServerConfig {
    /// 服务端名称
    pub name: String,
    
    /// 服务端版本
    pub version: String,
    
    /// 服务端描述
    pub description: Option<String>,
    
    /// 认证配置
    pub auth: Option<AuthConfig>,
    
    /// 日志级别
    pub log_level: String,
    
    /// 最大并发连接数
    pub max_connections: usize,
    
    /// 请求超时（秒）
    pub request_timeout: u64,
}
```

**默认值**:
```rust
impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "mcp-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            auth: None,
            log_level: "info".to_string(),
            max_connections: 100,
            request_timeout: 30,
        }
    }
}
```

### McpTool

工具定义结构。

```rust
pub struct McpTool {
    /// 工具名称（唯一标识符）
    pub name: String,
    
    /// 工具描述
    pub description: String,
    
    /// 输入参数的 JSON Schema
    pub input_schema: serde_json::Value,
    
    /// 工具处理器
    pub handler: ToolHandler,
}
```

**ToolHandler 类型**:
```rust
pub type ToolHandler = Box<
    dyn Fn(serde_json::Value, &ToolContext) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>>
    + Send
    + Sync
>;
```

### ResourceProvider

资源提供者 trait。

```rust
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    /// 列出所有可用资源
    async fn list_resources(&self) -> Result<Vec<McpResource>>;
    
    /// 读取指定资源
    async fn read_resource(&self, uri: &str) -> Result<String>;
    
    /// 订阅资源更新（可选）
    async fn subscribe_resource(&self, uri: &str) -> Result<ResourceStream> {
        Err("Not supported".into())
    }
}
```

### PromptProvider

提示词提供者 trait。

```rust
#[async_trait]
pub trait PromptProvider: Send + Sync {
    /// 列出所有可用提示词
    async fn list_prompts(&self) -> Result<Vec<McpPrompt>>;
    
    /// 获取指定提示词
    async fn get_prompt(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<Vec<PromptMessage>>;
}
```

## 客户端 API

### McpClient

MCP 客户端主类。

#### 构造函数

```rust
pub fn new(config: McpClientConfig) -> Self
```

创建新的 MCP 客户端实例。

**参数**:
- `config`: 客户端配置

**返回**: `McpClient` 实例

#### 方法

##### connect

```rust
pub async fn connect(&mut self) -> Result<()>
```

连接到 MCP 服务端。

**返回**: `Result<()>`

##### disconnect

```rust
pub async fn disconnect(&mut self) -> Result<()>
```

断开与服务端的连接。

**返回**: `Result<()>`

##### list_tools

```rust
pub async fn list_tools(&self) -> Result<Vec<McpTool>>
```

列出所有可用工具。

**返回**: `Result<Vec<McpTool>>`

##### call_tool

```rust
pub async fn call_tool(
    &self,
    name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value>
```

调用指定工具。

**参数**:
- `name`: 工具名称
- `arguments`: 工具参数

**返回**: `Result<serde_json::Value>` - 工具执行结果

##### list_resources

```rust
pub async fn list_resources(&self) -> Result<Vec<McpResource>>
```

列出所有可用资源。

**返回**: `Result<Vec<McpResource>>`

##### read_resource

```rust
pub async fn read_resource(&self, uri: &str) -> Result<String>
```

读取指定资源。

**参数**:
- `uri`: 资源 URI

**返回**: `Result<String>` - 资源内容

##### list_prompts

```rust
pub async fn list_prompts(&self) -> Result<Vec<McpPrompt>>
```

列出所有可用提示词。

**返回**: `Result<Vec<McpPrompt>>`

##### get_prompt

```rust
pub async fn get_prompt(
    &self,
    name: &str,
    arguments: serde_json::Value,
) -> Result<Vec<PromptMessage>>
```

获取指定提示词。

**参数**:
- `name`: 提示词名称
- `arguments`: 提示词参数

**返回**: `Result<Vec<PromptMessage>>` - 提示词消息列表

### McpClientConfig

客户端配置结构。

```rust
pub struct McpClientConfig {
    /// 服务端命令（Stdio 传输）
    pub server_command: String,
    
    /// 服务端参数
    pub server_args: Vec<String>,
    
    /// 服务端 URL（SSE/HTTP 传输）
    pub server_url: String,
    
    /// 传输类型
    pub transport: TransportType,
    
    /// 认证配置
    pub auth: Option<AuthConfig>,
    
    /// 连接超时（秒）
    pub connect_timeout: u64,
    
    /// 请求超时（秒）
    pub request_timeout: u64,
    
    /// 重连策略
    pub reconnect: ReconnectConfig,
}
```

## 数据类型

### McpResource

资源定义结构。

```rust
pub struct McpResource {
    /// 资源 URI
    pub uri: String,
    
    /// 资源名称
    pub name: String,
    
    /// 资源描述
    pub description: Option<String>,
    
    /// MIME 类型
    pub mime_type: Option<String>,
    
    /// 元数据
    pub metadata: Option<serde_json::Value>,
}
```

### McpPrompt

提示词定义结构。

```rust
pub struct McpPrompt {
    /// 提示词名称
    pub name: String,
    
    /// 提示词描述
    pub description: Option<String>,
    
    /// 参数定义
    pub arguments: Vec<serde_json::Value>,
}
```

### PromptMessage

提示词消息结构。

```rust
pub struct PromptMessage {
    /// 角色（system, user, assistant）
    pub role: String,
    
    /// 消息内容
    pub content: String,
}
```

### ToolContext

工具执行上下文。

```rust
pub struct ToolContext {
    /// 用户 ID
    pub user_id: String,
    
    /// 会话 ID
    pub session_id: String,
    
    /// 请求 ID
    pub request_id: String,
    
    /// 元数据
    pub metadata: HashMap<String, String>,
}
```

### AuthConfig

认证配置结构。

```rust
pub struct AuthConfig {
    /// 认证方法
    pub method: AuthMethod,
    
    /// API 密钥
    pub api_key: Option<String>,
    
    /// OAuth 客户端 ID
    pub oauth_client_id: Option<String>,
    
    /// OAuth 客户端密钥
    pub oauth_client_secret: Option<String>,
    
    /// OAuth 令牌 URL
    pub oauth_token_url: Option<String>,
}
```

### AuthMethod

认证方法枚举。

```rust
pub enum AuthMethod {
    /// 无认证
    None,
    
    /// API 密钥认证
    ApiKey,
    
    /// OAuth 2.0 认证
    OAuth2,
    
    /// 自定义认证
    Custom(String),
}
```

### TransportType

传输类型枚举。

```rust
pub enum TransportType {
    /// 标准输入/输出
    Stdio,
    
    /// Server-Sent Events
    Sse,
    
    /// HTTP
    Http,
}
```

## 错误类型

### McpError

MCP 错误类型。

```rust
pub enum McpError {
    /// 连接错误
    ConnectionError(String),
    
    /// 认证错误
    AuthenticationError(String),
    
    /// 工具未找到
    ToolNotFound(String),
    
    /// 资源未找到
    ResourceNotFound(String),
    
    /// 提示词未找到
    PromptNotFound(String),
    
    /// 无效参数
    InvalidArguments(String),
    
    /// 超时错误
    Timeout(String),
    
    /// 内部错误
    InternalError(String),
}
```

## 协议规范

### 请求格式

所有 MCP 请求使用 JSON-RPC 2.0 格式：

```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "method": "tools/call",
  "params": {
    "name": "calculator",
    "arguments": {
      "expression": "2 + 2"
    }
  }
}
```

### 响应格式

成功响应：

```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "result": {
    "result": 4
  }
}
```

错误响应：

```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": {
      "details": "Missing required field: expression"
    }
  }
}
```

### 方法列表

#### 工具相关

- `tools/list`: 列出所有工具
- `tools/call`: 调用工具

#### 资源相关

- `resources/list`: 列出所有资源
- `resources/read`: 读取资源
- `resources/subscribe`: 订阅资源更新

#### 提示词相关

- `prompts/list`: 列出所有提示词
- `prompts/get`: 获取提示词

#### 系统相关

- `initialize`: 初始化连接
- `ping`: 心跳检测

### 错误代码

| 代码 | 消息 | 说明 |
|------|------|------|
| -32700 | Parse error | JSON 解析错误 |
| -32600 | Invalid Request | 无效的请求 |
| -32601 | Method not found | 方法未找到 |
| -32602 | Invalid params | 无效的参数 |
| -32603 | Internal error | 内部错误 |
| -32000 | Server error | 服务端错误 |
| -32001 | Tool not found | 工具未找到 |
| -32002 | Resource not found | 资源未找到 |
| -32003 | Prompt not found | 提示词未找到 |
| -32004 | Authentication failed | 认证失败 |
| -32005 | Permission denied | 权限拒绝 |
| -32006 | Timeout | 超时 |

## 示例

### 完整的服务端示例

```rust
use agent_mem_tools::mcp::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = McpServerConfig {
        name: "example-server".to_string(),
        version: "1.0.0".to_string(),
        ..Default::default()
    };
    
    let mut server = McpServer::new(config);
    
    server.register_tool(McpTool {
        name: "echo".to_string(),
        description: "回显输入".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "message": {"type": "string"}
            },
            "required": ["message"]
        }),
        handler: Box::new(|args, _ctx| {
            Box::pin(async move {
                Ok(serde_json::json!({
                    "echo": args["message"]
                }))
            })
        }),
    }).await?;
    
    server.start_stdio().await?;
    
    Ok(())
}
```

### 完整的客户端示例

```rust
use agent_mem_tools::mcp::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = McpClientConfig {
        server_command: "example-server".to_string(),
        server_args: vec![],
        ..Default::default()
    };
    
    let mut client = McpClient::new(config);
    client.connect().await?;
    
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);
    
    let result = client.call_tool(
        "echo",
        serde_json::json!({
            "message": "Hello, MCP!"
        })
    ).await?;
    
    println!("Result: {:?}", result);
    
    client.disconnect().await?;
    
    Ok(())
}
```

## 参考资料

- [MCP 快速开始](QUICKSTART.md)
- [MCP 服务端开发指南](SERVER_GUIDE.md)
- [MCP 客户端开发指南](CLIENT_GUIDE.md)
- [MCP 最佳实践](BEST_PRACTICES.md)

