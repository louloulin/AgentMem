# 商品ID搜索失败问题分析报告

## 问题描述

**现象**: 搜索商品ID "P000257" 时，系统返回"没有找到商品信息"，但实际上记忆数据中存在多条 P000257 的商品记录。

**时间**: 2025-11-08 17:37:59

## 问题验证

### 1. 记忆数据确实存在

通过直接API调用验证，搜索 "P000257" 能找到以下记忆：

```json
{
  "data": [
    {
      "content": "商品ID: P000257, 名称: Li-Ning 存储卡 快充款 D257, 分类: 数码配件>存储卡, 品牌: Li-Ning, 价格: ¥564, 库存: 938件, 状态: 在售",
      "memory_type": "Semantic",
      "metadata": {
        "product_id": "P000257",
        "category": "数码配件",
        "subcategory": "存储卡",
        "brand": "Li-Ning",
        "price": "564",
        "stock": "938",
        "status": "active"
      }
    },
    // ... 还有多条 P000257 的记录
  ]
}
```

### 2. 搜索返回的问题

搜索 "P000257" 时，返回的第一条结果是：

```
"User: P000257商品详情\nAssistant: 根据您提供的记忆信息，并没有包含关于商品ID为P000257的详细信息..."
```

这是一个**工作记忆（working memory）**，内容是LLM之前的错误回复，而不是实际的商品记忆。

## 检索流程分析

### 当前检索流程（两阶段）

```
用户查询: "P000257"
    ↓
Phase 1: 检测精确查询
    ↓
detect_exact_query("P000257")
    ↓ (匹配 ^P\d{6}$ 模式 → true)
    ↓
Phase 2: LibSQL精确查询
    ↓
search_by_libsql_exact()
    ↓
repositories.memories.search("P000257", limit)
    ↓ (使用 LIKE 查询)
    ↓
如果找到 → 返回结果 ✅
如果未找到 → 降级到向量搜索
    ↓
Phase 3: 向量语义搜索
    ↓
memory_manager.search_memories()
    ↓
向量相似度计算
    ↓
阈值过滤 (threshold = 0.7)
    ↓
返回排序后的结果
```

### 问题根源分析

#### 问题1: LibSQL精确查询可能失败

**代码位置**: `agentmen/crates/agent-mem-server/src/routes/memory.rs:816-868`

```rust
async fn search_by_libsql_exact(
    repositories: &Arc<Repositories>,
    query: &str,
    limit: usize,
) -> Result<Vec<serde_json::Value>, String> {
    // 使用repositories.memories.search方法（支持content LIKE查询）
    match repositories.memories
        .search(query, limit as i64)
        .await
    {
        Ok(memories) if !memories.is_empty() => {
            // 返回结果
        }
        Ok(_) => {
            // ⚠️ 未找到结果，降级到向量搜索
            Err(format!("未找到匹配的记忆: {}", query))
        }
    }
}
```

**问题**:
- `repositories.memories.search()` 使用的是 **LIKE 查询**，不是精确匹配
- 如果 content 字段是 "商品ID: P000257, 名称: ..."，LIKE "P000257" 可能匹配不到
- 应该使用 `WHERE content LIKE '%P000257%'` 或 `WHERE metadata->>'product_id' = 'P000257'`

#### 问题2: 向量搜索阈值过高

**代码位置**: `agentmen/crates/agent-mem-server/src/routes/memory.rs:406`

```rust
let dynamic_threshold = get_adaptive_threshold(&query);
// 默认阈值可能是 0.7
```

**问题**:
- 对于精确ID查询，阈值 0.7 可能太高
- 商品ID "P000257" 的向量表示可能与包含 "P000257" 的工作记忆相似度更高
- 导致工作记忆排在商品记忆前面

#### 问题3: 工作记忆干扰

**问题**:
- 工作记忆包含 "P000257" 关键词
- 向量搜索时，工作记忆的相似度可能更高
- 导致工作记忆排在商品记忆前面

#### 问题4: 重复商品记录

**发现**:
- 同一个商品ID "P000257" 存在多条记录：
  - Li-Ning 存储卡
  - Haier 耳机
  - Want-Want 生鲜
  - HP 相机

**原因**:
- 商品记忆脚本可能多次运行
- 没有去重机制
- 导致同一商品ID有多个不同的商品信息

## 解决方案

### 方案1: 改进LibSQL精确查询（推荐）

**修改**: `search_by_libsql_exact()` 函数

```rust
async fn search_by_libsql_exact(
    repositories: &Arc<Repositories>,
    query: &str,
    limit: usize,
) -> Result<Vec<serde_json::Value>, String> {
    // 1. 先尝试 metadata 精确匹配（商品ID）
    if query.starts_with("P") && query.len() == 7 {
        // 商品ID格式：P000257
        match search_by_metadata_product_id(repositories, query, limit).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            _ => {}
        }
    }
    
    // 2. 使用 content LIKE 查询（包含查询）
    match repositories.memories
        .search(&format!("%{}%", query), limit as i64)  // 添加 % 通配符
        .await
    {
        Ok(memories) if !memories.is_empty() => {
            // 过滤：优先返回包含精确ID的记忆
            let exact_matches: Vec<_> = memories
                .iter()
                .filter(|m| {
                    m.content.contains(&format!("商品ID: {}", query)) ||
                    m.metadata.get("product_id") == Some(&serde_json::Value::String(query.to_string()))
                })
                .collect();
            
            if !exact_matches.is_empty() {
                // 返回精确匹配的结果
                return Ok(convert_to_json(exact_matches));
            }
            
            // 返回所有匹配结果
            Ok(convert_to_json(memories))
        }
        // ...
    }
}
```

### 方案2: 降低精确查询的阈值

**修改**: `search_memories()` 函数

```rust
// 对于精确查询，降低阈值
let dynamic_threshold = if is_exact_query {
    0.3  // 精确查询使用更低的阈值
} else {
    get_adaptive_threshold(&query)
};
```

### 方案3: 改进排序逻辑

**修改**: 向量搜索结果排序

```rust
// 对于精确查询，优先返回：
// 1. content 包含精确ID的记忆
// 2. metadata.product_id 匹配的记忆
// 3. 其他相关记忆

let mut sorted_results = results;
sorted_results.sort_by(|a, b| {
    let a_exact = a.content.contains(&format!("商品ID: {}", query)) ||
                  a.metadata.get("product_id") == Some(&serde_json::Value::String(query.to_string()));
    let b_exact = b.content.contains(&format!("商品ID: {}", query)) ||
                  b.metadata.get("product_id") == Some(&serde_json::Value::String(query.to_string()));
    
    match (a_exact, b_exact) {
        (true, false) => std::cmp::Ordering::Less,  // a 排在前面
        (false, true) => std::cmp::Ordering::Greater, // b 排在前面
        _ => b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
    }
});
```

### 方案4: 过滤工作记忆

**修改**: 搜索时排除工作记忆

```rust
// 在返回结果前，过滤掉工作记忆（如果查询是商品ID）
if is_exact_query && query.starts_with("P") {
    results.retain(|r| r.memory_type != "working");
}
```

### 方案5: 修复重复商品问题

**修改**: 商品记忆添加脚本

```bash
# 在添加商品记忆前，先检查是否已存在
# 如果存在，更新而不是创建新记录
```

## 立即修复建议

### 快速修复（最小改动）

1. **修改 `detect_exact_query()` 函数**，确保正确识别商品ID
2. **修改 `search_by_libsql_exact()` 函数**，使用 `LIKE '%P000257%'` 而不是 `LIKE 'P000257'`
3. **降低精确查询的阈值**，从 0.7 降到 0.3

### 完整修复（推荐）

1. 实现 metadata 精确匹配查询
2. 改进排序逻辑，优先返回精确匹配
3. 过滤工作记忆干扰
4. 添加商品记忆去重机制

## 测试验证

修复后，应该验证：

1. ✅ 搜索 "P000257" 能返回商品记忆（不是工作记忆）
2. ✅ 返回的第一条结果是商品记忆
3. ✅ 工作记忆不会干扰商品搜索
4. ✅ 精确查询使用 LibSQL，性能更好

## 相关文件

- `agentmen/crates/agent-mem-server/src/routes/memory.rs` - 搜索API实现
- `agentmen/scripts/add_product_memories.sh` - 商品记忆添加脚本
- `agentmen/crates/agent-mem-core/src/storage/` - 存储层实现

---

**分析日期**: 2025-11-08  
**问题状态**: 🔍 已分析，待修复

