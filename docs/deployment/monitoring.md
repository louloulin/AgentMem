# AgentMem 性能监控指南

**版本**: v1.0  
**更新日期**: 2025-11-03  
**适用版本**: AgentMem 2.0+

---

## 📋 目录

1. [性能基准](#性能基准)
2. [本地性能测试](#本地性能测试)
3. [CI/CD集成](#cicd集成)
4. [性能回归检测](#性能回归检测)
5. [性能分析工具](#性能分析工具)
6. [监控指标](#监控指标)
7. [性能优化建议](#性能优化建议)

---

## 🎯 性能基准

### 核心操作性能目标

| 操作 | 目标延迟 | P95延迟 | P99延迟 | 状态 |
|------|---------|---------|---------|------|
| **记忆创建** | < 5ms | < 8ms | < 15ms | ✅ 达标 |
| **记忆检索** | < 3ms | < 5ms | < 10ms | ✅ 达标 |
| **语义搜索** | < 25ms | < 40ms | < 60ms | ✅ 达标 |
| **批量操作(100)** | < 100ms | < 150ms | < 200ms | ✅ 达标 |
| **图遍历(100节点)** | < 20ms | < 30ms | < 50ms | ✅ 达标 |
| **LLM调用** | < 2000ms | < 3000ms | < 5000ms | ⏳ 依赖外部 |

### 吞吐量目标

| 指标 | 目标 | 当前 | 状态 |
|------|------|------|------|
| **QPS (记忆操作)** | > 1000 req/s | TBD | ⏳ |
| **并发连接** | > 1000 | TBD | ⏳ |
| **WebSocket连接** | > 10000 | TBD | ⏳ |

### 资源使用目标

| 资源 | 目标 | 限制 | 状态 |
|------|------|------|------|
| **内存使用** | < 2GB | < 4GB | ✅ |
| **CPU使用** | < 50% | < 80% | ✅ |
| **磁盘I/O** | < 100MB/s | < 200MB/s | ✅ |
| **数据库连接** | < 50 | < 100 | ✅ |

---

## 🚀 本地性能测试

### 1. 运行完整benchmark套件

```bash
# 运行所有benchmark
./scripts/run_benchmarks.sh

# 查看报告
ls -lh target/benchmark-reports/

# 打开最新报告
open target/benchmark-reports/benchmark_*.md
```

### 2. 运行特定benchmark

```bash
# 核心记忆操作
cargo bench --package agent-mem-core --bench memory_operations

# 图推理性能
cargo bench --package agent-mem-core --bench graph_reasoning

# 数据库性能
cargo bench --package agent-mem-server --bench database_performance

# 向量搜索性能
cargo bench --package agent-mem-server --bench vector_performance
```

### 3. 性能对比

```bash
# 保存当前baseline
cargo bench -- --save-baseline before

# 进行代码修改
# ...

# 对比性能变化
cargo bench -- --baseline before

# 查看详细对比
open target/criterion/report/index.html
```

### 4. 生成火焰图

```bash
# 安装flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin agent-mem-server

# 查看火焰图
open flamegraph.svg
```

---

## 🔄 CI/CD集成

### GitHub Actions工作流

已配置的CI/CD性能测试：

1. **benchmark** - 完整基准测试套件
   - 触发条件: push到main/develop, PR, 每日定时
   - 输出: benchmark报告和Criterion数据
   - 自动评论PR结果

2. **regression** - 性能回归检测
   - 触发条件: PR
   - 对比: PR分支 vs main分支
   - 失败条件: 性能退化 > 10%

3. **continuous-profiling** - 持续性能分析
   - 触发条件: push到main
   - 输出: flamegraph火焰图

4. **publish-results** - 发布性能报告
   - 触发条件: push到main
   - 输出: GitHub Pages性能仪表板

### 查看CI/CD结果

```bash
# 查看最新的workflow运行
gh run list --workflow=performance.yml

# 下载artifacts
gh run download <run-id>

# 查看性能趋势
# https://your-org.github.io/agentmem/performance/
```

---

## 🔍 性能回归检测

### 自动检测

性能回归测试会自动在PR中运行：

```bash
# 本地运行回归测试
./scripts/performance_regression_test.sh

# 查看报告
cat target/regression-report.md
```

### 性能退化阈值

- **警告**: 性能退化 5-10%
- **失败**: 性能退化 > 10%
- **阻止合并**: 失败状态

### 手动对比

```bash
# 保存main分支baseline
git checkout main
cargo bench -- --save-baseline main

# 切换到feature分支
git checkout feature-branch

# 对比性能
cargo bench -- --baseline main

# 分析差异
open target/criterion/report/index.html
```

---

## 🛠️ 性能分析工具

### 1. Criterion (基准测试)

```bash
# 运行并生成HTML报告
cargo bench

# 查看报告
open target/criterion/report/index.html
```

**优势**:
- ✅ 统计学准确的性能测量
- ✅ 自动检测性能变化
- ✅ 美观的HTML报告
- ✅ 历史趋势分析

### 2. Flamegraph (CPU分析)

```bash
# 生成CPU火焰图
cargo flamegraph --bin agent-mem-server -- [args]

# 查看火焰图
open flamegraph.svg
```

**优势**:
- ✅ 可视化CPU热点
- ✅ 快速定位性能瓶颈
- ✅ 支持递归调用分析

### 3. Tokio Console (异步分析)

```bash
# 启用tokio-console
RUSTFLAGS="--cfg tokio_unstable" cargo build --features tokio-console

# 启动应用
TOKIO_CONSOLE_BIND=127.0.0.1:6669 ./target/debug/agent-mem-server

# 在另一个终端
tokio-console http://127.0.0.1:6669
```

**优势**:
- ✅ 实时异步任务监控
- ✅ 任务生命周期追踪
- ✅ 资源使用分析

### 4. Valgrind/Callgrind (内存分析)

```bash
# 生成callgrind数据
cargo build --release
valgrind --tool=callgrind ./target/release/agent-mem-server

# 可视化分析
kcachegrind callgrind.out.*
```

### 5. perf (Linux性能分析)

```bash
# 记录性能数据
sudo perf record -F 99 -g ./target/release/agent-mem-server

# 查看报告
sudo perf report

# 生成火焰图
sudo perf script | stackcollapse-perf.pl | flamegraph.pl > perf-flamegraph.svg
```

---

## 📊 监控指标

### 应用层指标

通过Prometheus `/metrics` endpoint暴露：

```promql
# 请求速率
rate(agentmem_http_requests_total[5m])

# P95延迟
histogram_quantile(0.95, agentmem_http_request_duration_seconds)

# 错误率
rate(agentmem_errors_total[5m]) / rate(agentmem_http_requests_total[5m])

# 记忆操作速率
rate(agentmem_memory_operations_total[5m])

# 缓存命中率
agentmem_cache_hits / (agentmem_cache_hits + agentmem_cache_misses)

# 数据库连接池使用
agentmem_db_connections_active / agentmem_db_connections_max
```

### 系统层指标

```promql
# CPU使用率
process_cpu_seconds_total

# 内存使用
process_resident_memory_bytes

# 文件描述符
process_open_fds

# 线程数
process_threads
```

### 自定义业务指标

```promql
# 记忆数量
agentmem_memories_total

# Agent数量
agentmem_agents_total

# 平均记忆重要性
avg(agentmem_memory_importance)

# LLM调用延迟
histogram_quantile(0.95, agentmem_llm_call_duration_seconds)
```

---

## 🎯 性能优化建议

### 1. 数据库优化

#### 查询优化
```sql
-- 添加索引
CREATE INDEX idx_memories_agent_user ON memories(agent_id, user_id);
CREATE INDEX idx_memories_created_at ON memories(created_at DESC);
CREATE INDEX idx_memories_importance ON memories(importance DESC);

-- 使用EXPLAIN ANALYZE分析慢查询
EXPLAIN ANALYZE SELECT * FROM memories WHERE agent_id = 'xxx';
```

#### 连接池调优
```rust
// 在config中调整连接池
db_pool_size: 20,
db_max_connections: 100,
db_connection_timeout: 30,
```

### 2. 缓存优化

#### Redis缓存
```rust
// 启用Redis缓存
redis_url: "redis://localhost:6379",
redis_cache_ttl: 3600,  // 1小时

// 缓存热点数据
- Agent配置
- 用户信息
- 常用记忆
```

#### 应用层缓存
```rust
// 使用moka进行内存缓存
use moka::future::Cache;

let cache: Cache<String, Memory> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300))
    .build();
```

### 3. 并发优化

#### Tokio调优
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1

[dependencies.tokio]
features = ["full", "tracing"]
```

#### 异步批处理
```rust
// 使用批处理减少数据库往返
async fn batch_add_memories(memories: Vec<Memory>) -> Result<()> {
    // 单次事务插入多条记忆
    sqlx::query("INSERT INTO memories ...")
        .execute_many(&self.pool)
        .await?;
}
```

### 4. 向量搜索优化

#### Qdrant优化
```yaml
# qdrant配置
storage:
  performance:
    max_search_threads: 4
    
collection:
  hnsw_config:
    m: 16
    ef_construct: 200
  optimizers_config:
    indexing_threshold: 10000
```

#### 搜索参数调优
```rust
// 调整搜索参数
search_params: SearchParams {
    hnsw_ef: 128,  // 平衡速度和准确性
    exact: false,
    quantization: Some(QuantizationSearchParams {
        ignore: false,
        rescore: true,
    }),
}
```

### 5. 代码优化

#### 避免不必要的clone
```rust
// ❌ 不好
fn process(data: String) -> String {
    data.clone()
}

// ✅ 好
fn process(data: &str) -> String {
    data.to_string()
}
```

#### 使用零拷贝
```rust
// 使用Bytes避免内存拷贝
use bytes::Bytes;

fn handle_data(data: Bytes) -> Bytes {
    // 零拷贝操作
    data.slice(0..100)
}
```

---

## 📈 性能监控最佳实践

### 1. 持续监控

- ✅ 每次PR自动运行性能测试
- ✅ 每天定时运行完整benchmark
- ✅ 实时监控生产环境指标
- ✅ 设置性能告警阈值

### 2. 性能预算

为关键操作设置性能预算：

```yaml
performance_budget:
  memory_creation: 5ms
  memory_retrieval: 3ms
  semantic_search: 25ms
  graph_traversal: 20ms
```

### 3. 基线管理

```bash
# 每个版本保存baseline
git tag v1.0.0
cargo bench -- --save-baseline v1.0.0

# 版本间对比
cargo bench -- --baseline v1.0.0
```

### 4. 定期审查

- 每周: 审查性能趋势
- 每月: 全面性能评估
- 每季度: 性能优化冲刺

---

## 🔗 相关资源

### 内部文档
- [性能测试结果](../target/benchmark-reports/)
- [Criterion报告](../target/criterion/report/index.html)
- [故障排查指南](troubleshooting-guide.md)

### 外部工具
- [Criterion.rs](https://github.com/bheisler/criterion.rs) - Rust基准测试框架
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) - CPU火焰图
- [tokio-console](https://github.com/tokio-rs/console) - Tokio异步监控
- [Grafana](https://grafana.com/) - 可视化监控

---

**文档版本**: v1.0  
**最后更新**: 2025-11-03  
**维护团队**: AgentMem Performance Team

---

## ✅ 快速参考

### 运行完整benchmark
```bash
./scripts/run_benchmarks.sh
```

### 运行回归测试
```bash
./scripts/performance_regression_test.sh
```

### 查看Prometheus指标
```bash
curl http://localhost:8080/metrics/prometheus
```

### 生成火焰图
```bash
cargo flamegraph --bin agent-mem-server
```

---

🎯 **定期监控性能，保持系统健康！**

📊 **数据驱动优化，避免过早优化！**

⚡ **性能是特性，需要持续维护！**

