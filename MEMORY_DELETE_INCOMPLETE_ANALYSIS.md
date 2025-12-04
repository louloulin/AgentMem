# 记忆删除不彻底问题全面分析

## 问题描述

记忆同时存在于**向量存储（Embed/Vector Store）**和**LibSQL数据库**中，删除操作不彻底：
- 删除后，列表查询查不到 ✅
- 但搜索查询（向量搜索）还能查到 ❌
- 删除时返回 "Memory not found" 错误

## 系统架构

### 双层存储架构

```
记忆数据存储在两个地方：
1. LibSQL数据库（结构化数据）
   - 存储记忆的元数据、内容、类型等
   - 用于列表查询、精确查询

2. 向量存储（Vector Store/Embed）
   - 存储记忆的向量嵌入（embeddings）
   - 用于语义搜索、相似度搜索
```

## 当前删除流程分析

### 文件：`crates/agent-mem-server/src/routes/memory.rs`

```rust
pub async fn delete_memory(...) -> ServerResult<...> {
    info!("Deleting memory with ID: {}", id);

    // Step 1: 删除LibSQL Repository (主要存储)
    repositories.memories.delete(&id).await.map_err(|e| {
        error!("Failed to delete memory from repository: {}", e);
        ServerError::MemoryError(format!("Failed to delete memory: {}", e))
    })?;

    info!("✅ Memory deleted from LibSQL");

    // Step 2: 尝试删除Memory API (向量存储) - 如果失败不影响主流程
    if let Err(e) = memory_manager.delete_memory(&id).await {
        warn!(
            "Failed to delete memory from Memory API (non-critical): {}",
            e
        );
    }

    Ok(Json(crate::models::ApiResponse::success(response)))
}
```

### 问题分析

1. **删除顺序问题**：
   - ✅ Step 1: 先删除LibSQL（硬删除）
   - ⚠️ Step 2: 后删除向量存储，但失败时只记录警告

2. **错误处理问题**：
   - ❌ 向量存储删除失败时，LibSQL已经删除
   - ❌ 没有回滚机制
   - ❌ 导致数据不一致：LibSQL已删除，但向量存储还在

3. **"Memory not found" 错误原因**：
   - `find_by_id()` 查询条件：`WHERE id = ? AND is_deleted = 0`
   - 如果记录已经被软删除（is_deleted=1），`find_by_id` 查不到
   - 但 `DELETE FROM memories WHERE id = ?` 不检查 `is_deleted`，所以可能删除失败

## 查询流程分析

### 1. 列表查询（LibSQL）

**文件：`crates/agent-mem-server/src/routes/memory.rs:list_all_memories`**

```rust
let query = format!(
    "SELECT ... FROM memories WHERE is_deleted = 0 ORDER BY ... LIMIT ? OFFSET ?",
    ...
);
```

**特点：**
- ✅ 直接从LibSQL查询
- ✅ 过滤 `is_deleted = 0`
- ✅ 如果LibSQL已删除，查不到

### 2. 搜索查询（向量存储）

**文件：`crates/agent-mem-server/src/routes/memory.rs:search_memories`**

```rust
let results = memory_manager
    .search_memories(
        request.query,
        request.agent_id,
        request.user_id,
        request.limit,
        request.memory_type,
    )
    .await?;
```

**特点：**
- ❌ 从向量存储搜索
- ❌ 不检查LibSQL中的 `is_deleted` 状态
- ❌ 如果向量存储未删除，仍能查到

## 根本原因

### 问题1：删除不原子

```
删除流程：
1. 删除LibSQL ✅ (成功)
2. 删除向量存储 ❌ (失败，但只记录警告)

结果：
- LibSQL: 已删除 ✅
- 向量存储: 未删除 ❌
- 数据不一致！
```

### 问题2：查询路径不一致

```
列表查询：
LibSQL → 过滤 is_deleted=0 → 查不到 ✅

搜索查询：
向量存储 → 不检查LibSQL状态 → 还能查到 ❌
```

### 问题3：删除检查逻辑问题

```rust
// find_by_id 查询条件
WHERE id = ? AND is_deleted = 0

// delete 删除条件
DELETE FROM memories WHERE id = ?

// 问题：如果记录 is_deleted=1，find_by_id查不到，但DELETE可以删除
```

## 解决方案

### 方案1：确保向量存储删除成功（推荐）

**修改删除流程，确保向量存储删除成功，如果失败则回滚LibSQL删除：**

```rust
pub async fn delete_memory(...) -> ServerResult<...> {
    info!("Deleting memory with ID: {}", id);

    // Step 1: 先删除向量存储（如果失败，可以回滚）
    let vector_delete_result = memory_manager.delete_memory(&id).await;
    
    // Step 2: 删除LibSQL Repository
    let libsql_delete_result = repositories.memories.delete(&id).await;
    
    // Step 3: 检查结果
    match (vector_delete_result, libsql_delete_result) {
        (Ok(_), Ok(_)) => {
            info!("✅ Memory deleted from both LibSQL and Vector Store");
            Ok(Json(crate::models::ApiResponse::success(response)))
        }
        (Ok(_), Err(e)) => {
            // LibSQL删除失败，但向量存储已删除
            error!("Failed to delete from LibSQL after vector store deleted: {}", e);
            Err(ServerError::MemoryError(format!("Failed to delete memory: {}", e)))
        }
        (Err(e), Ok(_)) => {
            // 向量存储删除失败，但LibSQL已删除
            error!("Failed to delete from vector store after LibSQL deleted: {}", e);
            // 尝试回滚：重新创建记录（如果可能）
            // 或者返回错误，让用户知道删除不完整
            Err(ServerError::MemoryError(format!(
                "Memory deleted from LibSQL but failed to delete from vector store: {}", e
            )))
        }
        (Err(e1), Err(e2)) => {
            error!("Failed to delete from both stores: vector={}, libsql={}", e1, e2);
            Err(ServerError::MemoryError(format!("Failed to delete memory: {}", e2)))
        }
    }
}
```

### 方案2：在搜索时过滤已删除的记录（临时方案）

**在向量搜索结果中，检查LibSQL中的记录状态：**

```rust
pub async fn search_memories(...) -> Result<Vec<MemoryItem>, String> {
    // 执行向量搜索
    let raw_results = self
        .memory
        .search_with_options(query.clone(), options)
        .await?;

    // 过滤：检查LibSQL中是否存在且未删除
    let mut valid_results = Vec::new();
    for result in raw_results {
        // 检查LibSQL中的记录状态
        if let Some(memory) = repositories.memories.find_by_id(&result.id).await? {
            // 记录存在且未删除，添加到结果
            valid_results.push(result);
        }
        // 如果记录不存在或已删除，跳过
    }

    Ok(valid_results)
}
```

### 方案3：修复删除检查逻辑

**修改delete方法，不依赖find_by_id检查：**

```rust
async fn delete(&self, id: &str) -> Result<()> {
    let conn = self.conn.lock().await;

    // ✅ 修复：直接删除，不检查is_deleted状态
    // 因为硬删除应该删除所有状态的记录
    let rows_affected = conn
        .execute(
            "DELETE FROM memories WHERE id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| AgentMemError::StorageError(format!("Failed to delete memory: {e}")))?;

    if rows_affected == 0 {
        return Err(AgentMemError::NotFound(format!("Memory not found: {}", id)));
    }

    info!("✅ Memory physically deleted from database: {}", id);
    Ok(())
}
```

## 推荐修复方案

### 综合方案：确保删除原子性 + 修复检查逻辑

1. **修复删除流程**：确保向量存储和LibSQL都删除成功
2. **修复删除检查**：不依赖find_by_id，直接删除
3. **添加搜索过滤**：在向量搜索结果中检查LibSQL状态（作为额外保障）

## 修复优先级

1. **P0（立即修复）**：
   - 修复删除检查逻辑（方案3）
   - 确保向量存储删除成功（方案1）

2. **P1（重要）**：
   - 在搜索时过滤已删除记录（方案2）

3. **P2（优化）**：
   - 添加删除事务支持
   - 添加删除状态监控

## 测试建议

1. **删除测试**：
   - 删除一个记忆，验证LibSQL和向量存储都删除
   - 模拟向量存储删除失败，验证错误处理
   - 验证删除后列表查询和搜索查询都查不到

2. **一致性测试**：
   - 创建记忆，验证两个存储都有数据
   - 删除记忆，验证两个存储都删除
   - 部分删除失败，验证错误处理

3. **边界测试**：
   - 删除不存在的记忆
   - 删除已删除的记忆
   - 并发删除同一记忆

