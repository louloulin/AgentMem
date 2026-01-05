# 列表查询去重问题分析与修复

## 🔍 问题现象

UI列表页面显示重复的记忆（内容相同但类型不同）：
- 内容："林很厉害"
- 类型：一个是 `episodic`，一个是 `Semantic`
- 创建时间：相同（2025/12/9 11:22:38）
- ID：不同（因为类型不同）

## 📊 根本原因分析

### 1. **列表查询没有应用去重逻辑**

**位置**: `crates/agent-mem-server/src/routes/memory.rs:3725-3940`

**问题**:
```rust
pub async fn list_all_memories(...) {
    // ❌ 直接从LibSQL查询，没有去重
    let query = format!(
        "SELECT id, agent_id, user_id, content, memory_type, importance, \
         created_at, last_accessed, access_count, metadata, hash \
         FROM memories WHERE is_deleted = 0 ORDER BY {} {} LIMIT ? OFFSET ?",
        sort_by, order
    );
    
    // ❌ 直接返回查询结果，没有应用去重逻辑
    while let Some(row) = rows.next().await? {
        memories_json.push(serde_json::json!({...}));
    }
    
    // ❌ 没有基于ID去重
    // ❌ 没有基于hash/content去重
}
```

**对比搜索查询**:
```rust
pub async fn search_memories(...) {
    // ✅ 有完整的去重逻辑（1932-2001行）
    // 第一步：基于ID去重
    // 第二步：基于hash/content去重
}
```

### 2. **为什么会出现重复**

从日志看（502-503行）：
```
📋 List all memories: page=0, limit=20, agent_id=None
✅ Retrieved 2 memories (total: 2)
```

**原因链条**:
1. **数据写入**: 同一条内容可能被存储为两种类型（episodic和Semantic）
2. **ID不同**: 因为类型不同，所以ID不同
3. **hash可能相同**: 内容相同，hash可能相同
4. **列表查询**: 直接从LibSQL查询，不过滤重复
5. **没有去重**: `list_all_memories`没有应用去重逻辑

### 3. **为什么搜索查询没有这个问题**

搜索查询（`search_memories`）有完整的去重逻辑：
- ✅ 第一步：基于ID去重（1938-1953行）
- ✅ 第二步：基于hash/content去重（1958-1997行）

但列表查询（`list_all_memories`）没有这些逻辑。

## ✅ 修复方案

在`list_all_memories`中也应用去重逻辑，与`search_memories`保持一致。

### 修复策略

1. **基于hash/content去重**: 如果两条记忆的hash相同（或content相同），只保留一条
2. **保留规则**: 保留重要性（importance）最高的，如果重要性相同，保留创建时间最新的
3. **位置**: 在构建`memories_json`之后，返回之前应用去重

### 修复代码

```rust
pub async fn list_all_memories(...) {
    // ... 现有查询逻辑 ...
    
    let mut memories_json: Vec<serde_json::Value> = vec![];
    while let Some(row) = rows.next().await? {
        memories_json.push(serde_json::json!({...}));
    }
    
    // ✅ 新增：应用去重逻辑
    let original_count = memories_json.len();
    let mut deduplicated: Vec<serde_json::Value> = Vec::new();
    let mut seen_hashes: HashMap<String, usize> = HashMap::new();
    
    for memory in memories_json {
        // 获取去重key（优先使用hash，否则使用content）
        let dedup_key = memory.get("hash")
            .and_then(|v| v.as_str())
            .filter(|h| !h.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // 使用content的前100字符作为key
                memory.get("content")
                    .and_then(|v| v.as_str())
                    .map(|c| {
                        if c.len() > 100 {
                            c.chars().take(100).collect()
                        } else {
                            c.to_string()
                        }
                    })
                    .unwrap_or_default()
            });
        
        // 检查是否已存在
        match seen_hashes.get(&dedup_key) {
            Some(&existing_idx) => {
                // 已存在，比较重要性，保留更高的
                let existing = &deduplicated[existing_idx];
                let existing_importance = existing.get("importance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let current_importance = memory.get("importance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                
                if current_importance > existing_importance {
                    // 替换为重要性更高的
                    deduplicated[existing_idx] = memory;
                } else if current_importance == existing_importance {
                    // 重要性相同，保留创建时间更新的
                    let existing_created = existing.get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let current_created = memory.get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    if current_created > existing_created {
                        deduplicated[existing_idx] = memory;
                    }
                }
            }
            None => {
                // 新hash，直接添加
                let idx = deduplicated.len();
                seen_hashes.insert(dedup_key, idx);
                deduplicated.push(memory);
            }
        }
    }
    
    info!("🔄 列表去重: {} → {} 条结果", original_count, deduplicated.len());
    
    // 使用去重后的结果
    memories_json = deduplicated;
    
    // ... 返回结果 ...
}
```

## 🎯 修复效果

修复后：
- ✅ 列表查询也会应用去重逻辑
- ✅ 内容相同但类型不同的记忆只会显示一条
- ✅ 保留重要性最高的记忆
- ✅ 与搜索查询的去重逻辑保持一致

## 📝 注意事项

1. **去重key**: 优先使用hash，如果hash为空，使用content的前100字符
2. **保留规则**: 重要性 > 创建时间（重要性相同时保留更新的）
3. **性能**: 去重逻辑在内存中执行，对性能影响较小

