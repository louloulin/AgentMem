# AgentMem 监控系统

## 📋 概述

AgentMem 监控系统基于 Prometheus + Grafana，提供完整的系统可观测性。

### 监控指标

- **API 性能**: 请求率、错误率、延迟
- **数据库**: 连接池、查询性能
- **缓存**: 命中率、淘汰率
- **内存操作**: CRUD 操作性能
- **工具执行**: 工具调用性能
- **LLM 调用**: API 调用统计
- **系统资源**: CPU、内存、磁盘

---

## 🚀 快速开始

### 1. 启动 Prometheus

```bash
# 使用 Docker
docker run -d \
  --name prometheus \
  -p 9090:9090 \
  -v $(pwd)/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml \
  -v $(pwd)/prometheus/alerts:/etc/prometheus/alerts \
  prom/prometheus:latest

# 或使用 docker-compose
docker-compose up -d prometheus
```

### 2. 启动 Grafana

```bash
# 使用 Docker
docker run -d \
  --name grafana \
  -p 3000:3000 \
  -v $(pwd)/grafana/dashboards:/etc/grafana/provisioning/dashboards \
  grafana/grafana:latest

# 或使用 docker-compose
docker-compose up -d grafana
```

### 3. 配置 AgentMem

确保 AgentMem 服务器启用了 Prometheus metrics 端点：

```bash
# 启动 AgentMem 服务器
cd ../..
cargo run --release --bin agent-mem-server
```

Metrics 端点: `http://localhost:8080/metrics/prometheus`

### 4. 访问 Grafana

1. 打开浏览器访问: `http://localhost:3000`
2. 默认登录:
   - 用户名: `admin`
   - 密码: `admin`
3. 添加 Prometheus 数据源:
   - URL: `http://prometheus:9090`
4. 导入仪表板:
   - 导入 `grafana/dashboards/agentmem-dashboard.json`

---

## 📊 仪表板说明

### AgentMem System Dashboard

主仪表板包含以下面板：

#### 1. Request Rate (请求率)
- 显示每秒请求数
- 按方法和端点分组
- 用于监控系统负载

#### 2. Error Rate (错误率)
- 显示每秒错误数
- 按错误类型分组
- 配置了告警规则（> 10 errors/sec）

#### 3. Request Duration (请求延迟)
- 显示 p50、p95、p99 延迟
- 用于监控性能
- 目标: p95 < 1s

#### 4. Active Connections (活跃连接)
- 显示当前活跃连接数
- 用于监控并发负载

#### 5. Memory Operations (记忆操作)
- 显示 CRUD 操作速率
- 按操作类型分组

#### 6. Cache Hit Rate (缓存命中率)
- 显示缓存命中率百分比
- 目标: > 80%
- 配置了告警规则（< 70%）

#### 7. Database Connection Pool (数据库连接池)
- 显示活跃连接数
- 最大连接数: 50
- 配置了告警规则（>= 45）

#### 8. Tool Execution Duration (工具执行时间)
- 显示工具执行 p95 延迟
- 按工具名称分组

#### 9. LLM API Calls (LLM API 调用)
- 显示 LLM API 调用速率
- 按提供商和模型分组

#### 10. System Resource Usage (系统资源使用)
- 显示内存和 CPU 使用情况
- 用于监控资源消耗

#### 11. Agent Activity (Agent 活动)
- 显示最活跃的 10 个 Agent
- 按请求速率排序

---

## 🚨 告警规则

### 关键告警

#### 1. HighErrorRate (高错误率)
- **条件**: 错误率 > 10 errors/sec，持续 5 分钟
- **严重性**: Critical
- **处理**: 检查日志，定位错误原因

#### 2. HighRequestLatency (高请求延迟)
- **条件**: p95 延迟 > 1s，持续 10 分钟
- **严重性**: Warning
- **处理**: 检查数据库性能、缓存命中率

#### 3. DatabaseConnectionPoolExhausted (数据库连接池耗尽)
- **条件**: 活跃连接 >= 45，持续 5 分钟
- **严重性**: Critical
- **处理**: 增加连接池大小或优化查询

#### 4. LowCacheHitRate (低缓存命中率)
- **条件**: 命中率 < 70%，持续 15 分钟
- **严重性**: Warning
- **处理**: 检查缓存配置、预热策略

#### 5. ServiceDown (服务宕机)
- **条件**: 服务不可用，持续 1 分钟
- **严重性**: Critical
- **处理**: 立即重启服务，检查日志

### 警告告警

#### 6. HighMemoryUsage (高内存使用)
- **条件**: 内存使用 > 2GB，持续 10 分钟
- **严重性**: Warning
- **处理**: 检查内存泄漏

#### 7. HighToolExecutionTime (高工具执行时间)
- **条件**: 工具执行 p95 > 5s，持续 10 分钟
- **严重性**: Warning
- **处理**: 优化工具实现

#### 8. LLMAPIFailures (LLM API 失败)
- **条件**: LLM API 错误率 > 1 error/sec，持续 5 分钟
- **严重性**: Warning
- **处理**: 检查 API 密钥、配额

---

## 📈 性能目标

### SLA 目标

| 指标 | 目标 | 说明 |
|------|------|------|
| **可用性** | 99.9% | 每月最多 43 分钟宕机 |
| **请求延迟 (p95)** | < 1s | 95% 的请求在 1 秒内完成 |
| **请求延迟 (p99)** | < 2s | 99% 的请求在 2 秒内完成 |
| **错误率** | < 0.1% | 每 1000 个请求最多 1 个错误 |
| **缓存命中率** | > 80% | 80% 的请求命中缓存 |

### 容量规划

| 资源 | 当前 | 目标 | 扩展阈值 |
|------|------|------|----------|
| **请求率** | 100 req/s | 1000 req/s | 800 req/s |
| **并发连接** | 100 | 1000 | 800 |
| **数据库连接** | 20 | 50 | 45 |
| **内存使用** | 500 MB | 2 GB | 1.5 GB |
| **CPU 使用** | 20% | 80% | 70% |

---

## 🔧 配置文件

### Prometheus 配置

文件: `prometheus/prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - '/etc/prometheus/alerts/*.yml'

scrape_configs:
  - job_name: 'agentmem'
    static_configs:
      - targets: ['agentmem:8080']
    metrics_path: '/metrics/prometheus'
```

### Grafana 数据源配置

文件: `grafana/provisioning/datasources/prometheus.yml`

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

---

## 🐳 Docker Compose

文件: `docker-compose.yml`

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/alerts:/etc/prometheus/alerts
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/datasources:/etc/grafana/provisioning/datasources
      - grafana_data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager/config.yml:/etc/alertmanager/config.yml
      - alertmanager_data:/alertmanager

volumes:
  prometheus_data:
  grafana_data:
  alertmanager_data:
```

---

## 📝 运维手册

### 日常检查

1. **每日检查**
   - 查看 Grafana 仪表板
   - 检查告警状态
   - 查看错误日志

2. **每周检查**
   - 审查性能趋势
   - 检查资源使用
   - 更新告警规则

3. **每月检查**
   - 审查 SLA 达成情况
   - 容量规划
   - 优化建议

### 故障排查

#### 高错误率

1. 查看 Grafana Error Rate 面板
2. 检查 AgentMem 日志: `docker logs agentmem`
3. 查看具体错误类型
4. 定位问题代码
5. 修复并重启服务

#### 高延迟

1. 查看 Request Duration 面板
2. 检查数据库性能
3. 检查缓存命中率
4. 查看慢查询日志
5. 优化查询或增加缓存

#### 服务宕机

1. 检查服务状态: `docker ps`
2. 查看日志: `docker logs agentmem`
3. 重启服务: `docker restart agentmem`
4. 如果持续失败，回滚到上一个版本

---

## 🔗 相关链接

- [Prometheus 文档](https://prometheus.io/docs/)
- [Grafana 文档](https://grafana.com/docs/)
- [AgentMem API 文档](../../docs/api.md)
- [AgentMem 部署指南](../../docs/deployment.md)

---

**最后更新**: 2025-10-07  
**维护者**: AgentMem Team

