# 运行性能对比测试指南

## 快速开始

### 1. 运行 AgentMem 测试（已编译）

```bash
cd examples/mem0-performance-comparison
./run_agentmem_test.sh
```

或者直接运行：

```bash
cd examples/mem0-performance-comparison
./target/release/agentmem_benchmark
```

### 2. 运行 Mem0 测试

#### 安装 Mem0（如果未安装）

```bash
pip install mem0
```

#### 运行测试

```bash
cd examples/mem0-performance-comparison
python3 mem0_simple_benchmark.py
```

### 3. 运行完整对比

```bash
cd examples/mem0-performance-comparison
./run_comparison.sh
```

## 测试场景

### AgentMem 测试场景

1. **单个添加** (50项)
   - 使用 `mem.add_for_user()` 
   - 使用 `mem0_mode()` 初始化
   - 预期性能: ~19-43 ops/s

2. **批量添加** (100项)
   - 使用 `mem.add_batch_optimized()`
   - 预期性能: **~470 ops/s** (最佳性能)

3. **并发单个添加** (20并发×5项)
   - 使用 `tokio::spawn` 并发添加
   - 启用嵌入队列
   - 预期性能: **~235 ops/s**

### Mem0 测试场景

1. **单个添加** (50项)
   - 使用 `mem.add([{"role": "user", "content": ...}])`
   - 使用默认配置（FastEmbed + Qdrant）
   - 预期性能: ~100-500 ops/s（取决于环境）

2. **批量添加** (100项)
   - 循环调用 `mem.add()`
   - Mem0 没有批量 API
   - 预期性能: ~100-500 ops/s

## 预期结果对比

| 场景 | Mem0 | AgentMem | 差距 |
|------|------|----------|------|
| 单个添加 (50项) | ~100-500 ops/s | ~19-43 ops/s | 2-26x |
| 批量添加 (100项) | ~100-500 ops/s | **~470 ops/s** | 0.94-4.7x |
| 并发单个添加 | ~200-1000 ops/s | **~235 ops/s** | 0.85-4.3x |
| **Mem0 目标** | **10,000 ops/s** | - | - |

## 注意事项

1. **环境因素**: 测试结果受 CPU、内存、系统负载影响
2. **多次运行**: 建议多次运行取平均值
3. **Mem0 安装**: 确保已安装 mem0 (`pip install mem0`)
4. **AgentMem 编译**: 首次运行需要编译（约 5-10 分钟）

## 记录结果

运行测试后，请将结果记录到 `PERFORMANCE_COMPARISON_RESULTS.md`：

```markdown
## 实际测试结果 (日期: YYYY-MM-DD)

### AgentMem
- 单个添加 (50项): XX.XX ops/s
- 批量添加 (100项): XX.XX ops/s
- 并发单个添加 (20并发×5项): XX.XX ops/s

### Mem0
- 单个添加 (50项): XX.XX ops/s
- 批量添加 (100项): XX.XX ops/s

### 对比分析
- ...
```

## 故障排除

### AgentMem 编译失败
```bash
# 清理并重新编译
cd examples/mem0-performance-comparison
cargo clean
cargo build --bin agentmem_benchmark --release
```

### Mem0 导入失败
```bash
# 安装 mem0
pip install mem0

# 或使用虚拟环境
python3 -m venv venv
source venv/bin/activate  # Linux/Mac
# 或: venv\Scripts\activate  # Windows
pip install mem0
```

### 测试运行缓慢
- 这是正常的，嵌入生成是 CPU 密集型操作
- 单个添加测试可能需要几分钟
- 批量添加测试通常更快
