# MCP 快速开始指南

本指南将帮助您在 10 分钟内开始使用 AgentMem 的 Model Context Protocol (MCP) 功能。

## 什么是 MCP？

Model Context Protocol (MCP) 是一个开放协议，用于在 AI 应用和外部数据源/工具之间建立标准化的通信。AgentMem 实现了完整的 MCP 规范，提供：

- **工具调用** (Tools): AI 可以调用外部函数和 API
- **资源访问** (Resources): AI 可以访问文件、数据库等资源
- **提示词模板** (Prompts): 预定义的提示词模板
- **采样** (Sampling): AI 模型推理能力
- **日志记录** (Logging): 结构化日志和调试

## 前置要求

- Rust 1.70+
- Tokio 异步运行时
- 基本的 Rust 编程知识

## 安装

在您的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
agent-mem-tools = { path = "../../crates/agent-mem-tools" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 5 分钟快速开始

### 步骤 1: 创建 MCP 服务器

```rust
use agent_mem_tools::mcp::{McpServer, McpServerConfig, McpTool};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 MCP 服务器
    let config = McpServerConfig {
        name: "my-mcp-server".to_string(),
        version: "1.0.0".to_string(),
        ..Default::default()
    };
    
    let server = McpServer::new(config);
    
    // 注册一个简单的工具
    let calculator = McpTool {
        name: "calculator".to_string(),
        description: "执行基本的数学计算".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": { "type": "number" },
                "b": { "type": "number" }
            },
            "required": ["operation", "a", "b"]
        }),
    };
    
    server.register_tool(calculator).await?;
    
    // 启动服务器
    server.start("127.0.0.1:3000").await?;
    
    println!("MCP 服务器已启动在 http://127.0.0.1:3000");
    
    Ok(())
}
```

### 步骤 2: 实现工具处理器

```rust
use agent_mem_tools::mcp::{McpServer, ToolHandler};
use serde_json::{json, Value};
use async_trait::async_trait;

struct CalculatorHandler;

#[async_trait]
impl ToolHandler for CalculatorHandler {
    async fn handle(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let operation = params["operation"].as_str().unwrap();
        let a = params["a"].as_f64().unwrap();
        let b = params["b"].as_f64().unwrap();
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => a / b,
            _ => return Err("未知操作".into()),
        };
        
        Ok(json!({ "result": result }))
    }
}

// 注册处理器
server.register_handler("calculator", Box::new(CalculatorHandler)).await?;
```

### 步骤 3: 创建 MCP 客户端

```rust
use agent_mem_tools::mcp::{McpClient, McpClientConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 MCP 客户端
    let config = McpClientConfig {
        server_url: "http://127.0.0.1:3000".to_string(),
        ..Default::default()
    };
    
    let client = McpClient::new(config).await?;
    
    // 列出可用工具
    let tools = client.list_tools().await?;
    println!("可用工具: {:?}", tools);
    
    // 调用工具
    let result = client.call_tool(
        "calculator",
        json!({
            "operation": "add",
            "a": 10,
            "b": 20
        })
    ).await?;
    
    println!("计算结果: {:?}", result);
    // 输出: 计算结果: {"result": 30}
    
    Ok(())
}
```

### 步骤 4: 使用资源

```rust
use agent_mem_tools::mcp::{McpResource, ResourceUri};

// 注册资源
let file_resource = McpResource {
    uri: ResourceUri::parse("file:///data/users.json")?,
    name: "用户数据".to_string(),
    description: Some("用户信息数据库".to_string()),
    mime_type: Some("application/json".to_string()),
};

server.register_resource(file_resource).await?;

// 客户端读取资源
let content = client.read_resource("file:///data/users.json").await?;
println!("资源内容: {}", content);
```

### 步骤 5: 使用提示词模板

```rust
use agent_mem_tools::mcp::{McpPrompt, PromptArgument};

// 注册提示词模板
let prompt = McpPrompt {
    name: "code_review".to_string(),
    description: Some("代码审查提示词".to_string()),
    arguments: vec![
        PromptArgument {
            name: "code".to_string(),
            description: Some("要审查的代码".to_string()),
            required: true,
        },
        PromptArgument {
            name: "language".to_string(),
            description: Some("编程语言".to_string()),
            required: false,
        },
    ],
};

server.register_prompt(prompt).await?;

// 客户端获取提示词
let prompt_result = client.get_prompt(
    "code_review",
    json!({
        "code": "fn main() { println!(\"Hello\"); }",
        "language": "rust"
    })
).await?;

println!("提示词: {}", prompt_result.messages[0].content);
```

## 运行示例

AgentMem 提供了多个完整的 MCP 示例：

```bash
# 工具示例
cargo run --example mcp-tools-demo

# 资源示例
cargo run --example mcp-resources-demo

# 提示词示例
cargo run --example mcp-prompts-demo

# 传输层示例
cargo run --example mcp-transport-demo

# 认证示例
cargo run --example mcp-auth-demo

# 采样示例
cargo run --example mcp-sampling-demo

# 日志示例
cargo run --example mcp-logging-demo
```

## 核心概念

### 1. 工具 (Tools)

工具是 AI 可以调用的函数。每个工具包含：
- **名称**: 唯一标识符
- **描述**: 工具的功能说明
- **输入模式**: JSON Schema 定义的参数
- **处理器**: 实际执行逻辑

### 2. 资源 (Resources)

资源是 AI 可以访问的数据源。支持：
- 文件系统 (`file://`)
- HTTP/HTTPS (`http://`, `https://`)
- 数据库 (`db://`)
- 自定义协议

### 3. 提示词 (Prompts)

提示词模板允许预定义常用的提示词，支持：
- 参数化模板
- 多消息对话
- 模板继承

### 4. 传输层

支持多种传输协议：
- **stdio**: 标准输入/输出
- **HTTP**: RESTful API
- **SSE**: Server-Sent Events
- **WebSocket**: 双向通信

### 5. 安全和认证

支持多种认证方式：
- API Key
- OAuth 2.0
- JWT
- 自定义认证

## 最佳实践

### 1. 工具设计

```rust
// ✅ 好的工具设计
let good_tool = McpTool {
    name: "search_users".to_string(),
    description: "根据条件搜索用户".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "搜索关键词" },
            "limit": { "type": "integer", "default": 10, "minimum": 1, "maximum": 100 }
        },
        "required": ["query"]
    }),
};

// ❌ 不好的工具设计
let bad_tool = McpTool {
    name: "do_stuff".to_string(),  // 名称不清晰
    description: "做一些事情".to_string(),  // 描述不明确
    input_schema: json!({}),  // 没有参数定义
};
```

### 2. 错误处理

```rust
use agent_mem_tools::mcp::McpError;

async fn handle_tool(params: Value) -> Result<Value, McpError> {
    // 验证参数
    let query = params["query"]
        .as_str()
        .ok_or(McpError::InvalidParams("缺少 query 参数".to_string()))?;
    
    // 执行操作
    let result = search_database(query)
        .await
        .map_err(|e| McpError::InternalError(e.to_string()))?;
    
    Ok(json!({ "results": result }))
}
```

### 3. 资源缓存

```rust
use agent_mem_tools::mcp::ResourceCache;

// 启用资源缓存
let cache = ResourceCache::new(1000, Duration::from_secs(300));
server.set_resource_cache(cache).await?;
```

## 下一步

- 阅读 [MCP 服务端开发指南](SERVER.md)
- 阅读 [MCP 客户端开发指南](CLIENT.md)
- 阅读 [MCP 最佳实践](BEST_PRACTICES.md)
- 阅读 [MCP API 参考](API_REFERENCE.md)
- 查看 [示例代码](../../examples/)

## 常见问题

### Q: MCP 和 REST API 有什么区别？

A: MCP 是专门为 AI 应用设计的协议，提供了工具调用、资源访问、提示词模板等 AI 特定的功能。REST API 是通用的 Web API 协议。

### Q: 如何调试 MCP 应用？

A: 使用 MCP 日志功能：

```rust
use agent_mem_tools::mcp::McpLogger;

let logger = McpLogger::new();
server.set_logger(logger).await?;

// 日志会自动记录所有 MCP 消息
```

### Q: MCP 支持哪些编程语言？

A: AgentMem 的 MCP 实现是用 Rust 编写的，但 MCP 协议本身是语言无关的。您可以使用任何支持 JSON-RPC 的语言实现 MCP 客户端。

### Q: 如何处理大文件资源？

A: 使用流式传输：

```rust
let stream = client.read_resource_stream("file:///large-file.dat").await?;

while let Some(chunk) = stream.next().await {
    process_chunk(chunk?).await?;
}
```

## 支持

- [GitHub Issues](https://github.com/your-org/agentmem/issues)
- [文档](https://docs.agentmem.dev)
- [示例代码](../../examples/)

