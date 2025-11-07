# AgentMem MCP 2.0 生产级改造计划

**版本**: 2.0  
**日期**: 2025-11-07  
**目标**: 达到生产级别，真实对接Claude Code

---

## 📊 当前状态分析

### 对比分析：AgentMem vs mem0 vs MIRIX

| 特性 | AgentMem (现状) | mem0 | MIRIX | 目标状态 |
|------|----------------|------|-------|---------|
| **MCP Server** | ✅ STDIO实现 | ✅ FastMCP+SSE | ❌ 无 | ✅ 完整实现 |
| **MCP Client** | ❌ 无 | ❌ 无 | ✅ 完整客户端 | ✅ 添加客户端 |
| **传输协议** | STDIO | SSE | STDIO/SSE | STDIO+SSE |
| **工具数量** | 5个 | 4个 | N/A | 10+ 个 |
| **权限控制** | ❌ 无 | ✅ 完整ACL | ✅ 完整 | ✅ 实现ACL |
| **错误处理** | ⚠️ 基础 | ✅ 优雅降级 | ✅ 完整 | ✅ 生产级 |
| **日志记录** | ⚠️ 部分 | ✅ 完整 | ✅ 完整 | ✅ 完整 |
| **Mock代码** | ⚠️ 存在 | ❌ 无 | ❌ 无 | ❌ 删除 |
| **TODO项** | ⚠️ 2个 | ❌ 无 | ❌ 无 | ❌ 完成 |
| **数据库集成** | ✅ PostgreSQL | ✅ 多种 | ✅ SQLite | ✅ 保持 |
| **测试覆盖** | ⚠️ 部分 | ✅ 完整 | ✅ 完整 | ✅ 完整 |

---

## 🐛 识别的关键问题

### 1. Mock代码（必须删除）

**位置**: `agentmen/crates/agent-mem-tools/src/mcp/server.rs:381-455`

```rust
// ❌ 当前代码（Mock）
struct MockTool;

impl Tool for MockTool {
    fn name(&self) -> &str {
        "mock_tool"
    }
    // ...
}
```

**影响**: 测试代码混入生产代码，降低可信度

### 2. TODO项（必须完成）

**位置1**: `agentmen/crates/agent-mem-tools/src/execution_sandbox.rs:279-280`
```rust
// TODO: 实际执行工具代码
result = {{"status": "success", "message": "Tool executed", "args": args}}
```

**位置2**: `agentmen/crates/agent-mem-tools/src/execution_sandbox.rs:319-320`
```rust
// TODO: 创建虚拟环境
// 这里需要调用 python -m venv
```

**影响**: 核心功能未完成

### 3. HTTP Mock响应

**位置**: `agentmen/crates/agent-mem-tools/src/builtin/http.rs:146`
```rust
"{{\"message\": \"Mock response for {method} {url}\", \"success\": true}}"
```

**影响**: HTTP工具返回假数据

### 4. 缺失功能

- ❌ **没有SSE传输支持** (mem0有)
- ❌ **没有MCP客户端** (MIRIX有)
- ❌ **没有权限控制系统** (mem0有完整ACL)
- ❌ **没有访问日志** (mem0有MemoryAccessLog)
- ❌ **没有配置持久化** (MIRIX有)
- ❌ **没有优雅降级** (mem0有)

### 5. Bug修复（已完成✅）

- ✅ **Search工具缺少user_id** - 已修复
- ✅ **API响应解析错误** - 已修复

---

## 🎯 改造目标

### 核心目标

1. **删除所有Mock代码** - 100%生产代码
2. **完成所有TODO项** - 无未完成功能
3. **实现完整权限控制** - 类似mem0的ACL
4. **添加SSE传输支持** - 支持Web集成
5. **实现MCP客户端** - 类似MIRIX
6. **完整错误处理** - 优雅降级
7. **生产级日志** - 完整审计追踪
8. **完整测试覆盖** - 单元+集成测试
9. **性能优化** - 支持高并发
10. **Claude Code验证** - 真实对接测试

---

## 📋 详细改造计划

### Phase 1: 清理与修复 (1-2天)

#### 1.1 删除Mock代码

**文件**: `agentmen/crates/agent-mem-tools/src/mcp/server.rs`

```diff
--- a/crates/agent-mem-tools/src/mcp/server.rs
+++ b/crates/agent-mem-tools/src/mcp/server.rs
@@ -378,77 +378,0 @@
-    // Mock 工具
-    struct MockTool;
-    
-    #[async_trait]
-    impl Tool for MockTool {
-        fn name(&self) -> &str {
-            "mock_tool"
-        }
-        
-        fn description(&self) -> &str {
-            "A mock tool for testing"
-        }
-        
-        fn schema(&self) -> ToolSchema {
-            ToolSchema::new(self.name(), self.description())
-        }
-        
-        async fn execute(&self, _args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
-            Ok(json!({
-                "result": "mock",
-                "success": true
-            }))
-        }
-    }
-    
-    #[tokio::test]
-    async fn test_list_tools() {
-        let config = McpServerConfig {
-            name: "test-server".to_string(),
-            version: "1.0.0".to_string(),
-        };
-        
-        let server = McpServer::new(config);
-        let mock_tool = Arc::new(MockTool);
-        server.register_tool(mock_tool).await.unwrap();
-        
-        let response = server.list_tools().await.unwrap();
-        assert_eq!(response.tools.len(), 1);
-        assert_eq!(response.tools[0].name, "mock_tool");
-    }
```

**替换为真实测试**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_list_tools_real() {
        let config = McpServerConfig {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        };
        
        let server = McpServer::new(config);
        
        // 使用真实工具进行测试
        let add_memory_tool = Arc::new(AddMemoryTool::new(backend_url));
        server.register_tool(add_memory_tool).await.unwrap();
        
        let response = server.list_tools().await.unwrap();
        assert_eq!(response.tools.len(), 1);
        assert_eq!(response.tools[0].name, "agentmem_add_memory");
    }
}
```

#### 1.2 删除HTTP Mock

**文件**: `agentmen/crates/agent-mem-tools/src/builtin/http.rs`

```diff
--- a/crates/agent-mem-tools/src/builtin/http.rs
+++ b/crates/agent-mem-tools/src/builtin/http.rs
@@ -143,8 +143,12 @@
-        let response_body = format!(
-            "{{\"message\": \"Mock response for {method} {url}\", \"success\": true}}"
-        );
+        // 执行真实HTTP请求
+        let response = ureq::request(&method, &url)
+            .send()
+            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {}", e)))?;
+        
+        let response_body = response.into_string()
+            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read response: {}", e)))?;
```

#### 1.3 完成TODO: 工具执行

**文件**: `agentmen/crates/agent-mem-tools/src/execution_sandbox.rs`

**TODO 1** (Line 279-280):
```diff
--- a/crates/agent-mem-tools/src/execution_sandbox.rs
+++ b/crates/agent-mem-tools/src/execution_sandbox.rs
@@ -276,9 +276,45 @@
-    # TODO: 实际执行工具代码
-    result = {{"status": "success", "message": "Tool executed", "args": args}}
-    print(json.dumps(result))
+    # 实际执行工具代码
+    import importlib.util
+    import sys
+    
+    try:
+        # 加载工具模块
+        spec = importlib.util.spec_from_file_location("tool_module", tool_path)
+        if spec is None or spec.loader is None:
+            raise ImportError(f"Cannot load tool from {{tool_path}}")
+        
+        tool_module = importlib.util.module_from_spec(spec)
+        sys.modules["tool_module"] = tool_module
+        spec.loader.exec_module(tool_module)
+        
+        # 执行工具函数
+        if not hasattr(tool_module, "execute"):
+            raise AttributeError("Tool module must have an 'execute' function")
+        
+        execute_func = getattr(tool_module, "execute")
+        result = execute_func(**args)
+        
+        # 返回结果
+        print(json.dumps({{
+            "status": "success",
+            "result": result,
+            "args": args
+        }}))
+        
+    except Exception as e:
+        # 返回错误信息
+        print(json.dumps({{
+            "status": "error",
+            "error": str(e),
+            "error_type": type(e).__name__,
+            "args": args
+        }}), file=sys.stderr)
+        sys.exit(1)
```

**TODO 2** (Line 319-320):
```diff
--- a/crates/agent-mem-tools/src/execution_sandbox.rs
+++ b/crates/agent-mem-tools/src/execution_sandbox.rs
@@ -316,9 +316,32 @@
         #[cfg(target_os = "linux")]
         {
-            // TODO: 创建虚拟环境
-            // 这里需要调用 python -m venv
+            // 创建Python虚拟环境
+            use std::process::Command;
+            
+            let venv_path = sandbox_dir.join("venv");
+            
+            // 创建venv
+            let output = Command::new("python3")
+                .args(&["-m", "venv", venv_path.to_str().unwrap()])
+                .output()
+                .map_err(|e| format!("Failed to create venv: {}", e))?;
+            
+            if !output.status.success() {
+                return Err(format!(
+                    "venv creation failed: {}",
+                    String::from_utf8_lossy(&output.stderr)
+                ));
+            }
+            
+            // 安装基础依赖
+            let pip_path = venv_path.join("bin").join("pip");
+            let _install_output = Command::new(pip_path)
+                .args(&["install", "--upgrade", "pip", "setuptools", "wheel"])
+                .output()
+                .map_err(|e| format!("Failed to install dependencies: {}", e))?;
+            
+            tracing::info!("Virtual environment created at {:?}", venv_path);
         }
```

---

### Phase 2: 新功能实现 (3-5天)

#### 2.1 实现SSE传输支持

**新文件**: `agentmen/crates/agent-mem-tools/src/mcp/transport/sse.rs`

```rust
//! SSE (Server-Sent Events) 传输实现
//! 
//! 基于 mem0 的实现，提供 HTTP/SSE 双向通信

use async_trait::async_trait;
use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::{get, post},
    Router,
};
use futures::stream::Stream;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::Transport;
use crate::mcp::error::McpError;
use crate::mcp::JsonRpcRequest, JsonRpcResponse;

/// SSE传输层状态
struct SseState {
    /// 等待处理的请求队列
    request_queue: Arc<RwLock<Vec<JsonRpcRequest>>>,
    /// 响应发送通道
    response_tx: mpsc::Sender<JsonRpcResponse>,
}

/// SSE传输层实现
pub struct SseTransport {
    router: Router,
    state: Arc<SseState>,
}

impl SseTransport {
    pub fn new() -> Self {
        let (response_tx, _response_rx) = mpsc::channel(100);
        
        let state = Arc::new(SseState {
            request_queue: Arc::new(RwLock::new(Vec::new())),
            response_tx,
        });
        
        let router = Router::new()
            .route("/sse/:user_id", get(handle_sse_connection))
            .route("/messages", post(handle_post_message))
            .with_state(state.clone());
        
        Self { router, state }
    }
    
    pub fn router(&self) -> Router {
        self.router.clone()
    }
}

#[async_trait]
impl Transport for SseTransport {
    async fn send(&mut self, response: JsonRpcResponse) -> Result<(), McpError> {
        self.state
            .response_tx
            .send(response)
            .await
            .map_err(|e| McpError::TransportError(format!("Failed to send response: {}", e)))
    }
    
    async fn receive(&mut self) -> Result<JsonRpcRequest, McpError> {
        loop {
            let mut queue = self.state.request_queue.write().await;
            if let Some(request) = queue.pop() {
                return Ok(request);
            }
            drop(queue);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}

/// 处理SSE连接
async fn handle_sse_connection(
    State(state): State<Arc<SseState>>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let (tx, rx) = mpsc::channel(100);
    
    // 启动响应转发任务
    tokio::spawn(async move {
        let mut response_rx = state.response_tx.subscribe();
        while let Ok(response) = response_rx.recv().await {
            let json = serde_json::to_string(&response).unwrap();
            let _ = tx.send(Event::default().data(json)).await;
        }
    });
    
    Sse::new(tokio_stream::wrappers::ReceiverStream::new(rx))
}

/// 处理POST消息
async fn handle_post_message(
    State(state): State<Arc<SseState>>,
    axum::Json(request): axum::Json<JsonRpcRequest>,
) -> Result<axum::Json<Value>, axum::http::StatusCode> {
    state.request_queue.write().await.push(request);
    Ok(axum::Json(json!({"status": "queued"})))
}
```

#### 2.2 实现MCP客户端

**新文件**: `agentmen/crates/agent-mem-tools/src/mcp/client.rs`

```rust
//! MCP客户端实现
//! 
//! 基于 MIRIX 的设计，支持多服务器管理

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{JsonRpcRequest, JsonRpcResponse, Tool};
use crate::error::ToolError;

/// MCP服务器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub server_type: ServerType,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    Stdio,
    Sse,
    Http,
}

/// MCP客户端
pub struct McpClient {
    servers: Arc<RwLock<HashMap<String, Arc<dyn McpConnection>>>>,
    config_file: String,
}

#[async_trait]
pub trait McpConnection: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<Value>, ToolError>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ToolError>;
    async fn disconnect(&self) -> Result<(), ToolError>;
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            config_file: "~/.agentmem/mcp_connections.json".to_string(),
        }
    }
    
    /// 添加MCP服务器
    pub async fn add_server(&self, config: McpServerConfig) -> Result<(), ToolError> {
        let connection: Arc<dyn McpConnection> = match config.server_type {
            ServerType::Stdio => {
                Arc::new(StdioConnection::new(config).await?)
            }
            ServerType::Sse => {
                Arc::new(SseConnection::new(config).await?)
            }
            ServerType::Http => {
                Arc::new(HttpConnection::new(config).await?)
            }
        };
        
        self.servers.write().await.insert(config.name.clone(), connection);
        self.save_config().await?;
        
        Ok(())
    }
    
    /// 移除服务器
    pub async fn remove_server(&self, name: &str) -> Result<(), ToolError> {
        if let Some(connection) = self.servers.write().await.remove(name) {
            connection.disconnect().await?;
        }
        self.save_config().await?;
        Ok(())
    }
    
    /// 列出所有服务器
    pub async fn list_servers(&self) -> Vec<String> {
        self.servers.read().await.keys().cloned().collect()
    }
    
    /// 列出所有工具
    pub async fn list_all_tools(&self) -> Result<HashMap<String, Vec<Value>>, ToolError> {
        let mut all_tools = HashMap::new();
        
        for (name, connection) in self.servers.read().await.iter() {
            match connection.list_tools().await {
                Ok(tools) => {
                    all_tools.insert(name.clone(), tools);
                }
                Err(e) => {
                    tracing::warn!("Failed to list tools for server {}: {}", name, e);
                }
            }
        }
        
        Ok(all_tools)
    }
    
    /// 执行工具
    pub async fn execute_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        args: Value,
    ) -> Result<Value, ToolError> {
        let servers = self.servers.read().await;
        let connection = servers
            .get(server_name)
            .ok_or_else(|| ToolError::ExecutionFailed(format!("Server {} not found", server_name)))?;
        
        connection.call_tool(tool_name, args).await
    }
    
    /// 保存配置
    async fn save_config(&self) -> Result<(), ToolError> {
        // TODO: 实现配置持久化
        Ok(())
    }
    
    /// 加载配置
    async fn load_config(&self) -> Result<(), ToolError> {
        // TODO: 实现配置加载
        Ok(())
    }
}

/// STDIO连接实现
struct StdioConnection {
    config: McpServerConfig,
    // TODO: 添加进程句柄
}

#[async_trait]
impl McpConnection for StdioConnection {
    async fn list_tools(&self) -> Result<Vec<Value>, ToolError> {
        // TODO: 实现
        Ok(vec![])
    }
    
    async fn call_tool(&self, _name: &str, _args: Value) -> Result<Value, ToolError> {
        // TODO: 实现
        Ok(json!({}))
    }
    
    async fn disconnect(&self) -> Result<(), ToolError> {
        // TODO: 实现
        Ok(())
    }
}

impl StdioConnection {
    async fn new(config: McpServerConfig) -> Result<Self, ToolError> {
        Ok(Self { config })
    }
}

/// SSE连接实现
struct SseConnection {
    config: McpServerConfig,
}

#[async_trait]
impl McpConnection for SseConnection {
    async fn list_tools(&self) -> Result<Vec<Value>, ToolError> {
        // TODO: 实现
        Ok(vec![])
    }
    
    async fn call_tool(&self, _name: &str, _args: Value) -> Result<Value, ToolError> {
        // TODO: 实现
        Ok(json!({}))
    }
    
    async fn disconnect(&self) -> Result<(), ToolError> {
        // TODO: 实现
        Ok(())
    }
}

impl SseConnection {
    async fn new(config: McpServerConfig) -> Result<Self, ToolError> {
        Ok(Self { config })
    }
}

/// HTTP连接实现
struct HttpConnection {
    config: McpServerConfig,
}

#[async_trait]
impl McpConnection for HttpConnection {
    async fn list_tools(&self) -> Result<Vec<Value>, ToolError> {
        // TODO: 实现
        Ok(vec![])
    }
    
    async fn call_tool(&self, _name: &str, _args: Value) -> Result<Value, ToolError> {
        // TODO: 实现
        Ok(json!({}))
    }
    
    async fn disconnect(&self) -> Result<(), ToolError> {
        // TODO: 实现
        Ok(())
    }
}

impl HttpConnection {
    async fn new(config: McpServerConfig) -> Result<Self, ToolError> {
        Ok(Self { config })
    }
}
```

#### 2.3 实现权限控制系统

**新文件**: `agentmen/crates/agent-mem-server/src/acl.rs`

```rust
//! 访问控制列表 (ACL) 系统
//! 
//! 基于 mem0 的权限控制设计

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// 权限类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "permission_type", rename_all = "lowercase")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
}

/// ACL条目
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AclEntry {
    pub id: Uuid,
    pub memory_id: Uuid,
    pub app_id: Uuid,
    pub permission: Permission,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// ACL管理器
pub struct AclManager {
    pool: PgPool,
}

impl AclManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// 检查权限
    pub async fn check_permission(
        &self,
        memory_id: Uuid,
        app_id: Uuid,
        permission: Permission,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM acl_entries
            WHERE memory_id = $1 AND app_id = $2 AND permission >= $3
            "#,
        )
        .bind(memory_id)
        .bind(app_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(count > 0)
    }
    
    /// 授予权限
    pub async fn grant_permission(
        &self,
        memory_id: Uuid,
        app_id: Uuid,
        permission: Permission,
    ) -> Result<AclEntry, sqlx::Error> {
        let entry = sqlx::query_as::<_, AclEntry>(
            r#"
            INSERT INTO acl_entries (id, memory_id, app_id, permission, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(memory_id)
        .bind(app_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(entry)
    }
    
    /// 撤销权限
    pub async fn revoke_permission(
        &self,
        memory_id: Uuid,
        app_id: Uuid,
        permission: Permission,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM acl_entries
            WHERE memory_id = $1 AND app_id = $2 AND permission = $3
            "#,
        )
        .bind(memory_id)
        .bind(app_id)
        .bind(permission)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 列出所有权限
    pub async fn list_permissions(
        &self,
        memory_id: Uuid,
    ) -> Result<Vec<AclEntry>, sqlx::Error> {
        let entries = sqlx::query_as::<_, AclEntry>(
            r#"
            SELECT * FROM acl_entries
            WHERE memory_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(memory_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(entries)
    }
}

/// 创建ACL表的迁移SQL
pub const CREATE_ACL_TABLE_SQL: &str = r#"
CREATE TYPE permission_type AS ENUM ('read', 'write', 'delete', 'admin');

CREATE TABLE IF NOT EXISTS acl_entries (
    id UUID PRIMARY KEY,
    memory_id UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    app_id UUID NOT NULL,
    permission permission_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(memory_id, app_id, permission)
);

CREATE INDEX idx_acl_memory_id ON acl_entries(memory_id);
CREATE INDEX idx_acl_app_id ON acl_entries(app_id);
"#;
```

#### 2.4 实现访问日志系统

**新文件**: `agentmen/crates/agent-mem-server/src/audit_log.rs`

```rust
//! 审计日志系统
//! 
//! 记录所有记忆访问操作，用于审计和分析

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// 访问类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "access_type", rename_all = "lowercase")]
pub enum AccessType {
    Search,
    List,
    Get,
    Add,
    Update,
    Delete,
    DeleteAll,
}

/// 访问日志条目
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessLog {
    pub id: Uuid,
    pub memory_id: Option<Uuid>,
    pub app_id: Uuid,
    pub user_id: Uuid,
    pub access_type: AccessType,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 审计日志管理器
pub struct AuditLogManager {
    pool: PgPool,
}

impl AuditLogManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// 记录访问
    pub async fn log_access(
        &self,
        memory_id: Option<Uuid>,
        app_id: Uuid,
        user_id: Uuid,
        access_type: AccessType,
        metadata: serde_json::Value,
    ) -> Result<AccessLog, sqlx::Error> {
        let log = sqlx::query_as::<_, AccessLog>(
            r#"
            INSERT INTO access_logs (id, memory_id, app_id, user_id, access_type, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(memory_id)
        .bind(app_id)
        .bind(user_id)
        .bind(access_type)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(log)
    }
    
    /// 查询访问日志
    pub async fn query_logs(
        &self,
        memory_id: Option<Uuid>,
        app_id: Option<Uuid>,
        user_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<AccessLog>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM access_logs WHERE 1=1");
        
        if memory_id.is_some() {
            query.push_str(" AND memory_id = $1");
        }
        if app_id.is_some() {
            query.push_str(" AND app_id = $2");
        }
        if user_id.is_some() {
            query.push_str(" AND user_id = $3");
        }
        
        query.push_str(" ORDER BY created_at DESC LIMIT $4");
        
        let logs = sqlx::query_as::<_, AccessLog>(&query)
            .bind(memory_id)
            .bind(app_id)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        
        Ok(logs)
    }
    
    /// 统计访问次数
    pub async fn count_accesses(
        &self,
        memory_id: Uuid,
        access_type: Option<AccessType>,
    ) -> Result<i64, sqlx::Error> {
        let count = if let Some(at) = access_type {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM access_logs
                WHERE memory_id = $1 AND access_type = $2
                "#,
            )
            .bind(memory_id)
            .bind(at)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                r#"
                SELECT COUNT(*) FROM access_logs
                WHERE memory_id = $1
                "#,
            )
            .bind(memory_id)
            .fetch_one(&self.pool)
            .await?
        };
        
        Ok(count)
    }
}

/// 创建访问日志表的迁移SQL
pub const CREATE_ACCESS_LOG_TABLE_SQL: &str = r#"
CREATE TYPE access_type AS ENUM ('search', 'list', 'get', 'add', 'update', 'delete', 'delete_all');

CREATE TABLE IF NOT EXISTS access_logs (
    id UUID PRIMARY KEY,
    memory_id UUID REFERENCES memories(id) ON DELETE CASCADE,
    app_id UUID NOT NULL,
    user_id UUID NOT NULL,
    access_type access_type NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_access_logs_memory_id ON access_logs(memory_id);
CREATE INDEX idx_access_logs_app_id ON access_logs(app_id);
CREATE INDEX idx_access_logs_user_id ON access_logs(user_id);
CREATE INDEX idx_access_logs_created_at ON access_logs(created_at DESC);
"#;
```

---

### Phase 3: 优化与完善 (2-3天)

#### 3.1 完整错误处理

**文件**: `agentmen/crates/agent-mem-tools/src/agentmem_tools.rs`

```rust
// 在所有工具中添加优雅降级
async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
    // 尝试连接后端
    let api_url = get_api_url();
    
    // 检查后端是否可用
    match check_backend_health(&api_url).await {
        Ok(true) => {
            // 后端可用，执行正常流程
        }
        Ok(false) => {
            return Ok(json!({
                "success": false,
                "error": "Backend service is unhealthy",
                "message": "AgentMem backend is currently unavailable. Please try again later."
            }));
        }
        Err(e) => {
            tracing::error!("Failed to check backend health: {}", e);
            return Ok(json!({
                "success": false,
                "error": "Connection failed",
                "message": "Cannot connect to AgentMem backend. Please check your configuration."
            }));
        }
    }
    
    // ... 继续执行
}

async fn check_backend_health(api_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let url = format!("{}/health", api_url);
    let response = tokio::task::spawn_blocking(move || {
        ureq::get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .call()
    })
    .await??;
    
    Ok(response.status() == 200)
}
```

#### 3.2 性能优化

**缓存层**:
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;

pub struct CachedAgentMemClient {
    client: AgentMemClient,
    search_cache: Arc<RwLock<LruCache<String, Vec<Memory>>>>,
}

impl CachedAgentMemClient {
    pub fn new(client: AgentMemClient, cache_size: usize) -> Self {
        Self {
            client,
            search_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
        }
    }
    
    pub async fn search_memories_cached(
        &self,
        query: &str,
        user_id: &str,
    ) -> Result<Vec<Memory>, Error> {
        let cache_key = format!("{}:{}", user_id, query);
        
        // 检查缓存
        {
            let cache = self.search_cache.read().await;
            if let Some(cached) = cache.peek(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        // 缓存未命中，执行搜索
        let results = self.client.search_memories(query, user_id).await?;
        
        // 更新缓存
        {
            let mut cache = self.search_cache.write().await;
            cache.put(cache_key, results.clone());
        }
        
        Ok(results)
    }
}
```

---

### Phase 4: 测试与文档 (2-3天)

#### 4.1 单元测试

**新文件**: `agentmen/crates/agent-mem-tools/src/mcp/server_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_add_memory_tool() {
        // 启动测试后端
        let backend = TestBackend::start().await;
        
        // 创建工具
        let tool = AddMemoryTool::new(backend.url());
        
        // 执行工具
        let args = json!({
            "content": "Test memory",
            "user_id": "test_user",
            "agent_id": "test_agent"
        });
        
        let result = tool.execute(args, &ExecutionContext::default()).await;
        assert!(result.is_ok());
        
        let value = result.unwrap();
        assert_eq!(value["success"], true);
        
        backend.stop().await;
    }
    
    #[tokio::test]
    async fn test_search_memory_tool() {
        // ...类似的测试
    }
    
    #[tokio::test]
    async fn test_acl_permission_check() {
        // 测试权限控制
    }
    
    #[tokio::test]
    async fn test_audit_log_recording() {
        // 测试审计日志
    }
}
```

#### 4.2 集成测试

**新文件**: `agentmen/tests/mcp_integration_test.rs`

```rust
//! MCP集成测试
//! 
//! 测试完整的MCP工作流程

use agentmem::*;

#[tokio::test]
async fn test_complete_mcp_workflow() {
    // 1. 启动后端
    let backend = start_test_backend().await;
    
    // 2. 启动MCP服务器
    let mcp_server = start_mcp_server(backend.url()).await;
    
    // 3. 创建MCP客户端
    let client = McpClient::new();
    client.add_server(McpServerConfig {
        name: "agentmem".to_string(),
        server_type: ServerType::Stdio,
        command: Some("./target/release/agentmem-mcp-server".to_string()),
        args: None,
        url: None,
    }).await.unwrap();
    
    // 4. 列出工具
    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.contains_key("agentmem"));
    assert!(tools["agentmem"].len() >= 5);
    
    // 5. 添加记忆
    let add_result = client.execute_tool(
        "agentmem",
        "agentmem_add_memory",
        json!({
            "content": "Integration test memory",
            "user_id": "test_user",
            "agent_id": "test_agent"
        })
    ).await.unwrap();
    
    assert_eq!(add_result["success"], true);
    let memory_id = add_result["memory_id"].as_str().unwrap();
    
    // 6. 搜索记忆
    let search_result = client.execute_tool(
        "agentmem",
        "agentmem_search_memories",
        json!({
            "query": "integration test",
            "user_id": "test_user",
            "limit": 10
        })
    ).await.unwrap();
    
    assert!(search_result["total_results"].as_i64().unwrap() > 0);
    
    // 7. 清理
    client.remove_server("agentmem").await.unwrap();
    mcp_server.stop().await;
    backend.stop().await;
}
```

#### 4.3 Claude Code集成测试

**新文件**: `agentmen/tests/claude_code_integration.sh`

```bash
#!/bin/bash
# Claude Code 集成测试脚本

set -e

echo "🚀 Starting Claude Code Integration Test"

# 1. 编译MCP服务器
echo "📦 Building MCP server..."
cargo build --package mcp-stdio-server --release

# 2. 启动后端
echo "🔧 Starting backend..."
./start_server.sh &
BACKEND_PID=$!
sleep 5

# 3. 配置Claude Code
echo "⚙️  Configuring Claude Code..."
cat > .mcp.json << EOF
{
  "mcpServers": {
    "agentmem": {
      "command": "./target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "AGENTMEM_API_URL": "http://127.0.0.1:8080"
      }
    }
  }
}
EOF

# 4. 测试MCP工具列表
echo "📋 Testing tool list..."
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    ./target/release/agentmem-mcp-server | jq .

# 5. 测试添加记忆
echo "💾 Testing add memory..."
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"Claude Code integration test","user_id":"default","agent_id":"test_agent"}}}' | \
    ./target/release/agentmem-mcp-server | jq .

# 6. 等待索引
sleep 2

# 7. 测试搜索
echo "🔍 Testing search..."
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"Claude Code integration","user_id":"default","limit":5}}}' | \
    ./target/release/agentmem-mcp-server | jq .

# 8. 清理
echo "🧹 Cleaning up..."
kill $BACKEND_PID

echo "✅ Claude Code Integration Test Completed Successfully!"
```

---

## 📊 验收标准

### 代码质量标准

- [ ] **零Mock代码** - 100%生产代码
- [ ] **零TODO项** - 所有功能完整实现
- [ ] **测试覆盖率** ≥ 80%
- [ ] **文档覆盖率** ≥ 90%
- [ ] **Clippy警告** = 0
- [ ] **安全审计** - 通过cargo audit

### 功能完整性标准

- [ ] **STDIO传输** - 完整支持
- [ ] **SSE传输** - 完整支持
- [ ] **MCP客户端** - 完整实现
- [ ] **权限控制** - 完整ACL系统
- [ ] **审计日志** - 完整记录
- [ ] **错误处理** - 优雅降级
- [ ] **性能优化** - 缓存+批处理

### 性能标准

- [ ] **响应时间** - p50 < 50ms, p99 < 200ms
- [ ] **并发能力** - 支持1000+ QPS
- [ ] **内存使用** - < 500MB (正常负载)
- [ ] **启动时间** - < 2秒

### Claude Code集成标准

- [ ] **工具发现** - 100%工具可发现
- [ ] **工具执行** - 100%工具可执行
- [ ] **错误提示** - 清晰的错误信息
- [ ] **配置简单** - 一个JSON文件配置
- [ ] **文档完整** - 完整的使用指南

---

## 🗓️ 实施时间表

| 阶段 | 任务 | 工作量 | 截止日期 |
|------|------|--------|---------|
| Phase 1 | 清理Mock和TODO | 1-2天 | Day 2 |
| Phase 2.1 | SSE传输 | 1天 | Day 3 |
| Phase 2.2 | MCP客户端 | 2天 | Day 5 |
| Phase 2.3 | 权限控制 | 1天 | Day 6 |
| Phase 2.4 | 审计日志 | 1天 | Day 7 |
| Phase 3.1 | 错误处理 | 1天 | Day 8 |
| Phase 3.2 | 性能优化 | 1天 | Day 9 |
| Phase 4.1 | 单元测试 | 1天 | Day 10 |
| Phase 4.2 | 集成测试 | 1天 | Day 11 |
| Phase 4.3 | Claude Code测试 | 1天 | Day 12 |
| **总计** | | **12天** | **Day 12** |

---

## 📝 最小改动原则

### 保留现有功能

✅ **保留**:
- 现有的5个MCP工具
- STDIO传输实现
- PostgreSQL数据库
- 向量搜索功能
- Agent管理
- Memory类型系统

### 最小改动点

🔧 **只改动**:
1. 删除Mock代码（3处）
2. 完成TODO项（2处）
3. 修复HTTP工具（1处）
4. 添加新模块（不影响现有代码）

### 向后兼容

✅ **保证兼容**:
- 现有API接口不变
- 现有数据库schema扩展（不破坏）
- 现有配置文件兼容
- 现有测试脚本兼容

---

## 🎯 成功指标

### 定量指标

- **代码行数**: +3000行（新功能）
- **测试行数**: +2000行（测试覆盖）
- **文档页数**: +20页（API文档）
- **Bug修复**: 100%（已知bug）
- **性能提升**: 2x（缓存后）

### 定性指标

- **生产就绪**: ✅ 可直接部署
- **Claude Code集成**: ✅ 无缝对接
- **开发者体验**: ✅ 配置简单
- **维护性**: ✅ 代码清晰
- **可扩展性**: ✅ 易于扩展

---

## 🚀 立即开始

### Step 1: 创建分支

```bash
cd agentmen
git checkout -b feature/mcp-2.0-production
```

### Step 2: 删除Mock代码

```bash
# 运行自动化清理脚本
./scripts/cleanup_mock.sh
```

### Step 3: 完成TODO项

```bash
# 编辑execution_sandbox.rs
vim crates/agent-mem-tools/src/execution_sandbox.rs
```

### Step 4: 运行测试

```bash
cargo test --all
./test_mcp_integration_fixed.sh
```

### Step 5: 提交更改

```bash
git add .
git commit -m "feat(mcp): Phase 1 - Clean up mock code and complete TODOs"
git push origin feature/mcp-2.0-production
```

---

## 📚 参考资料

### 优秀实现参考

1. **mem0** (`source/mem0/openmemory/api/app/mcp_server.py`)
   - ✅ 完整的FastMCP实现
   - ✅ 优雅的错误处理
   - ✅ 完整的权限控制
   - ✅ SSE传输

2. **MIRIX** (`source/MIRIX/mirix/functions/mcp_client/manager.py`)
   - ✅ 完整的客户端实现
   - ✅ 配置持久化
   - ✅ 多服务器管理

3. **Anthropic MCP Spec**
   - https://modelcontextprotocol.io/
   - Protocol version: 2024-11-05

---

## ✅ 验收清单

### Phase 1完成标准

- [ ] 所有Mock代码已删除
- [ ] 所有TODO项已完成
- [ ] HTTP工具真实实现
- [ ] 所有测试通过
- [ ] 代码通过Clippy检查

### Phase 2完成标准

- [ ] SSE传输实现完成
- [ ] MCP客户端实现完成
- [ ] 权限控制系统完成
- [ ] 审计日志系统完成
- [ ] 集成测试通过

### Phase 3完成标准

- [ ] 完整错误处理
- [ ] 性能优化完成
- [ ] 缓存层实现
- [ ] 压力测试通过

### Phase 4完成标准

- [ ] 单元测试覆盖率≥80%
- [ ] 集成测试完成
- [ ] Claude Code测试通过
- [ ] 文档完整
- [ ] 生产就绪

---

## 📞 支持与反馈

**项目仓库**: `/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen`

**相关文档**:
- [SEARCH_BUG_FINAL_SOLUTION.md](SEARCH_BUG_FINAL_SOLUTION.md)
- [FINAL_COMPREHENSIVE_REPORT.md](FINAL_COMPREHENSIVE_REPORT.md)
- [MCP_DEEP_ANALYSIS_AND_VERIFICATION.md](MCP_DEEP_ANALYSIS_AND_VERIFICATION.md)

**联系方式**: 在项目Issue中提问

---

**让我们开始构建生产级的AgentMem MCP 2.0！** 🚀✨

