# Memory History 完整实现文档

**实现日期**: 2025-10-15  
**状态**: ✅ 数据库层完成，API 层简化实现  
**版本**: 1.0

---

## 📋 概述

Memory History 功能用于追踪记忆的所有变更历史，支持版本控制、审计追踪和版本回溯。

### 实现状态

| 组件 | 状态 | 说明 |
|------|------|------|
| 数据库表 | ✅ 完成 | `memory_history` 表已创建 |
| 数据库触发器 | ✅ 完成 | 自动追踪 INSERT/UPDATE/DELETE |
| 数据库索引 | ✅ 完成 | 优化查询性能 |
| API 端点 | ✅ 简化版 | 返回当前版本作为历史 |
| Repository Trait | ⏳ 待实现 | 需要扩展 trait 添加历史方法 |

---

## 🗄️ 数据库实现

### 1. memory_history 表结构

```sql
CREATE TABLE memory_history (
    id VARCHAR(255) PRIMARY KEY,
    memory_id VARCHAR(255) NOT NULL,
    version INTEGER NOT NULL,
    change_type VARCHAR(50) NOT NULL CHECK (change_type IN ('created', 'updated', 'deleted', 'restored')),
    change_reason TEXT,
    content TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    memory_type VARCHAR(50) NOT NULL,
    importance REAL NOT NULL DEFAULT 0.0,
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    changed_by_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 外键约束
    CONSTRAINT fk_memory_history_memory FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE,
    CONSTRAINT fk_memory_history_organization FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT fk_memory_history_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_memory_history_agent FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);
```

### 2. 索引

```sql
-- 按 memory_id 查询
CREATE INDEX idx_memory_history_memory_id ON memory_history(memory_id);

-- 按 memory_id 和 version 查询（降序）
CREATE INDEX idx_memory_history_memory_version ON memory_history(memory_id, version DESC);

-- 按 change_type 查询
CREATE INDEX idx_memory_history_change_type ON memory_history(change_type);

-- 按 created_at 查询（降序）
CREATE INDEX idx_memory_history_created_at ON memory_history(created_at DESC);

-- 租户隔离
CREATE INDEX idx_memory_history_tenant ON memory_history(organization_id, user_id, agent_id);
```

### 3. 触发器函数

```sql
CREATE OR REPLACE FUNCTION track_memory_changes()
RETURNS TRIGGER AS $$
DECLARE
    next_version INTEGER;
    change_type_val VARCHAR(50);
BEGIN
    -- 确定变更类型和版本号
    IF TG_OP = 'INSERT' THEN
        change_type_val := 'created';
        next_version := 1;
    ELSIF TG_OP = 'UPDATE' THEN
        change_type_val := 'updated';
        SELECT COALESCE(MAX(version), 0) + 1 INTO next_version
        FROM memory_history WHERE memory_id = NEW.id;
    ELSIF TG_OP = 'DELETE' THEN
        change_type_val := 'deleted';
        SELECT COALESCE(MAX(version), 0) + 1 INTO next_version
        FROM memory_history WHERE memory_id = OLD.id;
        
        -- 插入删除记录
        INSERT INTO memory_history (...) VALUES (...);
        RETURN OLD;
    END IF;
    
    -- 插入历史记录
    INSERT INTO memory_history (...) VALUES (...);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 4. 触发器

```sql
-- INSERT 触发器
CREATE TRIGGER trigger_memory_insert
AFTER INSERT ON memories
FOR EACH ROW
EXECUTE FUNCTION track_memory_changes();

-- UPDATE 触发器
CREATE TRIGGER trigger_memory_update
AFTER UPDATE ON memories
FOR EACH ROW
WHEN (OLD.* IS DISTINCT FROM NEW.*)
EXECUTE FUNCTION track_memory_changes();

-- DELETE 触发器
CREATE TRIGGER trigger_memory_delete
BEFORE DELETE ON memories
FOR EACH ROW
EXECUTE FUNCTION track_memory_changes();
```

---

## 🔧 代码实现

### 1. 迁移文件

**文件**: `agentmen/migrations/20251015_create_memory_history.sql`

包含完整的表、索引、触发器函数和触发器定义。

### 2. 迁移代码

**文件**: `agentmen/crates/agent-mem-core/src/storage/migrations.rs`

```rust
/// 创建 memory_history 表和触发器
async fn create_memory_history_table(pool: &PgPool) -> CoreResult<()> {
    // 1. 创建表
    // 2. 创建索引
    // 3. 创建触发器函数
    // 4. 创建触发器
    Ok(())
}
```

在 `run_migrations()` 中调用：

```rust
pub async fn run_migrations(pool: &PgPool) -> CoreResult<()> {
    // ... 其他迁移
    create_memory_history_table(pool).await?;
    Ok(())
}
```

### 3. API 端点

**文件**: `agentmen/crates/agent-mem-server/src/routes/memory.rs`

```rust
pub async fn get_memory_history(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
) -> ServerResult<Json<serde_json::Value>> {
    // 验证 memory 存在
    let memory = memory_manager.get_memory(&id).await?
        .ok_or_else(|| ServerError::not_found("Memory not found"))?;

    // 返回简化的历史记录
    let history = vec![serde_json::json!({
        "version": 1,
        "change_type": "created",
        "content": memory.get("content")...,
        // ...
    })];

    Ok(Json(serde_json::json!({
        "memory_id": id,
        "current_version": 1,
        "total_versions": history.len(),
        "history": history,
        "note": "Memory history table and triggers have been created..."
    })))
}
```

---

## 📝 使用示例

### 1. 查询记忆历史

```bash
curl -X GET http://localhost:3000/api/v1/memories/{memory_id}/history
```

**响应**:

```json
{
  "memory_id": "mem_123",
  "current_version": 1,
  "total_versions": 1,
  "history": [
    {
      "version": 1,
      "change_type": "created",
      "change_reason": "Initial version",
      "content": "...",
      "metadata": {},
      "memory_type": "episodic",
      "importance": 0.5,
      "created_at": "2025-10-15T10:00:00Z"
    }
  ],
  "current_content": "...",
  "current_metadata": {},
  "note": "Memory history table and triggers have been created..."
}
```

### 2. 数据库直接查询

```sql
-- 查询某个记忆的所有历史版本
SELECT * FROM memory_history
WHERE memory_id = 'mem_123'
ORDER BY version DESC;

-- 查询最近的变更
SELECT * FROM memory_history
ORDER BY created_at DESC
LIMIT 10;

-- 查询特定类型的变更
SELECT * FROM memory_history
WHERE change_type = 'updated'
ORDER BY created_at DESC;
```

---

## 🚀 下一步工作

### 1. 扩展 Repository Trait

在 `MemoryRepositoryTrait` 中添加历史方法：

```rust
pub trait MemoryRepositoryTrait: Send + Sync {
    // 现有方法...
    
    // 新增历史方法
    async fn get_history(&self, memory_id: &str) -> Result<Vec<MemoryHistory>>;
    async fn get_version(&self, memory_id: &str, version: i32) -> Result<Option<MemoryHistory>>;
    async fn restore_version(&self, memory_id: &str, version: i32) -> Result<()>;
}
```

### 2. 实现 Repository 方法

在 PostgreSQL 和 LibSQL 实现中添加历史查询方法。

### 3. 更新 API 端点

使用 Repository trait 方法替换简化实现：

```rust
pub async fn get_memory_history(
    Extension(repositories): Extension<Arc<Repositories>>,
    Path(id): Path<String>,
) -> ServerResult<Json<serde_json::Value>> {
    let history = repositories.memories.get_history(&id).await?;
    // 构建响应...
}
```

### 4. 添加版本回溯功能

```rust
pub async fn restore_memory_version(
    Extension(repositories): Extension<Arc<Repositories>>,
    Path((id, version)): Path<(String, i32)>,
) -> ServerResult<Json<ApiResponse<()>>> {
    repositories.memories.restore_version(&id, version).await?;
    Ok(Json(ApiResponse::success(())))
}
```

---

## ✅ 验证

### 1. 数据库迁移

```bash
# 运行迁移
cargo run --bin agent-mem-server

# 验证表存在
psql $DATABASE_URL -c "\d memory_history"

# 验证触发器存在
psql $DATABASE_URL -c "\df track_memory_changes"
```

### 2. 触发器测试

```sql
-- 插入记忆
INSERT INTO memories (...) VALUES (...);

-- 检查历史记录
SELECT * FROM memory_history WHERE memory_id = '...';

-- 更新记忆
UPDATE memories SET content = 'new content' WHERE id = '...';

-- 检查新版本
SELECT * FROM memory_history WHERE memory_id = '...' ORDER BY version DESC;
```

### 3. API 测试

```bash
# 创建记忆
curl -X POST http://localhost:3000/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{"content": "test", "agent_id": "...", "user_id": "..."}'

# 查询历史
curl -X GET http://localhost:3000/api/v1/memories/{id}/history
```

---

## 📊 性能考虑

1. **索引优化**: 已创建多个索引优化查询性能
2. **触发器性能**: 触发器在事务中执行，对性能影响最小
3. **存储空间**: 历史记录会占用额外空间，可考虑定期归档
4. **查询优化**: 使用 `version DESC` 索引优化最新版本查询

---

## 🔒 安全考虑

1. **租户隔离**: 历史记录包含 organization_id, user_id, agent_id
2. **外键约束**: 确保数据完整性
3. **审计追踪**: 记录 changed_by_id 和 change_reason
4. **级联删除**: 删除记忆时自动删除历史记录

---

## 📚 相关文档

- [数据库迁移文档](../migrations/20251015_create_memory_history.sql)
- [迁移代码](../crates/agent-mem-core/src/storage/migrations.rs)
- [API 文档](../crates/agent-mem-server/src/routes/memory.rs)
- [mem20.md](../../doc/technical-design/memory-systems/mem20.md)

