# AgentMem Working Memory 设计分析报告

## 核心问题
**Working Memory 应该复用 memories 表还是创建独立表？**

---

## 📊 现状分析

### 1. memories 表设计（长期记忆）

```sql
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    content TEXT NOT NULL,
    hash TEXT,
    metadata TEXT,
    score REAL,
    memory_type TEXT NOT NULL,      -- ✅ 支持多种类型
    scope TEXT NOT NULL,
    level TEXT NOT NULL,
    importance REAL NOT NULL,
    access_count INTEGER DEFAULT 0,
    last_accessed INTEGER,
    embedding TEXT,                  -- ✅ 向量嵌入
    expires_at INTEGER,              -- ✅ 支持过期
    version INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    is_deleted INTEGER DEFAULT 0
);
```

**特点**:
- ✅ 已有 `memory_type` 字段（支持 Working, Episodic, Semantic 等）
- ✅ 已有 `expires_at` 字段（支持临时记忆）
- ✅ 支持向量搜索（embedding）
- ✅ 统一的记忆管理
- ❌ **没有 session_id 字段**（这是关键！）

### 2. LibSqlWorkingStore 实现（假设独立表）

```rust
// 当前实现假设有独立的 working_memory 表
SELECT * FROM working_memory
WHERE session_id = ?  // ❌ memories 表没有这个字段！
AND (expires_at IS NULL OR expires_at > datetime('now'))
ORDER BY priority DESC, created_at ASC
```

**需要的字段**:
- ✅ id
- ✅ user_id, agent_id
- ✅ content
- ❌ **session_id** (memories 表没有！)
- ✅ priority (可以用 importance 代替)
- ✅ expires_at
- ✅ metadata
- ✅ created_at

---

## 🤔 方案对比

### 方案A: 复用 memories 表 + 添加 session_id

**实施方式**: 给 memories 表添加 `session_id` 字段

```sql
-- Migration 13
ALTER TABLE memories ADD COLUMN session_id TEXT;
CREATE INDEX idx_memories_session_id ON memories(session_id);
```

**优点**:
- ✅ **统一的记忆模型**（所有记忆类型在一张表）
- ✅ 可以查询跨类型记忆关联
- ✅ 统一的向量搜索和全文搜索
- ✅ 复用现有的索引和优化
- ✅ 符合 AgentMem 的**统一记忆架构**

**缺点**:
- ⚠️  修改现有表结构（但 SQLite 支持 ALTER TABLE）
- ⚠️  Working Memory 查询会扫描更大的表
- ⚠️  需要修改 WorkingMemoryStore 的 SQL

**改动**:
- migrations.rs: +10行（添加字段和索引）
- libsql_working.rs: ~30行（修改 SQL 从 memories 表查询）

---

### 方案B: 创建独立 working_memory 表

**实施方式**: 新建专门的 working_memory 表

```sql
-- Migration 13
CREATE TABLE working_memory (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    session_id TEXT NOT NULL,  -- ✅ 核心字段
    content TEXT NOT NULL,
    priority INTEGER DEFAULT 1,
    expires_at TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL
);
```

**优点**:
- ✅ **查询性能好**（表小，专用索引）
- ✅ 不影响现有 memories 表
- ✅ LibSqlWorkingStore 代码不需要改
- ✅ 清晰的临时/长期分离

**缺点**:
- ❌ **破坏统一记忆模型**（记忆分散在两张表）
- ❌ 无法跨类型查询记忆关联
- ❌ 不支持向量搜索（需要额外实现）
- ❌ 重复很多字段和逻辑

**改动**:
- migrations.rs: +40行（新建表+索引）
- libsql_working.rs: 0行（已经实现）

---

## 🎯 AgentMem 设计哲学分析

### 核心设计原则

1. **统一记忆模型** (Unified Memory Model)
   ```
   所有记忆类型（Episodic, Semantic, Procedural, Working, Core 等）
   都存储在同一个 memories 表中，通过 memory_type 字段区分
   ```

2. **trait-based 存储抽象**
   ```rust
   // 每种记忆类型有独立的 trait
   trait EpisodicMemoryStore { ... }
   trait WorkingMemoryStore { ... }
   
   // 但可以用同一个底层表实现
   ```

3. **灵活的过期机制**
   ```
   通过 expires_at 字段支持临时记忆
   不需要单独的表来存储临时数据
   ```

### 现有架构证据

```rust
// memories 表设计已经考虑了多种记忆类型
memory_type TEXT NOT NULL,  // Working, Episodic, Semantic, ...
expires_at INTEGER,         // 支持临时记忆
importance REAL,            // 优先级/重要性
```

**结论**: AgentMem 的设计**本意是统一表**，只是**缺少 session_id 字段**！

---

## 💡 推荐方案

### ✅ 方案A（统一模型）+ 最小改动

**理由**:
1. 符合 AgentMem 的**统一记忆架构**设计
2. 最小改动：只需添加一个字段
3. 保持架构一致性
4. 未来可扩展（session 可用于其他记忆类型）

**实施步骤**:

#### Step 1: 给 memories 表添加 session_id（Migration 13）
```sql
ALTER TABLE memories ADD COLUMN session_id TEXT;
CREATE INDEX idx_memories_session_id ON memories(session_id);
```

#### Step 2: 修改 LibSqlWorkingStore 使用 memories 表
```rust
// 修改查询 SQL
SELECT * FROM memories
WHERE session_id = ?
AND memory_type = 'working'
AND (expires_at IS NULL OR expires_at > datetime('now'))
ORDER BY importance DESC, created_at ASC
```

#### Step 3: 插入时设置 memory_type = 'working'
```rust
INSERT INTO memories (
    id, organization_id, user_id, agent_id, 
    content, memory_type, session_id, importance, expires_at, ...
)
VALUES (?, ?, ?, ?, ?, 'working', ?, ?, ?, ...)
```

**代码改动量**:
- migrations.rs: +15行
- libsql_working.rs: +30行（修改 SQL）
- **总计**: ~45行

---

## 🔍 对比分析

| 维度 | 方案A（统一表+session_id） | 方案B（独立表） |
|------|--------------------------|----------------|
| **架构一致性** | ✅ 完美（符合设计哲学） | ❌ 破坏统一模型 |
| **代码改动** | ~45行 | ~40行 |
| **查询性能** | ⚠️  稍慢（表更大） | ✅ 快（表小） |
| **功能完整性** | ✅ 支持向量搜索 | ❌ 需额外实现 |
| **可维护性** | ✅ 统一管理 | ⚠️  两套逻辑 |
| **扩展性** | ✅ session 可用于其他类型 | ❌ 仅限 working |
| **风险** | ⚠️  修改现有表 | ✅ 新表无风险 |

---

## 🎯 最终建议

### ✅ 推荐：方案A（统一模型）

**核心理由**:
1. **符合 AgentMem 设计哲学**：统一记忆模型
2. **最小架构变动**：只加一个字段
3. **保持一致性**：所有记忆类型统一管理
4. **未来可扩展**：session_id 可用于其他记忆类型

**实施风险评估**:
- ✅ SQLite 支持 ALTER TABLE（安全）
- ✅ 添加字段不影响现有数据
- ✅ 索引创建是幂等的
- ⚠️  需要修改 WorkingMemoryStore 实现（但改动小）

**性能优化**:
- ✅ 添加 `session_id` 索引
- ✅ 组合索引 `(session_id, memory_type)`
- ✅ 定期清理过期数据

---

## 📝 实施清单

### Phase 1: 数据库迁移
- [ ] Migration 13: 添加 session_id 字段
- [ ] 创建 session_id 索引
- [ ] （可选）创建组合索引 (session_id, memory_type)

### Phase 2: 修改 LibSqlWorkingStore
- [ ] 修改 add_item SQL（插入到 memories 表）
- [ ] 修改 get_session_items SQL（从 memories 表查询）
- [ ] 修改 remove_item SQL（从 memories 表删除）
- [ ] 修改 clear_expired SQL（清理 memories 表）

### Phase 3: 测试验证
- [ ] 测试 Working Memory 读写
- [ ] 测试 session 隔离
- [ ] 测试过期清理
- [ ] 测试与其他记忆类型的共存

---

## 🚀 后续优化

1. **性能监控**
   - 监控 session 查询性能
   - 根据实际情况调整索引

2. **定期清理**
   - 自动清理过期的 Working Memory
   - 可选：清理旧 session 数据

3. **跨类型功能**
   - 利用统一表实现跨记忆类型的关联查询
   - 实现 Working → Long-term 的升级机制

---

**结论**: **方案A** 是最佳选择，符合 AgentMem 的统一记忆架构，最小改动，最大收益。

**报告版本**: v1.0  
**分析日期**: 2025-11-02  
**分析人员**: AI Assistant  
**推荐方案**: ✅ 方案A（统一模型 + session_id）

