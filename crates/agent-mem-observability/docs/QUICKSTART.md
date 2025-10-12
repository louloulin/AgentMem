# AgentMem 监控快速开始指南

本指南将帮助您在 5 分钟内启动 AgentMem 的完整监控栈。

## 前置要求

- Docker 和 Docker Compose
- 至少 4GB 可用内存
- 端口 3000, 4317, 5000, 5601, 9090, 9091, 16686 可用

## 步骤 1: 启动监控栈

```bash
cd crates/agent-mem-observability
docker-compose -f docker-compose.monitoring.yml up -d
```

等待所有服务启动（约 30-60 秒）：

```bash
docker-compose -f docker-compose.monitoring.yml ps
```

## 步骤 2: 验证服务

访问以下 URL 验证服务是否正常运行：

| 服务 | URL | 凭据 |
|------|-----|------|
| Grafana | http://localhost:3000 | admin/admin |
| Prometheus | http://localhost:9090 | - |
| Jaeger | http://localhost:16686 | - |
| Kibana | http://localhost:5601 | - |
| AlertManager | http://localhost:9093 | - |

## 步骤 3: 配置 AgentMem

在您的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
agent-mem-observability = { path = "../agent-mem-observability" }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

在您的代码中初始化可观测性：

```rust
use agent_mem_observability::{init_observability, ObservabilityConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化可观测性
    let config = ObservabilityConfig {
        service_name: "agentmem".to_string(),
        otlp_endpoint: Some("http://localhost:4317".to_string()),
        enable_metrics: true,
        metrics_port: 9091,
        log_level: "info".to_string(),
        json_logging: true,
    };
    
    init_observability(config).await?;
    
    info!("AgentMem started with observability");
    
    // 您的应用代码
    
    Ok(())
}
```

## 步骤 4: 运行示例

运行 AgentMem 的可观测性示例：

```bash
cargo run --package agent-mem-observability --example basic_usage
```

## 步骤 5: 查看数据

### 查看指标 (Prometheus)

1. 访问 http://localhost:9090
2. 在查询框中输入: `agentmem_requests_total`
3. 点击 "Execute"
4. 切换到 "Graph" 标签查看图表

### 查看仪表板 (Grafana)

1. 访问 http://localhost:3000
2. 使用 admin/admin 登录
3. 进入 Dashboards > Browse
4. 选择 "AgentMem Observability Dashboard"

### 查看追踪 (Jaeger)

1. 访问 http://localhost:16686
2. 在 "Service" 下拉菜单中选择 "agentmem"
3. 点击 "Find Traces"
4. 点击任意 trace 查看详细信息

### 查看日志 (Kibana)

1. 访问 http://localhost:5601
2. 等待 Kibana 初始化（首次启动可能需要几分钟）
3. 进入 Management > Stack Management > Index Patterns
4. 创建索引模式: `agentmem-logs-*`
5. 选择时间字段: `@timestamp`
6. 进入 Discover 查看日志

## 步骤 6: 测试告警

触发一个测试告警：

```bash
# 停止 AgentMem 服务（如果正在运行）
# 等待 1 分钟，ServiceDown 告警将被触发

# 查看告警
curl http://localhost:9093/api/v2/alerts
```

## 常用命令

### 查看日志

```bash
# 查看所有服务日志
docker-compose -f docker-compose.monitoring.yml logs -f

# 查看特定服务日志
docker-compose -f docker-compose.monitoring.yml logs -f prometheus
docker-compose -f docker-compose.monitoring.yml logs -f grafana
docker-compose -f docker-compose.monitoring.yml logs -f jaeger
```

### 重启服务

```bash
# 重启所有服务
docker-compose -f docker-compose.monitoring.yml restart

# 重启特定服务
docker-compose -f docker-compose.monitoring.yml restart prometheus
```

### 停止服务

```bash
# 停止所有服务
docker-compose -f docker-compose.monitoring.yml down

# 停止并删除数据卷
docker-compose -f docker-compose.monitoring.yml down -v
```

### 更新配置

```bash
# 重新加载 Prometheus 配置（无需重启）
curl -X POST http://localhost:9090/-/reload

# 重启服务以应用配置更改
docker-compose -f docker-compose.monitoring.yml restart prometheus
```

## 下一步

- 阅读 [监控指南](MONITORING.md) 了解详细配置
- 自定义 Grafana 仪表板
- 配置告警通知（Email, Slack）
- 调整数据保留策略
- 配置生产环境的高可用性

## 故障排查

### 问题: 端口已被占用

```bash
# 查看端口占用
lsof -i :3000
lsof -i :9090

# 修改 docker-compose.monitoring.yml 中的端口映射
```

### 问题: 内存不足

```bash
# 检查 Docker 内存限制
docker stats

# 增加 Docker 内存限制（Docker Desktop）
# Settings > Resources > Memory
```

### 问题: Elasticsearch 启动失败

```bash
# 增加虚拟内存限制（Linux）
sudo sysctl -w vm.max_map_count=262144

# 永久设置
echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf
```

### 问题: 服务无法连接

```bash
# 检查 Docker 网络
docker network ls
docker network inspect agentmem-observability_monitoring

# 检查服务健康状态
docker-compose -f docker-compose.monitoring.yml ps
```

## 支持

如有问题，请查看：
- [监控指南](MONITORING.md)
- [README](../README.md)
- [GitHub Issues](https://github.com/your-org/agentmem/issues)

