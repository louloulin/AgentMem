# P1-5 实施计划：统一日志系统

**创建日期**: 2025-01-10  
**计划依据**: 完整工作流程指南 - 阶段 2

---

## 任务概述

- **任务编号**: P1-5
- **任务名称**: 统一日志系统和日志格式
- **优先级**: P1 - 重要但不紧急
- **预计工作量**: 2-3 小时
- **依赖关系**: 无（独立任务）

---

## 功能点分解

### 1. 集成结构化日志 Crate (0.5h)

**详细说明**:
- 添加 `tracing` crate 替代 `log`
- 添加 `tracing-subscriber` 用于日志订阅
- 添加 `tracing-appender` 用于文件输出
- 配置 JSON 格式输出

**技术要点**:
- 使用 `tracing` crate (v0.1+)
- 支持结构化日志（JSON 格式）
- 支持日志级别过滤
- 支持多个输出目标（控制台 + 文件）

### 2. 定义统一日志格式 (0.5h)

**详细说明**:
- **时间戳**: ISO 8601 格式
- **日志级别**: TRACE, DEBUG, INFO, WARN, ERROR
- **组件名称**: 标识日志来源
- **请求 ID**: 跟踪请求链路
- **结构化字段**: 使用 key-value 对

**日志格式示例**:
```json
{
  "timestamp": "2025-01-10T10:30:45.123Z",
  "level": "INFO",
  "component": "orchestrator",
  "request_id": "req-123456",
  "message": "Processing chat request",
  "user_id": "user-789",
  "duration_ms": 123
}
```

### 3. 替换现有日志调用 (1h)

**详细说明**:
- 将 `log::info!()` 替换为 `tracing::info!()`
- 添加结构化字段到日志调用
- 添加 span 跟踪请求生命周期

**替换范围**:
- `agent-mem-server/` - HTTP 服务器
- `agent-mem-core/` - 核心业务逻辑
- `agent-mem-storage/` - 存储层

### 4. 配置日志输出 (0.5h)

**详细说明**:
- 配置控制台输出（开发环境）
- 配置文件输出（生产环境）
- 配置日志轮转（按大小或时间）
- 配置日志级别（通过环境变量）

### 5. 添加请求跟踪 (0.5h)

**详细说明**:
- 为每个请求生成唯一 ID
- 在整个请求链路中传递 request_id
- 在所有日志中包含 request_id

---

## 实施步骤

### 步骤 1: 代码调研 (15 分钟)

**操作**:
```bash
# 搜索现有的日志使用
grep -r "log::" agentmen/crates/ | wc -l
grep -r "tracing::" agentmen/crates/ | wc -l
```

**预期结果**:
- 了解现有日志使用情况
- 识别需要替换的位置

---

### 步骤 2: 添加依赖 (15 分钟)

**操作**:
修改 `Cargo.toml`:
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
```

**预期结果**:
- 依赖添加成功
- `cargo build` 编译通过

---

### 步骤 3: 初始化 Tracing 系统 (20 分钟)

**操作**:
在 `crates/agent-mem-server/src/main.rs` 中初始化:
```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    let file_appender = tracing_appender::rolling::daily("./logs", "agentmem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().json().with_writer(non_blocking))
        .init();
    
    tracing::info!("AgentMem server starting");
    
    // ... 其他初始化代码
}
```

**预期结果**:
- Tracing 系统初始化成功
- 日志输出到控制台和文件

---

### 步骤 4: 创建日志辅助模块 (20 分钟)

**操作**:
创建 `crates/agent-mem-server/src/logging.rs`:
```rust
use tracing::{info, warn, error, debug};
use uuid::Uuid;

pub struct RequestContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_id: None,
            organization_id: None,
        }
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_org(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }
}

// 日志宏包装
#[macro_export]
macro_rules! log_info {
    ($ctx:expr, $msg:expr, $($key:tt = $value:expr),*) => {
        tracing::info!(
            request_id = %$ctx.request_id,
            user_id = ?$ctx.user_id,
            organization_id = ?$ctx.organization_id,
            $($key = $value,)*
            $msg
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($ctx:expr, $msg:expr, $($key:tt = $value:expr),*) => {
        tracing::error!(
            request_id = %$ctx.request_id,
            user_id = ?$ctx.user_id,
            organization_id = ?$ctx.organization_id,
            $($key = $value,)*
            $msg
        );
    };
}
```

**预期结果**:
- 日志辅助模块创建成功
- 编译通过

---

### 步骤 5: 添加请求跟踪中间件 (25 分钟)

**操作**:
创建 `crates/agent-mem-server/src/middleware/tracing.rs`:
```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{info_span, Instrument};
use uuid::Uuid;

pub async fn tracing_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    // 将 request_id 添加到请求扩展中
    req.extensions_mut().insert(request_id.clone());
    
    // 创建 span
    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        path = %path,
    );
    
    async move {
        tracing::info!("Request started");
        let response = next.run(req).await;
        tracing::info!(status = %response.status(), "Request completed");
        response
    }
    .instrument(span)
    .await
}
```

**预期结果**:
- 请求跟踪中间件创建成功
- 每个请求自动生成 request_id

---

### 步骤 6: 替换核心业务逻辑日志 (30 分钟)

**操作**:
在 `crates/agent-mem-core/src/orchestrator/mod.rs` 中替换:
```rust
// 旧代码
log::info!("Processing chat request");

// 新代码
tracing::info!(
    user_id = %request.user_id,
    message_count = request.messages.len(),
    "Processing chat request"
);

// 使用 span 跟踪操作
let span = tracing::info_span!(
    "orchestrator_step",
    user_id = %request.user_id,
);

async move {
    // 业务逻辑
    tracing::debug!("Retrieving memories");
    let memories = self.retrieve_memories(&request).await?;
    
    tracing::debug!(memory_count = memories.len(), "Memories retrieved");
    
    // ...
}
.instrument(span)
.await
```

**预期结果**:
- 核心业务逻辑日志已替换
- 包含结构化字段
- 编译通过

---

### 步骤 7: 替换存储层日志 (20 分钟)

**操作**:
在 `crates/agent-mem-storage/src/backends/postgres.rs` 中替换:
```rust
// 旧代码
log::info!("Executing query: {}", query);

// 新代码
tracing::debug!(
    query = %query,
    backend = "postgres",
    "Executing database query"
);

// 错误日志
tracing::error!(
    error = %e,
    query = %query,
    backend = "postgres",
    "Database query failed"
);
```

**预期结果**:
- 存储层日志已替换
- 包含后端类型标识
- 编译通过

---

### 步骤 8: 配置环境变量 (10 分钟)

**操作**:
创建 `.env.example`:
```bash
# 日志级别配置
RUST_LOG=info,agent_mem_core=debug,agent_mem_storage=debug

# 日志输出目录
LOG_DIR=./logs

# 日志轮转配置
LOG_ROTATION=daily  # daily, hourly, or size-based
```

**预期结果**:
- 环境变量配置文件创建
- 可通过环境变量控制日志级别

---

### 步骤 9: 创建测试用例 (15 分钟)

**操作**:
创建 `crates/agent-mem-server/tests/logging_test.rs`:
```rust
use tracing_subscriber::layer::SubscriberExt;

#[test]
fn test_structured_logging() {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json());
    
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(
            user_id = "test-user",
            action = "test",
            "Test log message"
        );
    });
}

#[test]
fn test_request_context() {
    let ctx = crate::logging::RequestContext::new()
        .with_user("user-123".to_string())
        .with_org("org-456".to_string());
    
    assert!(!ctx.request_id.is_empty());
    assert_eq!(ctx.user_id, Some("user-123".to_string()));
    assert_eq!(ctx.organization_id, Some("org-456".to_string()));
}
```

**预期结果**:
- 测试用例创建成功
- 测试通过

---

### 步骤 10: 运行测试和验证 (10 分钟)

**操作**:
```bash
# 运行测试
cargo test --package agent-mem-server logging

# 启动服务器验证日志输出
RUST_LOG=debug cargo run --package agent-mem-server

# 检查日志文件
cat logs/agentmem.log.* | jq .
```

**预期结果**:
- 所有测试通过
- 日志输出格式正确
- JSON 格式可解析

---

## 验收标准

### 必须满足的条件

- [ ] **依赖集成**: `tracing`, `tracing-subscriber`, `tracing-appender` 已添加
- [ ] **日志格式**: 统一使用 JSON 格式输出
- [ ] **结构化字段**: 所有日志包含 timestamp, level, component
- [ ] **请求跟踪**: 每个请求有唯一 request_id
- [ ] **日志级别**: 支持通过环境变量配置
- [ ] **文件输出**: 日志输出到文件并支持轮转
- [ ] **核心替换**: 至少 80% 的 `log::` 调用已替换为 `tracing::`
- [ ] **测试覆盖**: 至少 2 个测试用例通过
- [ ] **编译成功**: `cargo build --workspace` 无错误
- [ ] **文档更新**: 添加日志配置文档

### 可选的优化

- [ ] 集成到 ELK/Loki 等日志聚合系统
- [ ] 添加日志采样（减少高频日志）
- [ ] 添加敏感信息脱敏
- [ ] 性能优化（异步日志写入）

---

## 参考资料

### 相关代码位置

- **HTTP 服务器**: `crates/agent-mem-server/src/main.rs`
- **业务逻辑**: `crates/agent-mem-core/src/orchestrator/mod.rs`
- **存储层**: `crates/agent-mem-storage/src/backends/`

### 技术文档

- Tracing crate: https://docs.rs/tracing/
- Tracing subscriber: https://docs.rs/tracing-subscriber/
- Structured logging best practices: https://www.honeycomb.io/blog/structured-logging-and-your-team

---

## 风险和缓解措施

### 风险 1: 性能开销

**描述**: 结构化日志可能增加 CPU 和 I/O 开销

**缓解措施**:
- 使用异步日志写入
- 在生产环境降低日志级别
- 使用日志采样

### 风险 2: 敏感信息泄露

**描述**: 日志可能包含敏感信息（如密码、token）

**缓解措施**:
- 添加敏感字段过滤
- 定期审查日志内容
- 配置日志访问权限

---

## 下一步建议

完成 P1-5 后，建议：

1. **立即**: 部署到测试环境验证日志输出
2. **1 周内**: 集成到日志聚合系统（如 Loki）
3. **2 周内**: 配置日志告警规则
4. **后续**: 继续 P1-6 (添加访问控制)

---

**计划创建日期**: 2025-01-10  
**预计开始日期**: P1-4 完成后  
**预计完成日期**: 开始后 0.5 天内

