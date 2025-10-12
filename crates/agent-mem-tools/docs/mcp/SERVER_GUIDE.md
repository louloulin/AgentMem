# MCP 服务端开发指南

本指南介绍如何使用 AgentMem 开发 MCP (Model Context Protocol) 服务端。

## 目录

- [概述](#概述)
- [快速开始](#快速开始)
- [核心概念](#核心概念)
- [实现工具](#实现工具)
- [实现资源](#实现资源)
- [实现提示词](#实现提示词)
- [传输层](#传输层)
- [安全和认证](#安全和认证)
- [最佳实践](#最佳实践)
- [示例](#示例)

## 概述

MCP 服务端负责：
- 提供工具（Tools）供 AI 模型调用
- 提供资源（Resources）供 AI 模型访问
- 提供提示词（Prompts）模板
- 处理客户端请求
- 管理会话和状态

### 架构

```
┌─────────────────────────────────────────────────────────┐
│                    MCP 服务端                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   工具管理    │  │   资源管理    │  │  提示词管理   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                 │           │
│         └─────────────────┴─────────────────┘           │
│                          │                              │
│                  ┌───────▼────────┐                     │
│                  │   传输层        │                     │
│                  │ (Stdio/SSE/HTTP)│                     │
│                  └───────┬────────┘                     │
└──────────────────────────┼──────────────────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  MCP 客户端   │
                    └──────────────┘
```

## 快速开始

### 1. 添加依赖

```toml
[dependencies]
agent-mem-tools = { path = "../../crates/agent-mem-tools" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2. 创建基本服务端

```rust
use agent_mem_tools::mcp::{McpServer, McpServerConfig, McpTool, ToolContext};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建服务端配置
    let config = McpServerConfig {
        name: "my-mcp-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("My MCP Server".to_string()),
        ..Default::default()
    };
    
    // 创建服务端
    let mut server = McpServer::new(config);
    
    // 注册工具
    server.register_tool(create_calculator_tool()).await?;
    
    // 启动服务端（Stdio 传输）
    server.start_stdio().await?;
    
    Ok(())
}

fn create_calculator_tool() -> McpTool {
    McpTool {
        name: "calculator".to_string(),
        description: "执行数学计算".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式，例如: 2 + 2"
                }
            },
            "required": ["expression"]
        }),
        handler: Box::new(|args: Value, _ctx: &ToolContext| {
            Box::pin(async move {
                let expression = args["expression"].as_str()
                    .ok_or("Missing expression")?;
                
                // 实现计算逻辑
                let result = evaluate_expression(expression)?;
                
                Ok(serde_json::json!({
                    "result": result
                }))
            })
        }),
    }
}
```

## 核心概念

### 工具 (Tools)

工具是 AI 模型可以调用的函数。每个工具包含：

- **名称**: 唯一标识符
- **描述**: 工具的功能说明
- **输入模式**: JSON Schema 定义的参数
- **处理器**: 实际执行逻辑的异步函数

### 资源 (Resources)

资源是 AI 模型可以访问的数据源。每个资源包含：

- **URI**: 唯一资源标识符（例如：`file:///path/to/file`）
- **名称**: 人类可读的名称
- **描述**: 资源的说明
- **MIME 类型**: 资源的内容类型
- **内容**: 实际的资源数据

### 提示词 (Prompts)

提示词是预定义的模板，可以包含变量。每个提示词包含：

- **名称**: 唯一标识符
- **描述**: 提示词的用途
- **参数**: 模板变量定义
- **消息**: 提示词内容（支持变量替换）

## 实现工具

### 基本工具

```rust
use agent_mem_tools::mcp::{McpTool, ToolContext};
use serde_json::Value;

fn create_echo_tool() -> McpTool {
    McpTool {
        name: "echo".to_string(),
        description: "回显输入的文本".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "要回显的消息"
                }
            },
            "required": ["message"]
        }),
        handler: Box::new(|args: Value, _ctx: &ToolContext| {
            Box::pin(async move {
                let message = args["message"].as_str()
                    .ok_or("Missing message")?;
                
                Ok(serde_json::json!({
                    "echo": message
                }))
            })
        }),
    }
}
```

### 带状态的工具

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct CounterState {
    count: Arc<RwLock<i64>>,
}

fn create_counter_tool(state: Arc<CounterState>) -> McpTool {
    McpTool {
        name: "counter".to_string(),
        description: "计数器工具".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["increment", "decrement", "get"],
                    "description": "操作类型"
                }
            },
            "required": ["action"]
        }),
        handler: Box::new(move |args: Value, _ctx: &ToolContext| {
            let state = state.clone();
            Box::pin(async move {
                let action = args["action"].as_str()
                    .ok_or("Missing action")?;
                
                let mut count = state.count.write().await;
                
                match action {
                    "increment" => *count += 1,
                    "decrement" => *count -= 1,
                    "get" => {},
                    _ => return Err("Invalid action".into()),
                }
                
                Ok(serde_json::json!({
                    "count": *count
                }))
            })
        }),
    }
}
```

### 异步 I/O 工具

```rust
use tokio::fs;

fn create_file_reader_tool() -> McpTool {
    McpTool {
        name: "read_file".to_string(),
        description: "读取文件内容".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "文件路径"
                }
            },
            "required": ["path"]
        }),
        handler: Box::new(|args: Value, _ctx: &ToolContext| {
            Box::pin(async move {
                let path = args["path"].as_str()
                    .ok_or("Missing path")?;
                
                let content = fs::read_to_string(path).await
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                
                Ok(serde_json::json!({
                    "content": content,
                    "size": content.len()
                }))
            })
        }),
    }
}
```

## 实现资源

### 静态资源

```rust
use agent_mem_tools::mcp::{McpResource, ResourceProvider};

struct StaticResourceProvider;

#[async_trait::async_trait]
impl ResourceProvider for StaticResourceProvider {
    async fn list_resources(&self) -> Result<Vec<McpResource>, Box<dyn std::error::Error>> {
        Ok(vec![
            McpResource {
                uri: "config://app.json".to_string(),
                name: "应用配置".to_string(),
                description: Some("应用程序配置文件".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ])
    }
    
    async fn read_resource(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        match uri {
            "config://app.json" => {
                Ok(serde_json::json!({
                    "app_name": "My App",
                    "version": "1.0.0"
                }).to_string())
            },
            _ => Err("Resource not found".into()),
        }
    }
}
```

### 动态资源

```rust
use tokio::fs;
use std::path::PathBuf;

struct FileSystemResourceProvider {
    base_path: PathBuf,
}

#[async_trait::async_trait]
impl ResourceProvider for FileSystemResourceProvider {
    async fn list_resources(&self) -> Result<Vec<McpResource>, Box<dyn std::error::Error>> {
        let mut resources = Vec::new();
        
        let mut entries = fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let uri = format!("file://{}", path.display());
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                resources.push(McpResource {
                    uri,
                    name,
                    description: None,
                    mime_type: Some("text/plain".to_string()),
                });
            }
        }
        
        Ok(resources)
    }
    
    async fn read_resource(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(path) = uri.strip_prefix("file://") {
            let content = fs::read_to_string(path).await?;
            Ok(content)
        } else {
            Err("Invalid URI".into())
        }
    }
}
```

## 实现提示词

### 基本提示词

```rust
use agent_mem_tools::mcp::{McpPrompt, PromptMessage, PromptProvider};

struct BasicPromptProvider;

#[async_trait::async_trait]
impl PromptProvider for BasicPromptProvider {
    async fn list_prompts(&self) -> Result<Vec<McpPrompt>, Box<dyn std::error::Error>> {
        Ok(vec![
            McpPrompt {
                name: "code_review".to_string(),
                description: Some("代码审查提示词".to_string()),
                arguments: vec![
                    serde_json::json!({
                        "name": "code",
                        "description": "要审查的代码",
                        "required": true
                    }),
                    serde_json::json!({
                        "name": "language",
                        "description": "编程语言",
                        "required": false
                    }),
                ],
            },
        ])
    }
    
    async fn get_prompt(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<Vec<PromptMessage>, Box<dyn std::error::Error>> {
        match name {
            "code_review" => {
                let code = arguments["code"].as_str()
                    .ok_or("Missing code argument")?;
                let language = arguments["language"].as_str()
                    .unwrap_or("unknown");
                
                Ok(vec![
                    PromptMessage {
                        role: "system".to_string(),
                        content: "你是一个专业的代码审查员。".to_string(),
                    },
                    PromptMessage {
                        role: "user".to_string(),
                        content: format!(
                            "请审查以下 {} 代码：\n\n```{}\n{}\n```",
                            language, language, code
                        ),
                    },
                ])
            },
            _ => Err("Prompt not found".into()),
        }
    }
}
```

## 传输层

### Stdio 传输

```rust
// 使用标准输入/输出
server.start_stdio().await?;
```

### SSE 传输

```rust
// 使用 Server-Sent Events
let config = SseConfig {
    host: "127.0.0.1".to_string(),
    port: 8080,
};
server.start_sse(config).await?;
```

### HTTP 传输

```rust
// 使用 HTTP
let config = HttpConfig {
    host: "127.0.0.1".to_string(),
    port: 8080,
};
server.start_http(config).await?;
```

## 安全和认证

### API 密钥认证

```rust
use agent_mem_tools::mcp::{AuthConfig, AuthMethod};

let config = McpServerConfig {
    name: "secure-server".to_string(),
    version: "1.0.0".to_string(),
    auth: Some(AuthConfig {
        method: AuthMethod::ApiKey,
        api_key: Some("your-secret-key".to_string()),
        ..Default::default()
    }),
    ..Default::default()
};
```

### OAuth 2.0 认证

```rust
let config = McpServerConfig {
    name: "oauth-server".to_string(),
    version: "1.0.0".to_string(),
    auth: Some(AuthConfig {
        method: AuthMethod::OAuth2,
        oauth_client_id: Some("your-client-id".to_string()),
        oauth_client_secret: Some("your-client-secret".to_string()),
        oauth_token_url: Some("https://oauth.example.com/token".to_string()),
        ..Default::default()
    }),
    ..Default::default()
};
```

## 最佳实践

### 1. 错误处理

```rust
handler: Box::new(|args: Value, _ctx: &ToolContext| {
    Box::pin(async move {
        // 验证输入
        let input = args["input"].as_str()
            .ok_or_else(|| "Missing required field: input")?;
        
        // 执行操作
        let result = perform_operation(input)
            .await
            .map_err(|e| format!("Operation failed: {}", e))?;
        
        // 返回结果
        Ok(serde_json::json!({
            "success": true,
            "result": result
        }))
    })
}),
```

### 2. 日志记录

```rust
use tracing::{info, error};

handler: Box::new(|args: Value, ctx: &ToolContext| {
    Box::pin(async move {
        info!("Tool called with args: {:?}", args);
        
        match perform_operation(&args).await {
            Ok(result) => {
                info!("Tool succeeded");
                Ok(result)
            },
            Err(e) => {
                error!("Tool failed: {}", e);
                Err(e)
            }
        }
    })
}),
```

### 3. 超时处理

```rust
use tokio::time::{timeout, Duration};

handler: Box::new(|args: Value, _ctx: &ToolContext| {
    Box::pin(async move {
        let operation = perform_long_operation(&args);
        
        match timeout(Duration::from_secs(30), operation).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(format!("Operation failed: {}", e).into()),
            Err(_) => Err("Operation timed out".into()),
        }
    })
}),
```

### 4. 资源清理

```rust
impl Drop for MyServer {
    fn drop(&mut self) {
        // 清理资源
        info!("Cleaning up server resources");
    }
}
```

## 示例

完整的服务端示例请参考：
- `examples/mcp-resources-demo/`
- `examples/mcp-prompts-demo/`
- `examples/mcp-transport-demo/`

## 下一步

- 阅读 [MCP 客户端开发指南](CLIENT_GUIDE.md)
- 阅读 [MCP 最佳实践](BEST_PRACTICES.md)
- 查看 [MCP API 参考](API_REFERENCE.md)

