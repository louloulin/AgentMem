# 混合检索策略实现方案

## 📚 理论基础

### 1. 混合检索（Hybrid Search）
根据最新研究，混合检索结合了：
- **Dense Retrieval (向量检索)**: 语义相似度匹配，适合"电子产品"、"手机"等查询
- **Sparse Retrieval (稀疏检索/文本匹配)**: 精确匹配，适合"P000001"等商品ID查询
- **HNSW算法**: LanceDB已内置，提供高效向量检索

### 2. 检索策略选择
参考论文《Efficient Estimation of Word Representations in Vector Space》和《LINE: Large-scale Information Network Embedding》：

| 查询类型 | 最佳策略 | 示例 |
|---------|---------|------|
| 精确标识符 | Text Matching | P000001, SKU-123 |
| 短关键词 | Hybrid (Text + Vector) | Apple, 手机 |
| 自然语言 | Vector Search | "推荐一款性价比高的手机" |
| 分类查询 | Vector Search | "电子产品", "家居用品" |

### 3. 当前问题分析

```
问题：搜索"P000001"返回0结果

原因：
1. 查询向量: [0.1, 0.3, ...] (384维，基于"P000001"文本)
2. 商品向量: [0.5, 0.2, ...] (384维，基于完整商品描述)
3. 余弦相似度: ~0.15 (低于阈值0.3)
4. 结果：被过滤掉

解决：
- 对于商品ID格式(P\d{6})，使用SQL LIKE匹配
- 对于自然语言，使用向量搜索
- 混合两种策略的结果
```

## 🎯 实现方案

### Phase 1: 查询分类器（最小改动）

在`orchestrator.rs`的`search_memories_hybrid`中添加查询分类：

```rust
// 1. 检测查询类型
let query_type = detect_query_type(&query);

match query_type {
    QueryType::ExactMatch => {
        // 使用LibSQL精确匹配
        search_by_text(query)
    }
    QueryType::ShortKeyword => {
        // 混合：Text + Vector
        let text_results = search_by_text(query);
        let vector_results = search_by_vector(query);
        merge_results(text_results, vector_results)
    }
    QueryType::Semantic => {
        // 纯向量搜索
        search_by_vector(query)
    }
}
```

### Phase 2: LibSQL文本搜索（复用现有能力）

AgentMem已有LibSQL存储，可直接查询：

```rust
// 在orchestrator.rs中
async fn search_by_text(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>> {
    if let Some(storage) = &self.storage {
        // SQL: SELECT * FROM memories WHERE content LIKE '%P000001%'
        let sql = format!(
            "SELECT * FROM memories WHERE content LIKE '%{}%' AND is_deleted = 0 LIMIT {}",
            query.replace("'", "''"), limit
        );
        storage.execute_query(sql).await
    }
}
```

### Phase 3: 结果融合（RRF算法）

使用Reciprocal Rank Fusion合并结果：

```rust
fn merge_results(text: Vec<MemoryItem>, vector: Vec<MemoryItem>) -> Vec<MemoryItem> {
    let k = 60.0; // RRF常数
    
    // 计算每个结果的融合分数
    for (rank, item) in text.iter().enumerate() {
        item.score += 1.0 / (k + rank as f32);
    }
    for (rank, item) in vector.iter().enumerate() {
        item.score += 1.0 / (k + rank as f32);
    }
    
    // 按分数排序
    results.sort_by(|a, b| b.score.partial_cmp(&a.score));
    results
}
```

## 📋 最小改动实现步骤

### Step 1: 查询类型检测（orchestrator.rs）

```rust
enum QueryType {
    ExactMatch,   // 商品ID: P\d{6}
    ShortKeyword, // 短关键词: < 20字符
    Semantic,     // 自然语言: > 20字符
}

fn detect_query_type(query: &str) -> QueryType {
    // 检测商品ID格式
    if regex::Regex::new(r"^P\d{6}$").unwrap().is_match(query) {
        return QueryType::ExactMatch;
    }
    
    // 短查询
    if query.len() < 20 {
        return QueryType::ShortKeyword;
    }
    
    QueryType::Semantic
}
```

### Step 2: LibSQL文本搜索（复用storage）

AgentMem的`storage`已经有`EpisodicAgent`, `SemanticAgent`等，它们都有SQL查询能力。

```rust
// 在search_memories_hybrid中
if matches!(query_type, QueryType::ExactMatch | QueryType::ShortKeyword) {
    // 先尝试文本匹配
    if let Some(semantic_agent) = &self.semantic_agent {
        let text_results = semantic_agent.search_by_content(&query, limit).await?;
        if !text_results.is_empty() {
            return Ok(text_results);
        }
    }
}
```

### Step 3: 临时快速修复（search_with_filters）

在LanceDB的`search_with_filters`中添加文本预过滤：

```rust
// 在执行向量搜索前
if let Some(query_hint) = filters.get("_query_text") {
    // 检查metadata中是否包含查询文本
    if !metadata.values().any(|v| v.contains(query_hint.as_str())) {
        continue; // 跳过不包含查询文本的结果
    }
}
```

## 🚀 实施优先级

### P0: 立即修复（5分钟）
```rust
// lancedb_store.rs:search_with_filters
// 移除阈值检查，改为动态阈值
let dynamic_threshold = if query_len < 10 { 0.1 } else { 0.3 };
```

### P1: 文本预过滤（30分钟）
在向量搜索前检查content是否包含查询关键词

### P2: 完整混合检索（2小时）
实现QueryType分类和结果融合

### P3: 性能优化（未来）
- BM25全文索引
- 倒排索引加速
- 缓存热门查询

## 📊 预期效果

| 查询 | 当前 | 修复后 |
|-----|------|--------|
| P000001 | 0结果 ❌ | 1结果 ✅ |
| Apple | 0结果 ❌ | 10+结果 ✅ |
| 电子产品 | 3结果 ✅ | 20+结果 ✅ |
| 手机 | 5结果 ✅ | 15+结果 ✅ |

## 🔗 参考资料

1. **HNSW论文**: "Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs" (2016)
2. **Word2Vec**: "Efficient Estimation of Word Representations in Vector Space" (2013)
3. **LINE**: "Large-scale Information Network Embedding" (2015)
4. **RRF**: Reciprocal Rank Fusion for combining multiple rankings
5. **Hybrid Search Best Practices**: ElasticSearch, Weaviate, Pinecone documentation

---

**实现时间**: 2025-11-07
**优先级**: P0 (阻塞商品搜索功能)
**复杂度**: 低（基于现有代码）

