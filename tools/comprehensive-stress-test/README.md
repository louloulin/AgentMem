# AgentMem 综合压测工具

全面压测 AgentMem 系统性能，识别瓶颈，生成详细报告。

## 功能特性

### 8 大压测场景

1. **记忆构建压测** - 测试记忆创建的吞吐量和延迟
2. **记忆检索压测** - 测试向量搜索、全文搜索、混合搜索性能
3. **并发操作压测** - 测试高并发下的系统表现
4. **图推理压测** - 测试图记忆网络的查询性能
5. **智能处理压测** - 测试 LLM 集成和智能处理性能
6. **缓存性能压测** - 测试多级缓存系统效率
7. **批量操作压测** - 测试批量操作的性能优化
8. **长时间稳定性测试** - 测试系统长时间运行的稳定性

### 性能指标收集

- **吞吐量**: ops/sec, qps
- **延迟**: P50, P90, P95, P99, P999
- **资源使用**: CPU, 内存
- **错误率**: 成功率, 失败率

### 瓶颈分析

- CPU 瓶颈识别
- I/O 瓶颈识别
- 内存瓶颈识别
- 数据库瓶颈识别

### 报告生成

- Markdown 格式综合报告
- JSON 格式详细数据
- 优化建议

## 安装

```bash
cd tools/comprehensive-stress-test
cargo build --release
```

## 使用方法

### 运行所有场景

```bash
cargo run --release -- all
```

### 运行单个场景

#### 场景 1: 记忆构建压测

```bash
cargo run --release -- memory-creation --concurrency 100 --total 10000
```

#### 场景 2: 记忆检索压测

```bash
cargo run --release -- memory-retrieval --dataset-size 100000 --concurrency 100
```

#### 场景 3: 并发操作压测

```bash
cargo run --release -- concurrent-ops --users 1000 --duration 300
```

#### 场景 4: 图推理压测

```bash
cargo run --release -- graph-reasoning --nodes 10000 --edges 50000
```

#### 场景 5: 智能处理压测

```bash
cargo run --release -- intelligence-processing --concurrency 10
```

#### 场景 6: 缓存性能压测

```bash
cargo run --release -- cache-performance --cache-size-mb 500
```

#### 场景 7: 批量操作压测

```bash
cargo run --release -- batch-operations --batch-size 100
```

#### 场景 8: 长时间稳定性测试

```bash
cargo run --release -- stability-test --hours 24
```

### 生成报告

```bash
cargo run --release -- report --results-dir stress-test-results
```

## 配置文件

默认配置文件: `stress-test-config.json`

```json
{
  "memory_creation": {
    "concurrency": 100,
    "total_memories": 10000,
    "memory_sizes": [100, 1000, 10000]
  },
  "memory_retrieval": {
    "dataset_size": 100000,
    "concurrency": 100,
    "query_types": ["vector", "fulltext", "hybrid"],
    "top_k": 10
  },
  "concurrent_ops": {
    "concurrent_users": 1000,
    "duration_seconds": 300,
    "read_ratio": 0.7,
    "write_ratio": 0.2,
    "update_ratio": 0.1
  }
}
```

## 输出

### 结果目录

默认输出目录: `stress-test-results/`

```
stress-test-results/
├── memory_creation.json
├── memory_retrieval.json
├── concurrent_ops.json
├── graph_reasoning.json
├── intelligence_processing.json
├── cache_performance.json
├── batch_operations.json
├── stability.json
└── comprehensive-report.md
```

### 报告示例

```markdown
# AgentMem 综合压测报告

## 📊 总体摘要

| 场景 | 总操作数 | 成功率 | 吞吐量 (ops/s) | P95延迟 (ms) | P99延迟 (ms) |
|------|----------|--------|----------------|--------------|-------------|
| memory_creation | 10000 | 99.00% | 1250.50 | 25.30 | 32.10 |
| memory_retrieval | 1000 | 99.50% | 85.20 | 18.50 | 22.80 |
...

## 🔍 瓶颈分析

### CPU 瓶颈
- ⚠️ **memory_creation**: 峰值 CPU 使用率 85.20%，可能存在 CPU 瓶颈

### 延迟瓶颈
- ⚠️ **graph_reasoning**: P95 延迟 55.30ms，超过目标值 30ms

## 💡 优化建议

### memory_creation
- 🔧 **CPU 优化**: 考虑使用更高效的算法或并行处理
...
```

## 性能目标

- ✅ 响应时间 < 30ms (P95)
- ✅ 吞吐量 > 10K req/s
- ✅ 内存效率提升 3x
- ✅ 支持 10,000+ 并发用户

## 许可证

MIT

