# Mem0 vs AgentMem 性能对比测试

## 概述

这个目录包含用于对比 Mem0 和 AgentMem 性能的基准测试脚本。

## 文件说明

- `mem0_benchmark.py`: Mem0 (Python) 性能测试脚本
- `agentmem_benchmark.rs`: AgentMem (Rust) 性能测试脚本
- `run_comparison.sh`: 运行对比测试的脚本

## 运行测试

### 1. 运行 Mem0 测试

```bash
# 确保已安装 mem0
pip install mem0

# 运行测试
python mem0_benchmark.py
```

### 2. 运行 AgentMem 测试

```bash
# 运行测试
cargo run --bin agentmem_benchmark --release
```

### 3. 运行完整对比

```bash
# 运行对比脚本（如果存在）
./run_comparison.sh
```

## 测试场景

1. **单个添加** (50项)
   - Mem0: 使用 `mem.add()` 单个添加
   - AgentMem: 使用 `mem.add_for_user()` 单个添加

2. **批量添加** (100项)
   - Mem0: 循环调用 `mem.add()`（Mem0 没有批量 API）
   - AgentMem: 使用 `mem.add_batch_optimized()` 批量添加

3. **并发单个添加** (20并发×5项)
   - Mem0: 使用 ThreadPoolExecutor 并发添加
   - AgentMem: 使用 tokio::spawn 并发添加（启用队列）

## 预期结果

- **Mem0 目标性能**: 10,000 ops/s (infer=False)
- **AgentMem 当前性能**: 
  - 批量添加: ~470 ops/s
  - 并发单个添加: ~235 ops/s

## 注意事项

1. 测试环境可能影响结果（CPU、内存、系统负载等）
2. Mem0 使用 Python FastEmbed（ONNX Runtime 内部优化）
3. AgentMem 使用 Rust FastEmbed（需要自己处理并发）
4. 建议多次运行取平均值
