# 快速开始 - Mem0 vs AgentMem 性能对比测试

## 快速运行

### 1. 运行 AgentMem 测试（推荐先运行这个）

```bash
cd examples/mem0-performance-comparison
./run_agentmem_test.sh
```

**预期输出**:
```
============================================================
AgentMem 性能基准测试
============================================================

测试 1: 单个添加 (mem0_mode)
测试项数: 50
  已添加: 10/50
  已添加: 20/50
  ...
结果:
  成功: 50/50
  耗时: X.XXX 秒
  吞吐量: XX.XX ops/s
  平均延迟: XX.XX ms

测试 2: 批量添加
测试项数: 100
结果:
  成功: 100/100
  耗时: X.XXX 秒
  吞吐量: XXX.XX ops/s  (预期 ~470 ops/s)
  平均延迟: X.XX ms

测试 3: 并发单个添加（启用队列）
并发数: 20, 每任务项数: 5
结果:
  成功: 100/100
  耗时: X.XXX 秒
  吞吐量: XXX.XX ops/s  (预期 ~235 ops/s)
  平均延迟: X.XX ms
```

### 2. 运行 Mem0 测试（需要安装 mem0）

```bash
cd examples/mem0-performance-comparison

# 安装 mem0（如果未安装）
pip install mem0

# 运行测试
./run_mem0_test.sh
```

**预期输出**:
```
============================================================
Mem0 性能基准测试
============================================================

测试 1: 单个添加 (infer=False)
测试项数: 50
  已添加: 10/50
  ...
结果:
  成功: 50/50
  耗时: X.XXX 秒
  吞吐量: XXX.XX ops/s
  平均延迟: XX.XX ms

测试 2: 批量添加 (infer=False)
测试项数: 100
结果:
  成功: 100/100
  耗时: X.XXX 秒
  吞吐量: XXX.XX ops/s
  平均延迟: X.XX ms
```

### 3. 运行完整对比

```bash
cd examples/mem0-performance-comparison
./run_comparison.sh
```

## 预期性能对比

| 场景 | Mem0 | AgentMem | 说明 |
|------|------|----------|------|
| 单个添加 (50项) | ~100-500 ops/s | ~19-43 ops/s | Mem0 利用 ONNX Runtime |
| 批量添加 (100项) | ~100-500 ops/s | **~470 ops/s** | AgentMem 批量方法更好 |
| 并发单个添加 | ~200-1000 ops/s | **~235 ops/s** | Mem0 利用 Python FastEmbed |
| **Mem0 目标** | **10,000 ops/s** | - | Mem0 官方目标 |

## 性能分析

### AgentMem 性能特点
- ✅ **批量方法性能更好**: `add_batch_optimized()` 达到 ~470 ops/s
- ✅ **队列优化有效**: 并发场景下启用队列提供 1.74x 提升
- ⚠️ **单个 Mutex 锁瓶颈**: 所有嵌入请求需要通过同一个锁

### Mem0 性能特点
- ✅ **Python FastEmbed 内部优化**: 使用 ONNX Runtime (C++ 实现)，自动并行处理
- ✅ **C++ 扩展不受 GIL 限制**: 多个线程可以真正并行
- ✅ **内部批量处理**: FastEmbed 可能内部批量处理请求

### 性能差距
- **Mem0 目标**: 10,000 ops/s (infer=False)
- **AgentMem 批量方法**: ~470 ops/s
- **差距**: **21x** (10,000 / 470 ≈ 21)
- **主要瓶颈**: 单个 Mutex 锁导致串行执行嵌入生成

## 故障排除

### AgentMem 测试失败
1. **编译错误**: 运行 `cargo build --bin agentmem_benchmark --release`
2. **运行时错误**: 检查是否有足够的系统资源
3. **初始化失败**: 检查 FastEmbed 模型是否正确下载

### Mem0 测试失败
1. **未安装**: 运行 `pip install mem0`
2. **导入错误**: 检查 Python 环境和依赖
3. **初始化失败**: 检查配置是否正确

## 下一步

1. **运行测试**: 在实际环境中运行两个测试脚本
2. **记录结果**: 将实际测试结果记录到 `PERFORMANCE_COMPARISON_RESULTS.md`
3. **分析差异**: 对比实际结果与预期结果，分析性能差异
4. **优化实施**: 根据测试结果，实施 P2 优化（多个模型实例）
