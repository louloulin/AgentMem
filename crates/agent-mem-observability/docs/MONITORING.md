# AgentMem 监控和可观测性指南

本文档介绍如何部署和使用 AgentMem 的监控和可观测性系统。

## 目录

- [架构概览](#架构概览)
- [快速开始](#快速开始)
- [组件说明](#组件说明)
- [指标说明](#指标说明)
- [告警规则](#告警规则)
- [日志聚合](#日志聚合)
- [分布式追踪](#分布式追踪)
- [仪表板](#仪表板)
- [故障排查](#故障排查)

## 架构概览

AgentMem 的可观测性系统包含以下组件：

```
┌─────────────────────────────────────────────────────────────────┐
│                        AgentMem 应用                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Metrics     │  │   Tracing    │  │   Logging    │          │
│  │(Prometheus)  │  │(OpenTelemetry)│ │  (Structured)│          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
└─────────┼──────────────────┼──────────────────┼─────────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   Prometheus    │  │     Jaeger      │  │    Logstash     │
│  (指标收集)      │  │  (分布式追踪)    │  │   (日志处理)     │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │
         │                    │                    ▼
         │                    │           ┌─────────────────┐
         │                    │           │ Elasticsearch   │
         │                    │           │   (日志存储)     │
         │                    │           └────────┬────────┘
         │                    │                    │
         └────────────────────┴────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │     Grafana     │
                    │  (统一可视化)    │
                    └─────────────────┘
```

## 快速开始

### 1. 启动监控栈

使用 Docker Compose 启动完整的监控栈：

```bash
cd crates/agent-mem-observability
docker-compose -f docker-compose.monitoring.yml up -d
```

这将启动以下服务：

- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)
- **Jaeger UI**: http://localhost:16686
- **Kibana**: http://localhost:5601
- **AlertManager**: http://localhost:9093

### 2. 配置 AgentMem

在您的应用中初始化可观测性：

```rust
use agent_mem_observability::{init_observability, ObservabilityConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ObservabilityConfig {
        service_name: "agentmem".to_string(),
        otlp_endpoint: Some("http://localhost:4317".to_string()),
        enable_metrics: true,
        metrics_port: 9091,
        log_level: "info".to_string(),
        json_logging: true,
    };
    
    init_observability(config).await?;
    
    // 您的应用代码
    Ok(())
}
```

### 3. 访问仪表板

打开 Grafana (http://localhost:3000)，使用默认凭据登录：
- 用户名: `admin`
- 密码: `admin`

导入预配置的 AgentMem 仪表板。

## 组件说明

### Prometheus

**功能**: 指标收集和存储

**配置文件**: `prometheus/prometheus.yml`

**抓取间隔**: 15 秒

**数据保留**: 15 天（默认）

**主要抓取目标**:
- AgentMem 主服务 (`:9091/metrics`)
- Node Exporter (系统指标)
- Prometheus 自身
- Grafana
- Jaeger
- Elasticsearch

### Grafana

**功能**: 统一可视化平台

**配置文件**: `grafana/provisioning/`

**数据源**:
- Prometheus (指标)
- Jaeger (追踪)
- Elasticsearch (日志)

**预配置仪表板**:
- AgentMem 概览
- 性能监控
- 错误分析
- 资源使用

### Jaeger

**功能**: 分布式追踪

**端口**:
- UI: 16686
- OTLP gRPC: 4317
- OTLP HTTP: 4318
- Jaeger Collector: 14268

**采样策略**: 全量采样（生产环境建议调整）

### Elasticsearch + Logstash + Kibana (ELK)

**功能**: 日志聚合和分析

**Logstash 输入**:
- TCP: 5000
- UDP: 5000
- HTTP: 8080

**索引模式**:
- `agentmem-logs-*`: 所有日志
- `agentmem-errors-*`: 错误日志
- `agentmem-performance-*`: 性能日志

### AlertManager

**功能**: 告警管理和路由

**配置文件**: `alertmanager/config.yml`

**告警渠道**:
- Email
- Slack
- Webhook

## 指标说明

### 请求指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_requests_total` | Counter | 总请求数 |
| `agentmem_errors_total` | Counter | 总错误数 |
| `agentmem_request_duration_seconds` | Histogram | 请求延迟 |

**标签**: `method`, `endpoint`, `status`

### 工具执行指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_tool_execution_duration_seconds` | Histogram | 工具执行延迟 |

**标签**: `tool_name`

### 资源指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_memory_usage_bytes` | Gauge | 内存使用量 |
| `agentmem_active_connections` | Gauge | 活跃连接数 |
| `agentmem_system_cpu_usage_percent` | Gauge | CPU 使用率 |

### 缓存指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_cache_hits_total` | Counter | 缓存命中数 |
| `agentmem_cache_misses_total` | Counter | 缓存未命中数 |

### 数据库指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_db_connections_active` | Gauge | 活跃数据库连接 |
| `agentmem_db_connections_max` | Gauge | 最大数据库连接 |
| `agentmem_db_query_duration_seconds` | Histogram | 数据库查询延迟 |

### 向量搜索指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_vector_search_duration_seconds` | Histogram | 向量搜索延迟 |

### 图查询指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_graph_query_duration_seconds` | Histogram | 图查询延迟 |

### 批量操作指标

| 指标名称 | 类型 | 说明 |
|---------|------|------|
| `agentmem_batch_operations_total` | Counter | 批量操作总数 |
| `agentmem_batch_failures_total` | Counter | 批量操作失败数 |

## 告警规则

### 服务可用性告警

| 告警名称 | 条件 | 严重程度 | 说明 |
|---------|------|---------|------|
| ServiceDown | `up == 0` for 1m | Critical | 服务不可用 |
| HighErrorRate | `rate(errors) > 10/s` for 5m | Warning | 错误率过高 |

### 性能告警

| 告警名称 | 条件 | 严重程度 | 说明 |
|---------|------|---------|------|
| HighRequestLatency | `P95 > 1.0s` for 5m | Warning | 请求延迟过高 |
| HighToolExecutionLatency | `P95 > 5.0s` for 5m | Warning | 工具执行延迟过高 |
| HighMemoryUsage | `memory > 1GB` for 5m | Warning | 内存使用过高 |

### 资源告警

| 告警名称 | 条件 | 严重程度 | 说明 |
|---------|------|---------|------|
| HighCPUUsage | `CPU > 80%` for 5m | Warning | CPU 使用率过高 |
| TooManyActiveConnections | `connections > 1000` for 5m | Warning | 活跃连接数过多 |

### 缓存告警

| 告警名称 | 条件 | 严重程度 | 说明 |
|---------|------|---------|------|
| LowCacheHitRate | `hit_rate < 50%` for 10m | Warning | 缓存命中率过低 |

### 数据库告警

| 告警名称 | 条件 | 严重程度 | 说明 |
|---------|------|---------|------|
| DatabaseConnectionPoolExhausted | `active/max > 90%` for 5m | Critical | 数据库连接池即将耗尽 |
| SlowDatabaseQueries | `P95 > 1.0s` for 5m | Warning | 数据库查询过慢 |

## 日志聚合

### 日志格式

AgentMem 使用结构化 JSON 日志：

```json
{
  "timestamp": "2025-10-12T10:00:00Z",
  "level": "info",
  "service_name": "agentmem",
  "trace_id": "abc123",
  "span_id": "def456",
  "message": "Request processed",
  "method": "GET",
  "endpoint": "/api/users",
  "status_code": 200,
  "duration_ms": 45.2,
  "user_id": "user123"
}
```

### 发送日志到 Logstash

**TCP**:
```rust
// 配置日志输出到 TCP
let tcp_appender = TcpAppender::new("localhost:5000");
```

**HTTP**:
```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{"level":"info","message":"Test log"}'
```

### Kibana 查询

访问 Kibana (http://localhost:5601)，创建索引模式：

1. 进入 Management > Index Patterns
2. 创建索引模式: `agentmem-logs-*`
3. 选择时间字段: `@timestamp`
4. 开始查询日志

**常用查询**:
- 错误日志: `log_level:error`
- 特定用户: `user_id:"user123"`
- 慢请求: `duration_ms:>1000`
- 追踪关联: `trace_id:"abc123"`

## 分布式追踪

### 使用追踪

```rust
use tracing::{info, instrument};

#[instrument]
async fn process_request(user_id: &str) {
    info!("Processing request for user: {}", user_id);
    
    // 嵌套 span
    fetch_data(user_id).await;
}

#[instrument]
async fn fetch_data(user_id: &str) {
    info!("Fetching data");
    // 您的代码
}
```

### Jaeger UI

访问 Jaeger UI (http://localhost:16686)：

1. 选择服务: `agentmem`
2. 选择操作或留空查看所有
3. 点击 "Find Traces"
4. 点击任意 trace 查看详细信息

**Trace 视图**:
- Timeline: 时间线视图
- Graph: 依赖关系图
- JSON: 原始数据

## 仪表板

### AgentMem 概览仪表板

**面板**:
1. 请求率 (QPS)
2. 错误率
3. 请求延迟 (P50/P95/P99)
4. 活跃连接数
5. 内存使用
6. CPU 使用率

### 性能监控仪表板

**面板**:
1. 工具执行延迟
2. 向量搜索性能
3. 图查询性能
4. 批量操作吞吐量
5. 缓存命中率
6. 数据库查询延迟

### 错误分析仪表板

**面板**:
1. 错误率趋势
2. 错误类型分布
3. Top 错误端点
4. 错误日志流
5. 错误追踪关联

## 故障排查

### 问题: Prometheus 无法抓取指标

**检查**:
1. AgentMem 是否在运行？
2. 指标端口是否正确？(默认 9091)
3. 防火墙是否阻止？

**解决**:
```bash
# 检查指标端点
curl http://localhost:9091/metrics

# 检查 Prometheus 配置
docker exec agentmem-prometheus cat /etc/prometheus/prometheus.yml

# 查看 Prometheus 日志
docker logs agentmem-prometheus
```

### 问题: Jaeger 没有追踪数据

**检查**:
1. OTLP 端点配置是否正确？
2. 追踪是否已启用？

**解决**:
```rust
// 确保配置了 OTLP 端点
let config = ObservabilityConfig {
    otlp_endpoint: Some("http://localhost:4317".to_string()),
    ..Default::default()
};
```

### 问题: Elasticsearch 无法接收日志

**检查**:
1. Logstash 是否在运行？
2. Elasticsearch 是否健康？

**解决**:
```bash
# 检查 Elasticsearch 健康状态
curl http://localhost:9200/_cluster/health

# 检查 Logstash 日志
docker logs agentmem-logstash

# 测试 Logstash 输入
echo '{"test":"message"}' | nc localhost 5000
```

### 问题: Grafana 无法连接数据源

**检查**:
1. 数据源配置是否正确？
2. 网络是否可达？

**解决**:
```bash
# 从 Grafana 容器测试连接
docker exec agentmem-grafana curl http://prometheus:9090/-/healthy
docker exec agentmem-grafana curl http://jaeger:16686
docker exec agentmem-grafana curl http://elasticsearch:9200
```

## 生产环境建议

### 1. 数据保留策略

**Prometheus**:
```yaml
# prometheus.yml
global:
  storage.tsdb.retention.time: 30d
  storage.tsdb.retention.size: 50GB
```

**Elasticsearch**:
```bash
# 设置索引生命周期管理 (ILM)
# 30 天后删除旧日志
```

### 2. 采样策略

**Jaeger**:
```yaml
# 生产环境使用概率采样
sampling:
  type: probabilistic
  param: 0.1  # 10% 采样率
```

### 3. 资源限制

**Docker Compose**:
```yaml
services:
  prometheus:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
```

### 4. 高可用性

- Prometheus: 使用 Thanos 或 Cortex 实现高可用
- Elasticsearch: 使用集群模式
- Grafana: 使用外部数据库存储配置

### 5. 安全性

- 启用 HTTPS
- 配置认证和授权
- 限制网络访问
- 定期更新组件

## 参考资料

- [Prometheus 文档](https://prometheus.io/docs/)
- [Grafana 文档](https://grafana.com/docs/)
- [Jaeger 文档](https://www.jaegertracing.io/docs/)
- [Elasticsearch 文档](https://www.elastic.co/guide/en/elasticsearch/reference/current/index.html)
- [OpenTelemetry 文档](https://opentelemetry.io/docs/)

