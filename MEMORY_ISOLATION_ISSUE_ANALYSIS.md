# 记忆隔离问题分析报告

**日期**: 2025-11-07  
**问题**: 记忆有时候隔离，有时候不隔离  
**根本原因**: Scope推断逻辑与搜索过滤不一致

---

## 🔍 问题现象

### 用户反馈
1. **有时候**：不同用户的记忆互相看不到（隔离正常）
2. **有时候**：不同用户能看到彼此的记忆（隔离失败）
3. **Claude Code MCP**: 搜索"记忆林"没有找到任何结果

---

## 📊 数据库现状分析

### 当前记忆分布

```
总记忆数: 28条

按Scope统计:
- user:    15条 (53.6%)
- session:  8条 (28.6%)
- global:   3条 (10.7%)
- agent:    2条 (7.1%)

按Agent ID统计:
- default-agent-system:  11条
- agent-24139ba9...:      5条
- agent-261284b8...:      5条
- default-agent-default:  4条
- default-agent-张三:      1条
- default-agent:          1条
- agent-8d369c8b...:      1条

按User ID统计:
- (空): 28条  ⚠️ 所有记忆的metadata中user_id都是空的！
```

### ⚠️ 关键问题

**所有记忆的metadata中user_id字段都是空的！**

这导致：
1. Scope推断不准确
2. 搜索过滤时无法正确识别用户
3. 记忆隔离逻辑失效

---

## 🔧 Scope推断逻辑分析

### 当前实现 (`memory.rs:166-185`)

```rust
let scope_type = full_metadata
    .get("scope_type")
    .cloned()
    .unwrap_or_else(|| {
        // 自动推断scope类型
        if full_metadata.contains_key("run_id") {
            "run".to_string()
        } else if full_metadata.contains_key("session_id") {
            "session".to_string()                           // ✅ 会匹配
        } else if full_metadata.contains_key("org_id") {
            "organization".to_string()
        } else if user_id_val != "default" && effective_agent_id != "default" {
            "agent".to_string()
        } else if user_id_val != "default" {
            "user".to_string()
        } else {
            "global".to_string()                             // ⚠️ 默认
        }
    });
```

### 问题分析

#### 情况1: 有session_id时
```
metadata包含 session_id
  ↓
scope = "session"
  ↓
记忆被标记为session作用域
  ↓
只有相同session_id才能访问 ✅ 隔离正常
```

#### 情况2: 没有session_id，但有user_id
```
user_id = "test-user"
effective_agent_id = "default-agent-test-user"
  ↓
user_id_val != "default" && effective_agent_id != "default"
  ↓
scope = "agent"  ⚠️ 应该是"user"
```

#### 情况3: 都是默认值
```
user_id = None → user_id_val = "default"
effective_agent_id = "default-agent"
  ↓
scope = "global"  ⚠️ 全局可访问！
  ↓
所有人都能看到 ❌ 没有隔离
```

---

## 🔍 搜索过滤逻辑分析

### API搜索端点

**路径**: `GET /api/v1/memories/search`

**预期行为**:
```rust
// 根据scope过滤
WHERE scope = ? AND user_id = ? AND agent_id = ?
```

**实际问题**:
- metadata中的user_id字段为空
- 无法正确匹配user_id
- 导致搜索结果不一致

---

## 🎯 根本原因

### 原因1: metadata中user_id未正确存储

**当前代码** (`memory.rs:159-164`):
```rust
let mut full_metadata = metadata.unwrap_or_default();
full_metadata.insert("agent_id".to_string(), effective_agent_id.clone());
full_metadata.insert("user_id".to_string(), user_id_val.clone());  // ⚠️ 存储了
full_metadata.insert("data".to_string(), content.clone());
full_metadata.insert("hash".to_string(), content_hash.clone());
```

**问题**: 虽然代码里插入了user_id，但为什么数据库里是空的？

**可能原因**:
1. metadata字段的序列化问题
2. 数据库存储时metadata被覆盖
3. 旧数据遗留问题

---

### 原因2: Scope推断逻辑的优先级问题

**当前优先级**:
```
1. run_id        → "run"
2. session_id    → "session"      ⚠️ 优先级太高！
3. org_id        → "organization"
4. user+agent    → "agent"
5. user          → "user"
6. 默认           → "global"
```

**问题**:
- session_id优先级太高
- 即使提供了user_id，只要有session_id就会被标记为session scope
- 导致跨session无法访问

**正确优先级** (参考agentmem61.md):
```
1. 明确指定的scope_type
2. user_id + agent_id  → "agent" (长期记忆)
3. user_id            → "user"  (用户记忆)
4. session_id         → "session" (工作记忆)
5. 默认               → "global"
```

---

### 原因3: 搜索时的过滤条件不一致

**MCP搜索**:
```typescript
// MCP工具可能传递的参数
{
  query: "记忆林",
  // user_id可能缺失
  // agent_id可能缺失
}
```

**API搜索**:
```typescript
// UI调用可能传递的参数
{
  query: "记忆林",
  user_id: "test-user",
  agent_id: "agent-xxx"
}
```

**结果**: 不同的调用方式传递不同的参数，导致搜索结果不同

---

## 🛠️ 解决方案

### 方案1: 修复Scope推断优先级 ⭐ 推荐

**修改**: `crates/agent-mem-server/src/routes/memory.rs:166-185`

```rust
let scope_type = full_metadata
    .get("scope_type")
    .cloned()
    .unwrap_or_else(|| {
        // ✅ 正确的优先级（符合agentmem61.md设计）
        
        // 1. 如果有user_id和agent_id（非默认），这是长期记忆
        if user_id_val != "default" && effective_agent_id.starts_with("agent-") {
            "agent".to_string()
        } 
        // 2. 如果只有user_id（非默认），这是用户记忆
        else if user_id_val != "default" {
            "user".to_string()
        }
        // 3. 如果有session_id，这是工作记忆（临时）
        else if full_metadata.contains_key("session_id") {
            "session".to_string()
        }
        // 4. 如果有run_id
        else if full_metadata.contains_key("run_id") {
            "run".to_string()
        }
        // 5. 如果有org_id
        else if full_metadata.contains_key("org_id") {
            "organization".to_string()
        }
        // 6. 默认为全局
        else {
            "global".to_string()
        }
    });
```

**关键变化**:
- ✅ user_id优先于session_id
- ✅ 符合"Episodic-first"设计
- ✅ Session只是Working Memory

---

### 方案2: 修复metadata存储

**问题定位**: 需要检查为什么metadata中的user_id是空的

**检查点**:
1. Memory结构体的metadata字段序列化
2. LibSQL存储时的metadata处理
3. 是否有字段覆盖

**临时解决**: 清理旧数据，重新添加记忆

```bash
# 清理测试数据
rm -f data/agentmem.db
./start_server_no_auth.sh

# 重新添加记忆（确保user_id正确）
curl -X POST http://localhost:8080/api/v1/memories \
  -d '{"content": "蒋是CEO", "user_id": "test-user"}'
```

---

### 方案3: 统一搜索过滤逻辑

**目标**: MCP和API使用相同的搜索逻辑

**MCP工具改进**:
```rust
// agentmem_search_memories 工具
// 应该始终传递 user_id 和 agent_id
{
  "query": query,
  "user_id": user_id || "default",  // ✅ 添加默认值
  "agent_id": agent_id || null,     // ✅ 可选
  "limit": limit || 10
}
```

---

## 📊 测试验证

### 测试场景1: 不同用户隔离

```bash
# 用户A添加记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -d '{"content": "蒋是CEO", "user_id": "user-a"}'

# 用户B搜索
curl "http://localhost:8080/api/v1/memories/search?query=蒋&user_id=user-b"

# 预期: 0条结果 ✅ 隔离成功
```

### 测试场景2: 同用户跨session

```bash
# Session A添加记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -d '{"content": "蒋是CEO", "user_id": "user-a", "metadata": {"session_id": "session-a"}}'

# Session B搜索（同用户）
curl "http://localhost:8080/api/v1/memories/search?query=蒋&user_id=user-a"

# 预期: 1条结果 ✅ 跨session访问
```

### 测试场景3: MCP搜索

```bash
# 通过Claude Code MCP
claude mcp call agentmem agentmem_search_memories '{
  "query": "蒋",
  "user_id": "test-user"
}'

# 预期: 返回相关记忆
```

---

## 🎯 实施步骤

### Phase 1: 立即修复（30分钟）

1. ✅ **修改Scope推断优先级**
   - 文件: `memory.rs`
   - 改动: ~20行
   - 风险: 低

2. ✅ **清理测试数据**
   ```bash
   ./scripts/cleanup_and_restart.sh
   ```

3. ✅ **重新添加测试记忆**
   ```bash
   ./scripts/add_test_memories.sh
   ```

### Phase 2: 验证修复（30分钟）

1. ✅ API测试
2. ✅ MCP测试
3. ✅ UI测试

### Phase 3: 文档更新（15分钟）

1. ✅ 更新agentmem61.md
2. ✅ 更新API文档
3. ✅ 更新MCP文档

---

## 📈 预期效果

### 修复前

```
场景1: user_id="user-a", session_id="xxx"
  → scope="session"  ❌ 只能当前session访问

场景2: user_id=None, agent_id="default-agent"
  → scope="global"   ❌ 所有人都能访问

场景3: MCP搜索（没有user_id）
  → 找不到记忆     ❌ 参数缺失
```

### 修复后

```
场景1: user_id="user-a", session_id="xxx"
  → scope="user"    ✅ 用户级别，跨session

场景2: user_id=None, agent_id="default-agent"
  → scope="session" or "global"  ✅ 明确隔离

场景3: MCP搜索（提供user_id）
  → 找到记忆       ✅ 正确过滤
```

---

## 💡 长期优化建议

### 1. 显式Scope参数

允许用户明确指定scope：

```rust
pub struct MemoryRequest {
    pub content: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub scope: Option<String>,  // ✨ 新增：显式指定scope
    // ...
}
```

### 2. Scope验证

添加scope验证逻辑：

```rust
fn validate_scope(scope: &str, user_id: &Option<String>) -> Result<()> {
    match scope {
        "user" | "agent" => {
            if user_id.is_none() {
                return Err("user scope requires user_id");
            }
        },
        _ => {}
    }
    Ok(())
}
```

### 3. 迁移旧数据

为旧数据补充正确的scope：

```sql
-- 修复scope为global但实际应该是user的记忆
UPDATE memories 
SET scope = 'user' 
WHERE scope = 'global' 
  AND agent_id LIKE 'default-agent-%'
  AND agent_id != 'default-agent';
```

---

## ✅ 成功标准

- [ ] ⏳ 不同用户的记忆完全隔离
- [ ] ⏳ 同用户可以跨session访问记忆
- [ ] ⏳ MCP搜索能正确找到记忆
- [ ] ⏳ Scope推断逻辑符合设计文档
- [ ] ⏳ 测试用例全部通过

---

**状态**: 📝 分析完成，待实施修复  
**优先级**: 🔴 P0 - 影响核心功能  
**预计时间**: 1.5小时

