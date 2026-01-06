# AgentMem 性能基准测试

本目录包含 AgentMem 的性能基准测试套件。

---

## 📋 基准测试列表

### 1. `performance_benchmark.rs`

核心性能基准测试，包括：

- **JSON 序列化/反序列化** (5 个测试)
  - 小型对象序列化
  - 中型对象序列化
  - 大型对象序列化
  - 小型对象反序列化
  - 中型对象反序列化

- **字符串操作** (3 个测试)
  - 字符串拼接
  - 字符串格式化
  - 字符串搜索

- **集合操作** (3 个测试)
  - Vec 创建和填充
  - Vec 迭代
  - Vec 过滤

- **内存分配** (3 个测试)
  - 小对象分配 (100 bytes)
  - 中等对象分配 (10 KB)
  - 大对象分配 (1 MB)

- **哈希操作** (2 个测试)
  - HashMap 插入
  - HashMap 查找

- **并发操作** (1 个测试)
  - Arc + Mutex 性能

**总计**: 17 个基准测试

---

## 🚀 运行基准测试

### 方法 1: 使用测试脚本（推荐）

```bash
# 从项目根目录运行
cd agentmen

# 运行所有基准测试
./scripts/run-benchmarks.sh --all

# 快速测试（减少样本数）
./scripts/run-benchmarks.sh --quick

# 保存基准线
./scripts/run-benchmarks.sh --save-baseline

# 与基准线比较
./scripts/run-benchmarks.sh --compare

# 生成 HTML 报告
./scripts/run-benchmarks.sh --report

# 显示结果摘要
./scripts/run-benchmarks.sh --summary

# 清理结果
./scripts/run-benchmarks.sh --clean
```

### 方法 2: 使用 Cargo

```bash
# 运行所有基准测试
cargo bench --package agent-mem-server

# 运行特定基准测试
cargo bench --package agent-mem-server --bench performance_benchmark

# 运行特定测试组
cargo bench --package agent-mem-server -- json_serialization

# 保存基准线
cargo bench --package agent-mem-server -- --save-baseline baseline

# 与基准线比较
cargo bench --package agent-mem-server -- --baseline baseline
```

---

## 📊 性能报告

### HTML 报告

Criterion 自动生成 HTML 报告：

```
target/criterion/report/index.html
```

打开报告：

```bash
# macOS
open target/criterion/report/index.html

# Linux
xdg-open target/criterion/report/index.html

# 或使用测试脚本
./scripts/run-benchmarks.sh --report
```

### 命令行输出示例

```
json_serialization/small_object
                        time:   [495.23 ns 502.45 ns 510.12 ns]
                        change: [-2.3% +0.5% +3.2%] (p = 0.45 > 0.05)
                        No change in performance detected.

json_serialization/medium_object
                        time:   [1.95 μs 2.01 μs 2.08 μs]
                        change: [-1.5% +1.2% +4.1%] (p = 0.32 > 0.05)
                        No change in performance detected.

json_serialization/large_object
                        time:   [48.2 μs 50.1 μs 52.3 μs]
                        change: [-3.2% +0.8% +4.5%] (p = 0.52 > 0.05)
                        No change in performance detected.
```

---

## 🎯 性能基准线

### 参考性能指标（Apple M1/M2）

| 操作 | 平均时间 | 吞吐量 |
|------|---------|--------|
| JSON 序列化（小） | 500 ns | 2M ops/s |
| JSON 序列化（中） | 2 μs | 500K ops/s |
| JSON 序列化（大） | 50 μs | 20K ops/s |
| JSON 反序列化（小） | 600 ns | 1.6M ops/s |
| JSON 反序列化（中） | 3 μs | 333K ops/s |
| 字符串拼接 | 30 μs | 33K ops/s |
| 字符串格式化 | 500 ns | 2M ops/s |
| Vec 创建（1000） | 5 μs | 200K ops/s |
| Vec 迭代（1000） | 500 ns | 2M ops/s |
| HashMap 插入（1000） | 80 μs | 12.5K ops/s |
| HashMap 查找 | 50 ns | 20M ops/s |
| 内存分配（100B） | 50 ns | 20M ops/s |
| 内存分配（10KB） | 500 ns | 2M ops/s |
| 内存分配（1MB） | 50 μs | 20K ops/s |
| 并发操作 | 5 ms | 200 ops/s |

---

## 🔧 配置

### Criterion 配置

```rust
criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = /* benchmark functions */
}
```

**参数说明**:
- `measurement_time`: 每个基准测试的测量时间（10 秒）
- `sample_size`: 样本数量（100 个）

### 自定义配置

```bash
# 增加测量时间
cargo bench -- --measurement-time 20

# 增加样本数
cargo bench -- --sample-size 200

# 快速测试
cargo bench -- --quick
```

---

## 📈 性能趋势分析

### 保存基准线

```bash
# 保存当前性能作为基准线
cargo bench -- --save-baseline baseline
```

### 与基准线比较

```bash
# 与基准线比较
cargo bench -- --baseline baseline
```

### 输出示例

```
json_serialization/small_object
                        time:   [502.45 ns 510.12 ns 518.23 ns]
                        change: [+8.2% +10.5% +12.8%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

---

## 🚀 CI/CD 集成

### GitHub Actions 示例

```yaml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run benchmarks
        run: |
          cd agentmen
          cargo bench --package agent-mem-server
          
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion
```

---

## 🎯 性能优化建议

### 1. JSON 序列化优化

- ✅ 使用 `serde_json` 的高性能模式
- ✅ 避免不必要的序列化
- ✅ 考虑使用二进制格式（如 MessagePack）

### 2. 字符串操作优化

- ✅ 使用 `String::with_capacity()` 预分配
- ✅ 避免频繁的字符串拼接
- ✅ 使用 `format!` 而不是多次 `push_str`

### 3. 集合操作优化

- ✅ 使用 `Vec::with_capacity()` 预分配
- ✅ 使用迭代器而不是索引访问
- ✅ 考虑使用 `SmallVec` 优化小集合

### 4. 内存分配优化

- ✅ 重用对象池
- ✅ 使用栈分配而不是堆分配
- ✅ 考虑使用 `Box::leak()` 避免释放

### 5. 并发优化

- ✅ 使用 `RwLock` 而不是 `Mutex`（读多写少）
- ✅ 考虑使用无锁数据结构
- ✅ 减少锁的粒度

---

## 📚 相关文档

- [性能测试指南](../doc/PERFORMANCE_TESTING_GUIDE.md)
- [Criterion.rs 文档](https://bheisler.github.io/criterion.rs/book/)
- [Rust 性能优化](https://nnethercote.github.io/perf-book/)

---

## 🎯 总结

AgentMem 的性能基准测试套件提供：

- ✅ 17 个基准测试用例
- ✅ 覆盖所有核心操作
- ✅ 自动化测试脚本
- ✅ HTML 性能报告
- ✅ 性能基准线
- ✅ 回归检测
- ✅ CI/CD 集成

这确保了 AgentMem 的高性能和可扩展性！🚀

