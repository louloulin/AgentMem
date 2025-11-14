# AgentMem 性能全面改造计划

**创建日期**: 2025-11-14  
**目标**: 对标 Mem0，实现工业级性能，消除 Mock，使用真实验证  
**当前状态**: ⚠️ 性能严重不足，大量 Mock 实现

---

## 📊 性能对比分析

### AgentMem vs Mem0 vs 竞品

| 指标 | AgentMem (当前) | Mem0 | OpenAI Memory | MemGPT | 目标 |
|------|----------------|------|---------------|--------|------|
| **记忆检索 QPS** | 2,430 | **10,000+** | 8,000 | 5,000 | **15,000+** |
| **P95 延迟** | 20-34ms | **15ms** | 25ms | 30ms | **<15ms** |
| **并发用户** | 100 (测试) | **10,000+** | 5,000 | 3,000 | **20,000+** |
| **准确率** | 未测试 | **26% 领先** | 基准 | -8% | **>Mem0** |
| **Token 节省** | 未优化 | **90%** | 60% | 85% | **95%** |
| **图推理 QPS** | 29.47 | **500+** | N/A | 200 | **1,000+** |
| **批量操作 QPS** | 36.66 | **2,000+** | N/A | 800 | **3,000+** |

### 关键发现

#### ❌ AgentMem 的严重问题

1. **性能差距巨大**
   - 记忆检索 QPS 仅为 Mem0 的 24%
   - 图推理性能仅为目标的 3%
   - 批量操作性能仅为目标的 1.2%

2. **大量 Mock 实现**
   - ✅ LibSQL 存储：真实实现
   - ✅ PostgreSQL 存储：真实实现
   - ⚠️ LanceDB 向量存储：**部分 Mock**
   - ❌ 压测工具：**100% Mock**（所有场景都是 `tokio::time::sleep` 模拟）
   - ❌ 向量搜索：**Mock 嵌入**（`generate_mock_embedding`）
   - ❌ 图推理：**Mock 延迟**（`simulate_graph_query`）
   - ❌ 批量操作：**Mock 处理**（`simulate_batch_operation`）

3. **并发能力不足**
   - 仅测试了 100 并发用户
   - 实际生产需要支持 10,000+ 并发
   - 缺少真实的并发压力测试

#### ✅ Mem0 的优势

1. **两阶段记忆管线**
   - 快速提取 + 异步处理
   - 图增强变体（Mem0g）提供更强的时序推理

2. **生产级性能**
   - 10,000+ QPS
   - P95 延迟 15ms
   - 支持 10,000+ 并发用户

3. **高准确率**
   - 比 OpenAI Memory 高 26%
   - 90% Token 节省
   - 91% 更低的 P95 延迟

---

## 🔍 根本原因分析

### 1. Mock 实现的危害

**当前 Mock 代码示例**：

```rust
// tools/comprehensive-stress-test/src/scenarios/memory_creation.rs
async fn simulate_memory_creation(index: usize) -> bool {
    // ❌ 完全 Mock，没有真实数据库操作
    let delay_ms = 5 + (index % 20) as u64; // 5-25ms
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    index % 100 != 0  // 模拟 99% 成功率
}

// tools/comprehensive-stress-test/src/scenarios/memory_retrieval.rs
async fn simulate_vector_search(dataset_size: usize, query_index: usize) -> bool {
    // ❌ 完全 Mock，没有真实向量搜索
    let base_delay = 10;
    let scale_factor = (dataset_size as f64).log10() as u64;
    let delay_ms = base_delay + scale_factor + (query_index % 10) as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    query_index % 200 != 0
}

// examples/embedded-mode-demo/src/vector_search.rs
fn generate_mock_embedding(text: &str) -> Vec<f32> {
    // ❌ Mock 嵌入，不是真实的语义向量
    let mut vector = vec![0.0; 1536];
    for (i, byte) in text.bytes().enumerate() {
        let idx = (i * 7 + byte as usize) % 1536;
        vector[idx] += 0.1;
    }
    // 归一化...
    vector
}
```

**问题**：
- Mock 延迟不能反映真实性能瓶颈
- Mock 成功率掩盖了真实错误
- Mock 嵌入无法测试语义搜索质量
- 无法发现真实的数据库、网络、I/O 瓶颈

### 2. 架构瓶颈

**当前架构问题**：

1. **缺少连接池优化**
   - PostgreSQL 连接池配置不当
   - LibSQL 单连接模式
   - 向量数据库连接未复用

2. **缺少批量优化**
   - 单条插入，没有批量插入
   - 没有批量向量搜索
   - 没有批量嵌入生成

3. **缺少缓存层**
   - 没有查询结果缓存
   - 没有嵌入向量缓存
   - 没有热点数据缓存

4. **缺少异步优化**
   - 同步等待数据库操作
   - 没有异步批处理
   - 没有流式处理

### 3. 向量数据库问题

**LanceDB 实现不完整**：

```rust
// crates/agent-mem-storage/src/backends/lancedb.rs
async fn add_vectors(&self, _vectors: Vec<VectorData>) -> Result<Vec<String>> {
    // ❌ 未实现
    Err(AgentMemError::llm_error(
        "LanceDB provider not fully implemented yet",
    ))
}
```

**问题**：
- LanceDB 核心功能未实现
- 向量搜索性能未优化
- 缺少索引优化（HNSW, IVF）

---

## 🎯 改造计划

### Phase 1: 消除 Mock，真实验证 (P0 - 本周)

#### 1.1 真实数据库压测

**目标**: 使用真实的 PostgreSQL + LanceDB 进行压测

**任务**:
- [ ] 配置真实的 PostgreSQL 数据库
- [ ] 配置真实的 LanceDB 向量数据库
- [ ] 实现真实的记忆创建（插入数据库）
- [ ] 实现真实的记忆检索（查询数据库）
- [ ] 实现真实的向量搜索（LanceDB）

**代码改造**:
```rust
// 改造前 (Mock)
async fn simulate_memory_creation(index: usize) -> bool {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    index % 100 != 0
}

// 改造后 (真实)
async fn real_memory_creation(
    pool: &PgPool,
    memory: &Memory,
) -> Result<Memory> {
    let result = sqlx::query_as::<_, DbMemory>(
        "INSERT INTO memories (...) VALUES (...) RETURNING *"
    )
    .bind(&memory.id)
    // ... 真实的数据库插入
    .fetch_one(pool)
    .await?;
    Ok(result.into())
}
```

#### 1.2 真实嵌入生成

**目标**: 使用真实的嵌入模型替代 Mock

**任务**:
- [ ] 集成 FastEmbed 本地嵌入模型
- [ ] 实现批量嵌入生成
- [ ] 实现嵌入缓存
- [ ] 测试嵌入质量

**代码改造**:
```rust
// 改造前 (Mock)
fn generate_mock_embedding(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0; 1536];
    // ... Mock 逻辑
    vector
}

// 改造后 (真实)
async fn generate_real_embedding(
    embedder: &LocalEmbedder,
    text: &str,
) -> Result<Vec<f32>> {
    embedder.embed(text).await
}
```

#### 1.3 完善 LanceDB 实现

**目标**: 实现完整的 LanceDB 向量存储

**任务**:
- [ ] 实现 `add_vectors`
- [ ] 实现 `search_vectors`
- [ ] 实现 `delete_vectors`
- [ ] 实现 `update_vectors`
- [ ] 添加 HNSW 索引优化

**参考**: `crates/agent-mem-storage/src/backends/lancedb_store.rs` (已有真实实现)

### Phase 2: 性能优化 (P0 - 下周)

#### 2.1 数据库连接池优化

**目标**: 优化 PostgreSQL 连接池配置

**任务**:
- [ ] 配置连接池大小（min: 10, max: 100）
- [ ] 配置连接超时（5s）
- [ ] 配置空闲超时（10min）
- [ ] 实现连接健康检查

**配置**:
```rust
let pool = PgPoolOptions::new()
    .min_connections(10)
    .max_connections(100)
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
    .test_before_acquire(true)
    .connect(&database_url)
    .await?;
```

#### 2.2 批量操作优化

**目标**: 实现批量插入、批量查询、批量嵌入

**任务**:
- [ ] 实现批量记忆插入（100 条/批）
- [ ] 实现批量向量搜索（50 查询/批）
- [ ] 实现批量嵌入生成（32 文本/批）
- [ ] 实现批量删除和更新

**代码**:
```rust
// 批量插入
async fn batch_insert_memories(
    pool: &PgPool,
    memories: Vec<Memory>,
) -> Result<Vec<Memory>> {
    let mut query_builder = QueryBuilder::new(
        "INSERT INTO memories (...) "
    );
    
    for memory in &memories {
        query_builder.push_values([&memory.id, ...]);
    }
    
    query_builder.build().execute(pool).await?;
    Ok(memories)
}
```

#### 2.3 缓存层实现

**目标**: 实现多级缓存

**任务**:
- [ ] L1: 内存缓存（LRU, 10K 条）
- [ ] L2: Redis 缓存（100K 条）
- [ ] 嵌入向量缓存（1M 向量）
- [ ] 查询结果缓存（TTL: 5min）

**架构**:
```
查询 -> L1 Cache (内存) -> L2 Cache (Redis) -> Database
         ↓ 命中率 95%      ↓ 命中率 90%      ↓ 命中率 5%
```

### Phase 3: 图推理优化 (P0 - 下周)

#### 3.1 图索引实现

**目标**: 实现高性能图索引

**任务**:
- [ ] 实现邻接表索引
- [ ] 实现 CSR (Compressed Sparse Row) 格式
- [ ] 实现图缓存
- [ ] 优化图遍历算法（BFS/DFS）

**预期提升**: 3-5x

#### 3.2 图查询优化

**目标**: 优化图查询性能

**任务**:
- [ ] 实现查询计划优化
- [ ] 实现子图缓存
- [ ] 实现增量更新
- [ ] 实现并行图遍历

**预期提升**: 2-3x

### Phase 4: 大规模并发测试 (P1 - 下周)

#### 4.1 并发压测

**目标**: 测试 10,000+ 并发用户

**任务**:
- [ ] 实现 1,000 并发压测
- [ ] 实现 5,000 并发压测
- [ ] 实现 10,000 并发压测
- [ ] 实现 20,000 并发压测

#### 4.2 负载均衡

**目标**: 实现负载均衡和水平扩展

**任务**:
- [ ] 实现数据库读写分离
- [ ] 实现向量数据库分片
- [ ] 实现应用层负载均衡
- [ ] 实现自动扩缩容

---

## 📈 性能目标

### 短期目标 (1 周)

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| 记忆检索 QPS | 2,430 | 10,000 | 4.1x |
| P95 延迟 | 20-34ms | <20ms | 1.7x |
| 图推理 QPS | 29.47 | 500 | 17x |
| 批量操作 QPS | 36.66 | 2,000 | 54x |

### 中期目标 (2 周)

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| 记忆检索 QPS | 2,430 | 15,000 | 6.2x |
| P95 延迟 | 20-34ms | <15ms | 2.3x |
| 并发用户 | 100 | 10,000 | 100x |
| 图推理 QPS | 29.47 | 1,000 | 34x |

### 长期目标 (1 月)

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| 记忆检索 QPS | 2,430 | 20,000+ | 8.2x |
| P95 延迟 | 20-34ms | <10ms | 3.4x |
| 并发用户 | 100 | 20,000+ | 200x |
| 准确率 | 未测试 | >Mem0 | - |

---

## 🛠️ 实施步骤

### Week 1: 消除 Mock + 基础优化

**Day 1-2**: 真实数据库压测
- 配置 PostgreSQL + LanceDB
- 改造压测工具使用真实数据库
- 运行基准测试

**Day 3-4**: 真实嵌入 + LanceDB 完善
- 集成 FastEmbed
- 完善 LanceDB 实现
- 测试向量搜索性能

**Day 5-7**: 连接池 + 批量优化
- 优化数据库连接池
- 实现批量操作
- 运行性能对比测试

### Week 2: 高级优化 + 大规模测试

**Day 1-3**: 缓存层 + 图优化
- 实现多级缓存
- 优化图推理
- 测试缓存命中率

**Day 4-7**: 大规模并发测试
- 1,000 并发测试
- 5,000 并发测试
- 10,000 并发测试
- 性能调优

---

## 📝 验收标准

### 功能验收

- [ ] 所有压测场景使用真实数据库
- [ ] 所有嵌入使用真实模型
- [ ] LanceDB 完整实现
- [ ] 批量操作完整实现
- [ ] 缓存层完整实现

### 性能验收

- [ ] 记忆检索 QPS > 10,000
- [ ] P95 延迟 < 20ms
- [ ] 图推理 QPS > 500
- [ ] 批量操作 QPS > 2,000
- [ ] 支持 10,000+ 并发用户

### 质量验收

- [ ] 准确率测试通过
- [ ] 稳定性测试通过（24 小时）
- [ ] 内存泄漏测试通过
- [ ] 错误率 < 0.1%

---

## 🔧 技术实施细节

### 真实压测工具架构

**新架构设计**:

```
tools/real-stress-test/
├── Cargo.toml
├── src/
│   ├── main.rs                    # CLI 入口
│   ├── config.rs                  # 配置管理
│   ├── database/
│   │   ├── mod.rs
│   │   ├── postgres.rs            # PostgreSQL 连接池
│   │   ├── lancedb.rs             # LanceDB 连接
│   │   └── redis.rs               # Redis 缓存
│   ├── embeddings/
│   │   ├── mod.rs
│   │   ├── fastembed.rs           # FastEmbed 集成
│   │   └── cache.rs               # 嵌入缓存
│   ├── scenarios/
│   │   ├── mod.rs
│   │   ├── memory_creation.rs     # 真实记忆创建
│   │   ├── memory_retrieval.rs    # 真实记忆检索
│   │   ├── vector_search.rs       # 真实向量搜索
│   │   ├── graph_reasoning.rs     # 真实图推理
│   │   ├── batch_operations.rs    # 真实批量操作
│   │   └── concurrent_ops.rs      # 真实并发操作
│   ├── metrics/
│   │   ├── mod.rs
│   │   ├── collector.rs           # 指标收集
│   │   └── reporter.rs            # 报告生成
│   └── utils/
│       ├── mod.rs
│       └── data_generator.rs      # 测试数据生成
└── tests/
    └── integration_tests.rs
```

### 关键代码实现

#### 1. 真实记忆创建

```rust
// tools/real-stress-test/src/scenarios/memory_creation.rs

use agent_mem_core::storage::libsql::memory_repository::LibSqlMemoryRepository;
use agent_mem_traits::{Memory, MemoryRepositoryTrait};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct RealMemoryCreationTest {
    pg_pool: Arc<PgPool>,
    libsql_repo: Arc<LibSqlMemoryRepository>,
    embedder: Arc<LocalEmbedder>,
}

impl RealMemoryCreationTest {
    pub async fn run(
        &self,
        concurrency: usize,
        total: usize,
    ) -> Result<StressTestStats> {
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let stats = Arc::new(StatsCollector::new());

        let mut handles = Vec::new();

        for i in 0..total {
            let permit = semaphore.clone().acquire_owned().await?;
            let pg_pool = self.pg_pool.clone();
            let embedder = self.embedder.clone();
            let stats = stats.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let start = Instant::now();

                // 生成真实的测试数据
                let content = format!("Test memory content {}", i);

                // 生成真实的嵌入向量
                let embedding = embedder.embed(&content).await?;

                // 创建 Memory V4 对象
                let memory = Memory {
                    id: Uuid::new_v4().to_string(),
                    content,
                    embedding: Some(embedding),
                    memory_type: MemoryType::Episodic,
                    importance: 0.8,
                    created_at: Utc::now(),
                    // ... 其他字段
                };

                // 真实的数据库插入
                let result = sqlx::query(
                    "INSERT INTO memories (id, content, embedding, ...)
                     VALUES ($1, $2, $3, ...)"
                )
                .bind(&memory.id)
                .bind(&memory.content)
                .bind(&memory.embedding)
                .execute(pg_pool.as_ref())
                .await;

                let duration = start.elapsed();
                let success = result.is_ok();

                stats.record_operation(duration, success).await;

                Ok::<_, anyhow::Error>(())
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await??;
        }

        Ok(stats.get_stats().await)
    }
}
```

#### 2. 真实向量搜索

```rust
// tools/real-stress-test/src/scenarios/vector_search.rs

use agent_mem_storage::backends::lancedb_store::LanceDBStore;
use agent_mem_traits::VectorStore;

pub struct RealVectorSearchTest {
    lancedb: Arc<LanceDBStore>,
    embedder: Arc<LocalEmbedder>,
    cache: Arc<EmbeddingCache>,
}

impl RealVectorSearchTest {
    pub async fn run(
        &self,
        dataset_size: usize,
        concurrency: usize,
    ) -> Result<StressTestStats> {
        // 1. 准备测试数据集
        info!("准备 {} 条测试数据...", dataset_size);
        self.prepare_dataset(dataset_size).await?;

        // 2. 执行并发搜索
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let stats = Arc::new(StatsCollector::new());

        let total_queries = 1000;
        let mut handles = Vec::new();

        for i in 0..total_queries {
            let permit = semaphore.clone().acquire_owned().await?;
            let lancedb = self.lancedb.clone();
            let embedder = self.embedder.clone();
            let cache = self.cache.clone();
            let stats = stats.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let start = Instant::now();

                // 生成查询文本
                let query_text = format!("Search query {}", i);

                // 检查缓存
                let query_embedding = if let Some(cached) = cache.get(&query_text).await {
                    cached
                } else {
                    // 生成真实的查询嵌入
                    let emb = embedder.embed(&query_text).await?;
                    cache.set(&query_text, emb.clone()).await;
                    emb
                };

                // 真实的向量搜索
                let results = lancedb.search_vectors(
                    query_embedding,
                    10,  // top-k
                    Some(0.7),  // threshold
                ).await;

                let duration = start.elapsed();
                let success = results.is_ok();

                stats.record_operation(duration, success).await;

                Ok::<_, anyhow::Error>(())
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await??;
        }

        Ok(stats.get_stats().await)
    }

    async fn prepare_dataset(&self, size: usize) -> Result<()> {
        // 批量生成和插入测试数据
        let batch_size = 100;

        for batch_start in (0..size).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(size);
            let mut vectors = Vec::new();

            for i in batch_start..batch_end {
                let content = format!("Dataset item {}", i);
                let embedding = self.embedder.embed(&content).await?;

                vectors.push(VectorData {
                    id: format!("vec-{}", i),
                    vector: embedding,
                    metadata: HashMap::from([
                        ("content".to_string(), content.into()),
                        ("index".to_string(), i.into()),
                    ]),
                });
            }

            // 批量插入
            self.lancedb.add_vectors(vectors).await?;
        }

        info!("数据集准备完成: {} 条", size);
        Ok(())
    }
}
```

#### 3. 批量操作优化

```rust
// tools/real-stress-test/src/scenarios/batch_operations.rs

pub struct RealBatchOperationsTest {
    pg_pool: Arc<PgPool>,
    embedder: Arc<LocalEmbedder>,
}

impl RealBatchOperationsTest {
    pub async fn run(
        &self,
        batch_size: usize,
    ) -> Result<StressTestStats> {
        let stats = Arc::new(StatsCollector::new());
        let total_batches = 100;

        for batch_idx in 0..total_batches {
            let start = Instant::now();

            // 生成批量数据
            let mut memories = Vec::new();
            let mut contents = Vec::new();

            for i in 0..batch_size {
                let content = format!("Batch {} item {}", batch_idx, i);
                contents.push(content.clone());

                memories.push(Memory {
                    id: Uuid::new_v4().to_string(),
                    content,
                    // ... 其他字段
                });
            }

            // 批量生成嵌入（FastEmbed 支持批量）
            let embeddings = self.embedder.embed_batch(&contents).await?;

            // 更新嵌入
            for (memory, embedding) in memories.iter_mut().zip(embeddings) {
                memory.embedding = Some(embedding);
            }

            // 批量插入数据库
            let result = self.batch_insert_memories(&memories).await;

            let duration = start.elapsed();
            let success = result.is_ok();

            stats.record_operation(duration, success).await;
        }

        Ok(stats.get_stats().await)
    }

    async fn batch_insert_memories(
        &self,
        memories: &[Memory],
    ) -> Result<()> {
        // 使用 QueryBuilder 进行批量插入
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO memories (id, content, embedding, memory_type, importance, created_at) "
        );

        query_builder.push_values(memories, |mut b, memory| {
            b.push_bind(&memory.id)
             .push_bind(&memory.content)
             .push_bind(&memory.embedding)
             .push_bind(memory.memory_type.to_string())
             .push_bind(memory.importance)
             .push_bind(memory.created_at);
        });

        query_builder.build().execute(self.pg_pool.as_ref()).await?;

        Ok(())
    }
}
```

#### 4. 嵌入缓存实现

```rust
// tools/real-stress-test/src/embeddings/cache.rs

use dashmap::DashMap;
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EmbeddingCache {
    // L1: 内存 LRU 缓存
    l1_cache: Arc<RwLock<LruCache<String, Vec<f32>>>>,
    // L2: DashMap 用于并发访问
    l2_cache: Arc<DashMap<String, Vec<f32>>>,
    // 统计信息
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl EmbeddingCache {
    pub fn new(l1_capacity: usize, l2_capacity: usize) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(LruCache::new(l1_capacity))),
            l2_cache: Arc::new(DashMap::with_capacity(l2_capacity)),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<f32>> {
        // 先查 L1
        {
            let mut l1 = self.l1_cache.write().await;
            if let Some(value) = l1.get(key) {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(value.clone());
            }
        }

        // 再查 L2
        if let Some(value) = self.l2_cache.get(key) {
            // 提升到 L1
            let mut l1 = self.l1_cache.write().await;
            l1.put(key.to_string(), value.clone());
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Some(value.clone());
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub async fn set(&self, key: &str, value: Vec<f32>) {
        // 同时写入 L1 和 L2
        let mut l1 = self.l1_cache.write().await;
        l1.put(key.to_string(), value.clone());
        self.l2_cache.insert(key.to_string(), value);
    }

    pub fn get_hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}
```

### 数据库配置优化

#### PostgreSQL 配置

```toml
# .env
DATABASE_URL=postgresql://user:pass@localhost:5432/agentmem

# PostgreSQL 连接池配置
DB_MIN_CONNECTIONS=10
DB_MAX_CONNECTIONS=100
DB_ACQUIRE_TIMEOUT=5
DB_IDLE_TIMEOUT=600
DB_MAX_LIFETIME=1800

# PostgreSQL 性能配置
# postgresql.conf
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB
max_worker_processes = 8
max_parallel_workers_per_gather = 4
max_parallel_workers = 8
```

#### LanceDB 配置

```rust
// LanceDB 优化配置
let lancedb_config = LanceDBConfig {
    path: "./data/vectors.lance",
    table_name: "embeddings",
    // 向量维度
    dimension: 1536,
    // 索引类型: IVF_PQ (倒排文件 + 乘积量化)
    index_type: IndexType::IVF_PQ,
    // IVF 参数
    nlist: 1024,  // 聚类中心数
    nprobe: 32,   // 搜索的聚类数
    // PQ 参数
    m: 64,        // 子向量数
    nbits: 8,     // 每个子向量的位数
    // 缓存配置
    cache_size_mb: 512,
};
```

---

## 📊 性能监控和分析

### 监控指标

```rust
pub struct PerformanceMetrics {
    // 吞吐量
    pub qps: f64,
    pub ops_per_second: f64,

    // 延迟
    pub latency_p50: f64,
    pub latency_p90: f64,
    pub latency_p95: f64,
    pub latency_p99: f64,
    pub latency_p999: f64,

    // 资源使用
    pub cpu_usage: f64,
    pub memory_mb: f64,
    pub disk_io_mb_per_sec: f64,
    pub network_mb_per_sec: f64,

    // 数据库
    pub db_connections_active: usize,
    pub db_connections_idle: usize,
    pub db_query_time_ms: f64,

    // 缓存
    pub cache_hit_rate: f64,
    pub cache_size_mb: f64,

    // 错误
    pub error_rate: f64,
    pub timeout_rate: f64,
}
```

### 性能分析工具

```bash
# 1. 数据库性能分析
psql -d agentmem -c "
SELECT
    query,
    calls,
    total_time,
    mean_time,
    max_time
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 20;
"

# 2. 慢查询日志
tail -f /var/log/postgresql/postgresql-slow.log

# 3. 系统资源监控
htop
iotop
nethogs

# 4. 火焰图生成
cargo flamegraph --bin real-stress-test -- memory-retrieval --concurrency 1000

# 5. 性能剖析
cargo build --release
perf record -g ./target/release/real-stress-test memory-retrieval
perf report
```

---

**下一步**: 立即开始 Phase 1.1 - 真实数据库压测

**优先级**:
1. P0: 创建 `tools/real-stress-test` 项目
2. P0: 实现真实记忆创建压测
3. P0: 实现真实向量搜索压测
4. P0: 配置 PostgreSQL + LanceDB
5. P1: 实现批量操作优化
6. P1: 实现缓存层

