# AgentMem 部署指南

## 🚀 部署概述

本指南详细介绍了如何在不同环境中部署 AgentMem，包括单机部署、分布式部署和云原生部署。

## 📋 部署前准备

### 系统要求

#### 生产环境最低要求
- **CPU**: 4核心 2.0GHz+
- **内存**: 8GB RAM
- **存储**: 50GB SSD
- **网络**: 1Gbps 带宽
- **操作系统**: Ubuntu 20.04+, CentOS 8+, Windows Server 2019+

#### 推荐生产配置
- **CPU**: 8核心 3.0GHz+
- **内存**: 32GB RAM
- **存储**: 500GB NVMe SSD
- **网络**: 10Gbps 带宽
- **操作系统**: Ubuntu 22.04 LTS

### 依赖软件

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential curl git

# CentOS/RHEL
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl git

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安装 Zig
wget https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.xz
tar -xf zig-linux-x86_64-0.14.0.tar.xz
sudo mv zig-linux-x86_64-0.14.0 /opt/zig
echo 'export PATH="/opt/zig:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## 🏠 单机部署

### 1. 源码编译部署

```bash
# 克隆仓库
git clone https://github.com/louloulin/AgentMem.git
cd AgentMem

# 编译发布版本
cargo build --release

# 生成 C 头文件
cargo run --bin generate_bindings

# 编译 Zig 组件
zig build -Doptimize=ReleaseFast

# 运行测试验证
cargo test --release
zig build test
```

### 2. 配置文件设置

创建 `/etc/AgentMem/config.toml`:

```toml
[database]
path = "/var/lib/AgentMem/data"
max_connections = 100
connection_timeout = 30
query_timeout = 120
enable_wal = true
cache_size = 1073741824  # 1GB

[vector]
dimension = 384
similarity_algorithm = "cosine"
index_type = "hnsw"
ef_construction = 200
m = 16

[memory]
max_memories_per_agent = 50000
importance_threshold = 0.05
decay_factor = 0.001
cleanup_interval = 3600  # 1 hour

[security]
enable_auth = true
enable_encryption = true
jwt_secret = "your-production-secret-key-here"
session_timeout = 86400  # 24 hours

[performance]
enable_cache = true
batch_size = 5000
worker_threads = 8
io_threads = 4

[logging]
level = "info"
file = "/var/log/AgentMem/AgentMem.log"
max_size = "100MB"
max_files = 10

[monitoring]
enable_metrics = true
metrics_port = 9090
health_check_port = 8080
```

### 3. 系统服务配置

创建 `/etc/systemd/system/AgentMem.service`:

```ini
[Unit]
Description=AgentMem High-Performance AI Agent Database
After=network.target
Wants=network.target

[Service]
Type=simple
User=AgentMem
Group=AgentMem
WorkingDirectory=/opt/AgentMem
ExecStart=/opt/AgentMem/target/release/AgentMem-server --config /etc/AgentMem/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
LimitNOFILE=65536
LimitNPROC=32768

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/AgentMem /var/log/AgentMem

[Install]
WantedBy=multi-user.target
```

### 4. 启动服务

```bash
# 创建用户和目录
sudo useradd -r -s /bin/false AgentMem
sudo mkdir -p /var/lib/AgentMem/data
sudo mkdir -p /var/log/AgentMem
sudo mkdir -p /etc/AgentMem
sudo chown -R AgentMem:AgentMem /var/lib/AgentMem /var/log/AgentMem

# 复制二进制文件
sudo cp target/release/AgentMem-server /opt/AgentMem/
sudo chown AgentMem:AgentMem /opt/AgentMem/AgentMem-server
sudo chmod +x /opt/AgentMem/AgentMem-server

# 启动服务
sudo systemctl daemon-reload
sudo systemctl enable AgentMem
sudo systemctl start AgentMem

# 检查状态
sudo systemctl status AgentMem
```

## 🌐 分布式部署

### 1. 集群架构

```
┌─────────────────────────────────────────────────────────────┐
│                    AgentMem 分布式集群                        │
├─────────────────────────────────────────────────────────────┤
│  负载均衡层                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   HAProxy   │ │   Nginx     │ │   Consul    │           │
│  │   (主)      │ │   (备)      │ │  (服务发现)  │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  AgentMem 节点层                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │  Node-1     │ │  Node-2     │ │  Node-3     │           │
│  │  (主节点)   │ │  (工作节点)  │ │  (工作节点)  │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  存储层                                                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │  LanceDB    │ │   Redis     │ │   MinIO     │           │
│  │  (主存储)   │ │   (缓存)    │ │  (对象存储)  │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

### 2. 节点配置

#### 主节点配置 (node-1)

```toml
[cluster]
node_id = "node-1"
node_type = "master"
bind_address = "0.0.0.0:7000"
cluster_members = [
    "node-1:7000",
    "node-2:7000", 
    "node-3:7000"
]
election_timeout = 5000
heartbeat_interval = 1000

[replication]
enable_replication = true
replication_factor = 3
sync_mode = "async"
backup_interval = 3600

[sharding]
enable_sharding = true
shard_count = 16
hash_algorithm = "consistent"
```

#### 工作节点配置 (node-2, node-3)

```toml
[cluster]
node_id = "node-2"  # node-3 使用 "node-3"
node_type = "worker"
bind_address = "0.0.0.0:7000"
master_address = "node-1:7000"
cluster_members = [
    "node-1:7000",
    "node-2:7000",
    "node-3:7000"
]
```

### 3. 负载均衡配置

#### HAProxy 配置 (`/etc/haproxy/haproxy.cfg`)

```
global
    daemon
    maxconn 4096
    log stdout local0

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    option httplog

frontend AgentMem_frontend
    bind *:8080
    default_backend AgentMem_backend

backend AgentMem_backend
    balance roundrobin
    option httpchk GET /health
    server node1 node-1:8080 check
    server node2 node-2:8080 check
    server node3 node-3:8080 check

frontend AgentMem_api
    bind *:9000
    default_backend AgentMem_api_backend

backend AgentMem_api_backend
    balance leastconn
    server node1 node-1:9000 check
    server node2 node-2:9000 check
    server node3 node-3:9000 check
```

## ☁️ 云原生部署

### 1. Docker 容器化

#### Dockerfile

```dockerfile
# 多阶段构建
FROM rust:1.70 as rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM ziglang/zig:0.14.0 as zig-builder
WORKDIR /app
COPY build.zig ./
COPY src ./src
COPY --from=rust-builder /app/target/release/libagent_db_rust.so ./target/release/
RUN zig build -Doptimize=ReleaseFast

# 运行时镜像
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=rust-builder /app/target/release/AgentMem-server ./
COPY --from=zig-builder /app/zig-out/bin/* ./
COPY config/docker.toml ./config.toml

EXPOSE 8080 9000 9090
USER 1000:1000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./AgentMem-server", "--config", "config.toml"]
```

#### Docker Compose

```yaml
version: '3.8'

services:
  AgentMem-node1:
    build: .
    container_name: AgentMem-node1
    hostname: node1
    ports:
      - "8081:8080"
      - "9001:9000"
      - "9091:9090"
    volumes:
      - AgentMem-data1:/var/lib/AgentMem
      - ./config/node1.toml:/app/config.toml
    environment:
      - AgentMem_NODE_ID=node-1
      - AgentMem_NODE_TYPE=master
    networks:
      - AgentMem-network

  AgentMem-node2:
    build: .
    container_name: AgentMem-node2
    hostname: node2
    ports:
      - "8082:8080"
      - "9002:9000"
      - "9092:9090"
    volumes:
      - AgentMem-data2:/var/lib/AgentMem
      - ./config/node2.toml:/app/config.toml
    environment:
      - AgentMem_NODE_ID=node-2
      - AgentMem_NODE_TYPE=worker
    depends_on:
      - AgentMem-node1
    networks:
      - AgentMem-network

  AgentMem-node3:
    build: .
    container_name: AgentMem-node3
    hostname: node3
    ports:
      - "8083:8080"
      - "9003:9000"
      - "9093:9090"
    volumes:
      - AgentMem-data3:/var/lib/AgentMem
      - ./config/node3.toml:/app/config.toml
    environment:
      - AgentMem_NODE_ID=node-3
      - AgentMem_NODE_TYPE=worker
    depends_on:
      - AgentMem-node1
    networks:
      - AgentMem-network

  redis:
    image: redis:7-alpine
    container_name: AgentMem-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    networks:
      - AgentMem-network

  prometheus:
    image: prom/prometheus:latest
    container_name: AgentMem-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    networks:
      - AgentMem-network

  grafana:
    image: grafana/grafana:latest
    container_name: AgentMem-grafana
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    networks:
      - AgentMem-network

volumes:
  AgentMem-data1:
  AgentMem-data2:
  AgentMem-data3:
  redis-data:
  prometheus-data:
  grafana-data:

networks:
  AgentMem-network:
    driver: bridge
```

### 2. Kubernetes 部署

#### Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: AgentMem
  labels:
    name: AgentMem
```

#### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: AgentMem-config
  namespace: AgentMem
data:
  config.toml: |
    [database]
    path = "/var/lib/AgentMem/data"
    max_connections = 200
    connection_timeout = 30
    query_timeout = 120
    enable_wal = true
    cache_size = 2147483648  # 2GB
    
    [cluster]
    enable_cluster = true
    node_id = "${NODE_ID}"
    node_type = "${NODE_TYPE}"
    bind_address = "0.0.0.0:7000"
    
    [monitoring]
    enable_metrics = true
    metrics_port = 9090
    health_check_port = 8080
```

#### StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: AgentMem
  namespace: AgentMem
spec:
  serviceName: AgentMem-headless
  replicas: 3
  selector:
    matchLabels:
      app: AgentMem
  template:
    metadata:
      labels:
        app: AgentMem
    spec:
      containers:
      - name: AgentMem
        image: AgentMem:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9000
          name: api
        - containerPort: 9090
          name: metrics
        - containerPort: 7000
          name: cluster
        env:
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: NODE_TYPE
          value: "worker"
        volumeMounts:
        - name: data
          mountPath: /var/lib/AgentMem
        - name: config
          mountPath: /app/config.toml
          subPath: config.toml
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
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
      volumes:
      - name: config
        configMap:
          name: AgentMem-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 100Gi
```

## 📊 监控和运维

### 1. 健康检查

```bash
# 检查服务状态
curl http://localhost:8080/health

# 检查集群状态
curl http://localhost:8080/cluster/status

# 检查性能指标
curl http://localhost:9090/metrics
```

### 2. 日志管理

```bash
# 查看服务日志
sudo journalctl -u AgentMem -f

# 查看应用日志
tail -f /var/log/AgentMem/AgentMem.log

# 日志轮转配置
sudo logrotate -d /etc/logrotate.d/AgentMem
```

### 3. 备份策略

```bash
#!/bin/bash
# 备份脚本 backup.sh

BACKUP_DIR="/backup/AgentMem"
DATE=$(date +%Y%m%d_%H%M%S)
DATA_DIR="/var/lib/AgentMem/data"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 数据备份
tar -czf $BACKUP_DIR/AgentMem_data_$DATE.tar.gz -C $DATA_DIR .

# 配置备份
cp /etc/AgentMem/config.toml $BACKUP_DIR/config_$DATE.toml

# 清理旧备份 (保留30天)
find $BACKUP_DIR -name "*.tar.gz" -mtime +30 -delete
find $BACKUP_DIR -name "config_*.toml" -mtime +30 -delete

echo "备份完成: $DATE"
```

## 🔧 性能调优

### 1. 系统级优化

```bash
# 内核参数优化
echo 'net.core.somaxconn = 65535' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65535' >> /etc/sysctl.conf
echo 'fs.file-max = 1000000' >> /etc/sysctl.conf
sysctl -p

# 文件描述符限制
echo '* soft nofile 1000000' >> /etc/security/limits.conf
echo '* hard nofile 1000000' >> /etc/security/limits.conf
```

### 2. 应用级优化

```toml
[performance]
# 工作线程数 = CPU核心数
worker_threads = 16

# I/O线程数 = CPU核心数 / 2
io_threads = 8

# 批处理大小
batch_size = 10000

# 缓存大小 = 可用内存的 50%
cache_size = 16777216000  # 16GB

# 连接池大小
max_connections = 1000
```

---

**文档版本**: v1.0  
**最后更新**: 2025年6月19日  
**维护者**: AgentMem开发团队
