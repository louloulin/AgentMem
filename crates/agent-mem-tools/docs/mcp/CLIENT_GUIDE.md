# MCP 客户端开发指南

本指南介绍如何使用 AgentMem 开发 MCP (Model Context Protocol) 客户端。

## 目录

- [概述](#概述)
- [快速开始](#快速开始)
- [核心概念](#核心概念)
- [调用工具](#调用工具)
- [访问资源](#访问资源)
- [使用提示词](#使用提示词)
- [传输层](#传输层)
- [错误处理](#错误处理)
- [最佳实践](#最佳实践)
- [示例](#示例)

## 概述

MCP 客户端负责：
- 连接到 MCP 服务端
- 发现可用的工具、资源和提示词
- 调用工具执行任务
- 访问资源获取数据
- 使用提示词模板
- 处理响应和错误

### 架构

```
┌─────────────────────────────────────────────────────────┐
│                    MCP 客户端                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   工具调用    │  │   资源访问    │  │  提示词使用   │  │
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
                    │  MCP 服务端   │
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

### 2. 创建基本客户端

```rust
use agent_mem_tools::mcp::{McpClient, McpClientConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端配置
    let config = McpClientConfig {
        server_command: "my-mcp-server".to_string(),
        server_args: vec![],
        ..Default::default()
    };
    
    // 创建客户端
    let mut client = McpClient::new(config);
    
    // 连接到服务端
    client.connect().await?;
    
    // 列出可用工具
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);
    
    // 调用工具
    let result = client.call_tool(
        "calculator",
        json!({
            "expression": "2 + 2"
        })
    ).await?;
    println!("Result: {:?}", result);
    
    // 断开连接
    client.disconnect().await?;
    
    Ok(())
}
```

## 核心概念

### 连接管理

客户端需要管理与服务端的连接：

- **连接**: 建立与服务端的通信
- **心跳**: 保持连接活跃
- **重连**: 处理连接断开
- **断开**: 优雅地关闭连接

### 工具发现

客户端可以发现服务端提供的工具：

```rust
// 列出所有工具
let tools = client.list_tools().await?;

// 获取工具详情
let tool = client.get_tool("calculator").await?;
println!("Tool: {}", tool.name);
println!("Description: {}", tool.description);
println!("Schema: {}", tool.input_schema);
```

### 资源发现

客户端可以发现服务端提供的资源：

```rust
// 列出所有资源
let resources = client.list_resources().await?;

// 读取资源
let content = client.read_resource("file:///path/to/file").await?;
```

### 提示词发现

客户端可以发现服务端提供的提示词：

```rust
// 列出所有提示词
let prompts = client.list_prompts().await?;

// 获取提示词
let messages = client.get_prompt(
    "code_review",
    json!({
        "code": "fn main() { println!(\"Hello\"); }",
        "language": "rust"
    })
).await?;
```

## 调用工具

### 基本调用

```rust
use serde_json::json;

let result = client.call_tool(
    "calculator",
    json!({
        "expression": "10 * 5"
    })
).await?;

println!("Result: {}", result["result"]);
```

### 带超时的调用

```rust
use tokio::time::{timeout, Duration};

let call = client.call_tool(
    "slow_operation",
    json!({
        "input": "data"
    })
);

match timeout(Duration::from_secs(30), call).await {
    Ok(Ok(result)) => println!("Success: {:?}", result),
    Ok(Err(e)) => eprintln!("Tool error: {}", e),
    Err(_) => eprintln!("Timeout"),
}
```

### 批量调用

```rust
use futures::future::join_all;

let calls = vec![
    client.call_tool("tool1", json!({"arg": "value1"})),
    client.call_tool("tool2", json!({"arg": "value2"})),
    client.call_tool("tool3", json!({"arg": "value3"})),
];

let results = join_all(calls).await;

for (i, result) in results.iter().enumerate() {
    match result {
        Ok(value) => println!("Tool {} result: {:?}", i + 1, value),
        Err(e) => eprintln!("Tool {} error: {}", i + 1, e),
    }
}
```

### 流式调用

```rust
use futures::StreamExt;

let mut stream = client.call_tool_stream(
    "streaming_tool",
    json!({
        "input": "data"
    })
).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(data) => println!("Chunk: {:?}", data),
        Err(e) => eprintln!("Stream error: {}", e),
    }
}
```

## 访问资源

### 列出资源

```rust
let resources = client.list_resources().await?;

for resource in resources {
    println!("URI: {}", resource.uri);
    println!("Name: {}", resource.name);
    if let Some(desc) = resource.description {
        println!("Description: {}", desc);
    }
    if let Some(mime) = resource.mime_type {
        println!("MIME Type: {}", mime);
    }
    println!();
}
```

### 读取资源

```rust
let content = client.read_resource("file:///path/to/file.txt").await?;
println!("Content: {}", content);
```

### 订阅资源更新

```rust
use futures::StreamExt;

let mut updates = client.subscribe_resource("config://app.json").await?;

while let Some(update) = updates.next().await {
    match update {
        Ok(content) => println!("Resource updated: {}", content),
        Err(e) => eprintln!("Update error: {}", e),
    }
}
```

### 缓存资源

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct ResourceCache {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl ResourceCache {
    async fn get_or_fetch(
        &self,
        client: &McpClient,
        uri: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(content) = cache.get(uri) {
                return Ok(content.clone());
            }
        }
        
        // 从服务端获取
        let content = client.read_resource(uri).await?;
        
        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(uri.to_string(), content.clone());
        }
        
        Ok(content)
    }
}
```

## 使用提示词

### 获取提示词

```rust
let messages = client.get_prompt(
    "code_review",
    json!({
        "code": "fn add(a: i32, b: i32) -> i32 { a + b }",
        "language": "rust"
    })
).await?;

for message in messages {
    println!("{}: {}", message.role, message.content);
}
```

### 与 AI 模型集成

```rust
use openai_api_rust::chat::*;

// 获取提示词
let messages = client.get_prompt(
    "code_review",
    json!({
        "code": code_to_review,
        "language": "rust"
    })
).await?;

// 转换为 OpenAI 格式
let openai_messages: Vec<ChatMessage> = messages
    .into_iter()
    .map(|m| ChatMessage {
        role: m.role,
        content: m.content,
    })
    .collect();

// 调用 OpenAI API
let response = openai_client.chat_completion(
    "gpt-4",
    openai_messages,
    None,
).await?;

println!("AI Response: {}", response.choices[0].message.content);
```

## 传输层

### Stdio 传输

```rust
let config = McpClientConfig {
    server_command: "my-mcp-server".to_string(),
    server_args: vec![],
    transport: TransportType::Stdio,
    ..Default::default()
};

let client = McpClient::new(config);
```

### SSE 传输

```rust
let config = McpClientConfig {
    server_url: "http://localhost:8080/sse".to_string(),
    transport: TransportType::Sse,
    ..Default::default()
};

let client = McpClient::new(config);
```

### HTTP 传输

```rust
let config = McpClientConfig {
    server_url: "http://localhost:8080".to_string(),
    transport: TransportType::Http,
    ..Default::default()
};

let client = McpClient::new(config);
```

## 错误处理

### 基本错误处理

```rust
match client.call_tool("calculator", json!({"expression": "2+2"})).await {
    Ok(result) => println!("Result: {:?}", result),
    Err(e) => {
        eprintln!("Error: {}", e);
        
        // 根据错误类型处理
        if e.to_string().contains("timeout") {
            eprintln!("Operation timed out, retrying...");
        } else if e.to_string().contains("connection") {
            eprintln!("Connection error, reconnecting...");
            client.reconnect().await?;
        }
    }
}
```

### 重试机制

```rust
use tokio::time::{sleep, Duration};

async fn call_with_retry<T>(
    client: &McpClient,
    tool_name: &str,
    args: serde_json::Value,
    max_retries: u32,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut retries = 0;
    
    loop {
        match client.call_tool(tool_name, args.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(e);
                }
                
                eprintln!("Retry {}/{}: {}", retries, max_retries, e);
                sleep(Duration::from_secs(2u64.pow(retries))).await;
            }
        }
    }
}
```

### 断路器模式

```rust
use std::sync::atomic::{AtomicU32, Ordering};

struct CircuitBreaker {
    failure_count: AtomicU32,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            threshold,
            timeout,
        }
    }
    
    async fn call<F, T>(&self, f: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        // 检查断路器状态
        if self.failure_count.load(Ordering::Relaxed) >= self.threshold {
            return Err("Circuit breaker open".into());
        }
        
        // 执行操作
        match f.await {
            Ok(result) => {
                self.failure_count.store(0, Ordering::Relaxed);
                Ok(result)
            },
            Err(e) => {
                self.failure_count.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }
}
```

## 最佳实践

### 1. 连接池

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

struct ClientPool {
    clients: Vec<Arc<McpClient>>,
    semaphore: Arc<Semaphore>,
}

impl ClientPool {
    fn new(size: usize, config: McpClientConfig) -> Self {
        let clients = (0..size)
            .map(|_| Arc::new(McpClient::new(config.clone())))
            .collect();
        
        Self {
            clients,
            semaphore: Arc::new(Semaphore::new(size)),
        }
    }
    
    async fn execute<F, T>(&self, f: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce(&McpClient) -> Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error>>>>>,
    {
        let _permit = self.semaphore.acquire().await?;
        let client = &self.clients[0]; // 简化示例
        f(client).await
    }
}
```

### 2. 请求去重

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct RequestDeduplicator {
    pending: Arc<RwLock<HashMap<String, Arc<tokio::sync::Notify>>>>,
}

impl RequestDeduplicator {
    async fn call_tool(
        &self,
        client: &McpClient,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let key = format!("{}:{}", tool_name, args.to_string());
        
        // 检查是否有相同的请求正在进行
        {
            let pending = self.pending.read().await;
            if let Some(notify) = pending.get(&key) {
                let notify = notify.clone();
                drop(pending);
                notify.notified().await;
                // 返回缓存的结果
            }
        }
        
        // 执行请求
        let notify = Arc::new(tokio::sync::Notify::new());
        {
            let mut pending = self.pending.write().await;
            pending.insert(key.clone(), notify.clone());
        }
        
        let result = client.call_tool(tool_name, args).await;
        
        // 清理并通知等待者
        {
            let mut pending = self.pending.write().await;
            pending.remove(&key);
        }
        notify.notify_waiters();
        
        result
    }
}
```

### 3. 监控和日志

```rust
use tracing::{info, error, instrument};

#[instrument]
async fn call_tool_with_logging(
    client: &McpClient,
    tool_name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    info!("Calling tool: {}", tool_name);
    
    let start = std::time::Instant::now();
    let result = client.call_tool(tool_name, args).await;
    let duration = start.elapsed();
    
    match &result {
        Ok(_) => info!("Tool call succeeded in {:?}", duration),
        Err(e) => error!("Tool call failed in {:?}: {}", duration, e),
    }
    
    result
}
```

## 示例

完整的客户端示例请参考：
- `examples/mcp-tool-discovery-demo/`
- `examples/mcp-auth-demo/`
- `examples/mcp-transport-demo/`

## 下一步

- 阅读 [MCP 服务端开发指南](SERVER_GUIDE.md)
- 阅读 [MCP 最佳实践](BEST_PRACTICES.md)
- 查看 [MCP API 参考](API_REFERENCE.md)

