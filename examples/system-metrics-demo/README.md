# System Metrics Demo - 系统指标监控演示

本示例演示如何使用 AgentMem 的监控系统收集和导出系统指标。

## 功能特性

### 1. CPU 使用率监控
- ✅ 实时收集 CPU 使用率（0-100%）
- ✅ 使用 `sysinfo` crate 获取系统信息
- ✅ 定期更新（可配置间隔）

### 2. 内存使用率监控
- ✅ 系统总内存（bytes）
- ✅ 已使用内存（bytes）
- ✅ 可用内存（bytes）
- ✅ 自动计算使用率百分比

### 3. Prometheus 指标导出
- ✅ HTTP 服务器（默认端口 9090）
- ✅ `/metrics` 端点
- ✅ 标准 Prometheus 格式

## 运行示例

```bash
# 从 agentmen 目录运行
cargo run --package system-metrics-demo

# 或者从项目根目录运行
cd agentmen
cargo run --example system-metrics-demo
```

## 查看指标

启动示例后，访问以下 URL 查看 Prometheus 指标：

```
http://localhost:9090/metrics
```

## 可用指标

### 系统指标

| 指标名称 | 类型 | 描述 |
|---------|------|------|
| `agentmem_cpu_usage_percent` | Gauge | CPU 使用率（0-100） |
| `agentmem_system_memory_total_bytes` | Gauge | 系统总内存（bytes） |
| `agentmem_system_memory_used_bytes` | Gauge | 已使用内存（bytes） |
| `agentmem_system_memory_available_bytes` | Gauge | 可用内存（bytes） |

### 应用指标

| 指标名称 | 类型 | 描述 |
|---------|------|------|
| `agentmem_requests_total` | Counter | 请求总数 |
| `agentmem_errors_total` | Counter | 错误总数 |
| `agentmem_active_connections` | Gauge | 活动连接数 |
| `agentmem_memory_usage_bytes` | Gauge | 应用内存使用（bytes） |
| `agentmem_request_duration_seconds` | Histogram | 请求延迟（秒） |
| `agentmem_tool_execution_duration_seconds` | Histogram | 工具执行时间（秒） |

## 示例输出

```
# HELP agentmem_cpu_usage_percent CPU usage percentage (0-100)
# TYPE agentmem_cpu_usage_percent gauge
agentmem_cpu_usage_percent 23.45

# HELP agentmem_system_memory_total_bytes Total system memory in bytes
# TYPE agentmem_system_memory_total_bytes gauge
agentmem_system_memory_total_bytes 17179869184

# HELP agentmem_system_memory_used_bytes Used system memory in bytes
# TYPE agentmem_system_memory_used_bytes gauge
agentmem_system_memory_used_bytes 12884901888

# HELP agentmem_system_memory_available_bytes Available system memory in bytes
# TYPE agentmem_system_memory_available_bytes gauge
agentmem_system_memory_available_bytes 4294967296

# HELP agentmem_requests_total Total number of requests
# TYPE agentmem_requests_total counter
agentmem_requests_total{method="GET",endpoint="/api/health",status="200"} 10

# HELP agentmem_active_connections Number of active connections
# TYPE agentmem_active_connections gauge
agentmem_active_connections 50
```

## 集成到生产环境

### 1. 启动系统监控

```rust
use agent_mem_observability::metrics::{MetricsRegistry, SystemMetricsMonitor};
use std::time::Duration;

// 创建指标注册表
let registry = MetricsRegistry::new();
let collector = registry.collector();

// 创建系统监控器（每 5 秒收集一次）
let monitor = SystemMetricsMonitor::new(
    collector.clone(),
    Duration::from_secs(5)
);

// 启动监控（返回 JoinHandle）
let monitor_handle = monitor.start();
```

### 2. 启动 Prometheus 服务器

```rust
// 启动指标服务器
tokio::spawn(async move {
    agent_mem_observability::metrics::start_metrics_server(
        registry.registry(),
        9090
    ).await
});
```

### 3. 记录自定义指标

```rust
// 记录请求
collector.record_request("GET", "/api/users", 200).await;

// 记录请求延迟
collector.record_request_duration("GET", "/api/users", 0.123).await;

// 设置活动连接数
collector.set_active_connections(42).await;

// 设置应用内存使用
collector.set_memory_usage(1024 * 1024 * 256).await;

// 手动设置 CPU 使用率（如果不使用 SystemMetricsMonitor）
collector.set_cpu_usage(45.5).await;

// 手动设置系统内存（如果不使用 SystemMetricsMonitor）
collector.set_system_memory(
    16_000_000_000,  // total
    8_000_000_000,   // used
    8_000_000_000    // available
).await;
```

## 配置 Prometheus 抓取

在 `prometheus.yml` 中添加：

```yaml
scrape_configs:
  - job_name: 'agentmem'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 5s
```

## 配置 Grafana 仪表板

### CPU 使用率面板

```
Query: agentmem_cpu_usage_percent
Visualization: Gauge
Unit: percent (0-100)
```

### 内存使用率面板

```
Query: (agentmem_system_memory_used_bytes / agentmem_system_memory_total_bytes) * 100
Visualization: Gauge
Unit: percent (0-100)
```

### 请求速率面板

```
Query: rate(agentmem_requests_total[5m])
Visualization: Graph
Unit: requests/sec
```

### 请求延迟面板

```
Query: histogram_quantile(0.95, rate(agentmem_request_duration_seconds_bucket[5m]))
Visualization: Graph
Unit: seconds
Legend: P95 Latency
```

## 性能优化

### 1. 调整收集间隔

```rust
// 生产环境建议 5-10 秒
let monitor = SystemMetricsMonitor::new(
    collector,
    Duration::from_secs(5)
);

// 开发环境可以更频繁
let monitor = SystemMetricsMonitor::new(
    collector,
    Duration::from_secs(1)
);
```

### 2. 减少指标基数

避免使用高基数标签（如用户 ID、请求 ID）：

```rust
// ❌ 不推荐：高基数
collector.record_request("GET", &format!("/api/users/{}", user_id), 200).await;

// ✅ 推荐：低基数
collector.record_request("GET", "/api/users/:id", 200).await;
```

## 故障排查

### 问题：指标服务器无法启动

```
Error: Address already in use
```

**解决方案**：更改端口号

```rust
start_metrics_server(registry.registry(), 9091).await
```

### 问题：CPU 使用率始终为 0

**原因**：`sysinfo` 需要时间初始化

**解决方案**：等待几个收集周期后再查看指标

### 问题：内存指标不准确

**原因**：不同操作系统的内存计算方式不同

**解决方案**：使用 `used_memory` 和 `total_memory` 的比率而不是绝对值

## 相关文档

- [Prometheus 文档](https://prometheus.io/docs/)
- [Grafana 文档](https://grafana.com/docs/)
- [sysinfo crate 文档](https://docs.rs/sysinfo/)
- [AgentMem 监控系统设计](../../../doc/technical-design/memory-systems/mem15.md)

## 许可证

MIT License

