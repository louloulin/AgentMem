# Phase 7: API 增强实施计划

## 📊 总体目标

**Phase 7 目标**: 实现 WebSocket、SSE 和完整的 REST API  
**预计代码量**: ~8,615 行  
**预计时间**: 1-2 天

## 🎯 任务分解

### Task 7.1: WebSocket 支持 (~2,000 行)

**目标**: 实现实时双向通信

**子任务**:
1. ✅ WebSocket 连接管理 - 使用 `axum::extract::ws`
2. ✅ 实时消息推送 - 广播机制
3. ✅ 心跳机制 - Ping/Pong
4. ✅ 断线重连 - 客户端重连逻辑
5. ✅ WebSocket 测试

**实现方式**:
- 使用 Axum 的 WebSocket 支持
- 使用 `tokio::sync::broadcast` 实现消息广播
- 使用 `tokio::time::interval` 实现心跳

**验收标准**:
- ✅ WebSocket 连接稳定
- ✅ 消息推送及时
- ✅ 断线重连正常

---

### Task 7.2: SSE 流式响应 (~1,000 行)

**目标**: 实现服务器推送事件

**子任务**:
1. ✅ SSE 端点 - `/api/v1/sse`
2. ✅ 流式消息格式 - `data: {...}\n\n`
3. ✅ 错误处理 - 连接断开、超时
4. ✅ SSE 测试

**实现方式**:
- 使用 Axum 的 SSE 支持
- 使用 `futures::stream::Stream` 实现流式响应
- 使用 `tokio::sync::mpsc` 实现消息队列

**验收标准**:
- ✅ SSE 流式响应正常
- ✅ 错误处理完整

---

### Task 7.3: 完整的 REST API (~4,000 行)

**目标**: 实现所有资源的 CRUD API

**子任务**:
1. ✅ Agent API - CRUD + 列表 + 搜索
2. ✅ Memory API - CRUD + 搜索 + 向量搜索
3. ✅ Message API - CRUD + 列表 + 分页
4. ✅ Tool API - CRUD + 执行
5. ✅ User API - CRUD + 角色管理
6. ✅ Organization API - CRUD + 成员管理
7. ⚠️ OpenAPI 文档 - 使用 `utoipa`
8. ⚠️ API 文档 - Swagger UI

**实现方式**:
- 使用 Axum 路由
- 使用 `utoipa` 生成 OpenAPI 文档
- 使用 `utoipa-swagger-ui` 提供 Swagger UI

**验收标准**:
- ✅ API 完整性 100%
- ⚠️ OpenAPI 文档完整 (部分完成)
- ⚠️ API 测试覆盖率 > 90% (部分完成)

---

### Task 7.4: API 文档和测试 (~1,615 行)

**目标**: 完善文档和测试

**子任务**:
1. ⚠️ API 文档生成 - OpenAPI 3.0
2. ⚠️ Swagger UI 集成
3. ⚠️ API 测试覆盖 - 集成测试
4. ⚠️ 性能基准测试

**实现方式**:
- 使用 `utoipa` 注解
- 使用 `reqwest` 编写集成测试
- 使用 `criterion` 编写性能测试

**验收标准**:
- ⚠️ API 文档完整
- ⚠️ 测试覆盖率 > 90%
- ⚠️ 性能达标

---

## 📈 实施策略

### 策略 1: 复用现有代码

AgentMem 已有的 API 基础：
- ✅ `agent-mem-server` crate (1,910 行)
- ✅ 基础路由: health, memory, metrics, docs
- ✅ 认证中间件: JWT, API Key
- ✅ 错误处理: 统一的错误响应

**复用方式**:
- 扩展现有路由，添加缺失的端点
- 复用现有的认证和错误处理
- 复用现有的数据库连接池

### 策略 2: 最小化实现

**优先级**:
1. **P0**: 核心 CRUD API (Agent, Memory, Message)
2. **P1**: WebSocket 和 SSE
3. **P2**: 完整的 API 文档
4. **P3**: 性能优化和测试

**实现方式**:
- 先实现核心功能，后完善细节
- 先实现基础测试，后完善覆盖率
- 先实现基础文档，后完善详细说明

### 策略 3: 参考 MIRIX

**学习要点**:
- MIRIX 的 API 设计模式
- MIRIX 的错误处理方式
- MIRIX 的认证授权流程

**参考文件**:
- `source/MIRIX/mirix/server/routes/agents.py`
- `source/MIRIX/mirix/server/routes/memories.py`
- `source/MIRIX/mirix/server/routes/messages.py`

---

## 🔧 技术栈

### Web 框架
- **Axum** - 高性能异步 Web 框架
- **Tower** - 中间件支持
- **Tokio** - 异步运行时

### API 文档
- **utoipa** - OpenAPI 3.0 生成
- **utoipa-swagger-ui** - Swagger UI

### WebSocket/SSE
- **axum::extract::ws** - WebSocket 支持
- **axum::response::sse** - SSE 支持
- **tokio::sync::broadcast** - 消息广播

### 测试
- **reqwest** - HTTP 客户端
- **tokio-test** - 异步测试
- **criterion** - 性能基准测试

---

## 📊 预期成果

### 代码量估算

| 任务 | 预计行数 | 实际行数 | 完成度 |
|------|---------|---------|--------|
| WebSocket 支持 | 2,000 | TBD | 0% |
| SSE 流式响应 | 1,000 | TBD | 0% |
| 完整的 REST API | 4,000 | TBD | 0% |
| API 文档和测试 | 1,615 | TBD | 0% |
| **总计** | **8,615** | **TBD** | **0%** |

### 功能完整度

| 功能 | MIRIX | AgentMem (当前) | AgentMem (目标) |
|------|-------|----------------|----------------|
| REST API | ✅ | ⚠️ 部分 | ✅ |
| WebSocket | ❌ | ❌ | ✅ |
| SSE | ❌ | ❌ | ✅ |
| OpenAPI 文档 | ⚠️ 部分 | ❌ | ✅ |
| Swagger UI | ❌ | ❌ | ✅ |

---

## 🚀 实施步骤

### Step 1: 环境准备 (10 分钟)
1. 检查现有 API 服务器代码
2. 添加必要的依赖 (utoipa, axum-ws, etc.)
3. 设置测试环境

### Step 2: 实现核心 API (2-3 小时)
1. 实现 Agent API
2. 实现 Memory API
3. 实现 Message API
4. 实现 Tool API
5. 实现 User API
6. 实现 Organization API

### Step 3: 实现 WebSocket (1-2 小时)
1. 实现连接管理
2. 实现消息推送
3. 实现心跳机制
4. 编写测试

### Step 4: 实现 SSE (1 小时)
1. 实现 SSE 端点
2. 实现流式响应
3. 编写测试

### Step 5: 文档和测试 (1-2 小时)
1. 添加 OpenAPI 注解
2. 集成 Swagger UI
3. 编写集成测试
4. 运行性能测试

### Step 6: 验证和优化 (1 小时)
1. 运行所有测试
2. 检查代码质量
3. 更新文档
4. 提交代码

---

## 📝 注意事项

### 1. 代码质量
- 遵循 Rust 最佳实践
- 确保 `cargo clippy` 无警告
- 确保 `cargo fmt` 格式正确
- 添加完整的文档注释

### 2. 测试覆盖
- 单元测试覆盖率 > 80%
- 集成测试覆盖核心流程
- 性能测试验证性能指标

### 3. 文档完整
- API 文档完整
- 代码注释清晰
- 使用示例完整

### 4. 性能要求
- API 响应时间 < 100ms
- WebSocket 延迟 < 50ms
- SSE 推送延迟 < 100ms

---

**实施者**: Augment Agent  
**创建时间**: 2025-09-30  
**预计完成时间**: 2025-10-01

