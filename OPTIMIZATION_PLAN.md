# AgentMem 优化实施计划

## 现状分析（2025-10-31）

### ✅ 已实现的优化模块

#### 1. 多层缓存系统 (`cache/multi_level.rs`)
**状态**: 90% 完成
- ✅ L1内存缓存（MemoryCache）
- ✅ L1/L2缓存提升机制
- ✅ 写穿透支持
- ❌ L2 Redis缓存未连接（代码中 `let l2 = None;`）

**最小改造方案**: 保持现状，L1缓存已经足够好

#### 2. 缓存预热系统 (`cache/warming.rs`)
**状态**: 100% 完成
- ✅ Eager warming（启动时加载）
- ✅ Scheduled warming（定期刷新）
- ✅ Lazy warming（按需加载）
- ✅ 统计信息

**无需改造**

#### 3. 批处理系统
**状态**: 80% 完成
- ✅ 批量数据库操作 (`storage/batch.rs`)
- ✅ 通用批处理器 (`performance/batch.rs`)
- ✅ 并发控制
- ❌ 缺少批量嵌入生成

**最小改造方案**: 添加批量嵌入接口包装

#### 4. 搜索排序器 (`search/ranker.rs`)
**状态**: 100% 完成
- ✅ RRF (Reciprocal Rank Fusion) 排序
- ✅ 加权平均排序
- ✅ 权重归一化
- ✅ 保留原始分数（vector_score/fulltext_score）

**无需改造**

#### 5. 向量搜索引擎 (`search/vector_search.rs`)
**状态**: 90% 完成
- ✅ 搜索缓存
- ✅ 性能统计
- ✅ pgvector索引支持（IVFFlat/HNSW）
- ✅ 批量搜索
- ⚠️ 缺少查询优化器
- ⚠️ 缺少结果重排序

**最小改造方案**: 添加简单的查询优化和重排序

---

## 优化实施方案（最小改造）

### 阶段1: 补充缺失功能（1天）

#### 1.1 批量嵌入生成包装器
**文件**: `crates/agent-mem-embeddings/src/batch.rs` (新建)

```rust
/// 批量嵌入生成器
pub struct BatchEmbeddingGenerator {
    client: Arc<dyn EmbeddingClient>,
    batch_processor: BatchProcessor,
}

impl BatchEmbeddingGenerator {
    pub async fn generate_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // 利用现有的 BatchProcessor
        self.batch_processor.batch_execute(texts, |text| {
            let client = self.client.clone();
            async move { client.embed(text).await }
        }).await
    }
}
```

#### 1.2 简单查询优化器
**文件**: `crates/agent-mem-core/src/search/query_optimizer.rs` (新建)

```rust
/// 查询优化器
pub struct QueryOptimizer {
    stats: Arc<RwLock<SearchStats>>,
}

impl QueryOptimizer {
    /// 优化搜索查询
    pub fn optimize(&self, query: &SearchQuery) -> OptimizedQuery {
        // 基于查询特征选择策略
        let strategy = if query.text.contains('"') {
            SearchStrategy::ExactMatch  // 精确匹配
        } else if query.text.split_whitespace().count() < 3 {
            SearchStrategy::Semantic    // 短查询用语义
        } else {
            SearchStrategy::Hybrid      // 默认混合
        };
        
        OptimizedQuery { strategy, ..Default::default() }
    }
}
```

#### 1.3 结果重排序
**文件**: 在 `crates/agent-mem-core/src/search/vector_search.rs` 中添加

```rust
/// 重排序结果（结合元数据和时间衰减）
pub async fn rerank_results(
    &self,
    results: Vec<SearchResult>,
    query: &SearchQuery,
) -> Result<Vec<SearchResult>> {
    let mut reranked = results.into_iter().map(|mut r| {
        // 计算综合分数
        let mut final_score = r.score;
        
        // 时间衰减
        if let Some(created_at) = r.metadata.get("created_at") {
            let age_days = calculate_age_days(created_at);
            let decay = (-age_days as f32 / 30.0).exp(); // 30天半衰期
            final_score *= 0.9 + 0.1 * decay; // 10%时间权重
        }
        
        // 重要性加成
        if let Some(importance) = r.metadata.get("importance") {
            final_score *= 0.9 + 0.1 * importance; // 10%重要性权重
        }
        
        r.score = final_score;
        r
    }).collect::<Vec<_>>();
    
    reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    Ok(reranked)
}
```

### 阶段2: 综合测试（1天）

#### 2.1 缓存测试
**文件**: `crates/agent-mem-core/tests/cache_test.rs` (新建)

测试内容：
- L1缓存命中率
- 缓存预热效果
- 缓存统计

#### 2.2 批处理测试
**文件**: `crates/agent-mem-core/tests/batch_test.rs` (新建)

测试内容：
- 批量嵌入生成性能
- 批量插入性能
- 并发控制正确性

#### 2.3 搜索测试
**文件**: `crates/agent-mem-core/tests/search_test.rs` (新建)

测试内容：
- 查询优化器效果
- 重排序准确性
- RRF融合正确性

#### 2.4 性能基准测试
**文件**: `crates/agent-mem-core/benches/performance_bench.rs` (新建)

测试场景：
- 1000/10000/100000 条记忆的搜索性能
- 批处理 vs 单条处理性能对比
- 缓存命中 vs 未命中延迟对比

### 阶段3: 文档更新（0.5天）

更新 `agentmem40.md`：
- ✅ 标记已完成的优化
- 📊 添加性能测试结果
- 📝 更新架构图

---

## 预期效果

### 性能提升
- ✅ 缓存命中率: 60-80%
- ✅ 批处理吞吐量: 3-5x 提升
- ✅ 查询优化: 15-20% 准确率提升
- ✅ 重排序: 10-15% 相关性提升

### 代码质量
- ✅ 测试覆盖率: 80%+
- ✅ 文档完整性: 100%
- ✅ 可维护性: 高

---

## 实施时间表

| 阶段 | 任务 | 时间 | 状态 |
|------|------|------|------|
| 分析 | 现有代码分析 | 0.5天 | ✅ 完成 |
| 实施1 | 批量嵌入包装器 | 0.5天 | ⏳ 待开始 |
| 实施2 | 查询优化器 | 0.5天 | ⏳ 待开始 |
| 实施3 | 结果重排序 | 0.5天 | ⏳ 待开始 |
| 测试1 | 单元测试 | 0.5天 | ⏳ 待开始 |
| 测试2 | 集成测试 | 0.5天 | ⏳ 待开始 |
| 测试3 | 性能基准测试 | 0.5天 | ⏳ 待开始 |
| 文档 | 更新文档 | 0.5天 | ⏳ 待开始 |
| **总计** | | **4天** | |

---

## 总结

**优势**:
- ✅ 系统已有90%的优化基础设施
- ✅ 代码质量高，架构清晰
- ✅ 只需最小改造即可完成

**策略**:
- 🎯 充分利用现有代码
- 🎯 最小化修改范围
- 🎯 高内聚低耦合
- 🎯 完善测试验证
- 🎯 持续文档更新

**风险**:
- ⚠️ 低（改动小，影响面小）

**建议**:
- ✅ 优先完成测试，验证现有功能
- ✅ 补充缺失的小功能
- ✅ 改进而非重写现有代码

