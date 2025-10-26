# AgentMem 后端服务器启动指南

**日期**: 2025-10-26  
**目的**: 解决后端启动的PostgreSQL配置问题

---

## 问题分析

### 遇到的错误

```
Failed to create server: Server error: Failed to create repositories: 
Configuration error: PostgreSQL support is not enabled. 
Enable the 'postgres' feature to use PostgreSQL backend.
```

### 根本原因

后端服务器默认尝试使用PostgreSQL数据库，但：
1. ❌ `postgres` feature 未在编译时启用
2. ❌ 默认配置指向PostgreSQL
3. ✅ `libsql` feature 已启用（默认）

---

## 解决方案

### 方案1: 使用环境变量（推荐）

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 设置环境变量强制使用LibSQL
DATABASE_TYPE=libsql cargo run --release --bin agent-mem-server

# 或者使用.env文件
echo "DATABASE_TYPE=libsql" > .env
cargo run --release --bin agent-mem-server
```

### 方案2: 修改配置文件

查找并修改配置文件（如果存在）：
- `config/default.toml`
- `Rocket.toml`
- `.env`

修改数据库类型为 `libsql`。

### 方案3: 使用命令行参数（如果支持）

```bash
cargo run --release --bin agent-mem-server -- --database-type libsql
```

---

## 验证启动

### 1. 检查进程

```bash
ps aux | grep agent-mem-server
```

### 2. 测试健康检查

```bash
curl http://localhost:8080/health
```

预期响应：
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### 3. 测试API

```bash
# 获取Agents列表
curl http://localhost:8080/api/v1/agents

# 获取Swagger文档
curl http://localhost:8080/swagger-ui/
```

---

## 正确的启动流程

### Step 1: 准备环境

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 确保依赖已安装
cargo build --release --bin agent-mem-server
```

### Step 2: 启动服务器

```bash
# 前台启动（用于调试）
DATABASE_TYPE=libsql cargo run --release --bin agent-mem-server

# 后台启动（用于生产）
DATABASE_TYPE=libsql cargo run --release --bin agent-mem-server &> /tmp/agentmem-server.log &
```

### Step 3: 验证启动

```bash
# 等待5秒让服务器启动
sleep 5

# 测试健康检查
curl http://localhost:8080/health

# 检查日志
tail -f /tmp/agentmem-server.log
```

---

## 配置说明

### LibSQL 配置（推荐）

**优点**:
- ✅ 零配置
- ✅ 嵌入式数据库
- ✅ 适合开发和小规模部署
- ✅ 无需外部数据库服务

**配置**:
```bash
DATABASE_TYPE=libsql
DATABASE_PATH=./agentmem.db  # 可选，默认为 ./agentmem.db
```

### PostgreSQL 配置（生产环境）

**优点**:
- ✅ 企业级数据库
- ✅ 高性能
- ✅ 适合大规模部署

**前提条件**:
1. 重新编译，启用 `postgres` feature:
   ```bash
   cargo build --release --bin agent-mem-server --features postgres
   ```

2. 启动PostgreSQL服务器

3. 配置环境变量:
   ```bash
   DATABASE_TYPE=postgres
   DATABASE_URL=postgres://user:password@localhost:5432/agentmem
   ```

---

## 常见问题

### Q1: 后端启动后立即退出

**原因**: 配置错误或端口被占用

**解决**:
```bash
# 检查端口是否被占用
lsof -i :8080

# 查看详细日志
tail -f /tmp/agentmem-server.log
```

### Q2: CORS 错误

**原因**: 前端（localhost:3001）无法访问后端（localhost:8080）

**解决**: 后端代码中已配置CORS，确保允许localhost:3001

### Q3: API 404 错误

**原因**: API路径不正确

**正确路径**:
- Health: `GET /health`
- Agents: `GET /api/v1/agents`
- Memories: `GET /api/v1/memories`
- Swagger: `GET /swagger-ui/`

---

## 端口配置

### 默认端口

- **后端**: 8080
- **前端**: 3001

### 修改端口

```bash
# 后端
PORT=8090 DATABASE_TYPE=libsql cargo run --release --bin agent-mem-server

# 前端（修改 package.json）
PORT=3002 npm run dev
```

**注意**: 修改端口后需要更新前端的API Client配置（`src/lib/api-client.ts`）

---

## 完整启动脚本

### start-backend.sh

```bash
#!/bin/bash

cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 停止旧进程
pkill -f agent-mem-server

# 启动后端
DATABASE_TYPE=libsql \
PORT=8080 \
RUST_LOG=info \
cargo run --release --bin agent-mem-server &> /tmp/agentmem-server.log &

echo "AgentMem Backend starting..."
echo "PID: $!"
echo "Log: /tmp/agentmem-server.log"

# 等待启动
sleep 5

# 验证
if curl -s http://localhost:8080/health > /dev/null; then
    echo "✅ Backend started successfully!"
    echo "   Health: http://localhost:8080/health"
    echo "   Swagger: http://localhost:8080/swagger-ui/"
else
    echo "❌ Backend failed to start"
    echo "   Check logs: tail -f /tmp/agentmem-server.log"
fi
```

### 使用方法

```bash
chmod +x start-backend.sh
./start-backend.sh
```

---

## 总结

**推荐启动方式**:
```bash
DATABASE_TYPE=libsql cargo run --release --bin agent-mem-server
```

**验证方式**:
```bash
curl http://localhost:8080/health
```

**前端配置**:
- API Base URL: `http://localhost:8080`
- 已配置在: `agentmem-website/src/lib/api-client.ts`

---

**创建日期**: 2025-10-26  
**状态**: ✅ 完成  
**下一步**: 启动后端并验证前后端对接

