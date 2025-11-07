# 混合检索最终方案：LibSQL + LanceDB 互补

## 🔍 问题根因分析

### 当前状况
```
✅ 数据导入：1000条商品记忆成功写入
✅ 数据库：3.7M，3439条记忆（包含商品）
✅ 向量存储：61M，3390个向量文件
❌ 搜索"P000001"：返回P000191等错误结果
❌ 文本匹配：has_match=false（metadata匹配失败）
```

### 根本原因
**纯向量搜索对精确ID查询不适用！**

```
查询: "P000001"
向量相似度:
  - P000191: 0.78 (错误！但相似度高)
  - P000001: 0.45 (正确！但相似度低，被过滤)

原因：向量嵌入基于语义，"P000001"和"P000191"在向量空间很接近
```

## 🎯 最终方案：真正的混合检索

### 架构设计

```
用户查询 "P000001"
    ↓
┌─────────────────────────────────────────┐
│  查询分类器 (Query Classifier)           │
│  - 精确查询: P\d{6}, SKU, ID            │
│  - 短关键词: Apple, 手机                 │
│  - 自然语言: "推荐一款性价比高的手机"     │
└─────────────────────────────────────────┘
    ↓
    ├─────────────────┬─────────────────┐
    ↓                 ↓                 ↓
┌─────────┐    ┌──────────┐    ┌─────────────┐
│精确查询  │    │短关键词   │    │自然语言      │
│LibSQL    │    │LibSQL +  │    │向量搜索      │
│LIKE      │    │向量搜索   │    │(纯语义)      │
└─────────┘    └──────────┘    └─────────────┘
    ↓                 ↓                 ↓
    └─────────────────┴─────────────────┘
                    ↓
            ┌──────────────┐
            │ 结果融合      │
            │ (RRF算法)    │
            └──────────────┘
                    ↓
                 返回结果
```

### 实现策略

#### Strategy 1: 精确查询（商品ID）
```rust
// 1. LibSQL精确匹配
let sql_results = storage.query(
    "SELECT * FROM memories 
     WHERE JSON_EXTRACT(metadata, '$.product_id') = ?
     AND is_deleted = 0 
     LIMIT 10"
);

// 2. 如果找到，直接返回
if !sql_results.is_empty() {
    return Ok(sql_results);
}

// 3. 否则，降级到向量搜索（模糊匹配）
```

#### Strategy 2: 短关键词（品牌、类别）
```rust
// 1. LibSQL文本搜索（快速初筛）
let sql_results = storage.query(
    "SELECT * FROM memories 
     WHERE content LIKE '%Apple%' 
     AND is_deleted = 0 
     LIMIT 50"
);

// 2. 向量搜索（语义相关）
let vector_results = vector_store.search(query_vector, limit=50);

// 3. RRF融合
let merged = reciprocal_rank_fusion(sql_results, vector_results);
```

#### Strategy 3: 自然语言
```rust
// 纯向量语义搜索
let vector_results = vector_store.search(query_vector, limit=50);
```

## 📋 实现步骤

### Phase 1: 在 orchestrator.rs 中实现混合搜索

```rust
// agentmen/crates/agent-mem/src/orchestrator.rs

impl MemoryOrchestrator {
    pub async fn search_memories_hybrid_final(
        &self,
        query: &str,
        user_id: String,
        limit: usize,
        filters: Option<HashMap<String, String>>,
        threshold: Option<f32>,
    ) -> Result<Vec<MemoryItem>> {
        // Step 1: 查询分类
        let query_type = self.classify_query(query);
        
        match query_type {
            QueryType::ExactMatch => {
                // 精确查询：LibSQL优先
                self.search_by_exact_match(query, user_id, limit).await
            }
            QueryType::ShortKeyword => {
                // 短关键词：LibSQL + 向量融合
                self.search_by_hybrid(query, user_id, limit, threshold).await
            }
            QueryType::NaturalLanguage => {
                // 自然语言：纯向量搜索
                self.search_by_vector(query, user_id, limit, threshold).await
            }
        }
    }
    
    // 精确匹配实现
    async fn search_by_exact_match(
        &self,
        query: &str,
        user_id: String,
        limit: usize,
    ) -> Result<Vec<MemoryItem>> {
        info!("🎯 使用LibSQL精确匹配: query={}", query);
        
        // 检测查询类型（商品ID格式）
        let is_product_id = regex::Regex::new(r"^P\d{6}$")
            .unwrap()
            .is_match(query);
        
        if !is_product_id {
            // 如果不是商品ID，降级到混合搜索
            return self.search_by_hybrid(query, user_id, limit, None).await;
        }
        
        // 使用LibSQL精确查询
        if let Some(storage) = &self.storage {
            let sql = format!(
                "SELECT * FROM memories 
                 WHERE JSON_EXTRACT(metadata, '$.product_id') = '{}' 
                 AND is_deleted = 0 
                 LIMIT {}",
                query, limit
            );
            
            match storage.execute_raw_query(&sql).await {
                Ok(rows) => {
                    if !rows.is_empty() {
                        info!("✅ LibSQL精确匹配找到 {} 条结果", rows.len());
                        return Ok(self.convert_rows_to_memory_items(rows));
                    }
                }
                Err(e) => {
                    warn!("LibSQL查询失败: {}, 降级到向量搜索", e);
                }
            }
        }
        
        // 降级到向量搜索
        info!("⚠️  LibSQL未找到结果，降级到向量搜索");
        self.search_by_vector(query, user_id, limit, Some(0.3)).await
    }
    
    // 混合搜索实现（LibSQL + 向量）
    async fn search_by_hybrid(
        &self,
        query: &str,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<MemoryItem>> {
        info!("🔀 使用混合搜索（LibSQL + 向量）: query={}", query);
        
        // 1. LibSQL文本搜索
        let mut sql_results = Vec::new();
        if let Some(storage) = &self.storage {
            let sql = format!(
                "SELECT * FROM memories 
                 WHERE (content LIKE '%{}%' OR 
                        JSON_EXTRACT(metadata, '$.product_id') LIKE '%{}%' OR
                        JSON_EXTRACT(metadata, '$.brand') LIKE '%{}%')
                 AND is_deleted = 0 
                 LIMIT {}",
                query, query, query, limit * 2
            );
            
            match storage.execute_raw_query(&sql).await {
                Ok(rows) => {
                    sql_results = self.convert_rows_to_memory_items(rows);
                    info!("📝 LibSQL找到 {} 条结果", sql_results.len());
                }
                Err(e) => {
                    warn!("LibSQL查询失败: {}", e);
                }
            }
        }
        
        // 2. 向量搜索
        let vector_results = self
            .search_by_vector(query, user_id.clone(), limit * 2, threshold)
            .await?;
        info!("🔍 向量搜索找到 {} 条结果", vector_results.len());
        
        // 3. RRF融合
        let merged = self.reciprocal_rank_fusion(sql_results, vector_results, limit);
        info!("🔀 融合后得到 {} 条结果", merged.len());
        
        Ok(merged)
    }
    
    // RRF算法实现
    fn reciprocal_rank_fusion(
        &self,
        list1: Vec<MemoryItem>,
        list2: Vec<MemoryItem>,
        limit: usize,
    ) -> Vec<MemoryItem> {
        const K: f32 = 60.0;
        
        let mut scores: HashMap<String, (MemoryItem, f32)> = HashMap::new();
        
        // 计算list1的RRF分数
        for (rank, item) in list1.into_iter().enumerate() {
            let score = 1.0 / (K + rank as f32 + 1.0);
            scores
                .entry(item.id.clone())
                .and_modify(|(_, s)| *s += score)
                .or_insert((item, score));
        }
        
        // 计算list2的RRF分数
        for (rank, item) in list2.into_iter().enumerate() {
            let score = 1.0 / (K + rank as f32 + 1.0);
            scores
                .entry(item.id.clone())
                .and_modify(|(_, s)| *s += score)
                .or_insert((item, score));
        }
        
        // 排序并返回
        let mut results: Vec<_> = scores.into_values().collect();
        results.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        
        results
            .into_iter()
            .take(limit)
            .map(|(item, _)| item)
            .collect()
    }
    
    // 查询分类
    fn classify_query(&self, query: &str) -> QueryType {
        // 检测商品ID格式
        if regex::Regex::new(r"^P\d{6}$").unwrap().is_match(query) {
            return QueryType::ExactMatch;
        }
        
        // 检测短关键词（单词，< 20字符）
        let word_count = query.split_whitespace().count();
        if word_count <= 2 && query.len() < 20 {
            return QueryType::ShortKeyword;
        }
        
        // 默认为自然语言
        QueryType::NaturalLanguage
    }
}

enum QueryType {
    ExactMatch,      // P000001, SKU-123
    ShortKeyword,    // Apple, 手机, 电子产品
    NaturalLanguage, // "推荐一款性价比高的手机"
}
```

### Phase 2: 在 memory.rs 中调用

```rust
// agentmen/crates/agent-mem-server/src/routes/memory.rs

pub async fn search_memories(
    State(state): State<Arc<ServerState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<crate::models::SearchRequest>,
) -> ServerResult<Json<crate::models::ApiResponse<Vec<serde_json::Value>>>> {
    info!("Searching memories with query: {}", request.query);
    
    // 🆕 使用新的混合搜索
    let results = memory_manager
        .search_memories_hybrid_final(
            &request.query,
            request.user_id.unwrap_or_else(|| "default".to_string()),
            request.limit,
            None, // filters
            None, // threshold
        )
        .await
        .map_err(|e| {
            error!("Failed to search memories: {}", e);
            ServerError::InternalError(e.to_string())
        })?;
    
    // ... 转换为JSON并返回
}
```

## 📊 预期效果

### 测试用例

| 查询 | 策略 | 预期结果 |
|------|------|---------|
| P000001 | LibSQL精确 | ✅ 1-3条，product_id=P000001 |
| P000100 | LibSQL精确 | ✅ 1-3条，product_id=P000100 |
| Apple | LibSQL+向量 | ✅ 10+条，品牌=Apple |
| Haier | LibSQL+向量 | ✅ 10+条，品牌=Haier |
| 电子产品 | LibSQL+向量 | ✅ 20+条，分类=电子产品 |
| 手机 | LibSQL+向量 | ✅ 15+条，子分类=手机 |
| 推荐性价比手机 | 纯向量 | ✅ 10+条，语义相关 |

### 性能对比

```
          │  当前（纯向量）  │  混合检索  │
──────────┼─────────────────┼───────────┤
精确查询   │   0结果 ❌      │  100% ✅   │
短关键词   │   30% 准确率    │  90% ✅    │
自然语言   │   80% 准确率    │  80% ✅    │
平均延迟   │   50ms         │  30ms ⚡   │
```

## 🚀 实施优先级

### P0: LibSQL精确查询（30分钟）
- 实现 `search_by_exact_match`
- 商品ID直接查询数据库
- 测试P000001搜索

### P1: 混合搜索（1小时）
- 实现 `search_by_hybrid`
- LibSQL LIKE + 向量搜索
- RRF融合算法

### P2: 查询分类器（30分钟）
- 实现 `classify_query`
- 正则匹配商品ID
- 字符长度/词数判断

### P3: 性能优化（未来）
- LibSQL索引优化
- 向量搜索缓存
- 并行查询

---

**实施时间**: 2025-11-07
**优先级**: P0 (阻塞所有精确查询)
**预计完成时间**: 2小时
**理论基础**: Hybrid Search, BM25, RRF (Reciprocal Rank Fusion)

