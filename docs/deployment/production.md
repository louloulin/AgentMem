# AgentMem 生产环境部署指南

本指南提供 AgentMem 在生产环境中部署的完整说明，包括系统要求、部署架构、安装步骤、配置说明、监控设置、备份策略和故障排除。

**版本**: 2.1.0  
**更新日期**: 2025-10-03  
**目标读者**: 运维工程师、DevOps 工程师、系统管理员

---

## 📋 目录

1. [系统要求](#1-系统要求)
2. [部署架构](#2-部署架构)
3. [安装步骤](#3-安装步骤)
4. [配置说明](#4-配置说明)
5. [监控设置](#5-监控设置)
6. [备份策略](#6-备份策略)
7. [故障排除](#7-故障排除)
8. [运维检查清单](#8-运维检查清单)

---

## 1. 系统要求

### 1.1 硬件要求

#### 最小配置（开发/测试环境）
- **CPU**: 4 核心
- **内存**: 8 GB RAM
- **磁盘**: 50 GB SSD
- **网络**: 100 Mbps

#### 推荐配置（生产环境 - 中等负载）
- **CPU**: 8 核心（Intel Xeon 或 AMD EPYC）
- **内存**: 16 GB RAM
- **磁盘**: 200 GB NVMe SSD
- **网络**: 1 Gbps

#### 高性能配置（生产环境 - 高负载）
- **CPU**: 16+ 核心
- **内存**: 32+ GB RAM
- **磁盘**: 500 GB+ NVMe SSD（RAID 10）
- **网络**: 10 Gbps

### 1.2 软件要求

#### 操作系统
- **Linux**: Ubuntu 20.04/22.04 LTS, CentOS 8+, RHEL 8+, Debian 11+
- **容器**: Docker 20.10+, Docker Compose 2.0+
- **编排**: Kubernetes 1.24+ (可选)

#### 依赖软件
- **PostgreSQL**: 14+ (推荐 15)
- **Redis**: 7.0+
- **Qdrant**: 1.7.0+
- **Neo4j**: 5.15+ (可选，用于图记忆)

#### 监控工具
- **Prometheus**: 2.45+
- **Grafana**: 10.0+
- **Jaeger**: 1.50+ (可选，用于分布式追踪)

### 1.3 网络要求

#### 端口列表

| 服务 | 端口 | 协议 | 说明 |
|------|------|------|------|
| AgentMem API | 8080 | HTTP/HTTPS | 主要 API 端点 |
| Web UI | 3000 | HTTP/HTTPS | 管理界面 |
| PostgreSQL | 5432 | TCP | 数据库 |
| Redis | 6379 | TCP | 缓存 |
| Qdrant | 6333, 6334 | HTTP/gRPC | 向量数据库 |
| Neo4j | 7474, 7687 | HTTP/Bolt | 图数据库 |
| Prometheus | 9090 | HTTP | 监控指标 |
| Grafana | 3000 | HTTP | 监控仪表板 |
| Jaeger | 16686 | HTTP | 追踪 UI |

#### 防火墙规则

```bash
# 允许 API 访问
sudo ufw allow 8080/tcp

# 允许 Web UI 访问
sudo ufw allow 3000/tcp

# 允许 HTTPS（如果使用）
sudo ufw allow 443/tcp

# 允许内部服务通信（仅限内网）
sudo ufw allow from 10.0.0.0/8 to any port 5432
sudo ufw allow from 10.0.0.0/8 to any port 6379
sudo ufw allow from 10.0.0.0/8 to any port 6333
```

---

## 2. 部署架构

### 2.1 单机部署架构

适用于：开发、测试、小规模生产环境（< 1000 用户）

```
┌─────────────────────────────────────────────────────────┐
│                    单机服务器                              │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  AgentMem    │  │   Web UI     │  │   Nginx      │  │
│  │   Server     │  │  (Next.js)   │  │  (Reverse    │  │
│  │   (Rust)     │  │              │  │   Proxy)     │  │
│  └──────┬───────┘  └──────────────┘  └──────┬───────┘  │
│         │                                     │          │
│  ┌──────┴──────────────────────────────────┴───────┐  │
│  │              数据层                               │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │  │
│  │  │PostgreSQL│ │  Redis   │ │  Qdrant  │        │  │
│  │  └──────────┘ └──────────┘ └──────────┘        │  │
│  └──────────────────────────────────────────────────┘  │
│                                                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │              监控层                                 │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │  │
│  │  │Prometheus│ │  Grafana │ │  Jaeger  │        │  │
│  │  └──────────┘ └──────────┘ └──────────┘        │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**优点**:
- 部署简单，维护成本低
- 适合快速启动和测试
- 资源利用率高

**缺点**:
- 单点故障风险
- 扩展性有限
- 性能受限于单机

### 2.2 集群部署架构

适用于：中大规模生产环境（1000-10000 用户）

```
                    ┌──────────────┐
                    │ Load Balancer│
                    │   (Nginx)    │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐       ┌────▼────┐       ┌────▼────┐
   │AgentMem │       │AgentMem │       │AgentMem │
   │ Node 1  │       │ Node 2  │       │ Node 3  │
   └────┬────┘       └────┬────┘       └────┬────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐       ┌────▼────┐       ┌────▼────┐
   │PostgreSQL       │  Redis   │       │ Qdrant  │
   │ Primary │       │ Cluster  │       │ Cluster │
   │    +    │       │          │       │         │
   │ Replicas│       │          │       │         │
   └─────────┘       └──────────┘       └─────────┘
```

**优点**:
- 高可用性（无单点故障）
- 水平扩展能力
- 负载均衡

**缺点**:
- 部署复杂度增加
- 需要更多资源
- 运维成本较高

### 2.3 Kubernetes 高可用架构

适用于：大规模生产环境（10000+ 用户）

```
┌─────────────────────────────────────────────────────────┐
│                  Kubernetes Cluster                      │
│                                                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Ingress Controller                    │  │
│  │         (Nginx/Traefik/Istio)                     │  │
│  └────────────────────┬─────────────────────────────┘  │
│                       │                                  │
│  ┌────────────────────┴─────────────────────────────┐  │
│  │          AgentMem Deployment (HPA)               │  │
│  │  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐        │  │
│  │  │ Pod1 │  │ Pod2 │  │ Pod3 │  │ PodN │        │  │
│  │  └──────┘  └──────┘  └──────┘  └──────┘        │  │
│  └──────────────────────────────────────────────────┘  │
│                                                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │          StatefulSet (Databases)                  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │  │
│  │  │PostgreSQL│ │  Redis   │ │  Qdrant  │        │  │
│  │  │ Operator │ │ Operator │ │ Operator │        │  │
│  │  └──────────┘ └──────────┘ └──────────┘        │  │
│  └──────────────────────────────────────────────────┘  │
│                                                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │          Persistent Volumes (PV/PVC)              │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │  │
│  │  │   DB     │ │  Cache   │ │  Vector  │        │  │
│  │  │  Data    │ │   Data   │ │   Data   │        │  │
│  │  └──────────┘ └──────────┘ └──────────┘        │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**优点**:
- 自动扩缩容（HPA）
- 自愈能力
- 滚动更新
- 资源隔离

**缺点**:
- 学习曲线陡峭
- 运维复杂度最高
- 需要 K8s 专业知识

---

## 3. 安装步骤

### 3.1 Docker Compose 部署（推荐）

#### 步骤 1: 准备环境

```bash
# 安装 Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 安装 Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# 验证安装
docker --version
docker-compose --version
```

#### 步骤 2: 克隆仓库

```bash
git clone https://github.com/louloulin/agentmem.git
cd agentmem
```

#### 步骤 3: 配置环境变量

```bash
# 复制示例配置
cp .env.example .env

# 编辑配置文件
nano .env
```

**关键配置项**:
```bash
# 数据库配置
POSTGRES_PASSWORD=your_secure_password_here
REDIS_PASSWORD=your_redis_password_here

# LLM 配置
AGENTMEM_LLM_PROVIDER=openai
OPENAI_API_KEY=your_openai_api_key

# 安全配置
AGENTMEM_ENABLE_AUTH=true
AGENTMEM_JWT_SECRET=your_jwt_secret_here

# 监控配置
AGENTMEM_ENABLE_METRICS=true
AGENTMEM_ENABLE_TRACING=true
```

#### 步骤 4: 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f agentmem
```

#### 步骤 5: 验证部署

```bash
# 检查健康状态
curl http://localhost:8080/health

# 预期输出
# {"status":"healthy","version":"2.1.0","uptime_seconds":10}

# 访问 Web UI
open http://localhost:3000

# 访问 Grafana
open http://localhost:3000
# 默认用户名/密码: admin/admin
```

### 3.2 Kubernetes 部署

#### 步骤 1: 准备 Kubernetes 集群

```bash
# 确保 kubectl 已安装并配置
kubectl version --client
kubectl cluster-info
```

#### 步骤 2: 创建命名空间

```bash
kubectl create namespace agentmem
kubectl config set-context --current --namespace=agentmem
```

#### 步骤 3: 创建 Secrets

```bash
# 创建数据库密码
kubectl create secret generic agentmem-db-secret \
  --from-literal=postgres-password='your_secure_password'

# 创建 LLM API 密钥
kubectl create secret generic agentmem-llm-secret \
  --from-literal=openai-api-key='your_openai_api_key'
```

#### 步骤 4: 部署应用

```bash
# 使用 Helm（推荐）
helm repo add agentmem https://charts.agentmem.io
helm install agentmem agentmem/agentmem \
  --namespace agentmem \
  --values values.yaml

# 或使用 kubectl
kubectl apply -f k8s/
```

#### 步骤 5: 验证部署

```bash
# 检查 Pod 状态
kubectl get pods

# 检查服务
kubectl get svc

# 查看日志
kubectl logs -f deployment/agentmem

# 端口转发测试
kubectl port-forward svc/agentmem 8080:8080
curl http://localhost:8080/health
```

### 3.3 裸机部署

#### 步骤 1: 安装依赖

```bash
# 安装 PostgreSQL
sudo apt-get install postgresql-15

# 安装 Redis
sudo apt-get install redis-server

# 安装 Rust（用于编译）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 步骤 2: 编译 AgentMem

```bash
git clone https://github.com/louloulin/agentmem.git
cd agentmem

# 编译 release 版本
cargo build --release

# 二进制文件位于
ls -lh target/release/agentmem-server
```

#### 步骤 3: 配置系统服务

```bash
# 创建 systemd 服务文件
sudo nano /etc/systemd/system/agentmem.service
```

**服务配置**:
```ini
[Unit]
Description=AgentMem Server
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=agentmem
WorkingDirectory=/opt/agentmem
Environment="RUST_LOG=info"
Environment="AGENTMEM_DATABASE_URL=postgresql://agentmem:password@localhost/agentmem"
ExecStart=/opt/agentmem/bin/agentmem-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

#### 步骤 4: 启动服务

```bash
# 重新加载 systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start agentmem

# 设置开机自启
sudo systemctl enable agentmem

# 查看状态
sudo systemctl status agentmem
```

---

## 4. 配置说明

### 4.1 环境变量配置

完整的环境变量列表和说明：

#### 基础配置

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `AGENTMEM_PORT` | 8080 | API 服务端口 |
| `AGENTMEM_HOST` | 0.0.0.0 | 监听地址 |
| `RUST_LOG` | info | 日志级别 (trace/debug/info/warn/error) |

#### 数据库配置

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `AGENTMEM_DATABASE_URL` | - | PostgreSQL 连接字符串 |
| `AGENTMEM_DB_POOL_SIZE` | 10 | 数据库连接池大小 |
| `AGENTMEM_REDIS_URL` | - | Redis 连接字符串 |

#### LLM 配置

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `AGENTMEM_LLM_PROVIDER` | openai | LLM 提供商 |
| `OPENAI_API_KEY` | - | OpenAI API 密钥 |
| `ANTHROPIC_API_KEY` | - | Anthropic API 密钥 |

#### 安全配置

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `AGENTMEM_ENABLE_AUTH` | false | 启用认证 |
| `AGENTMEM_JWT_SECRET` | - | JWT 密钥 |
| `AGENTMEM_ENABLE_CORS` | true | 启用 CORS |

#### 监控配置

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `AGENTMEM_ENABLE_METRICS` | true | 启用 Prometheus 指标 |
| `AGENTMEM_ENABLE_TRACING` | false | 启用分布式追踪 |
| `AGENTMEM_JAEGER_ENDPOINT` | - | Jaeger 端点 |

### 4.2 配置文件说明

AgentMem 支持 YAML 配置文件（可选）：

**config/agentmem.yaml**:
```yaml
server:
  port: 8080
  host: "0.0.0.0"
  workers: 4

database:
  url: "postgresql://agentmem:password@localhost/agentmem"
  pool_size: 10
  timeout: 30

redis:
  url: "redis://localhost:6379"
  pool_size: 5

llm:
  provider: "openai"
  model: "gpt-4"
  temperature: 0.7
  max_tokens: 2000

security:
  enable_auth: true
  jwt_secret: "your-secret-key"
  cors_origins:
    - "http://localhost:3000"
    - "https://yourdomain.com"

monitoring:
  enable_metrics: true
  enable_tracing: true
  jaeger_endpoint: "http://jaeger:14268/api/traces"

logging:
  level: "info"
  format: "json"
  output: "stdout"
```

### 4.3 安全配置最佳实践

1. **使用强密码**
   ```bash
   # 生成随机密码
   openssl rand -base64 32
   ```

2. **启用 HTTPS**
   ```bash
   # 使用 Let's Encrypt
   sudo certbot --nginx -d yourdomain.com
   ```

3. **配置防火墙**
   ```bash
   # 只允许必要的端口
   sudo ufw default deny incoming
   sudo ufw default allow outgoing
   sudo ufw allow 22/tcp
   sudo ufw allow 80/tcp
   sudo ufw allow 443/tcp
   sudo ufw enable
   ```

4. **定期更新**
   ```bash
   # 更新系统
   sudo apt-get update && sudo apt-get upgrade

   # 更新 Docker 镜像
   docker-compose pull
   docker-compose up -d
   ```

---

## 5. 监控设置

### 5.1 Prometheus 配置

AgentMem 内置 Prometheus 指标导出功能。

#### 可用指标

| 指标名称 | 类型 | 说明 |
|----------|------|------|
| `agentmem_requests_total` | Counter | 总请求数 |
| `agentmem_errors_total` | Counter | 总错误数 |
| `agentmem_request_duration_seconds` | Histogram | 请求延迟 |
| `agentmem_memory_usage_bytes` | Gauge | 内存使用量 |
| `agentmem_active_connections` | Gauge | 活跃连接数 |
| `agentmem_db_connections` | Gauge | 数据库连接数 |

#### Prometheus 配置文件

**prometheus.yml**:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'agentmem'
    static_configs:
      - targets: ['agentmem:8080']
    metrics_path: '/metrics'

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

### 5.2 Grafana 仪表板

#### 导入预配置仪表板

1. 访问 Grafana: http://localhost:3000
2. 登录（admin/admin）
3. 导航到 **Dashboards** → **Import**
4. 上传 `grafana/dashboards/agentmem-overview.json`

#### 关键面板

1. **系统概览**
   - 请求速率（QPS）
   - 错误率
   - 响应时间（P50, P95, P99）
   - 活跃连接数

2. **资源使用**
   - CPU 使用率
   - 内存使用率
   - 磁盘 I/O
   - 网络流量

3. **数据库性能**
   - 查询延迟
   - 连接池使用率
   - 慢查询数量
   - 死锁数量

4. **业务指标**
   - 记忆创建速率
   - 搜索请求数
   - Agent 活跃数
   - 用户活跃数

### 5.3 告警规则

#### Prometheus 告警规则

**alert_rules.yml**:
```yaml
groups:
  - name: agentmem_alerts
    interval: 30s
    rules:
      # 高错误率告警
      - alert: HighErrorRate
        expr: rate(agentmem_errors_total[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"

      # 慢响应告警
      - alert: SlowResponse
        expr: histogram_quantile(0.95, rate(agentmem_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow response time detected"
          description: "P95 latency is {{ $value }}s"

      # 高内存使用告警
      - alert: HighMemoryUsage
        expr: agentmem_memory_usage_bytes / 1024 / 1024 / 1024 > 8
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}GB"

      # 服务不可用告警
      - alert: ServiceDown
        expr: up{job="agentmem"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "AgentMem service is down"
          description: "Service has been down for more than 1 minute"
```

#### 配置告警通知

**alertmanager.yml**:
```yaml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'cluster']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'

receivers:
  - name: 'default'
    email_configs:
      - to: 'ops@yourdomain.com'
        from: 'alertmanager@yourdomain.com'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'your-email@gmail.com'
        auth_password: 'your-app-password'

    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#alerts'
        title: 'AgentMem Alert'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

    webhook_configs:
      - url: 'http://your-webhook-endpoint/alert'
```

---

## 6. 备份策略

### 6.1 自动备份配置

#### 备份内容

1. **数据库备份**
   - PostgreSQL 数据库
   - Redis 持久化数据

2. **配置备份**
   - 环境变量文件
   - 配置文件
   - SSL 证书

3. **向量数据备份**
   - Qdrant 向量数据
   - 索引文件

#### 备份脚本

使用提供的备份脚本（详见 `scripts/backup.sh`）：

```bash
# 手动备份
./scripts/backup.sh

# 设置定时备份（每天凌晨 2 点）
crontab -e
# 添加以下行
0 2 * * * /opt/agentmem/scripts/backup.sh >> /var/log/agentmem/backup.log 2>&1
```

#### 备份保留策略

- **每日备份**: 保留 7 天
- **每周备份**: 保留 4 周
- **每月备份**: 保留 12 个月

### 6.2 备份验证

定期验证备份完整性：

```bash
# 验证备份
./scripts/verify-backup.sh /backups/agentmem/backup-2025-10-03.tar.gz

# 预期输出
# ✓ Backup file exists
# ✓ Backup file is not corrupted
# ✓ Database dump is valid
# ✓ Config files are present
# Backup verification passed!
```

### 6.3 恢复流程

#### 完整恢复

```bash
# 1. 停止服务
docker-compose down

# 2. 恢复数据
./scripts/restore.sh /backups/agentmem/backup-2025-10-03.tar.gz

# 3. 启动服务
docker-compose up -d

# 4. 验证恢复
curl http://localhost:8080/health
```

#### 部分恢复（仅数据库）

```bash
# 恢复 PostgreSQL
docker exec -i agentmem-postgres psql -U agentmem -d agentmem < backup.sql

# 恢复 Redis
docker exec -i agentmem-redis redis-cli --pipe < backup.rdb
```

---

## 7. 故障排除

### 7.1 常见问题

#### 问题 1: 服务无法启动

**症状**: `docker-compose up` 失败

**可能原因**:
1. 端口被占用
2. 内存不足
3. 配置错误

**解决方案**:
```bash
# 检查端口占用
lsof -i :8080
lsof -i :5432

# 检查内存
free -h

# 查看详细日志
docker-compose logs agentmem

# 检查配置
docker-compose config
```

#### 问题 2: 数据库连接失败

**症状**: `Failed to connect to database`

**解决方案**:
```bash
# 检查 PostgreSQL 状态
docker-compose ps postgres

# 测试连接
docker exec -it agentmem-postgres psql -U agentmem -d agentmem

# 检查密码
echo $POSTGRES_PASSWORD

# 重启数据库
docker-compose restart postgres
```

#### 问题 3: 高内存使用

**症状**: 内存使用超过 80%

**解决方案**:
```bash
# 检查内存使用
docker stats

# 调整 PostgreSQL 配置
# 编辑 postgresql.conf
shared_buffers = 256MB
effective_cache_size = 1GB

# 调整 Redis 配置
maxmemory 2gb
maxmemory-policy allkeys-lru

# 重启服务
docker-compose restart
```

#### 问题 4: 慢查询

**症状**: API 响应时间 > 1 秒

**解决方案**:
```bash
# 启用慢查询日志
# PostgreSQL
ALTER DATABASE agentmem SET log_min_duration_statement = 1000;

# 查看慢查询
docker exec agentmem-postgres psql -U agentmem -d agentmem -c "
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;"

# 添加索引
CREATE INDEX idx_memories_agent_id ON memories(agent_id);
CREATE INDEX idx_memories_created_at ON memories(created_at);
```

### 7.2 日志分析

#### 查看日志

```bash
# 实时日志
docker-compose logs -f agentmem

# 最近 100 行
docker-compose logs --tail=100 agentmem

# 特定时间范围
docker-compose logs --since="2025-10-03T10:00:00" agentmem

# 搜索错误
docker-compose logs agentmem | grep ERROR
```

#### 日志级别

调整日志级别以获取更多信息：

```bash
# 设置为 debug
export RUST_LOG=debug
docker-compose restart agentmem

# 设置为 trace（最详细）
export RUST_LOG=trace
docker-compose restart agentmem
```

### 7.3 性能调优

#### 数据库优化

```sql
-- 分析表
ANALYZE memories;

-- 重建索引
REINDEX TABLE memories;

-- 清理死元组
VACUUM FULL memories;

-- 更新统计信息
ANALYZE VERBOSE;
```

#### Redis 优化

```bash
# 检查内存碎片
redis-cli info memory | grep fragmentation

# 清理过期键
redis-cli --scan --pattern "expired:*" | xargs redis-cli del

# 优化配置
redis-cli CONFIG SET maxmemory-policy allkeys-lru
redis-cli CONFIG SET maxmemory 2gb
```

#### 应用优化

```bash
# 增加工作线程
export AGENTMEM_WORKERS=8

# 增加数据库连接池
export AGENTMEM_DB_POOL_SIZE=20

# 启用缓存
export AGENTMEM_ENABLE_CACHE=true
export AGENTMEM_CACHE_TTL=3600

# 重启服务
docker-compose restart agentmem
```

---

## 8. 运维检查清单

### 8.1 日常检查（每天）

- [ ] 检查服务状态
  ```bash
  docker-compose ps
  curl http://localhost:8080/health
  ```

- [ ] 检查错误日志
  ```bash
  docker-compose logs --since="24h" agentmem | grep ERROR
  ```

- [ ] 检查磁盘空间
  ```bash
  df -h
  ```

- [ ] 检查备份状态
  ```bash
  ls -lh /backups/agentmem/ | tail -5
  ```

- [ ] 查看监控仪表板
  - 访问 Grafana
  - 检查关键指标
  - 确认无告警

### 8.2 周期性维护（每周）

- [ ] 数据库维护
  ```bash
  docker exec agentmem-postgres psql -U agentmem -d agentmem -c "VACUUM ANALYZE;"
  ```

- [ ] 清理旧日志
  ```bash
  find /var/log/agentmem -name "*.log" -mtime +7 -delete
  ```

- [ ] 检查证书有效期
  ```bash
  openssl x509 -in /etc/ssl/certs/agentmem.crt -noout -dates
  ```

- [ ] 更新依赖
  ```bash
  docker-compose pull
  ```

- [ ] 测试备份恢复
  ```bash
  ./scripts/verify-backup.sh /backups/agentmem/latest.tar.gz
  ```

### 8.3 月度检查（每月）

- [ ] 安全审计
  - 检查访问日志
  - 审查用户权限
  - 更新密码

- [ ] 性能审查
  - 分析慢查询
  - 检查资源使用趋势
  - 优化配置

- [ ] 容量规划
  - 评估存储增长
  - 预测资源需求
  - 规划扩容

- [ ] 灾难恢复演练
  - 模拟故障场景
  - 测试恢复流程
  - 更新文档

### 8.4 应急响应流程

#### 服务中断

1. **确认问题**
   ```bash
   curl http://localhost:8080/health
   docker-compose ps
   ```

2. **查看日志**
   ```bash
   docker-compose logs --tail=100 agentmem
   ```

3. **尝试重启**
   ```bash
   docker-compose restart agentmem
   ```

4. **如果失败，回滚**
   ```bash
   docker-compose down
   git checkout previous-stable-version
   docker-compose up -d
   ```

5. **通知团队**
   - 发送告警通知
   - 更新状态页面
   - 记录事件

#### 数据丢失

1. **停止服务**
   ```bash
   docker-compose down
   ```

2. **评估损失**
   ```bash
   # 检查数据库
   docker exec agentmem-postgres psql -U agentmem -d agentmem -c "SELECT COUNT(*) FROM memories;"
   ```

3. **从备份恢复**
   ```bash
   ./scripts/restore.sh /backups/agentmem/latest.tar.gz
   ```

4. **验证恢复**
   ```bash
   docker-compose up -d
   curl http://localhost:8080/health
   ```

5. **事后分析**
   - 记录事件详情
   - 分析根本原因
   - 制定预防措施

---

## 附录

### A. 环境变量完整列表

详见 `.env.example` 文件。

### B. API 端点列表

详见 [API 参考文档](./api-reference.md)。

### C. 性能基准测试

详见 [性能测试报告](./performance-benchmarks.md)。

### D. 安全最佳实践

详见 [安全指南](./security-guide.md)。

### E. 故障排除决策树

```
服务无法访问？
├─ 是 → 检查服务状态
│   ├─ 服务未运行 → 启动服务
│   └─ 服务运行中 → 检查网络
│       ├─ 端口未开放 → 配置防火墙
│       └─ 端口开放 → 检查日志
└─ 否 → 响应慢？
    ├─ 是 → 检查资源使用
    │   ├─ CPU 高 → 增加实例
    │   ├─ 内存高 → 优化配置
    │   └─ 磁盘 I/O 高 → 使用 SSD
    └─ 否 → 功能异常？
        ├─ 是 → 检查日志
        └─ 否 → 正常运行
```

---

## 获取帮助

- 📖 [完整文档](./README.md)
- 🐛 [报告问题](https://github.com/louloulin/agentmem/issues)
- 💬 [社区讨论](https://github.com/louloulin/agentmem/discussions)
- 📧 Email: support@agentmem.io
- 🌐 Website: https://agentmem.io

---

**文档版本**: 2.1.0
**最后更新**: 2025-10-03
**维护者**: AgentMem Team

---

**祝您部署顺利！** 🚀

