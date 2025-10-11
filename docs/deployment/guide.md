# AgentMem 部署指南

**版本:** 2.0  
**更新时间:** 2025年9月4日  
**适用环境:** 生产环境、测试环境、开发环境

## 概述

本指南提供了 AgentMem 系统的完整部署方案，包括单机部署、集群部署、容器化部署和云原生部署等多种方式。

## 系统要求

### 最低配置
- **CPU:** 2核心
- **内存:** 4GB RAM
- **存储:** 20GB 可用空间
- **操作系统:** Linux (Ubuntu 20.04+), macOS (10.15+), Windows 10+

### 推荐配置
- **CPU:** 8核心
- **内存:** 16GB RAM
- **存储:** 100GB SSD
- **网络:** 1Gbps

### 依赖软件
- **Rust:** 1.70+
- **PostgreSQL:** 13+ (可选)
- **Redis:** 6+ (可选)
- **Docker:** 20+ (容器化部署)
- **Kubernetes:** 1.20+ (云原生部署)

## 快速开始

### 1. 源码部署

```bash
# 克隆项目
git clone https://gitcode.com/louloulin/agentmem.git
cd agentmem

# 安装依赖
cargo build --release

# 配置环境
cp config/default.toml config/local.toml
# 编辑 config/local.toml 设置你的配置

# 运行服务
cargo run --release --bin agentmem-server
```

### 2. Docker 部署

```bash
# 构建镜像
docker build -t agentmem:latest .

# 运行容器
docker run -d \
  --name agentmem \
  -p 8080:8080 \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/data:/app/data \
  agentmem:latest
```

### 3. Docker Compose 部署

```yaml
# docker-compose.yml
version: '3.8'

services:
  agentmem:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - AGENT_MEM_ENV=production
    volumes:
      - ./config:/app/config
      - ./data:/app/data
      - ./logs:/app/logs
    depends_on:
      - postgres
      - redis

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: agentmem
      POSTGRES_USER: agentmem
      POSTGRES_PASSWORD: your_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f agentmem
```

## 配置管理

### 配置文件结构

```toml
# config/production.toml

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgresql://user:password@localhost/agentmem"
max_connections = 10
min_connections = 1

[redis]
url = "redis://localhost:6379"
pool_size = 10

[llm]
provider = "openai"
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"
temperature = 0.7

[vector_store]
provider = "pinecone"
api_key = "${PINECONE_API_KEY}"
environment = "us-west1-gcp"
index_name = "agentmem-index"

[logging]
level = "info"
format = "json"
file = "/app/logs/agentmem.log"

[security]
jwt_secret = "${JWT_SECRET}"
cors_origins = ["https://your-frontend.com"]
rate_limit_requests = 1000
rate_limit_window = 3600

[monitoring]
metrics_enabled = true
metrics_port = 9090
health_check_path = "/health"
```

### 环境变量

```bash
# 必需的环境变量
export OPENAI_API_KEY="your-openai-api-key"
export PINECONE_API_KEY="your-pinecone-api-key"
export JWT_SECRET="your-jwt-secret"
export DATABASE_URL="postgresql://user:password@localhost/agentmem"

# 可选的环境变量
export RUST_LOG="info"
export AGENT_MEM_ENV="production"
export REDIS_URL="redis://localhost:6379"
```

## 生产环境部署

### 1. 系统服务配置

```ini
# /etc/systemd/system/agentmem.service
[Unit]
Description=AgentMem Service
After=network.target

[Service]
Type=simple
User=agentmem
Group=agentmem
WorkingDirectory=/opt/agentmem
ExecStart=/opt/agentmem/target/release/agentmem-server
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=AGENT_MEM_ENV=production
EnvironmentFile=/opt/agentmem/.env

[Install]
WantedBy=multi-user.target
```

```bash
# 启用和启动服务
sudo systemctl enable agentmem
sudo systemctl start agentmem
sudo systemctl status agentmem
```

### 2. Nginx 反向代理

```nginx
# /etc/nginx/sites-available/agentmem
server {
    listen 80;
    server_name your-domain.com;
    
    # 重定向到 HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    # SSL 配置
    ssl_certificate /path/to/your/certificate.crt;
    ssl_certificate_key /path/to/your/private.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    # 安全头
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains";
    
    # 代理配置
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket 支持
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # 超时设置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # 健康检查
    location /health {
        proxy_pass http://127.0.0.1:8080/health;
        access_log off;
    }
    
    # 静态文件缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### 3. 数据库初始化

```sql
-- 创建数据库和用户
CREATE DATABASE agentmem;
CREATE USER agentmem WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE agentmem TO agentmem;

-- 连接到 agentmem 数据库
\c agentmem

-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "vector";

-- 运行迁移
-- (AgentMem 会自动创建表结构)
```

## Kubernetes 部署

### 1. 命名空间和配置

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: agentmem
---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: agentmem-config
  namespace: agentmem
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    workers = 4
    
    [database]
    url = "postgresql://agentmem:password@postgres:5432/agentmem"
    max_connections = 10
    
    [redis]
    url = "redis://redis:6379"
    pool_size = 10
---
# k8s/secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: agentmem-secrets
  namespace: agentmem
type: Opaque
data:
  openai-api-key: <base64-encoded-key>
  pinecone-api-key: <base64-encoded-key>
  jwt-secret: <base64-encoded-secret>
```

### 2. 部署配置

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agentmem
  namespace: agentmem
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
      - name: agentmem
        image: agentmem:latest
        ports:
        - containerPort: 8080
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: agentmem-secrets
              key: openai-api-key
        - name: PINECONE_API_KEY
          valueFrom:
            secretKeyRef:
              name: agentmem-secrets
              key: pinecone-api-key
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: agentmem-secrets
              key: jwt-secret
        volumeMounts:
        - name: config
          mountPath: /app/config
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: agentmem-config
---
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: agentmem-service
  namespace: agentmem
spec:
  selector:
    app: agentmem
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP
---
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: agentmem-ingress
  namespace: agentmem
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - your-domain.com
    secretName: agentmem-tls
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: agentmem-service
            port:
              number: 80
```

### 3. 部署命令

```bash
# 应用配置
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/deployment.yaml

# 检查部署状态
kubectl get pods -n agentmem
kubectl logs -f deployment/agentmem -n agentmem

# 扩缩容
kubectl scale deployment agentmem --replicas=5 -n agentmem
```

## 监控和日志

### 1. Prometheus 监控

```yaml
# k8s/monitoring.yaml
apiVersion: v1
kind: ServiceMonitor
metadata:
  name: agentmem-metrics
  namespace: agentmem
spec:
  selector:
    matchLabels:
      app: agentmem
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

### 2. 日志收集

```yaml
# k8s/logging.yaml
apiVersion: logging.coreos.com/v1
kind: ClusterLogForwarder
metadata:
  name: agentmem-logs
spec:
  outputs:
  - name: elasticsearch
    type: elasticsearch
    url: http://elasticsearch:9200
  pipelines:
  - name: agentmem-pipeline
    inputRefs:
    - application
    filterRefs:
    - agentmem-filter
    outputRefs:
    - elasticsearch
```

## 备份和恢复

### 1. 数据库备份

```bash
#!/bin/bash
# backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups"
DB_NAME="agentmem"

# 创建备份
pg_dump -h localhost -U agentmem -d $DB_NAME > $BACKUP_DIR/agentmem_$DATE.sql

# 压缩备份
gzip $BACKUP_DIR/agentmem_$DATE.sql

# 清理旧备份 (保留30天)
find $BACKUP_DIR -name "agentmem_*.sql.gz" -mtime +30 -delete

echo "Backup completed: agentmem_$DATE.sql.gz"
```

### 2. 配置备份

```bash
#!/bin/bash
# config-backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/config"

# 备份配置文件
tar -czf $BACKUP_DIR/config_$DATE.tar.gz /opt/agentmem/config/

echo "Config backup completed: config_$DATE.tar.gz"
```

## 故障排除

### 常见问题

1. **服务启动失败**
   ```bash
   # 检查日志
   journalctl -u agentmem -f
   
   # 检查配置
   /opt/agentmem/target/release/agentmem-server --check-config
   ```

2. **数据库连接问题**
   ```bash
   # 测试数据库连接
   psql -h localhost -U agentmem -d agentmem -c "SELECT 1;"
   ```

3. **内存不足**
   ```bash
   # 检查内存使用
   free -h
   ps aux | grep agentmem
   ```

### 性能调优

1. **数据库优化**
   ```sql
   -- 调整 PostgreSQL 配置
   ALTER SYSTEM SET shared_buffers = '256MB';
   ALTER SYSTEM SET effective_cache_size = '1GB';
   ALTER SYSTEM SET maintenance_work_mem = '64MB';
   SELECT pg_reload_conf();
   ```

2. **应用优化**
   ```toml
   # config/production.toml
   [server]
   workers = 8  # 根据 CPU 核心数调整
   
   [database]
   max_connections = 20  # 根据负载调整
   ```

## 安全最佳实践

1. **网络安全**
   - 使用 HTTPS
   - 配置防火墙
   - 限制数据库访问

2. **认证和授权**
   - 强密码策略
   - JWT 令牌过期时间
   - API 访问限制

3. **数据保护**
   - 数据加密
   - 定期备份
   - 访问审计

---

**文档维护者:** AgentMem 运维团队  
**下次更新:** 2025年10月4日
