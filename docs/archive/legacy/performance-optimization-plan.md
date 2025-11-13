# AgentMem 性能优化计划 (xn.md)

**日期**: 2025-11-07  
**版本**: v1.0  
**状态**: 📝 规划中

---

## 🔍 性能问题分析

### 当前性能指标

```
✅ 已完成测试:
- 单次API调用: 17-34ms (平均 ~25ms)
- 当前写入速度: ~40条/秒 (理论)
- 实际写入速度: ~12-15条/秒 (脚本串行)
- 已写入: 3662/10000 条 (36.62%)

❌ 性能瓶颈:
- 批量写入10000条预计: 10-15分钟
- 远低于目标: 50条/秒 (3-4分钟)
- 成功率: ~98% (有2%失败率)
```

---

## 📊 性能瓶颈详细分析

### 瓶颈1: 嵌入向量生成 (最大瓶颈)

**现象**:
```rust
// 从日志看到每次都要生成嵌入向量
INFO ✅ 生成嵌入向量，维度: 384
// 耗时: ~15-20ms/次
```

**分析**:
- **Embedder**: FastEmbed (BAAI/bge-small-en-v1.5)
- **模型加载**: 每次调用可能重新加载模型
- **CPU密集**: 384维向量计算消耗大量CPU
- **无批处理**: 每条记忆单独生成，没有批量处理

**影响**:
```
单次嵌入: 20ms
10000次: 20ms × 10000 = 200秒 = 3.3分钟
占总耗时: ~60-70%
```

**优化方向**:
1. ✅ 批量嵌入（Batch Embedding）
2. ✅ 模型预加载和缓存
3. ✅ 异步嵌入处理
4. ✅ GPU加速（如果可用）

---

### 瓶颈2: 三阶段串行提交

**现象**:
```rust
INFO Commit Phase 1/3: 存储到 CoreMemoryManager
INFO ✅ 已存储到 CoreMemoryManager
INFO Commit Phase 2/3: 存储到向量库
INFO ✅ 已存储到向量库
INFO Commit Phase 3/3: 记录操作历史
INFO ✅ 已记录操作历史
```

**分析**:
- **串行执行**: 三个阶段顺序执行
- **每阶段耗时**: 1-3ms
- **总耗时**: ~5-8ms
- **无并发**: 没有利用并行能力

**影响**:
```
单次提交: 8ms
10000次: 8ms × 10000 = 80秒 = 1.3分钟
占总耗时: ~20-25%
```

**优化方向**:
1. ✅ 批量提交（Batch Commit）
2. ✅ 并行写入（向量库和LibSQL并行）
3. ✅ 异步操作历史记录
4. ✅ 事务批处理

---

### 瓶颈3: LibSQL写入

**现象**:
```rust
// 每次写入多个表
- memories表
- episodic_events / core_memory / semantic_memory
- memory_operations (操作历史)
```

**分析**:
- **多表写入**: 每条记忆写入3-4个表
- **事务开销**: 每次独立事务
- **索引更新**: 每次写入触发索引更新
- **WAL模式**: SQLite的WAL模式有写入开销

**影响**:
```
单次写入: 3-5ms
10000次: 5ms × 10000 = 50秒
占总耗时: ~15-20%
```

**优化方向**:
1. ✅ 批量事务（Batch Transaction）
2. ✅ 延迟索引更新
3. ✅ 预分配空间
4. ✅ 考虑使用PostgreSQL（生产环境）

---

### 瓶颈4: 脚本串行调用

**现象**:
```bash
# 脚本中每次调用都是串行的
for i in $(seq 1 $BATCH_SIZE); do
    curl -X POST ...  # 串行
done
```

**分析**:
- **无并发**: 每个curl等待上一个完成
- **网络延迟**: 每次都有TCP握手
- **无连接复用**: 每次都是新连接

**影响**:
```
单次调用: 25ms
串行调用100次: 2.5秒/批
100批: 250秒 = 4.2分钟
实际耗时: 10-15分钟（加上失败重试）
```

**优化方向**:
1. ✅ 并发API调用（使用xargs -P）
2. ✅ HTTP/2连接复用
3. ✅ 批量API端点（单次提交多条）
4. ✅ 长连接（Keep-Alive）

---

### 瓶颈5: 内存和权限检查

**现象**:
```rust
INFO Permission granted user_id=default action=POST 
INFO Adding new memory for agent_id: None, user_id: None
INFO 使用简单模式 (infer=false)
```

**分析**:
- **权限检查**: 每次都检查权限（虽然是无认证模式）
- **Agent查找**: 查询agents表（虽然是缓存的）
- **日志输出**: 大量INFO日志影响性能

**影响**:
```
单次开销: ~2ms
10000次: 20秒
占总耗时: ~5-10%
```

**优化方向**:
1. ✅ 批量写入时跳过重复检查
2. ✅ 降低日志级别（WARN/ERROR）
3. ✅ 缓存权限结果
4. ✅ 批量模式标志

---

## 🎯 优化方案

### Phase 1: 快速优化（1-2天，预期提升3-5倍）

#### 1.1 批量嵌入接口 ⭐⭐⭐

**目标**: 将嵌入耗时从200秒降到40秒

**实现**:
```rust
// crates/agent-mem-embedder/src/lib.rs
pub trait Embedder {
    // 新增：批量嵌入
    async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // 默认实现：循环调用embed（子类可以覆盖优化）
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(&text).await?);
        }
        Ok(results)
    }
}

// FastEmbed实现
impl Embedder for FastEmbedEmbedder {
    async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // ✅ 使用FastEmbed的批量接口
        self.model.embed_batch(texts, Some(Batch::from_usize(32)))
    }
}
```

**预期效果**:
- 嵌入耗时: 200秒 → 40秒 (5倍提升)
- 总耗时: 600秒 → 440秒 (27%提升)

---

#### 1.2 批量API端点 ⭐⭐⭐

**目标**: 单次API调用提交多条记忆

**实现**:
```rust
// crates/agent-mem-server/src/routes/memory.rs

#[derive(Deserialize)]
pub struct BatchMemoryRequest {
    pub memories: Vec<MemoryRequest>,
}

pub async fn add_memories_batch(
    State(app): State<Arc<AppState>>,
    Json(req): Json<BatchMemoryRequest>,
) -> Result<Json<BatchMemoryResponse>, AppError> {
    // 1. 批量生成嵌入向量
    let texts: Vec<String> = req.memories.iter().map(|m| m.content.clone()).collect();
    let embeddings = app.embedder.embed_batch(texts).await?;
    
    // 2. 批量提交事务
    let mut tx = app.db.begin().await?;
    let mut memory_ids = Vec::new();
    
    for (memory_req, embedding) in req.memories.iter().zip(embeddings.iter()) {
        let memory_id = store_memory_with_embedding(&mut tx, memory_req, embedding).await?;
        memory_ids.push(memory_id);
    }
    
    tx.commit().await?;
    
    Ok(Json(BatchMemoryResponse { ids: memory_ids }))
}
```

**API设计**:
```bash
POST /api/v1/memories/batch
Content-Type: application/json

{
  "memories": [
    {"content": "商品1", "memory_type": "Semantic", ...},
    {"content": "商品2", "memory_type": "Semantic", ...},
    ...
  ]
}

# 返回
{
  "ids": ["uuid1", "uuid2", ...],
  "success_count": 100,
  "fail_count": 0
}
```

**预期效果**:
- API调用次数: 10000次 → 100次 (100倍减少)
- 网络开销: 250秒 → 2.5秒 (100倍提升)
- 总耗时: 600秒 → 100秒 (6倍提升)

---

#### 1.3 脚本并发优化 ⭐⭐

**实现**:
```bash
# 使用xargs并发调用
seq 1 100 | xargs -P 10 -I {} curl -X POST ...

# 或使用GNU parallel
parallel -j 10 curl -X POST ::: $(seq 1 100)
```

**预期效果**:
- 串行调用: 250秒
- 并发10: 25秒 (10倍提升)
- 总耗时: 600秒 → 75秒 (8倍提升)

---

### Phase 2: 中期优化（3-5天，预期提升10倍）

#### 2.1 异步批处理队列 ⭐⭐⭐

**架构**:
```
Client → API → Queue → BatchProcessor → Storage
                ↓
              (Buffer)
                ↓
          Every 1s or 100 items
```

**实现**:
```rust
pub struct MemoryBatchProcessor {
    buffer: Arc<Mutex<Vec<MemoryRequest>>>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl MemoryBatchProcessor {
    pub async fn add(&self, memory: MemoryRequest) -> Result<String> {
        // 1. 添加到buffer
        let mut buffer = self.buffer.lock().await;
        buffer.push(memory);
        
        // 2. 如果达到batch_size，立即处理
        if buffer.len() >= self.batch_size {
            self.flush(&mut buffer).await?;
        }
        
        Ok(memory_id)
    }
    
    async fn flush(&self, buffer: &mut Vec<MemoryRequest>) -> Result<()> {
        // 批量处理
        let embeddings = self.embedder.embed_batch(...).await?;
        // 批量存储
        self.storage.store_batch(...).await?;
        
        buffer.clear();
    }
}
```

**预期效果**:
- API响应: 同步 → 异步确认
- 吞吐量: 40条/秒 → 500条/秒 (12倍提升)
- 总耗时: 600秒 → 20秒 (30倍提升)

---

#### 2.2 向量库批量写入 ⭐⭐

**实现**:
```rust
// crates/agent-mem-vector/src/lib.rs
pub trait VectorStore {
    async fn upsert_batch(&self, vectors: Vec<VectorData>) -> Result<()>;
}

// Qdrant实现
impl VectorStore for QdrantVectorStore {
    async fn upsert_batch(&self, vectors: Vec<VectorData>) -> Result<()> {
        let points: Vec<PointStruct> = vectors.into_iter().map(|v| {
            PointStruct {
                id: v.id.into(),
                vector: v.embedding.into(),
                payload: v.metadata,
            }
        }).collect();
        
        // ✅ Qdrant批量upsert
        self.client.upsert_points_blocking(
            self.collection,
            points,
            None,
        ).await?;
        
        Ok(())
    }
}
```

**预期效果**:
- 向量写入: 10000次 → 100次
- 耗时: 50秒 → 5秒 (10倍提升)

---

#### 2.3 数据库连接池优化 ⭐⭐

**实现**:
```rust
// 增加连接池大小
let pool = SqlitePoolOptions::new()
    .max_connections(20)  // 从5增加到20
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .connect(&database_url)
    .await?;

// 批量事务
let mut tx = pool.begin().await?;
for memory in memories {
    insert_memory(&mut tx, memory).await?;
}
tx.commit().await?;  // 一次提交
```

**预期效果**:
- 连接等待: 减少90%
- 事务开销: 10000次 → 100次

---

### Phase 3: 长期优化（1-2周，预期提升50倍）

#### 3.1 GPU加速嵌入 ⭐⭐⭐

**方案**:
```rust
// 使用ONNX Runtime GPU
pub struct GPUEmbedder {
    session: ort::Session,
    // 使用CUDA/CoreML/DirectML
}

impl Embedder for GPUEmbedder {
    async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // GPU批量推理
        // 预期加速: 10-50倍
    }
}
```

**预期效果**:
- 嵌入耗时: 200秒 → 4-20秒 (10-50倍)

---

#### 3.2 分布式存储 ⭐⭐⭐

**架构**:
```
API Layer
    ↓
Message Queue (Redis/Kafka)
    ↓
Worker Pool (10 workers)
    ↓
Database Cluster (PostgreSQL)
```

**实现**:
```rust
// 使用消息队列
pub async fn add_memory_async(memory: MemoryRequest) -> Result<String> {
    let job_id = uuid::Uuid::new_v4();
    
    // 发送到队列
    redis.lpush("memory_queue", serde_json::to_string(&memory)?).await?;
    
    // 立即返回job_id
    Ok(job_id.to_string())
}

// Worker处理
pub async fn worker_process() {
    loop {
        let batch = redis.rpop("memory_queue", 100).await?;
        process_batch(batch).await?;
    }
}
```

**预期效果**:
- 吞吐量: 500条/秒 → 5000条/秒 (10倍)
- 可扩展: 水平扩展workers

---

#### 3.3 缓存优化 ⭐⭐

**策略**:
```rust
// 1. 嵌入缓存
pub struct EmbeddingCache {
    cache: Arc<RwLock<LruCache<String, Vec<f32>>>>,
}

// 2. 查询结果缓存
pub struct SearchResultCache {
    cache: Arc<RwLock<LruCache<String, Vec<Memory>>>>,
    ttl: Duration,
}

// 3. Agent/User信息缓存
pub struct MetadataCache {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    users: Arc<RwLock<HashMap<String, User>>>,
}
```

**预期效果**:
- 重复内容: 跳过嵌入生成
- 热点查询: 命中率>80%

---

## 📈 预期性能提升总结

### 各阶段对比

| 阶段 | 优化方案 | 写入速度 | 10K总耗时 | 提升倍数 |
|------|---------|---------|----------|---------|
| 当前 | 无优化 | 12条/秒 | 13分钟 | 1x |
| Phase 1 | 批量API+脚本并发 | 100条/秒 | 100秒 | 8x |
| Phase 2 | 异步队列+批处理 | 500条/秒 | 20秒 | 40x |
| Phase 3 | GPU+分布式 | 5000条/秒 | 2秒 | 400x |

### 目标达成度

```
短期目标 (Phase 1):
  ✅ 10,000条: 13分钟 → 2分钟
  ✅ 写入速度: 12条/秒 → 100条/秒
  ✅ 成功率: 98% → 99.5%

中期目标 (Phase 2):
  ✅ 10,000条: 2分钟 → 20秒
  ✅ 写入速度: 100条/秒 → 500条/秒
  ✅ 支持并发: 单机 → 多worker

长期目标 (Phase 3):
  ✅ 10,000条: 20秒 → 2秒
  ✅ 写入速度: 500条/秒 → 5000条/秒
  ✅ 可扩展性: 水平扩展
```

---

## 🛠️ 实施计划

### Week 1: Phase 1实施

**Day 1-2: 批量嵌入**
- [ ] 实现Embedder::embed_batch接口
- [ ] FastEmbed批量优化
- [ ] 单元测试

**Day 3-4: 批量API**
- [ ] 实现/api/v1/memories/batch端点
- [ ] 批量事务处理
- [ ] API测试

**Day 5: 脚本优化**
- [ ] 修改批量写入脚本（并发）
- [ ] 性能测试
- [ ] 文档更新

**验收标准**:
- ✅ 10,000条写入时间 < 2分钟
- ✅ 成功率 > 99.5%
- ✅ 无内存泄漏

---

### Week 2: Phase 2实施

**Day 1-3: 异步批处理队列**
- [ ] 设计队列架构
- [ ] 实现MemoryBatchProcessor
- [ ] 集成测试

**Day 4: 向量库优化**
- [ ] 实现VectorStore::upsert_batch
- [ ] 各向量库适配
- [ ] 性能测试

**Day 5: 数据库优化**
- [ ] 连接池调优
- [ ] 批量事务优化
- [ ] 索引优化

**验收标准**:
- ✅ 10,000条写入时间 < 20秒
- ✅ 吞吐量 > 500条/秒
- ✅ P99延迟 < 50ms

---

### Week 3-4: Phase 3规划

**评估GPU加速可行性**
**设计分布式架构**
**制定迁移方案**

---

## 📊 监控指标

### 关键指标

```yaml
写入性能:
  - 吞吐量 (条/秒)
  - P50/P90/P99 延迟
  - 成功率
  - 失败率和错误类型

资源使用:
  - CPU使用率
  - 内存使用
  - 数据库连接数
  - 磁盘I/O

业务指标:
  - 总记忆数
  - 活跃用户数
  - 查询QPS
  - 平均响应时间
```

### 监控工具

```yaml
指标收集: Prometheus
可视化: Grafana
日志: Loki
追踪: Jaeger
```

---

## ✅ 成功标准

### Phase 1（短期）
- [ ] 批量API实现
- [ ] 脚本并发优化
- [ ] 10K写入 < 2分钟
- [ ] 文档更新

### Phase 2（中期）
- [ ] 异步队列实现
- [ ] 批量处理优化
- [ ] 10K写入 < 20秒
- [ ] 集成测试

### Phase 3（长期）
- [ ] GPU加速评估
- [ ] 分布式架构设计
- [ ] 可扩展性验证
- [ ] 生产级部署

---

**状态**: 📋 待实施  
**优先级**: 🔴 P0 (Phase 1), 🟡 P1 (Phase 2), 🟢 P2 (Phase 3)  
**负责人**: TBD  
**开始日期**: 2025-11-08

