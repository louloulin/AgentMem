# P1-4 实施计划：添加 Metrics 指标

**创建日期**: 2025-01-10  
**计划依据**: 完整工作流程指南 - 阶段 2

---

## 任务概述

- **任务编号**: P1-4
- **任务名称**: 添加 Metrics 指标收集和监控
- **优先级**: P1 - 重要但不紧急
- **预计工作量**: 4-5 小时
- **依赖关系**: 无（独立任务）

---

## 功能点分解

### 1. 集成 Metrics Crate (1h)

**详细说明**:
- 添加 `metrics` crate 到 `Cargo.toml`
- 添加 `metrics-exporter-prometheus` 用于 Prometheus 导出
- 配置 metrics 初始化和导出端点

**技术要点**:
- 使用 `metrics` crate (v0.21+)
- 支持 Prometheus 格式导出
- 配置 HTTP 端点 `/metrics`

### 2. 添加核心指标 (2h)

**详细说明**:
- **请求计数器** (Counter): 跟踪 API 请求总数
- **响应时间** (Histogram): 记录请求处理时间分布
- **错误率** (Counter): 跟踪错误发生次数
- **活跃连接数** (Gauge): 监控当前活跃连接
- **缓存命中率** (Gauge): 监控缓存效率

**指标命名规范**:
```
agentmem_requests_total{method, endpoint, status}
agentmem_request_duration_seconds{method, endpoint}
agentmem_errors_total{type, component}
agentmem_active_connections
agentmem_cache_hit_rate{cache_type}
```

### 3. 集成到现有代码 (1h)

**详细说明**:
- 在 HTTP 服务器中添加 metrics 中间件
- 在关键业务逻辑中添加指标记录
- 在存储层添加性能指标

**集成点**:
- `agent-mem-server/src/routes/` - API 路由
- `agent-mem-core/src/orchestrator/` - 业务逻辑
- `agent-mem-storage/src/` - 存储层

### 4. 配置 Prometheus 导出器 (0.5h)

**详细说明**:
- 配置 Prometheus 导出端点
- 设置指标刷新间隔
- 配置标签和维度

### 5. 创建测试用例 (0.5h)

**详细说明**:
- 测试指标收集功能
- 测试 Prometheus 导出格式
- 测试指标准确性

---

## 实施步骤

### 步骤 1: 代码调研 (15 分钟)

**操作**:
```bash
# 搜索现有的 metrics 使用
grep -r "metrics::" agentmen/crates/
grep -r "prometheus" agentmen/crates/
```

**预期结果**:
- 了解现有 metrics 使用情况
- 识别需要添加指标的位置

---

### 步骤 2: 添加依赖 (15 分钟)

**操作**:
修改 `crates/agent-mem-server/Cargo.toml`:
```toml
[dependencies]
metrics = "0.21"
metrics-exporter-prometheus = "0.12"
```

**预期结果**:
- 依赖添加成功
- `cargo build` 编译通过

---

### 步骤 3: 初始化 Metrics 系统 (30 分钟)

**操作**:
在 `crates/agent-mem-server/src/main.rs` 中初始化:
```rust
use metrics_exporter_prometheus::PrometheusBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 Prometheus 导出器
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(([0, 0, 0, 0], 9090))
        .install()
        .expect("Failed to install Prometheus exporter");
    
    // ... 其他初始化代码
}
```

**预期结果**:
- Metrics 系统初始化成功
- `/metrics` 端点可访问

---

### 步骤 4: 添加请求计数器 (30 分钟)

**操作**:
创建 `crates/agent-mem-server/src/metrics.rs`:
```rust
use metrics::{counter, histogram, gauge};

pub fn record_request(method: &str, endpoint: &str, status: u16) {
    counter!("agentmem_requests_total", 
        "method" => method, 
        "endpoint" => endpoint, 
        "status" => status.to_string()
    ).increment(1);
}

pub fn record_request_duration(method: &str, endpoint: &str, duration: f64) {
    histogram!("agentmem_request_duration_seconds",
        "method" => method,
        "endpoint" => endpoint
    ).record(duration);
}

pub fn record_error(error_type: &str, component: &str) {
    counter!("agentmem_errors_total",
        "type" => error_type,
        "component" => component
    ).increment(1);
}

pub fn set_active_connections(count: i64) {
    gauge!("agentmem_active_connections").set(count as f64);
}

pub fn set_cache_hit_rate(cache_type: &str, rate: f64) {
    gauge!("agentmem_cache_hit_rate",
        "cache_type" => cache_type
    ).set(rate);
}
```

**预期结果**:
- Metrics 辅助函数创建成功
- 编译通过

---

### 步骤 5: 集成到 HTTP 中间件 (45 分钟)

**操作**:
创建 `crates/agent-mem-server/src/middleware/metrics.rs`:
```rust
use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

pub async fn metrics_middleware(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();
    
    crate::metrics::record_request(&method, &path, status);
    crate::metrics::record_request_duration(&method, &path, duration);
    
    response
}
```

**预期结果**:
- 中间件创建成功
- 自动记录所有 HTTP 请求

---

### 步骤 6: 集成到业务逻辑 (45 分钟)

**操作**:
在 `crates/agent-mem-core/src/orchestrator/mod.rs` 中添加:
```rust
use metrics::counter;

impl AgentOrchestrator {
    pub async fn step(&mut self, request: ChatRequest) -> Result<ChatResponse> {
        // 验证输入
        request.validate()?;
        
        // 记录请求
        counter!("agentmem_orchestrator_requests_total").increment(1);
        
        let start = std::time::Instant::now();
        
        // ... 业务逻辑
        
        let duration = start.elapsed().as_secs_f64();
        histogram!("agentmem_orchestrator_duration_seconds").record(duration);
        
        Ok(response)
    }
}
```

**预期结果**:
- 业务逻辑指标记录成功
- 编译通过

---

### 步骤 7: 添加存储层指标 (30 分钟)

**操作**:
在 `crates/agent-mem-storage/src/backends/postgres.rs` 中添加:
```rust
use metrics::{counter, histogram};

impl PostgresStore {
    async fn execute_query(&self, query: &str) -> Result<()> {
        let start = std::time::Instant::now();
        
        let result = sqlx::query(query).execute(&self.pool).await;
        
        let duration = start.elapsed().as_secs_f64();
        histogram!("agentmem_storage_query_duration_seconds",
            "backend" => "postgres"
        ).record(duration);
        
        match result {
            Ok(_) => {
                counter!("agentmem_storage_queries_total",
                    "backend" => "postgres",
                    "status" => "success"
                ).increment(1);
            }
            Err(e) => {
                counter!("agentmem_storage_queries_total",
                    "backend" => "postgres",
                    "status" => "error"
                ).increment(1);
                return Err(e.into());
            }
        }
        
        Ok(())
    }
}
```

**预期结果**:
- 存储层指标记录成功
- 编译通过

---

### 步骤 8: 创建测试用例 (30 分钟)

**操作**:
创建 `crates/agent-mem-server/tests/metrics_test.rs`:
```rust
#[tokio::test]
async fn test_metrics_collection() {
    // 初始化 metrics
    let _ = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install();
    
    // 记录一些指标
    crate::metrics::record_request("GET", "/api/memories", 200);
    crate::metrics::record_request_duration("GET", "/api/memories", 0.123);
    
    // 验证指标已记录
    // (Prometheus 导出器会自动收集)
}

#[tokio::test]
async fn test_prometheus_export() {
    // 测试 Prometheus 导出格式
    let client = reqwest::Client::new();
    let response = client.get("http://localhost:9090/metrics")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body = response.text().await.unwrap();
    assert!(body.contains("agentmem_requests_total"));
}
```

**预期结果**:
- 测试用例创建成功
- 测试通过

---

### 步骤 9: 运行测试 (15 分钟)

**操作**:
```bash
cargo test --package agent-mem-server metrics
```

**预期结果**:
- 所有 metrics 测试通过
- 无编译错误

---

### 步骤 10: 验证 Prometheus 导出 (15 分钟)

**操作**:
```bash
# 启动服务器
cargo run --package agent-mem-server

# 访问 metrics 端点
curl http://localhost:9090/metrics
```

**预期结果**:
- Metrics 端点返回 Prometheus 格式数据
- 包含所有定义的指标

---

## 验收标准

### 必须满足的条件

- [ ] **依赖集成**: `metrics` 和 `metrics-exporter-prometheus` crate 已添加
- [ ] **核心指标**: 至少 5 个核心指标已实现
  - [ ] `agentmem_requests_total` - 请求计数
  - [ ] `agentmem_request_duration_seconds` - 响应时间
  - [ ] `agentmem_errors_total` - 错误计数
  - [ ] `agentmem_active_connections` - 活跃连接
  - [ ] `agentmem_cache_hit_rate` - 缓存命中率
- [ ] **HTTP 集成**: Metrics 中间件已集成到 HTTP 服务器
- [ ] **业务逻辑集成**: Orchestrator 中已添加指标记录
- [ ] **存储层集成**: 存储层查询已添加性能指标
- [ ] **Prometheus 导出**: `/metrics` 端点可访问并返回正确格式
- [ ] **测试覆盖**: 至少 2 个测试用例通过
- [ ] **编译成功**: `cargo build --workspace` 无错误
- [ ] **文档更新**: 添加 Metrics 使用文档

### 可选的优化

- [ ] 添加更多业务指标（如记忆创建数、搜索次数等）
- [ ] 配置 Grafana 仪表板
- [ ] 添加告警规则
- [ ] 性能优化（减少指标记录开销）

---

## 参考资料

### 相关代码位置

- **HTTP 服务器**: `crates/agent-mem-server/src/main.rs`
- **路由处理**: `crates/agent-mem-server/src/routes/`
- **业务逻辑**: `crates/agent-mem-core/src/orchestrator/mod.rs`
- **存储层**: `crates/agent-mem-storage/src/backends/`

### 技术文档

- Metrics crate: https://docs.rs/metrics/
- Prometheus exporter: https://docs.rs/metrics-exporter-prometheus/
- Prometheus 格式: https://prometheus.io/docs/instrumenting/exposition_formats/

---

## 风险和缓解措施

### 风险 1: 性能开销

**描述**: Metrics 收集可能增加请求延迟

**缓解措施**:
- 使用异步指标记录
- 避免在热路径中进行复杂计算
- 使用采样（如果必要）

### 风险 2: 指标爆炸

**描述**: 过多的标签维度可能导致指标数量爆炸

**缓解措施**:
- 限制标签基数（cardinality）
- 避免使用高基数标签（如 user_id）
- 定期审查指标使用情况

---

## 下一步建议

完成 P1-4 后，建议：

1. **立即**: 部署到测试环境验证 metrics 收集
2. **1 周内**: 配置 Grafana 仪表板
3. **2 周内**: 设置告警规则
4. **后续**: 继续 P1-5 (统一日志系统)

---

**计划创建日期**: 2025-01-10  
**预计开始日期**: 待定  
**预计完成日期**: 开始后 1 天内

