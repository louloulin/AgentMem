# 长期记忆检索问题分析报告

**问题日期**: 2025-11-03  
**问题描述**: UI Chat 功能无法检索到长期记忆（Semantic Memory）  
**严重程度**: 🔴 高 - 影响核心功能

---

## 问题现象

用户在 UI 中进行对话时，系统无法检索到之前存储的长期记忆（Semantic Memory），导致 AI 无法利用历史知识提供个性化回复。

### 用户反馈截图分析

从用户提供的截图可以看到：
- 用户询问："你先生/女士是谁？"
- AI 回复："抱歉，我无法提供关于'你'的具体信息..."
- **预期行为**: AI 应该从长期记忆中检索到相关信息并回答

---

## 根本原因分析

### 1. 数据库状态检查

```sql
-- 查询记忆类型分布
SELECT memory_type, COUNT(*) as count FROM memories GROUP BY memory_type;

结果:
Semantic | 13    ← 有长期记忆
Working  | 1
working  | 91
```

```sql
-- 查询 Semantic 记忆详情
SELECT id, memory_type, user_id, agent_id, content, importance 
FROM memories 
WHERE memory_type != 'working' 
LIMIT 10;

结果:
user_id: default-user  ← 关键发现！
agent_id: agent-a23bfd10-b1be-4848-8b4e-f3d34f4aae0e
content: "林很厉害", "蒋是林的朋友", "蒋的小孩很厉害" 等
importance: 0.8
```

### 2. UI 代码检查

**文件**: `agentmem-ui/src/app/admin/chat/page.tsx`

```typescript
// 第 150 行 - 流式请求
body: JSON.stringify({
  message: messageContent,
  user_id: 'default',  // ❌ 问题所在！
  session_id: sessionId,
  stream: true,
}),

// 第 255 行 - 普通请求
const response = await apiClient.sendChatMessage(selectedAgentId, {
  message: messageContent,
  user_id: 'default',  // ❌ 问题所在！
  session_id: sessionId,
});
```

### 3. 用户隔离机制

**文件**: `crates/agent-mem-core/src/engine.rs` (第 287-300 行)

```rust
// ✅ 计算用户匹配权重
let user_match_boost = if let Some(ref mem_user_id) = memory.user_id {
    if let Some(target_uid) = target_user_id {
        if mem_user_id == target_uid {
            2.0  // 同一用户：加倍权重
        } else {
            0.3  // 不同用户：大幅降权 ← 这里导致过滤
        }
    } else {
        1.0  // 无user_id过滤：保持原权重
    }
} else {
    1.0
};
```

### 4. 问题链条

```
UI 发送请求
  ↓
user_id: 'default'
  ↓
MemoryEngine.search_memories()
  ↓
用户匹配检查: 'default' != 'default-user'
  ↓
user_match_boost = 0.3 (大幅降权)
  ↓
final_score = relevance * time_decay * 0.3 * importance
  ↓
分数过低，被过滤掉
  ↓
返回 0 条长期记忆
```

---

## 详细技术分析

### 1. 记忆检索流程

**调用链**:
```
UI Chat (user_id='default')
  ↓
POST /api/v1/agents/{agent_id}/chat
  ↓
AgentOrchestrator.step()
  ↓
retrieve_memories() (第 662 行)
  ↓
MemoryIntegrator.retrieve_relevant_memories_with_session()
  ↓
MemoryEngine.search_memories()
  ↓
LibSQL Repository.find_by_user_id('default')
  ↓
返回 0 条记录 (因为所有 Semantic 记忆的 user_id='default-user')
```

### 2. 用户隔离权重计算

**代码位置**: `crates/agent-mem-core/src/engine.rs:287-300`

```rust
// 当前逻辑
if mem_user_id == target_uid {
    2.0  // 完全匹配：加倍
} else {
    0.3  // 不匹配：降权 70%
}

// 实际效果
'default-user' != 'default'
  ↓
user_match_boost = 0.3
  ↓
final_score = 0.5 * 1.0 * 0.3 * 0.8 = 0.12
  ↓
低于 relevance_threshold (0.1)，被过滤
```

### 3. 相关性阈值

**代码位置**: `crates/agent-mem-core/src/orchestrator/memory_integration.rs:27`

```rust
impl Default for MemoryIntegratorConfig {
    fn default() -> Self {
        Self {
            max_memories: 10,
            relevance_threshold: 0.1,  // ✅ 已降低阈值
            include_timestamp: true,
            sort_by_importance: true,
        }
    }
}
```

---

## 影响范围

### 受影响的功能
1. ✅ **Working Memory**: 正常工作（同一 session 内）
2. ❌ **Semantic Memory**: 无法检索（user_id 不匹配）
3. ❌ **跨会话记忆**: 无法检索（user_id 不匹配）
4. ❌ **个性化回复**: 无法实现（缺少长期记忆）

### 受影响的用户场景
1. 用户询问之前对话中提到的信息
2. 用户期望 AI 记住个人偏好
3. 用户期望 AI 记住关系信息（如"我的朋友是谁"）
4. 用户期望 AI 记住事实信息（如"我喜欢什么"）

---

## 解决方案

### 方案 1: 统一 user_id（推荐）✅

**修改文件**: `agentmem-ui/src/app/admin/chat/page.tsx`

```typescript
// 修改前
user_id: 'default',

// 修改后
user_id: 'default-user',
```

**优点**:
- 最小改动
- 立即生效
- 保持用户隔离机制

**缺点**:
- 需要确保所有地方使用统一的 user_id

### 方案 2: 放宽用户隔离策略

**修改文件**: `crates/agent-mem-core/src/engine.rs`

```rust
// 修改前
} else {
    0.3  // 不同用户：大幅降权
}

// 修改后
} else {
    0.8  // 不同用户：轻微降权
}
```

**优点**:
- 允许跨用户检索
- 更灵活的记忆共享

**缺点**:
- 可能影响多租户隔离
- 需要仔细评估安全性

### 方案 3: 添加 user_id 映射机制

**新增功能**: 用户别名映射

```rust
// 伪代码
let normalized_user_id = normalize_user_id(user_id);
// 'default' -> 'default-user'
// 'admin' -> 'default-user'
```

**优点**:
- 兼容性最好
- 支持多种 user_id 格式

**缺点**:
- 需要额外的映射逻辑
- 增加系统复杂度

### 方案 4: 修改记忆提取逻辑

**修改文件**: `crates/agent-mem-core/src/orchestrator/memory_extraction.rs`

确保提取记忆时使用正确的 user_id。

---

## 推荐实施步骤

### 第一步: 立即修复（5 分钟）

1. 修改 UI 代码，统一 user_id 为 `'default-user'`
2. 重启前端服务
3. 验证长期记忆检索

### 第二步: 数据迁移（可选，10 分钟）

```sql
-- 将所有 'default' 用户的记忆迁移到 'default-user'
UPDATE memories 
SET user_id = 'default-user' 
WHERE user_id = 'default';
```

### 第三步: 添加测试（30 分钟）

1. 创建长期记忆检索测试
2. 验证用户隔离机制
3. 验证跨会话记忆检索

### 第四步: 文档更新（15 分钟）

1. 更新 API 文档，说明 user_id 规范
2. 更新开发指南，说明用户隔离机制
3. 添加故障排查指南

---

## 验证方法

### 1. 创建测试记忆

```bash
# 使用 API 创建 Semantic 记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent-6812f152-16c0-4637-8fc0-714efee147f3",
    "user_id": "default-user",
    "content": "用户的名字是 Alice",
    "memory_type": "Semantic",
    "importance": 0.9
  }'
```

### 2. 测试检索

```bash
# 发送对话请求
curl -X POST http://localhost:8080/api/v1/agents/agent-6812f152-16c0-4637-8fc0-714efee147f3/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "我的名字是什么？",
    "user_id": "default-user",
    "session_id": "test-session-001",
    "stream": false
  }'
```

### 3. 验证数据库

```sql
-- 检查记忆是否被检索
SELECT * FROM memories 
WHERE user_id = 'default-user' 
  AND memory_type = 'Semantic'
ORDER BY last_accessed DESC 
LIMIT 5;
```

---

## 预期结果

修复后，系统应该能够：

1. ✅ 检索到长期记忆（Semantic Memory）
2. ✅ AI 能够回答之前对话中提到的信息
3. ✅ 提供个性化的回复
4. ✅ 保持用户隔离机制正常工作

---

## 相关代码文件

1. **UI 代码**:
   - `agentmem-ui/src/app/admin/chat/page.tsx` (第 150, 255 行)

2. **后端代码**:
   - `crates/agent-mem-core/src/engine.rs` (第 184-340 行)
   - `crates/agent-mem-core/src/orchestrator/mod.rs` (第 662-685 行)
   - `crates/agent-mem-core/src/orchestrator/memory_integration.rs` (第 67-118 行)

3. **数据库**:
   - `data/agentmem.db` - memories 表

---

## 总结

**问题根源**: UI 使用 `user_id='default'`，而长期记忆存储时使用 `user_id='default-user'`，导致用户隔离机制过滤掉所有长期记忆。

**解决方案**: 统一 user_id 为 `'default-user'`，确保 UI 和后端使用相同的用户标识。

**优先级**: 🔴 高 - 建议立即修复

**预计修复时间**: 5 分钟（代码修改） + 5 分钟（验证）

---

**报告生成时间**: 2025-11-03 21:10:00  
**分析人员**: AgentMem 技术团队  
**状态**: 待修复

