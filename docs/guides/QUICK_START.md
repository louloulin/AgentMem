# AgentMem 快速启动和验证指南

## 🚀 快速启动（3分钟）

### 方法 1：自动化脚本（推荐）

```bash
# 一键启动和验证
cd agentmen
bash scripts/start_and_verify.sh
```

这个脚本会自动：
1. ✅ 检查环境（protoc, Rust, jq）
2. ✅ 编译项目
3. ✅ 运行测试
4. ✅ 启动服务器
5. ✅ 验证核心 API
6. ✅ 显示访问信息

### 方法 2：手动启动

```bash
# 1. 设置环境变量
export PROTOC=/opt/homebrew/bin/protoc

# 2. 编译服务器
cargo build --release -p agent-mem-server

# 3. 启动服务器
./target/release/agent-mem-server \
    --host 127.0.0.1 \
    --port 8080 \
    --log-level info

# 4. 在另一个终端验证
curl http://localhost:8080/health | jq '.'
```

---

## 📊 核心功能验证

### 1. Health Check

```bash
# 基础健康检查
curl http://localhost:8080/health

# 存活检查
curl http://localhost:8080/health/live

# 就绪检查
curl http://localhost:8080/health/ready
```

**期望输出**:
```json
{
  "status": "healthy",
  "version": "2.0.0",
  "components": {
    "database": "healthy",
    "vector_store": "healthy"
  }
}
```

### 2. API 文档

访问 Swagger UI 查看完整 API 文档：
```
http://localhost:8080/swagger-ui
```

下载 OpenAPI 规范：
```bash
curl http://localhost:8080/api-docs/openapi.json > openapi.json
```

### 3. Memory 管理（核心功能）

#### 创建记忆

```bash
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "content": "AgentMem 是一个企业级 AI Agent 记忆管理平台",
    "metadata": {
      "source": "quick_start",
      "category": "documentation"
    }
  }'
```

**期望输出**:
```json
{
  "id": "mem_xxx",
  "content": "AgentMem 是一个企业级 AI Agent 记忆管理平台",
  "metadata": {...},
  "created_at": "2025-10-30T...",
  "status": "success"
}
```

#### 搜索记忆

```bash
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "AgentMem",
    "limit": 10
  }'
```

#### 批量创建

```bash
curl -X POST http://localhost:8080/api/v1/memories/batch \
  -H "Content-Type: application/json" \
  -d '{
    "memories": [
      {
        "content": "记忆 1",
        "metadata": {"batch": "test"}
      },
      {
        "content": "记忆 2",
        "metadata": {"batch": "test"}
      }
    ]
  }'
```

### 4. 统计和监控

#### Dashboard 统计

```bash
curl http://localhost:8080/api/v1/stats/dashboard | jq '.'
```

#### Memory 增长趋势

```bash
curl http://localhost:8080/api/v1/stats/memories/growth | jq '.'
```

#### Agent 活动统计

```bash
curl http://localhost:8080/api/v1/stats/agents/activity | jq '.'
```

### 5. Metrics

#### JSON 格式

```bash
curl http://localhost:8080/metrics | jq '.'
```

#### Prometheus 格式

```bash
curl http://localhost:8080/metrics/prometheus
```

---

## 🧪 自动化测试

### 运行核心 API 测试

```bash
# 确保服务器正在运行
bash scripts/test_core_api.sh
```

这个脚本会测试：
- ✅ Health & Monitoring (3个端点)
- ✅ Metrics (2个端点)
- ✅ API Documentation (1个端点)
- ✅ Memory Management (4个端点)
- ✅ Statistics (3个端点)
- ✅ MCP Server (2个端点)

**总计**: 15+ 个核心 API 端点

### 运行单元测试

```bash
export PROTOC=/opt/homebrew/bin/protoc
cargo test --workspace --lib
```

**期望结果**: 1148 passed; 0 failed; 56 ignored (100% 通过率)

---

## 🎯 核心功能清单

### ✅ 已验证功能

- [x] **Health & Monitoring**: 健康检查、存活检查、就绪检查
- [x] **Memory Management**: 创建、查询、更新、删除、搜索记忆
- [x] **Batch Operations**: 批量创建、批量删除
- [x] **Statistics**: Dashboard 统计、增长趋势、活动分析
- [x] **Metrics**: JSON 和 Prometheus 格式的指标
- [x] **API Documentation**: OpenAPI/Swagger 文档
- [x] **MCP Server**: Model Context Protocol 服务器

### 🔄 待验证功能（需要认证）

- [ ] **User Management**: 用户注册、登录、权限管理
- [ ] **Organization Management**: 组织创建、成员管理
- [ ] **Agent Management**: Agent 创建、状态管理
- [ ] **Chat**: 对话流式响应
- [ ] **Tool Management**: 工具注册和执行

---

## 📈 性能指标

### 预期性能

- **Health Check**: < 10ms
- **Memory Create**: < 100ms
- **Memory Search**: < 100ms (1000条记录)
- **Batch Create**: < 500ms (100条记录)

### 性能测试

```bash
# 使用 Apache Bench 测试
ab -n 1000 -c 10 http://localhost:8080/health

# 使用 wrk 测试
wrk -t4 -c100 -d30s http://localhost:8080/health
```

---

## 🐛 故障排查

### 服务器无法启动

1. **检查端口占用**:
   ```bash
   lsof -i :8080
   ```

2. **查看日志**:
   ```bash
   tail -f /tmp/agentmem_server.log
   ```

3. **检查数据库**:
   ```bash
   # LibSQL 数据库文件
   ls -lh ~/.agentmem/data/
   ```

### API 返回错误

1. **检查请求格式**:
   ```bash
   # 使用 -v 查看详细信息
   curl -v http://localhost:8080/api/v1/memories
   ```

2. **查看服务器日志**:
   ```bash
   tail -f /tmp/agentmem_server.log
   ```

### 测试失败

1. **重新编译**:
   ```bash
   cargo clean
   cargo build --release -p agent-mem-server
   ```

2. **运行特定测试**:
   ```bash
   cargo test --lib -p agent-mem-core -- --nocapture
   ```

---

## 📚 下一步

### 开发

1. **查看 API 文档**: http://localhost:8080/swagger-ui
2. **阅读代码**: `crates/agent-mem-server/src/routes/`
3. **运行示例**: `cargo run --example <example_name>`

### 集成

1. **Python 客户端**: 使用 `requests` 库调用 API
2. **JavaScript 客户端**: 使用 `fetch` 或 `axios`
3. **其他语言**: 使用 OpenAPI 生成客户端代码

### 部署

1. **Docker**: 构建 Docker 镜像
2. **Kubernetes**: 部署到 K8s 集群
3. **云服务**: AWS/GCP/Azure 部署

---

## 🎉 成功标志

如果你看到以下输出，说明 AgentMem 已成功启动：

```
✅ 所有测试通过！

下一步建议：
  1. 访问 Swagger UI: http://localhost:8080/swagger-ui
  2. 查看 API 文档: http://localhost:8080/api-docs/openapi.json
  3. 运行集成测试: cargo test --workspace
```

**恭喜！AgentMem 核心功能已验证通过！** 🚀

