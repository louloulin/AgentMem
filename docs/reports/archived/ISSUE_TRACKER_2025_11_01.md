# AgentMem Issues Tracker - 2025-11-01

**Generated**: 2025-11-01  
**Last Updated**: 2025-11-01  
**Total Issues**: 13  

---

## 🔴 Critical Issues (P0) - 需要立即修复

### Issue #1: 记忆列表API缺失
- **ID**: AGM-001
- **Priority**: P0 🔴
- **Category**: API缺失
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 2小时

**Description**:
GET `/api/v1/memories` 端点返回405 Method Not Allowed，导致前端"记忆管理"页面无法加载记忆列表。

**Technical Details**:
```rust
// 当前配置 (mod.rs:66)
.route("/api/v1/memories", post(memory::add_memory))

// 缺少:
.route("/api/v1/memories", get(memory::list_memories))
```

**Impact**:
- ❌ 前端无法列表查询记忆
- ❌ Dashboard记忆统计不完整
- ❌ 用户体验严重受损

**Solution**:
1. 在 `memory.rs` 创建 `list_memories` 路由处理器
2. 支持分页 (offset, limit)
3. 支持过滤 (agent_id, user_id, type)
4. 支持排序 (created_at, importance)
5. 更新 `mod.rs` 路由配置

**Acceptance Criteria**:
- [ ] `GET /api/v1/memories` 返回200
- [ ] 支持 `?limit=10&offset=0` 分页
- [ ] 支持 `?agent_id=xxx` 过滤
- [ ] 响应时间 < 100ms
- [ ] 测试覆盖率 > 80%

**Related**:
- agentmem41.md - Task 1.1.1
- crates/agent-mem-server/src/routes/memory.rs:280

---

### Issue #2: 聊天会话管理API完全缺失
- **ID**: AGM-002
- **Priority**: P0 🔴
- **Category**: API缺失
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 17小时

**Description**:
聊天功能完全不可用，所有会话管理API都缺失，前端"聊天界面"是空壳。

**Missing Endpoints**:
- `GET /api/v1/chat/sessions` - 会话列表
- `POST /api/v1/chat/sessions` - 创建会话
- `GET /api/v1/chat/sessions/:id` - 会话详情
- `DELETE /api/v1/chat/sessions/:id` - 删除会话
- `GET /api/v1/chat/sessions/:id/messages` - 消息列表
- `POST /api/v1/chat/messages` - 发送消息

**Current Situation**:
后端只有基于Agent的聊天API:
```rust
POST /api/v1/agents/:agent_id/chat
POST /api/v1/agents/:agent_id/chat/stream
GET  /api/v1/agents/:agent_id/chat/history
```

这些API需要提前知道agent_id，不适合通用聊天场景。

**Impact**:
- ❌ 前端聊天功能完全不可用
- ❌ 无法创建和管理对话
- ❌ 无法查看历史对话
- ❌ 核心功能缺失

**Solution**:
1. 设计 `ChatSession` 和 `ChatMessage` 数据模型
2. 创建数据库migration
3. 实现会话CRUD操作
4. 实现消息管理
5. 集成Agent响应
6. 支持多轮对话上下文

**Database Schema** (建议):
```sql
CREATE TABLE chat_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    agent_id TEXT,
    title TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    metadata TEXT
);

CREATE TABLE chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,  -- 'user' | 'assistant' | 'system'
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    metadata TEXT,
    FOREIGN KEY (session_id) REFERENCES chat_sessions(id)
);
```

**Acceptance Criteria**:
- [ ] 所有6个端点都实现并返回正确状态码
- [ ] 支持创建、查询、删除会话
- [ ] 支持发送消息并获取AI响应
- [ ] 支持多轮对话上下文
- [ ] 前端聊天界面正常工作
- [ ] E2E测试通过

**Related**:
- agentmem41.md - Task 1.2.1, 1.2.2, 1.2.3
- agentmem-ui/src/app/admin/chat/page.tsx

---

### Issue #3: 历史记录功能数据库连接失败
- **ID**: AGM-003
- **Priority**: P0 🔴
- **Category**: 数据库
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 2小时

**Description**:
服务器启动时报错：
```
WARN 创建 HistoryManager 失败: Storage error: 连接数据库失败: 
(code: 14) unable to open database file, 历史记录功能将不可用
```

**Impact**:
- ❌ 无法查看记忆的历史版本
- ❌ 无法追踪记忆变更
- ❌ 审计功能受影响

**Root Cause Analysis**:
可能原因：
1. 数据库文件路径配置错误
2. 目录权限不足
3. 数据库文件不存在且无法自动创建
4. DATABASE_URL环境变量配置错误

**Solution**:
1. 检查 `DATABASE_URL` 环境变量
2. 确保数据目录存在且有写权限
3. 添加自动创建数据库目录的逻辑
4. 添加优雅降级：历史功能不可用时继续运行
5. 改进错误提示

**Suggested Fix**:
```rust
// 启动时检查并创建目录
let db_path = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "file:./data/agentmem.db".to_string());

if let Some(dir) = std::path::Path::new(&db_path).parent() {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("Failed to create database directory: {}", e))?;
}
```

**Acceptance Criteria**:
- [ ] 服务器启动无WARN日志
- [ ] 历史记录功能正常工作
- [ ] 数据目录自动创建
- [ ] 优雅降级处理

**Related**:
- agentmem41.md - Task 1.1.3
- start_server_with_correct_onnx.sh
- crates/agent-mem-server/src/routes/memory.rs

---

## 🟠 High Priority Issues (P1) - 本周内修复

### Issue #4: 用户创建API路径不一致
- **ID**: AGM-004
- **Priority**: P1 🟠
- **Category**: API不一致
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 1小时

**Description**:
前端调用 `POST /api/v1/users`，但后端路由是 `POST /api/v1/users/register`，导致405错误。

**Technical Details**:
```typescript
// 前端调用 (api-client.ts)
const response = await fetch('/api/v1/users', {
    method: 'POST',
    body: JSON.stringify(userData)
});
```

```rust
// 后端路由 (mod.rs:96)
.route("/api/v1/users/register", post(users::register_user))
```

**Impact**:
- ⚠️ 新用户无法注册
- ⚠️ 用户管理功能受限

**Solution** (2个方案):

**方案A** (推荐): 添加路由别名
```rust
.route("/api/v1/users", post(users::register_user))
.route("/api/v1/users/register", post(users::register_user))
```

**方案B**: 修改前端
```typescript
const response = await fetch('/api/v1/users/register', {
    method: 'POST',
    body: JSON.stringify(userData)
});
```

**Acceptance Criteria**:
- [ ] `POST /api/v1/users` 返回201
- [ ] 可以成功创建用户
- [ ] 前端用户管理页面正常

**Related**:
- agentmem41.md - Task 1.1.2
- crates/agent-mem-server/src/routes/mod.rs:96

---

### Issue #5: 图谱可视化API未实现 (LibSQL)
- **ID**: AGM-005
- **Priority**: P1 🟠
- **Category**: Feature缺失
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 34小时

**Description**:
图谱可视化功能被条件编译为PostgreSQL专用，当前使用LibSQL后端时不可用。

**Technical Details**:
```rust
// mod.rs:180
#[cfg(feature = "postgres")]  // ❌ 条件编译
pub mod graph;
```

**Impact**:
- ❌ 前端图谱页面无数据
- ❌ 无法展示记忆关联关系
- ❌ 知识图谱功能完全不可用

**Solution Options**:

**方案A: LibSQL图谱实现** (推荐)
1. 设计图谱数据模型 (nodes, edges)
2. 创建LibSQL schema
3. 实现图查询算法
4. 移除条件编译限制
5. 实现API端点

**方案B: 启用PostgreSQL**
1. 添加PostgreSQL依赖
2. 数据迁移
3. 启用 `postgres` feature

**Database Schema** (LibSQL版本):
```sql
CREATE TABLE graph_nodes (
    id TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,  -- 'memory' | 'agent' | 'user'
    entity_id TEXT NOT NULL,
    metadata TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE graph_edges (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type TEXT NOT NULL,  -- 'related_to' | 'created_by' | 'used_by'
    weight REAL DEFAULT 1.0,
    metadata TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (source_id) REFERENCES graph_nodes(id),
    FOREIGN KEY (target_id) REFERENCES graph_nodes(id)
);
```

**Acceptance Criteria**:
- [ ] GET `/api/v1/graph/memories` 返回图谱数据
- [ ] 支持节点和边的查询
- [ ] 前端可视化正常展示
- [ ] 性能 < 200ms (1000节点)

**Related**:
- agentmem41.md - Phase 2
- crates/agent-mem-server/src/routes/graph.rs

---

## 🟡 Medium Priority Issues (P2) - 下一迭代

### Issue #6: WebSocket连接需要验证
- **ID**: AGM-006
- **Priority**: P2 🟡
- **Category**: 实时通信
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 6小时

**Description**:
WebSocket端点存在但握手行为未验证，实时更新功能可能异常。

**Current Status**:
- ✅ 端点存在: `/api/v1/ws`
- ⚠️ 握手未验证
- ⚠️ 消息传递未测试
- ⚠️ 重连机制未测试

**Testing Plan**:
1. 使用浏览器DevTools验证连接
2. 编写E2E测试
3. 测试消息传递
4. 测试重连机制
5. 性能压测

**Acceptance Criteria**:
- [ ] E2E测试通过
- [ ] 连接稳定性 > 99%
- [ ] 消息延迟 < 100ms
- [ ] 支持1000+并发连接

**Related**:
- agentmem41.md - Task 3.1.1, 3.1.2
- crates/agent-mem-server/src/websocket.rs

---

### Issue #7: SSE流式响应需要验证
- **ID**: AGM-007
- **Priority**: P2 🟡
- **Category**: 实时通信
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 4小时

**Description**:
SSE端点存在但流式响应行为未验证，可能影响流式对话体验。

**Current Status**:
- ✅ 端点存在: `/api/v1/chat/stream`
- ⚠️ 流式响应未测试
- ⚠️ 断线重连未验证
- ⚠️ 性能未评估

**Testing Plan**:
1. 测试流式响应
2. 测试断线重连
3. 测试取消功能
4. 性能压测

**Acceptance Criteria**:
- [ ] 流式响应正常
- [ ] 支持中途取消
- [ ] 断线自动重连
- [ ] 延迟 < 50ms

**Related**:
- agentmem41.md - Task 3.2.1, 3.2.2
- crates/agent-mem-server/src/sse.rs

---

### Issue #8: API文档与实际路由不一致
- **ID**: AGM-008
- **Priority**: P2 🟡
- **Category**: 文档
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 2小时

**Description**:
Swagger文档中的端点信息与实际路由配置不一致，影响开发体验。

**Problems**:
- 缺失的端点仍在文档中
- 新增端点未更新到文档
- 请求/响应示例不准确

**Solution**:
1. 使用 utoipa 自动生成文档
2. 添加CI检查确保同步
3. 提供Postman collection

**Acceptance Criteria**:
- [ ] 文档与实际路由100%一致
- [ ] 所有端点有示例
- [ ] 提供Postman collection

**Related**:
- http://localhost:8080/swagger-ui/
- crates/agent-mem-server/src/routes/mod.rs:224

---

### Issue #9: 认证授权未实现
- **ID**: AGM-009
- **Priority**: P1 🟠 (生产环境)
- **Category**: 安全
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 26小时

**Description**:
当前使用mock认证，所有请求都使用默认用户，存在严重安全风险。

**Current Code**:
```rust
// middleware.rs
pub async fn default_auth_middleware(...) {
    let auth_user = AuthUser::default();  // ❌ 不安全
    // ...
}
```

**Impact**:
- 🔓 任何人都可以访问所有数据
- 🔓 无法区分用户权限
- 🔓 生产环境不可用

**Solution**:
1. 实现JWT认证
2. 添加用户权限系统 (RBAC)
3. 实现细粒度访问控制
4. 添加速率限制
5. 添加API Key支持

**Acceptance Criteria**:
- [ ] JWT认证正常工作
- [ ] 权限控制生效
- [ ] 支持角色管理
- [ ] 通过安全审计

**Related**:
- agentmem41.md - Phase 4
- crates/agent-mem-server/src/middleware.rs

---

## 🔵 Low Priority Issues (P3) - 优化项

### Issue #10: 查询优化器未集成到API
- **ID**: AGM-010
- **Priority**: P3 🔵
- **Category**: 性能
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 4小时

**Description**:
Phase 3-D实现的QueryOptimizer和ResultReranker未集成到实际API中。

**Impact**:
- ⚠️ 大数据集性能差
- ⚠️ 搜索精度不理想
- ⚠️ 优化功能未使用

**Solution**:
在 `search_memories` 中集成优化器：
```rust
pub async fn search_memories(...) {
    // 1. 使用QueryOptimizer选择策略
    let plan = optimizer.optimize_query(&query)?;
    
    // 2. 执行搜索
    let results = execute_search(&plan).await?;
    
    // 3. 使用ResultReranker重排序
    if plan.should_rerank {
        results = reranker.rerank(results, &query_vector, &query).await?;
    }
    
    Ok(results)
}
```

**Acceptance Criteria**:
- [ ] 集成QueryOptimizer
- [ ] 集成ResultReranker
- [ ] 性能提升 > 30%
- [ ] 搜索精度提升

**Related**:
- agentmem40.md - Phase 3-D
- agentmem41.md - Task 5.1.1, 5.1.2
- crates/agent-mem-core/src/search/query_optimizer.rs
- crates/agent-mem-core/src/search/reranker.rs

---

### Issue #11: 缺少E2E测试
- **ID**: AGM-011
- **Priority**: P2 🟡
- **Category**: 测试
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 12小时

**Description**:
缺少端到端测试，回归风险高。

**Testing Gaps**:
- ❌ 无UI自动化测试
- ❌ 无API集成测试套件
- ❌ 无性能基准测试
- ❌ 无并发测试

**Solution**:
1. 使用Playwright编写UI测试
2. 使用cargo test编写API测试
3. 使用criterion编写性能测试
4. 集成到CI/CD

**Acceptance Criteria**:
- [ ] E2E测试覆盖率 > 80%
- [ ] 所有关键路径有测试
- [ ] CI自动运行测试

**Related**:
- agentmem41.md - 测试策略

---

### Issue #12: 缺少实时监控面板
- **ID**: AGM-012
- **Priority**: P3 🔵
- **Category**: 运维
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 20小时

**Description**:
缺少实时监控和告警系统，运维困难。

**Missing Features**:
- ❌ 系统指标监控 (CPU, 内存)
- ❌ 业务指标监控 (QPS, 延迟)
- ❌ 错误监控
- ❌ 告警系统

**Solution**:
1. 实现指标收集API
2. 集成Prometheus
3. 创建Grafana Dashboard
4. 添加告警规则

**Related**:
- agentmem41.md - Task 5.2.1, 5.2.2
- crates/agent-mem-observability/

---

### Issue #13: 前端错误处理不完善
- **ID**: AGM-013
- **Priority**: P3 🔵
- **Category**: 用户体验
- **Status**: 🟡 Open
- **Assigned**: 待分配
- **Estimated**: 8小时

**Description**:
前端错误处理不友好，用户体验差。

**Problems**:
- 错误提示不清晰
- 无loading状态
- 无重试机制
- 无离线提示

**Solution**:
1. 统一错误处理
2. 添加loading组件
3. 实现自动重试
4. 添加离线检测

**Related**:
- agentmem-ui/src/lib/api-client.ts

---

## 📊 Issues Summary

### By Priority
- 🔴 P0 (Critical): 3 issues
- 🟠 P1 (High): 3 issues
- 🟡 P2 (Medium): 3 issues
- 🔵 P3 (Low): 4 issues

### By Category
- API缺失: 3
- 数据库: 1
- 安全: 1
- 性能: 1
- 测试: 1
- 文档: 1
- 实时通信: 2
- 用户体验: 1
- 运维: 1
- Feature缺失: 1

### By Status
- 🟡 Open: 13
- 🟢 In Progress: 0
- ✅ Resolved: 0
- ❌ Closed: 0

### Total Estimated Hours
- **P0**: 21小时
- **P1**: 63小时
- **P2**: 24小时
- **P3**: 32小时
- **Total**: **140小时** (~17.5个工作日)

---

## 🎯 Sprint Planning

### Sprint 1 (Week 1): Core Fixes
**Focus**: AGM-001, AGM-002, AGM-003
**Goal**: 让所有UI页面可用
**Estimated**: 21小时

### Sprint 2 (Week 2): Graph & User Management
**Focus**: AGM-004, AGM-005
**Goal**: 图谱可视化和用户管理完善
**Estimated**: 35小时

### Sprint 3 (Week 3): Real-time Communication
**Focus**: AGM-006, AGM-007, AGM-008
**Goal**: 验证实时功能
**Estimated**: 12小时

### Sprint 4 (Week 4): Security
**Focus**: AGM-009
**Goal**: 认证授权系统
**Estimated**: 26小时

### Sprint 5 (Week 5): Optimization & Testing
**Focus**: AGM-010, AGM-011, AGM-012, AGM-013
**Goal**: 性能优化和完善测试
**Estimated**: 44小时

---

## 📞 Communication

### Daily Standup
- 每日同步进度
- 讨论阻塞问题
- 调整优先级

### Weekly Review
- 每周五review完成情况
- 演示新功能
- 规划下周任务

### Issue Updates
- 每个issue更新时通知相关人员
- 重要决策记录在issue中
- 代码review后更新状态

---

**Tracker Owner**: AI Assistant  
**Last Sync**: 2025-11-01  
**Next Review**: 2025-11-07

