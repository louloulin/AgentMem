# AgentMem 部署指南

## 概述

AgentMem 采用前后端分离架构：
- **前端**: Next.js 应用 (agentmem-ui)
- **后端**: Rust 服务器 (agent-mem-server)

本文档提供完整的部署指南，支持多种部署方式。

## 目录

1. [快速开始](#快速开始)
2. [构建发布包](#构建发布包)
3. [部署方式](#部署方式)
4. [环境变量配置](#环境变量配置)
5. [生产环境优化](#生产环境优化)
6. [故障排查](#故障排查)

---

## 快速开始

### 前置要求

- **Node.js**: >= 18.0.0
- **Rust**: >= 1.70.0
- **npm** 或 **yarn**
- **SQLite** 或 **PostgreSQL**（可选）

### 一键构建

```bash
# 克隆仓库
git clone https://github.com/your-org/agentmem.git
cd agentmem

# 构建所有组件（发布模式）
./build-release.sh

# 构建产物位于 dist/ 目录
```

---

## 构建发布包

### 使用构建脚本

```bash
# 查看帮助
./build-release.sh --help

# 构建所有组件（默认）
./build-release.sh

# 仅构建前端
./build-release.sh --ui-only

# 仅构建后端
./build-release.sh --server-only

# 开发模式构建（更快）
./build-release.sh --dev

# 清理并重新构建
./build-release.sh --clean --all
```

### 手动构建

#### 构建前端

```bash
cd agentmem-ui

# 安装依赖
npm install

# 构建生产版本
npm run build

# 启动生产服务器
npm start
```

#### 构建后端

```bash
# 发布模式
cargo build --package agent-mem-server --release --exclude agent-mem-python

# 二进制文件位于
# target/release/agent-mem-server
```

---

## 部署方式

### 方式 1: 独立部署（推荐）

前端和后端分别部署在不同的服务器或容器中。

#### 部署后端

```bash
cd dist/server

# 1. 配置环境变量
export SERVER_HOST=0.0.0.0
export SERVER_PORT=8080
export DATABASE_URL=sqlite://agentmem.db
export RUST_LOG=info

# 2. 启动服务
./start.sh

# 或直接运行二进制
./agent-mem-server
```

#### 部署前端

```bash
cd dist/ui

# 1. 配置环境变量
export PORT=3000
export NEXT_PUBLIC_API_URL=http://your-backend-server:8080

# 2. 安装依赖（首次）
npm install --production

# 3. 启动服务
./start.sh

# 或使用 npm
npm start
```

### 方式 2: Docker 部署

#### 后端 Dockerfile

```dockerfile
# dist/server/Dockerfile
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 复制二进制文件
COPY agent-mem-server /app/
COPY config.example.toml /app/config.toml

# 暴露端口
EXPOSE 8080

# 启动服务
CMD ["/app/agent-mem-server"]
```

#### 前端 Dockerfile

```dockerfile
# dist/ui/Dockerfile
FROM node:18-alpine

WORKDIR /app

# 复制文件
COPY package.json ./
COPY .next ./.next
COPY public ./public
COPY next.config.ts ./

# 安装生产依赖
RUN npm install --production

# 暴露端口
EXPOSE 3000

# 启动服务
CMD ["npm", "start"]
```

#### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  backend:
    build: ./dist/server
    ports:
      - "8080:8080"
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - DATABASE_URL=sqlite://agentmem.db
      - RUST_LOG=info
    volumes:
      - ./data:/app/data
    restart: unless-stopped

  frontend:
    build: ./dist/ui
    ports:
      - "3000:3000"
    environment:
      - PORT=3000
      - NEXT_PUBLIC_API_URL=http://backend:8080
    depends_on:
      - backend
    restart: unless-stopped
```

启动：

```bash
docker-compose up -d
```

### 方式 3: systemd 服务

#### 后端服务

创建 `/etc/systemd/system/agentmem-server.service`:

```ini
[Unit]
Description=AgentMem Server
After=network.target

[Service]
Type=simple
User=agentmem
Group=agentmem
WorkingDirectory=/opt/agentmem/server
Environment="RUST_LOG=info"
Environment="SERVER_HOST=0.0.0.0"
Environment="SERVER_PORT=8080"
Environment="DATABASE_URL=sqlite:///var/lib/agentmem/agentmem.db"
ExecStart=/opt/agentmem/server/agent-mem-server
Restart=always
RestartSec=10

# 安全加固
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/agentmem

[Install]
WantedBy=multi-user.target
```

#### 前端服务

创建 `/etc/systemd/system/agentmem-ui.service`:

```ini
[Unit]
Description=AgentMem UI
After=network.target agentmem-server.service

[Service]
Type=simple
User=agentmem
Group=agentmem
WorkingDirectory=/opt/agentmem/ui
Environment="NODE_ENV=production"
Environment="PORT=3000"
Environment="NEXT_PUBLIC_API_URL=http://localhost:8080"
ExecStart=/usr/bin/npm start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

#### 安装和启动

```bash
# 创建用户
sudo useradd -r -s /bin/false agentmem

# 创建目录
sudo mkdir -p /opt/agentmem/{server,ui}
sudo mkdir -p /var/lib/agentmem
sudo chown -R agentmem:agentmem /opt/agentmem /var/lib/agentmem

# 复制文件
sudo cp -r dist/server/* /opt/agentmem/server/
sudo cp -r dist/ui/* /opt/agentmem/ui/

# 安装前端依赖
cd /opt/agentmem/ui
sudo -u agentmem npm install --production

# 启动服务
sudo systemctl daemon-reload
sudo systemctl enable agentmem-server agentmem-ui
sudo systemctl start agentmem-server agentmem-ui

# 查看状态
sudo systemctl status agentmem-server
sudo systemctl status agentmem-ui
```

### 方式 4: Nginx 反向代理

```nginx
# /etc/nginx/sites-available/agentmem
upstream backend {
    server localhost:8080;
}

upstream frontend {
    server localhost:3000;
}

server {
    listen 80;
    server_name agentmem.example.com;

    # 前端
    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # 后端 API
    location /api {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS 头（如果需要）
        add_header Access-Control-Allow-Origin *;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
        add_header Access-Control-Allow-Headers "Content-Type, Authorization";
    }

    # 静态文件缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        proxy_pass http://frontend;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

启用配置：

```bash
sudo ln -s /etc/nginx/sites-available/agentmem /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## 环境变量配置

### 后端环境变量

| 变量名 | 说明 | 默认值 | 示例 |
|--------|------|--------|------|
| `SERVER_HOST` | 服务器监听地址 | `0.0.0.0` | `127.0.0.1` |
| `SERVER_PORT` | 服务器端口 | `8080` | `8080` |
| `DATABASE_URL` | 数据库连接字符串 | `sqlite://agentmem.db` | `postgres://user:pass@localhost/agentmem` |
| `RUST_LOG` | 日志级别 | `info` | `debug`, `warn`, `error` |
| `CORS_ALLOWED_ORIGINS` | CORS 允许的源 | `*` | `http://localhost:3000` |
| `MAX_CONNECTIONS` | 数据库最大连接数 | `10` | `50` |

### 前端环境变量

| 变量名 | 说明 | 默认值 | 示例 |
|--------|------|--------|------|
| `PORT` | 前端端口 | `3000` | `3000` |
| `NODE_ENV` | 运行环境 | `production` | `development`, `production` |
| `NEXT_PUBLIC_API_URL` | 后端 API 地址 | `http://localhost:8080` | `https://api.example.com` |

---

## 生产环境优化

### 1. 数据库优化

#### 使用 PostgreSQL（推荐）

```bash
# 安装 PostgreSQL
sudo apt-get install postgresql

# 创建数据库
sudo -u postgres createdb agentmem
sudo -u postgres createuser agentmem

# 设置环境变量
export DATABASE_URL=postgres://agentmem:password@localhost/agentmem
```

#### 连接池配置

```toml
# config.toml
[database]
url = "postgres://agentmem:password@localhost/agentmem"
max_connections = 50
min_connections = 5
connect_timeout = 30
idle_timeout = 600
```

### 2. 性能优化

#### 启用 Gzip 压缩

```nginx
# Nginx 配置
gzip on;
gzip_vary on;
gzip_min_length 1024;
gzip_types text/plain text/css text/xml text/javascript application/json application/javascript application/xml+rss;
```

#### 使用 CDN

```javascript
// next.config.ts
const nextConfig = {
  assetPrefix: process.env.CDN_URL || '',
};
```

### 3. 安全加固

#### HTTPS 配置

```bash
# 使用 Let's Encrypt
sudo apt-get install certbot python3-certbot-nginx
sudo certbot --nginx -d agentmem.example.com
```

#### 防火墙配置

```bash
# UFW
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### 4. 监控和日志

#### 日志配置

```bash
# 后端日志
export RUST_LOG=info,agent_mem_server=debug

# 前端日志
# 使用 PM2
pm2 start npm --name "agentmem-ui" -- start
pm2 logs agentmem-ui
```

#### 健康检查

```bash
# 后端健康检查
curl http://localhost:8080/health

# 前端健康检查
curl http://localhost:3000/api/health
```

---

## 故障排查

### 后端问题

#### 1. 端口被占用

```bash
# 查看端口占用
lsof -i :8080

# 杀死进程
kill -9 <PID>
```

#### 2. 数据库连接失败

```bash
# 检查数据库连接
psql -U agentmem -d agentmem -h localhost

# 查看日志
RUST_LOG=debug ./agent-mem-server
```

#### 3. 权限问题

```bash
# 检查文件权限
ls -la /opt/agentmem/server/

# 修复权限
sudo chown -R agentmem:agentmem /opt/agentmem/
```

### 前端问题

#### 1. API 连接失败

```bash
# 检查环境变量
echo $NEXT_PUBLIC_API_URL

# 测试 API 连接
curl http://your-backend:8080/api/health
```

#### 2. 构建失败

```bash
# 清理缓存
rm -rf .next node_modules
npm install
npm run build
```

#### 3. 内存不足

```bash
# 增加 Node.js 内存限制
NODE_OPTIONS="--max-old-space-size=4096" npm start
```

### 日志查看

```bash
# systemd 日志
sudo journalctl -u agentmem-server -f
sudo journalctl -u agentmem-ui -f

# Docker 日志
docker-compose logs -f backend
docker-compose logs -f frontend
```

---

## 附录

### A. 完整的配置文件示例

#### config.toml

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://agentmem:password@localhost/agentmem"
max_connections = 50

[cors]
allowed_origins = ["http://localhost:3000", "https://agentmem.example.com"]

[mcp]
enabled = true

[logging]
level = "info"
```

### B. 性能基准

- **后端**: ~10,000 请求/秒（单核）
- **前端**: ~1,000 并发用户
- **数据库**: SQLite（开发）/ PostgreSQL（生产）

### C. 支持和反馈

- GitHub Issues: https://github.com/your-org/agentmem/issues
- 文档: https://docs.agentmem.com
- 社区: https://community.agentmem.com

---

**最后更新**: 2025-11-12

