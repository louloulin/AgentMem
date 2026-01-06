# AgentMem 生产部署指南

**版本**: v1.0  
**更新日期**: 2025-10-27  
**状态**: ✅ 生产就绪  

---

## 📋 目录

1. [快速开始](#快速开始)
2. [Docker部署](#docker部署)
3. [Kubernetes部署](#kubernetes部署)
4. [云服务部署](#云服务部署)
5. [配置说明](#配置说明)
6. [监控和日志](#监控和日志)
7. [故障排除](#故障排除)

---

## 🚀 快速开始

### 系统要求

| 组件 | 最低要求 | 推荐配置 |
|------|---------|---------|
| **CPU** | 2核 | 4核+ |
| **内存** | 4GB | 8GB+ |
| **存储** | 20GB | 50GB+ |
| **操作系统** | Linux (Ubuntu 20.04+) | Ubuntu 22.04 LTS |

### 前置依赖

```bash
# 必需
- Docker 20.10+
- Docker Compose 2.0+

# 可选（K8s部署）
- Kubernetes 1.24+
- Helm 3.0+
- kubectl
```

---

## 🐳 Docker部署

### 方式1: Docker Compose（推荐）

#### 1.1 创建docker-compose.yml

```yaml
version: '3.8'

services:
  agentmem-server:
    image: agentmem/server:latest
    container_name: agentmem-server
    ports:
      - "8080:8080"
    environment:
      # 数据库配置
      - DATABASE_URL=libsql://local/agentmem.db
      - LIBSQL_PATH=/data/agentmem.db
      
      # LLM配置
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      
      # 服务配置
      - RUST_LOG=info
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      
      # 安全配置
      - JWT_SECRET=${JWT_SECRET:-your-secret-key-change-in-production}
      - ENABLE_AUTH=true
    volumes:
      - agentmem-data:/data
      - agentmem-logs:/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped
    networks:
      - agentmem-network

  # 可选: PostgreSQL（用于高级功能）
  postgres:
    image: postgres:16-alpine
    container_name: agentmem-postgres
    environment:
      - POSTGRES_DB=agentmem
      - POSTGRES_USER=agentmem
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-changeme}
    volumes:
      - postgres-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U agentmem"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped
    networks:
      - agentmem-network

  # 可选: Redis（用于缓存）
  redis:
    image: redis:7-alpine
    container_name: agentmem-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped
    networks:
      - agentmem-network

  # 可选: Prometheus（监控）
  prometheus:
    image: prom/prometheus:latest
    container_name: agentmem-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped
    networks:
      - agentmem-network

  # 可选: Grafana（可视化）
  grafana:
    image: grafana/grafana:latest
    container_name: agentmem-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD:-admin}
    volumes:
      - grafana-data:/var/lib/grafana
    depends_on:
      - prometheus
    restart: unless-stopped
    networks:
      - agentmem-network

volumes:
  agentmem-data:
  agentmem-logs:
  postgres-data:
  redis-data:
  prometheus-data:
  grafana-data:

networks:
  agentmem-network:
    driver: bridge
```

#### 1.2 创建环境变量文件

```bash
# .env
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx
JWT_SECRET=your-super-secret-jwt-key-min-32-chars
POSTGRES_PASSWORD=secure-password-here
GRAFANA_PASSWORD=admin-password-here
```

#### 1.3 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f agentmem-server

# 检查状态
docker-compose ps

# 停止服务
docker-compose down

# 停止并删除数据
docker-compose down -v
```

#### 1.4 验证部署

```bash
# 健康检查
curl http://localhost:8080/health

# API测试
curl http://localhost:8080/api/v1/health

# 查看Swagger文档
open http://localhost:8080/swagger-ui
```

### 方式2: 单容器部署

```bash
# 拉取镜像
docker pull agentmem/server:latest

# 运行容器
docker run -d \
  --name agentmem-server \
  -p 8080:8080 \
  -e OPENAI_API_KEY=sk-xxx \
  -e DATABASE_URL=libsql://local/agentmem.db \
  -e LIBSQL_PATH=/data/agentmem.db \
  -v agentmem-data:/data \
  -v agentmem-logs:/logs \
  --restart unless-stopped \
  agentmem/server:latest

# 查看日志
docker logs -f agentmem-server

# 停止容器
docker stop agentmem-server
docker rm agentmem-server
```

---

## ☸️ Kubernetes部署

### 方式1: Helm Charts（推荐）

#### 2.1 添加Helm仓库

```bash
# 添加AgentMem Helm仓库
helm repo add agentmem https://charts.agentmem.dev
helm repo update

# 搜索可用版本
helm search repo agentmem
```

#### 2.2 创建values.yaml

```yaml
# values.yaml
replicaCount: 3

image:
  repository: agentmem/server
  tag: "latest"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080
  targetPort: 8080

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: agentmem.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: agentmem-tls
      hosts:
        - agentmem.example.com

resources:
  limits:
    cpu: 2000m
    memory: 4Gi
  requests:
    cpu: 500m
    memory: 1Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

env:
  - name: RUST_LOG
    value: "info"
  - name: OPENAI_API_KEY
    valueFrom:
      secretKeyRef:
        name: agentmem-secrets
        key: openai-api-key
  - name: DATABASE_URL
    value: "postgresql://agentmem:password@postgres:5432/agentmem"

persistence:
  enabled: true
  storageClass: "standard"
  accessMode: ReadWriteOnce
  size: 50Gi

postgresql:
  enabled: true
  auth:
    username: agentmem
    password: changeme
    database: agentmem
  primary:
    persistence:
      enabled: true
      size: 50Gi

redis:
  enabled: true
  auth:
    enabled: false
  master:
    persistence:
      enabled: true
      size: 10Gi

monitoring:
  prometheus:
    enabled: true
  grafana:
    enabled: true
    adminPassword: admin

healthCheck:
  liveness:
    enabled: true
    initialDelaySeconds: 30
    periodSeconds: 30
    timeoutSeconds: 5
    failureThreshold: 3
  readiness:
    enabled: true
    initialDelaySeconds: 10
    periodSeconds: 10
    timeoutSeconds: 3
    failureThreshold: 3
```

#### 2.3 创建Secrets

```bash
# 创建secrets
kubectl create secret generic agentmem-secrets \
  --from-literal=openai-api-key=sk-xxx \
  --from-literal=anthropic-api-key=sk-ant-xxx \
  --from-literal=jwt-secret=your-secret-key

# 或从文件创建
kubectl create secret generic agentmem-secrets \
  --from-file=.env
```

#### 2.4 部署应用

```bash
# 安装
helm install agentmem agentmem/agentmem -f values.yaml

# 升级
helm upgrade agentmem agentmem/agentmem -f values.yaml

# 查看状态
helm status agentmem

# 卸载
helm uninstall agentmem
```

### 方式2: 原生Kubernetes YAML

#### 2.5 创建部署配置

**deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agentmem-server
  labels:
    app: agentmem
spec:
  replicas: 3
  selector:
    matchLabels:
      app: agentmem
  template:
    metadata:
      labels:
        app: agentmem
    spec:
      containers:
      - name: server
        image: agentmem/server:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: agentmem-secrets
              key: openai-api-key
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        volumeMounts:
        - name: data
          mountPath: /data
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: agentmem-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: agentmem-service
spec:
  selector:
    app: agentmem
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: agentmem-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 50Gi
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: agentmem-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: agentmem-server
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

#### 2.6 应用配置

```bash
# 应用所有配置
kubectl apply -f deployment.yaml

# 查看部署状态
kubectl get deployments
kubectl get pods
kubectl get services

# 查看日志
kubectl logs -f deployment/agentmem-server

# 扩容
kubectl scale deployment agentmem-server --replicas=5

# 删除
kubectl delete -f deployment.yaml
```

---

## ☁️ 云服务部署

### AWS部署

#### ECS Fargate

```bash
# 1. 创建ECR仓库
aws ecr create-repository --repository-name agentmem/server

# 2. 推送镜像
aws ecr get-login-password | docker login --username AWS --password-stdin <account-id>.dkr.ecr.region.amazonaws.com
docker tag agentmem/server:latest <account-id>.dkr.ecr.region.amazonaws.com/agentmem/server:latest
docker push <account-id>.dkr.ecr.region.amazonaws.com/agentmem/server:latest

# 3. 创建任务定义
aws ecs register-task-definition --cli-input-json file://task-definition.json

# 4. 创建服务
aws ecs create-service \
  --cluster agentmem-cluster \
  --service-name agentmem-service \
  --task-definition agentmem-task \
  --desired-count 3 \
  --launch-type FARGATE
```

**task-definition.json**:
```json
{
  "family": "agentmem-task",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "containerDefinitions": [
    {
      "name": "agentmem-server",
      "image": "<account-id>.dkr.ecr.region.amazonaws.com/agentmem/server:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "OPENAI_API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account-id:secret:agentmem-secrets"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/agentmem",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

### Azure部署

#### Azure Container Instances

```bash
# 创建资源组
az group create --name agentmem-rg --location eastus

# 创建容器实例
az container create \
  --resource-group agentmem-rg \
  --name agentmem-server \
  --image agentmem/server:latest \
  --cpu 2 \
  --memory 4 \
  --ports 8080 \
  --environment-variables \
    RUST_LOG=info \
  --secure-environment-variables \
    OPENAI_API_KEY=sk-xxx \
  --restart-policy Always

# 查看状态
az container show --resource-group agentmem-rg --name agentmem-server

# 查看日志
az container logs --resource-group agentmem-rg --name agentmem-server
```

### GCP部署

#### Google Cloud Run

```bash
# 1. 构建并推送到GCR
gcloud builds submit --tag gcr.io/PROJECT_ID/agentmem-server

# 2. 部署到Cloud Run
gcloud run deploy agentmem-server \
  --image gcr.io/PROJECT_ID/agentmem-server \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars RUST_LOG=info \
  --set-secrets OPENAI_API_KEY=agentmem-secrets:latest \
  --memory 4Gi \
  --cpu 2 \
  --min-instances 1 \
  --max-instances 10

# 查看服务
gcloud run services describe agentmem-server --region us-central1
```

---

## ⚙️ 配置说明

### 环境变量配置

| 变量名 | 必需 | 默认值 | 说明 |
|--------|------|--------|------|
| `DATABASE_URL` | 是 | - | 数据库连接URL |
| `OPENAI_API_KEY` | 否 | - | OpenAI API密钥 |
| `ANTHROPIC_API_KEY` | 否 | - | Anthropic API密钥 |
| `JWT_SECRET` | 是 | - | JWT签名密钥 |
| `RUST_LOG` | 否 | `info` | 日志级别 |
| `SERVER_HOST` | 否 | `0.0.0.0` | 服务器监听地址 |
| `SERVER_PORT` | 否 | `8080` | 服务器端口 |
| `ENABLE_AUTH` | 否 | `false` | 启用认证 |

### 数据库配置

#### LibSQL (默认)
```env
DATABASE_URL=libsql://local/agentmem.db
LIBSQL_PATH=/data/agentmem.db
```

#### PostgreSQL
```env
DATABASE_URL=postgresql://user:password@host:5432/database
```

#### Turso (云端LibSQL)
```env
DATABASE_URL=libsql://your-database.turso.io
LIBSQL_AUTH_TOKEN=your-token
```

---

## 📊 监控和日志

### Prometheus配置

**prometheus.yml**:
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'agentmem'
    static_configs:
      - targets: ['agentmem-server:8080']
    metrics_path: '/metrics/prometheus'
```

### Grafana仪表板

访问 http://localhost:3000 并导入AgentMem仪表板:
- Dashboard ID: `agentmem-overview`
- 默认用户名/密码: `admin/admin`

### 日志管理

```bash
# Docker日志
docker logs -f agentmem-server

# Kubernetes日志
kubectl logs -f deployment/agentmem-server

# 日志级别
export RUST_LOG=debug  # trace, debug, info, warn, error
```

---

## 🔧 故障排除

### 常见问题

#### 1. 服务无法启动

```bash
# 检查日志
docker logs agentmem-server

# 常见原因:
# - 端口被占用
# - 环境变量缺失
# - 数据库连接失败
```

#### 2. 健康检查失败

```bash
# 手动测试
curl http://localhost:8080/health/ready

# 检查依赖:
# - 数据库是否可用
# - API密钥是否正确
```

#### 3. 性能问题

```bash
# 检查资源使用
docker stats agentmem-server

# 调整资源限制
docker update --memory 4g --cpus 2 agentmem-server
```

### 支持渠道

- **文档**: https://agentmem.cc
- **GitHub Issues**: https://github.com/louloulin/agentmem/issues
- **Discord**: https://discord.gg/agentmem
- **Email**: support@agentmem.dev

---

## 📝 生产就绪清单

- ✅ 使用环境变量管理敏感信息
- ✅ 配置持久化存储
- ✅ 启用健康检查
- ✅ 配置资源限制
- ✅ 启用自动重启
- ✅ 配置日志收集
- ✅ 启用监控告警
- ✅ 配置备份策略
- ✅ 使用HTTPS/TLS
- ✅ 配置负载均衡

---

**文档版本**: v1.0  
**维护团队**: AgentMem DevOps Team  
**最后更新**: 2025-10-27

