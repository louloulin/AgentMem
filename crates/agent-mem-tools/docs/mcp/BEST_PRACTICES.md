# MCP 最佳实践

本文档介绍使用 AgentMem MCP 的最佳实践和设计模式。

## 目录

- [架构设计](#架构设计)
- [工具设计](#工具设计)
- [资源管理](#资源管理)
- [性能优化](#性能优化)
- [安全性](#安全性)
- [错误处理](#错误处理)
- [测试](#测试)
- [监控和日志](#监控和日志)
- [部署](#部署)

## 架构设计

### 1. 单一职责原则

每个工具应该只做一件事，并且做好：

**❌ 不好的设计**:
```rust
// 一个工具做太多事情
McpTool {
    name: "file_operations".to_string(),
    description: "文件操作（读取、写入、删除、列表）".to_string(),
    // ...
}
```

**✅ 好的设计**:
```rust
// 每个工具专注于一个操作
McpTool {
    name: "read_file".to_string(),
    description: "读取文件内容".to_string(),
    // ...
}

McpTool {
    name: "write_file".to_string(),
    description: "写入文件内容".to_string(),
    // ...
}
```

### 2. 模块化设计

将相关的工具组织成模块：

```rust
// tools/file_system.rs
pub fn create_file_tools() -> Vec<McpTool> {
    vec![
        create_read_file_tool(),
        create_write_file_tool(),
        create_list_files_tool(),
    ]
}

// tools/database.rs
pub fn create_database_tools() -> Vec<McpTool> {
    vec![
        create_query_tool(),
        create_insert_tool(),
        create_update_tool(),
    ]
}

// main.rs
let mut server = McpServer::new(config);
server.register_tools(create_file_tools()).await?;
server.register_tools(create_database_tools()).await?;
```

### 3. 依赖注入

使用依赖注入提高可测试性：

```rust
struct ToolDependencies {
    database: Arc<Database>,
    cache: Arc<Cache>,
    logger: Arc<Logger>,
}

fn create_query_tool(deps: Arc<ToolDependencies>) -> McpTool {
    McpTool {
        name: "query".to_string(),
        description: "查询数据库".to_string(),
        handler: Box::new(move |args, _ctx| {
            let deps = deps.clone();
            Box::pin(async move {
                let result = deps.database.query(&args).await?;
                deps.logger.log("Query executed").await;
                Ok(result)
            })
        }),
    }
}
```

## 工具设计

### 1. 清晰的命名

使用动词开头的命名，清楚地表达工具的功能：

**✅ 好的命名**:
- `read_file`
- `send_email`
- `calculate_sum`
- `fetch_weather`

**❌ 不好的命名**:
- `file` (不清楚做什么)
- `email` (不清楚是发送还是接收)
- `math` (太宽泛)

### 2. 详细的描述

提供清晰、详细的描述，帮助 AI 模型理解工具的用途：

```rust
McpTool {
    name: "send_email".to_string(),
    description: "发送电子邮件。支持 HTML 和纯文本格式，可以添加附件。\
                  邮件将通过配置的 SMTP 服务器发送。".to_string(),
    // ...
}
```

### 3. 严格的输入验证

使用 JSON Schema 定义清晰的输入格式：

```rust
input_schema: serde_json::json!({
    "type": "object",
    "properties": {
        "to": {
            "type": "string",
            "format": "email",
            "description": "收件人邮箱地址"
        },
        "subject": {
            "type": "string",
            "minLength": 1,
            "maxLength": 200,
            "description": "邮件主题"
        },
        "body": {
            "type": "string",
            "description": "邮件正文"
        },
        "format": {
            "type": "string",
            "enum": ["text", "html"],
            "default": "text",
            "description": "邮件格式"
        }
    },
    "required": ["to", "subject", "body"]
}),
```

### 4. 结构化的输出

返回结构化的 JSON 数据，而不是纯文本：

**❌ 不好的输出**:
```rust
Ok(serde_json::json!({
    "result": "Email sent successfully to user@example.com"
}))
```

**✅ 好的输出**:
```rust
Ok(serde_json::json!({
    "success": true,
    "message_id": "abc123",
    "recipient": "user@example.com",
    "sent_at": "2025-10-12T10:00:00Z"
}))
```

### 5. 幂等性

确保工具可以安全地重复调用：

```rust
handler: Box::new(|args, _ctx| {
    Box::pin(async move {
        let id = args["id"].as_str().ok_or("Missing id")?;
        
        // 检查是否已经处理过
        if already_processed(id).await? {
            return Ok(serde_json::json!({
                "success": true,
                "already_processed": true
            }));
        }
        
        // 处理请求
        process_request(id).await?;
        
        Ok(serde_json::json!({
            "success": true,
            "already_processed": false
        }))
    })
}),
```

## 资源管理

### 1. URI 设计

使用清晰、一致的 URI 方案：

```rust
// 文件系统资源
"file:///path/to/file.txt"

// 配置资源
"config://app/database"

// 数据库资源
"db://users/123"

// HTTP 资源
"http://api.example.com/data"
```

### 2. 资源缓存

实现智能缓存减少重复读取：

```rust
struct CachedResourceProvider {
    cache: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    ttl: Duration,
}

impl CachedResourceProvider {
    async fn read_resource(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some((content, timestamp)) = cache.get(uri) {
                if timestamp.elapsed() < self.ttl {
                    return Ok(content.clone());
                }
            }
        }
        
        // 从源读取
        let content = self.fetch_from_source(uri).await?;
        
        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(uri.to_string(), (content.clone(), Instant::now()));
        }
        
        Ok(content)
    }
}
```

### 3. 资源版本控制

为资源提供版本信息：

```rust
McpResource {
    uri: "config://app.json".to_string(),
    name: "应用配置".to_string(),
    description: Some("应用程序配置文件".to_string()),
    mime_type: Some("application/json".to_string()),
    metadata: Some(serde_json::json!({
        "version": "1.2.3",
        "last_modified": "2025-10-12T10:00:00Z",
        "etag": "abc123"
    })),
}
```

## 性能优化

### 1. 异步处理

使用异步 I/O 提高并发性能：

```rust
handler: Box::new(|args, _ctx| {
    Box::pin(async move {
        // 并发执行多个操作
        let (result1, result2, result3) = tokio::join!(
            fetch_data_1(&args),
            fetch_data_2(&args),
            fetch_data_3(&args),
        );
        
        Ok(serde_json::json!({
            "data1": result1?,
            "data2": result2?,
            "data3": result3?,
        }))
    })
}),
```

### 2. 批量操作

支持批量处理减少往返次数：

```rust
McpTool {
    name: "batch_query".to_string(),
    description: "批量查询数据库".to_string(),
    input_schema: serde_json::json!({
        "type": "object",
        "properties": {
            "queries": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "table": {"type": "string"}
                    }
                }
            }
        }
    }),
    handler: Box::new(|args, _ctx| {
        Box::pin(async move {
            let queries = args["queries"].as_array()
                .ok_or("Invalid queries")?;
            
            let results = futures::future::join_all(
                queries.iter().map(|q| execute_query(q))
            ).await;
            
            Ok(serde_json::json!({
                "results": results
            }))
        })
    }),
}
```

### 3. 连接池

使用连接池管理资源：

```rust
struct DatabasePool {
    pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
}

impl DatabasePool {
    async fn query(&self, sql: &str) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
        let conn = self.pool.get().await?;
        let rows = conn.query(sql, &[]).await?;
        Ok(rows)
    }
}
```

### 4. 超时控制

为所有操作设置合理的超时：

```rust
use tokio::time::{timeout, Duration};

handler: Box::new(|args, _ctx| {
    Box::pin(async move {
        let operation = perform_operation(&args);
        
        match timeout(Duration::from_secs(30), operation).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Operation timed out".into()),
        }
    })
}),
```

## 安全性

### 1. 输入验证

严格验证所有输入：

```rust
handler: Box::new(|args, _ctx| {
    Box::pin(async move {
        // 验证必需字段
        let path = args["path"].as_str()
            .ok_or("Missing path")?;
        
        // 验证路径安全性
        if path.contains("..") {
            return Err("Invalid path: contains ..".into());
        }
        
        // 验证路径在允许的目录内
        let canonical = std::fs::canonicalize(path)?;
        if !canonical.starts_with("/allowed/directory") {
            return Err("Path outside allowed directory".into());
        }
        
        // 执行操作
        let content = tokio::fs::read_to_string(canonical).await?;
        Ok(serde_json::json!({"content": content}))
    })
}),
```

### 2. 权限检查

实现细粒度的权限控制：

```rust
struct PermissionChecker {
    user_permissions: HashMap<String, Vec<String>>,
}

impl PermissionChecker {
    fn check(&self, user_id: &str, permission: &str) -> bool {
        self.user_permissions
            .get(user_id)
            .map(|perms| perms.contains(&permission.to_string()))
            .unwrap_or(false)
    }
}

handler: Box::new(move |args, ctx| {
    let checker = checker.clone();
    Box::pin(async move {
        // 检查权限
        if !checker.check(&ctx.user_id, "read_file") {
            return Err("Permission denied".into());
        }
        
        // 执行操作
        // ...
    })
}),
```

### 3. 速率限制

防止滥用：

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

struct RateLimiter {
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
    
    async fn execute<F, T>(&self, f: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let _permit = self.semaphore.acquire().await?;
        f.await
    }
}
```

### 4. 敏感数据处理

避免在日志中泄露敏感信息：

```rust
use tracing::info;

handler: Box::new(|args, _ctx| {
    Box::pin(async move {
        let password = args["password"].as_str()
            .ok_or("Missing password")?;
        
        // ❌ 不要这样做
        // info!("Authenticating with password: {}", password);
        
        // ✅ 应该这样做
        info!("Authenticating user");
        
        let result = authenticate(password).await?;
        
        Ok(serde_json::json!({
            "success": true,
            // ❌ 不要返回敏感信息
            // "token": result.token,
            
            // ✅ 只返回必要信息
            "user_id": result.user_id,
        }))
    })
}),
```

## 错误处理

### 1. 错误分类

使用不同的错误类型：

```rust
#[derive(Debug)]
enum ToolError {
    InvalidInput(String),
    NotFound(String),
    PermissionDenied(String),
    InternalError(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ToolError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ToolError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ToolError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ToolError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ToolError {}
```

### 2. 错误恢复

实现优雅的错误恢复：

```rust
handler: Box::new(|args, _ctx| {
    Box::pin(async move {
        // 尝试主要方法
        match primary_method(&args).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                warn!("Primary method failed: {}, trying fallback", e);
            }
        }
        
        // 尝试备用方法
        match fallback_method(&args).await {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("All methods failed: {}", e).into()),
        }
    })
}),
```

### 3. 错误上下文

提供丰富的错误上下文：

```rust
Ok(serde_json::json!({
    "success": false,
    "error": {
        "code": "FILE_NOT_FOUND",
        "message": "File not found",
        "details": {
            "path": "/path/to/file",
            "attempted_at": "2025-10-12T10:00:00Z"
        },
        "suggestions": [
            "Check if the file path is correct",
            "Verify file permissions"
        ]
    }
}))
```

## 测试

### 1. 单元测试

为每个工具编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = create_calculator_tool();
        
        let args = serde_json::json!({
            "expression": "2 + 2"
        });
        
        let ctx = ToolContext::default();
        let result = (tool.handler)(args, &ctx).await.unwrap();
        
        assert_eq!(result["result"], 4);
    }
    
    #[tokio::test]
    async fn test_calculator_tool_invalid_input() {
        let tool = create_calculator_tool();
        
        let args = serde_json::json!({
            "expression": "invalid"
        });
        
        let ctx = ToolContext::default();
        let result = (tool.handler)(args, &ctx).await;
        
        assert!(result.is_err());
    }
}
```

### 2. 集成测试

测试工具之间的交互：

```rust
#[tokio::test]
async fn test_file_workflow() {
    let mut server = McpServer::new(config);
    server.register_tool(create_write_file_tool()).await.unwrap();
    server.register_tool(create_read_file_tool()).await.unwrap();
    
    // 写入文件
    let write_result = server.call_tool(
        "write_file",
        serde_json::json!({
            "path": "/tmp/test.txt",
            "content": "Hello, World!"
        })
    ).await.unwrap();
    
    assert!(write_result["success"].as_bool().unwrap());
    
    // 读取文件
    let read_result = server.call_tool(
        "read_file",
        serde_json::json!({
            "path": "/tmp/test.txt"
        })
    ).await.unwrap();
    
    assert_eq!(read_result["content"], "Hello, World!");
}
```

### 3. 性能测试

测试工具的性能：

```rust
#[tokio::test]
async fn test_tool_performance() {
    let tool = create_query_tool();
    let ctx = ToolContext::default();
    
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let args = serde_json::json!({"query": "SELECT 1"});
        (tool.handler)(args, &ctx).await.unwrap();
    }
    
    let duration = start.elapsed();
    let avg_duration = duration / 1000;
    
    assert!(avg_duration < std::time::Duration::from_millis(10));
}
```

## 监控和日志

### 1. 结构化日志

使用结构化日志记录关键信息：

```rust
use tracing::{info, error, instrument};

#[instrument]
async fn execute_tool(
    tool_name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    info!(
        tool_name = %tool_name,
        args = %args,
        "Executing tool"
    );
    
    let start = std::time::Instant::now();
    let result = perform_operation(&args).await;
    let duration = start.elapsed();
    
    match &result {
        Ok(_) => info!(
            tool_name = %tool_name,
            duration_ms = duration.as_millis(),
            "Tool execution succeeded"
        ),
        Err(e) => error!(
            tool_name = %tool_name,
            duration_ms = duration.as_millis(),
            error = %e,
            "Tool execution failed"
        ),
    }
    
    result
}
```

### 2. 指标收集

收集关键性能指标：

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref TOOL_CALLS: Counter = Counter::new(
        "mcp_tool_calls_total",
        "Total number of tool calls"
    ).unwrap();
    
    static ref TOOL_DURATION: Histogram = Histogram::new(
        "mcp_tool_duration_seconds",
        "Tool execution duration"
    ).unwrap();
}

handler: Box::new(|args, _ctx| {
    Box::pin(async move {
        TOOL_CALLS.inc();
        
        let timer = TOOL_DURATION.start_timer();
        let result = perform_operation(&args).await;
        timer.observe_duration();
        
        result
    })
}),
```

## 部署

### 1. 配置管理

使用环境变量和配置文件：

```rust
use config::{Config, Environment, File};

fn load_config() -> Result<McpServerConfig, Box<dyn std::error::Error>> {
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name("config/production").required(false))
        .add_source(Environment::with_prefix("MCP"))
        .build()?;
    
    Ok(config.try_deserialize()?)
}
```

### 2. 健康检查

实现健康检查端点：

```rust
server.register_tool(McpTool {
    name: "health_check".to_string(),
    description: "健康检查".to_string(),
    input_schema: serde_json::json!({}),
    handler: Box::new(|_args, _ctx| {
        Box::pin(async move {
            Ok(serde_json::json!({
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": get_uptime(),
            }))
        })
    }),
}).await?;
```

### 3. 优雅关闭

实现优雅关闭：

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = McpServer::new(config);
    
    // 启动服务端
    let server_handle = tokio::spawn(async move {
        server.start().await
    });
    
    // 等待关闭信号
    signal::ctrl_c().await?;
    info!("Shutting down gracefully...");
    
    // 等待服务端关闭
    server_handle.await??;
    
    Ok(())
}
```

## 总结

遵循这些最佳实践可以帮助您：
- 构建可维护的 MCP 服务
- 提高性能和可靠性
- 增强安全性
- 简化测试和调试
- 优化部署和运维

## 参考资料

- [MCP 服务端开发指南](SERVER_GUIDE.md)
- [MCP 客户端开发指南](CLIENT_GUIDE.md)
- [MCP API 参考](API_REFERENCE.md)

