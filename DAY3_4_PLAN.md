# Day 3-4 工作计划 - 性能优化和可观测性

**日期**: 2025-10-09 - 2025-10-10  
**目标**: 完善智能功能集成，添加性能优化和可观测性  
**前置条件**: Day 1-2 架构重构完成

---

## 🎯 总体目标

1. ✅ 添加性能优化机制
2. ✅ 集成可观测性（Prometheus 指标）
3. ✅ 实现缓存机制
4. ✅ 编写集成测试
5. ✅ 完善文档

---

## 📋 详细任务

### Day 3: 性能优化和缓存

#### 任务 3.1: 实现缓存机制 (4 小时)

**目标**: 为智能组件添加 LRU 缓存，减少 LLM 调用

**子任务**:

1. **设计缓存接口** (30 分钟)
   ```rust
   // crates/agent-mem-traits/src/cache.rs
   pub trait IntelligenceCache: Send + Sync {
       async fn get_facts(&self, key: &str) -> Option<Vec<ExtractedFact>>;
       async fn set_facts(&self, key: &str, facts: Vec<ExtractedFact>);
       async fn get_decision(&self, key: &str) -> Option<MemoryDecision>;
       async fn set_decision(&self, key: &str, decision: MemoryDecision);
   }
   ```

2. **实现 LRU 缓存** (1.5 小时)
   ```rust
   // crates/agent-mem-intelligence/src/cache.rs
   pub struct LRUIntelligenceCache {
       facts_cache: Arc<RwLock<LruCache<String, Vec<ExtractedFact>>>>,
       decision_cache: Arc<RwLock<LruCache<String, MemoryDecision>>>,
       max_size: usize,
   }
   ```

3. **集成缓存到 FactExtractor** (1 小时)
   ```rust
   impl FactExtractor {
       pub fn with_cache(self, cache: Arc<dyn IntelligenceCache>) -> Self {
           // ...
       }
       
       async fn extract_facts(&self, messages: &[Message]) -> Result<Vec<ExtractedFact>> {
           let cache_key = self.compute_cache_key(messages);
           
           // 尝试从缓存获取
           if let Some(cached) = self.cache.get_facts(&cache_key).await {
               return Ok(cached);
           }
           
           // 调用 LLM
           let facts = self.extract_facts_impl(messages).await?;
           
           // 存入缓存
           self.cache.set_facts(&cache_key, facts.clone()).await;
           
           Ok(facts)
       }
   }
   ```

4. **集成缓存到 DecisionEngine** (1 小时)
   ```rust
   impl DecisionEngine {
       pub fn with_cache(self, cache: Arc<dyn IntelligenceCache>) -> Self {
           // ...
       }
   }
   ```

**验收标准**:
- ✅ 缓存命中率 > 50% (在重复查询场景)
- ✅ LLM 调用次数减少 50%+
- ✅ 响应时间减少 30%+

---

#### 任务 3.2: 批处理优化 (2 小时)

**目标**: 支持批量处理记忆，提高吞吐量

**子任务**:

1. **添加批处理接口** (30 分钟)
   ```rust
   impl MemoryManager {
       pub async fn add_memories_batch(
           &self,
           memories: Vec<AddMemoryRequest>,
       ) -> Result<Vec<String>> {
           // 批量提取事实
           // 批量做决策
           // 批量执行操作
       }
   }
   ```

2. **实现批量事实提取** (45 分钟)
   ```rust
   impl FactExtractor {
       pub async fn extract_facts_batch(
           &self,
           messages_batch: Vec<Vec<Message>>,
       ) -> Result<Vec<Vec<ExtractedFact>>> {
           // 并发提取
           let futures: Vec<_> = messages_batch.iter()
               .map(|messages| self.extract_facts(messages))
               .collect();
           
           futures::future::try_join_all(futures).await
       }
   }
   ```

3. **实现批量决策** (45 分钟)
   ```rust
   impl DecisionEngine {
       pub async fn decide_batch(
           &self,
           facts: Vec<ExtractedFact>,
           existing_memories: &[MemoryItem],
       ) -> Result<Vec<MemoryDecision>> {
           // 并发决策
       }
   }
   ```

**验收标准**:
- ✅ 批处理吞吐量 > 单个处理的 5 倍
- ✅ 支持至少 100 个并发请求

---

#### 任务 3.3: 性能基准测试 (2 小时)

**目标**: 建立性能基准，验证优化效果

**子任务**:

1. **创建基准测试** (1 小时)
   ```rust
   // benches/intelligent_integration.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   
   fn bench_fact_extraction(c: &mut Criterion) {
       c.bench_function("extract_facts", |b| {
           b.iter(|| {
               // 测试事实提取性能
           });
       });
   }
   
   fn bench_decision_making(c: &mut Criterion) {
       c.bench_function("make_decision", |b| {
           b.iter(|| {
               // 测试决策性能
           });
       });
   }
   
   criterion_group!(benches, bench_fact_extraction, bench_decision_making);
   criterion_main!(benches);
   ```

2. **运行基准测试** (30 分钟)
   ```bash
   cargo bench --package agent-mem-core
   ```

3. **分析结果并优化** (30 分钟)

**验收标准**:
- ✅ 事实提取: < 500ms (单条消息)
- ✅ 决策制定: < 300ms (单个事实)
- ✅ 完整流程: < 1s (单条记忆)

---

### Day 4: 可观测性和集成测试

#### 任务 4.1: 集成 Prometheus 指标 (3 小时)

**目标**: 添加详细的性能和业务指标

**子任务**:

1. **定义指标** (30 分钟)
   ```rust
   // crates/agent-mem-core/src/metrics.rs
   use prometheus::{Counter, Histogram, IntGauge};
   
   lazy_static! {
       // 事实提取指标
       pub static ref FACT_EXTRACTION_TOTAL: Counter = 
           Counter::new("agentmem_fact_extraction_total", "Total fact extractions").unwrap();
       
       pub static ref FACT_EXTRACTION_DURATION: Histogram = 
           Histogram::new("agentmem_fact_extraction_duration_seconds", "Fact extraction duration").unwrap();
       
       pub static ref FACTS_EXTRACTED: Counter = 
           Counter::new("agentmem_facts_extracted_total", "Total facts extracted").unwrap();
       
       // 决策引擎指标
       pub static ref DECISIONS_MADE: Counter = 
           Counter::new("agentmem_decisions_made_total", "Total decisions made").unwrap();
       
       pub static ref DECISION_DURATION: Histogram = 
           Histogram::new("agentmem_decision_duration_seconds", "Decision making duration").unwrap();
       
       // 缓存指标
       pub static ref CACHE_HITS: Counter = 
           Counter::new("agentmem_cache_hits_total", "Total cache hits").unwrap();
       
       pub static ref CACHE_MISSES: Counter = 
           Counter::new("agentmem_cache_misses_total", "Total cache misses").unwrap();
       
       // 操作指标
       pub static ref MEMORY_ACTIONS: Counter = 
           Counter::new("agentmem_memory_actions_total", "Total memory actions").unwrap();
   }
   ```

2. **集成指标到代码** (1.5 小时)
   ```rust
   impl FactExtractor {
       async fn extract_facts(&self, messages: &[Message]) -> Result<Vec<ExtractedFact>> {
           let timer = FACT_EXTRACTION_DURATION.start_timer();
           FACT_EXTRACTION_TOTAL.inc();
           
           let result = self.extract_facts_impl(messages).await;
           
           if let Ok(ref facts) = result {
               FACTS_EXTRACTED.inc_by(facts.len() as f64);
           }
           
           timer.observe_duration();
           result
       }
   }
   ```

3. **添加指标导出端点** (1 小时)
   ```rust
   // crates/agent-mem-server/src/metrics.rs
   use prometheus::{Encoder, TextEncoder};
   use warp::Filter;
   
   pub fn metrics_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
       warp::path!("metrics")
           .and(warp::get())
           .map(|| {
               let encoder = TextEncoder::new();
               let metric_families = prometheus::gather();
               let mut buffer = vec![];
               encoder.encode(&metric_families, &mut buffer).unwrap();
               String::from_utf8(buffer).unwrap()
           })
   }
   ```

**验收标准**:
- ✅ 所有关键操作都有指标
- ✅ 指标可以通过 /metrics 端点访问
- ✅ 可以在 Grafana 中可视化

---

#### 任务 4.2: 添加结构化日志 (1 小时)

**目标**: 使用 tracing 添加详细的结构化日志

**子任务**:

1. **添加 tracing spans** (30 分钟)
   ```rust
   use tracing::{info, warn, error, instrument};
   
   impl MemoryManager {
       #[instrument(skip(self), fields(agent_id, user_id))]
       pub async fn add_memory_intelligent(
           &self,
           agent_id: String,
           user_id: Option<String>,
           content: String,
           // ...
       ) -> Result<String> {
           info!("Starting intelligent memory addition");
           
           // 提取事实
           let facts = self.extract_facts_from_content(&content).await?;
           info!(facts_count = facts.len(), "Facts extracted");
           
           // ...
       }
   }
   ```

2. **配置日志输出** (30 分钟)
   ```rust
   use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
   
   tracing_subscriber::registry()
       .with(tracing_subscriber::EnvFilter::new(
           std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
       ))
       .with(tracing_subscriber::fmt::layer().json())
       .init();
   ```

**验收标准**:
- ✅ 所有关键操作都有日志
- ✅ 日志包含上下文信息 (agent_id, user_id, etc.)
- ✅ 支持 JSON 格式输出

---

#### 任务 4.3: 编写集成测试 (2 小时)

**目标**: 编写端到端集成测试

**子任务**:

1. **创建测试框架** (30 分钟)
   ```rust
   // tests/intelligent_integration_test.rs
   use agent_mem_core::MemoryManager;
   use agent_mem_intelligence::{FactExtractor, MemoryDecisionEngine};
   use agent_mem_llm::providers::LocalTestProvider;
   
   async fn setup_test_manager() -> MemoryManager {
       let llm_provider = Arc::new(LocalTestProvider::new());
       let fact_extractor = Arc::new(FactExtractor::new(llm_provider.clone()));
       let decision_engine = Arc::new(MemoryDecisionEngine::new(llm_provider.clone()));
       
       MemoryManager::with_intelligent_components(
           MemoryConfig::default(),
           Some(fact_extractor),
           Some(decision_engine),
           Some(llm_provider),
       )
   }
   ```

2. **编写测试用例** (1 小时)
   ```rust
   #[tokio::test]
   async fn test_intelligent_memory_flow() {
       let manager = setup_test_manager().await;
       
       // 测试添加记忆
       let memory_id = manager.add_memory(
           "agent1".to_string(),
           Some("user1".to_string()),
           "我喜欢 Rust 编程".to_string(),
           None,
           None,
           None,
       ).await.unwrap();
       
       assert!(!memory_id.is_empty());
       
       // 测试更新记忆
       let memory_id2 = manager.add_memory(
           "agent1".to_string(),
           Some("user1".to_string()),
           "我特别喜欢 Rust 的异步编程".to_string(),
           None,
           None,
           None,
       ).await.unwrap();
       
       // 验证决策引擎是否正确工作
       // (应该更新而不是添加新记忆)
   }
   ```

3. **运行测试** (30 分钟)
   ```bash
   cargo test --package agent-mem-core --test intelligent_integration_test
   ```

**验收标准**:
- ✅ 所有测试通过
- ✅ 测试覆盖率 > 80%
- ✅ 包含正常和异常场景

---

## 📊 成功指标

### 性能指标

| 指标 | 目标 | 测量方法 |
|------|------|---------|
| 事实提取延迟 | < 500ms | Prometheus histogram |
| 决策制定延迟 | < 300ms | Prometheus histogram |
| 完整流程延迟 | < 1s | Prometheus histogram |
| 缓存命中率 | > 50% | cache_hits / (cache_hits + cache_misses) |
| LLM 调用减少 | > 50% | 对比启用/禁用缓存 |
| 批处理吞吐量 | > 5x | 对比单个/批量处理 |

### 质量指标

| 指标 | 目标 | 测量方法 |
|------|------|---------|
| 测试覆盖率 | > 80% | cargo tarpaulin |
| 文档完整性 | 100% | 手动检查 |
| 代码质量 | A | cargo clippy |

---

## 🎯 交付物

### Day 3

- [ ] LRU 缓存实现
- [ ] 批处理接口
- [ ] 性能基准测试
- [ ] 性能优化报告

### Day 4

- [ ] Prometheus 指标集成
- [ ] 结构化日志
- [ ] 集成测试套件
- [ ] 可观测性文档

---

## 📝 文档更新

- [ ] 更新 README.md (添加性能数据)
- [ ] 更新 INTELLIGENT_INTEGRATION_GUIDE.md (添加缓存和监控章节)
- [ ] 创建 PERFORMANCE_TUNING.md
- [ ] 创建 OBSERVABILITY.md

---

## 🔄 迭代计划

如果时间充裕，可以继续：

### 额外优化

- [ ] 实现智能预取
- [ ] 添加自适应缓存策略
- [ ] 实现请求合并 (request coalescing)
- [ ] 添加断路器模式

### 额外监控

- [ ] 添加分布式追踪 (OpenTelemetry)
- [ ] 添加错误追踪 (Sentry)
- [ ] 添加性能分析 (pprof)

---

**总结**: Day 3-4 将完善智能功能集成，添加生产级的性能优化和可观测性支持，为 MVP 发布做好准备。

