# AgentMem 学术论文基准测试框架

本目录包含用于学术论文的所有基准测试代码和脚本。

## 📁 目录结构

```
benchmarks/
├── README.md                    # 本文件
├── rust/                        # Rust 基准测试
│   ├── memory_operations.rs     # 记忆操作基准测试
│   ├── concurrent_ops.rs        # 并发操作基准测试
│   └── hierarchical_arch.rs     # 分层架构基准测试
├── python/                      # Python 对照组基准测试
│   ├── bench_mem0.py            # Mem0 基准测试
│   ├── bench_mirix.py           # MIRIX 基准测试
│   └── bench_flat_arch.py       # 扁平架构基准测试
├── comparison/                  # 对比分析脚本
│   ├── analyze_results.py       # 结果分析
│   ├── generate_plots.py        # 图表生成
│   └── generate_report.py       # 报告生成
├── data/                        # 测试数据集
│   ├── synthetic/               # 合成数据集
│   └── real/                    # 真实数据集
├── results/                     # 实验结果
│   ├── e1_rust_vs_python/       # 实验 E1 结果
│   ├── e2_hierarchical/         # 实验 E2 结果
│   ├── e3_temporal_graph/       # 实验 E3 结果
│   ├── e4_intelligent_reasoning/# 实验 E4 结果
│   └── e5_storage_backends/     # 实验 E5 结果
└── scripts/                     # 执行脚本
    ├── run_all_experiments.sh   # 运行所有实验
    ├── run_e1.sh                # 运行实验 E1
    ├── run_e2.sh                # 运行实验 E2
    ├── run_e3.sh                # 运行实验 E3
    ├── run_e4.sh                # 运行实验 E4
    └── run_e5.sh                # 运行实验 E5
```

## 🚀 快速开始

### 1. 环境准备

```bash
# 安装 Rust 依赖
cd agentmen
cargo build --release

# 安装 Python 依赖
cd benchmarks/python
pip install -r requirements.txt

# 启动 Qdrant
docker-compose up -d qdrant
```

### 2. 运行所有实验

```bash
cd benchmarks
./scripts/run_all_experiments.sh
```

### 3. 查看结果

```bash
# 生成对比报告
cd comparison
python generate_report.py

# 查看报告
open ../results/final_report.html
```

## 📊 实验说明

### 实验 E1: Rust vs Python 性能对比

**目标**: 量化 AgentMem (Rust) 相比 Mem0/MIRIX (Python) 的性能优势

**运行命令**:
```bash
./scripts/run_e1.sh
```

**测试场景**:
1. 记忆添加 (10K/100K/1M 记忆)
2. 记忆搜索 (1K 查询)
3. 并发操作 (1/10/100/1000 并发)

**预期结果**:
- 吞吐量提升: 10-50x
- 延迟降低: 5-20x
- 内存节省: 50-80%

---

### 实验 E2: 分层架构有效性验证

**目标**: 验证四层分层架构相比扁平架构的优势

**运行命令**:
```bash
./scripts/run_e2.sh
```

**测试场景**:
1. 多智能体协作 (10/50/100 智能体)
2. 记忆检索准确率
3. 记忆冲突率

**预期结果**:
- 检索准确率提升: 15-30%
- 冲突率降低: 40-60%
- 存储空间节省: 20-40%

---

### 实验 E3: 时序知识图谱性能评估

**目标**: 评估时序图谱在记忆版本管理中的性能

**运行命令**:
```bash
./scripts/run_e3.sh
```

**测试场景**:
1. 版本查询
2. 时间窗口查询
3. 历史追溯

**预期结果**:
- 版本查询延迟: < 100ms
- 存储开销: < 20%
- 支持任意时间点查询

---

### 实验 E4: 智能推理引擎评估

**目标**: 评估 LLM 驱动的智能推理引擎效果

**运行命令**:
```bash
./scripts/run_e4.sh
```

**测试场景**:
1. 事实提取准确率
2. 记忆决策正确率
3. 冲突解决成功率

**预期结果**:
- 事实提取 F1: > 90%
- 决策准确率: > 85%
- 冲突解决成功率: > 80%

---

### 实验 E5: 多后端存储对比

**目标**: 对比不同向量数据库的性能

**运行命令**:
```bash
./scripts/run_e5.sh
```

**测试后端**:
- Qdrant
- Pinecone
- Milvus
- Weaviate
- pgvector

**预期结果**:
- Qdrant 和 Pinecone 搜索性能领先
- pgvector 小规模数据集性价比高
- Milvus 大规模数据集扩展性好

## 📈 结果分析

### 生成对比图表

```bash
cd comparison
python generate_plots.py --experiment e1
```

### 生成统计报告

```bash
python generate_report.py --all
```

### 导出 LaTeX 表格

```bash
python export_latex_tables.py --output ../results/tables.tex
```

## 🔧 自定义实验

### 修改测试参数

编辑 `config.yaml`:

```yaml
experiments:
  e1:
    data_sizes: [10000, 100000, 1000000]
    concurrency_levels: [1, 10, 100, 1000]
    iterations: 3
  
  e2:
    num_agents: [10, 50, 100]
    memories_per_agent: 100
    queries: 1000
  
  e3:
    simulation_days: 365
    query_timestamps: [30, 180, 365]
  
  e4:
    datasets: ["PersonaChat", "DailyDialog"]
    sample_size: 1000
  
  e5:
    backends: ["qdrant", "pinecone", "milvus", "weaviate", "pgvector"]
    data_sizes: [10000, 100000, 1000000]
```

### 添加新的基准测试

1. 创建 Rust 基准测试:

```rust
// benchmarks/rust/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use agent_mem_core::AgentMemClient;

fn bench_my_operation(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            // 你的基准测试代码
        });
    });
}

criterion_group!(benches, bench_my_operation);
criterion_main!(benches);
```

2. 创建 Python 对照组:

```python
# benchmarks/python/bench_my_operation.py
import time
from mem0 import Memory

def bench_my_operation():
    client = Memory()
    
    start = time.time()
    # 你的基准测试代码
    duration = time.time() - start
    
    print(f"Duration: {duration}s")

if __name__ == "__main__":
    bench_my_operation()
```

## 📝 数据格式

### 实验结果 CSV 格式

```csv
timestamp,operation,throughput,latency_p50,latency_p95,latency_p99,cpu_percent,memory_mb
1697123456.789,add,50000,2.5,5.0,10.0,45.2,512.3
1697123457.890,search,10000,8.2,15.3,25.1,38.7,520.1
```

### 对比报告 JSON 格式

```json
{
  "experiment": "e1_rust_vs_python",
  "timestamp": "2025-10-14T10:30:00Z",
  "results": {
    "rust": {
      "throughput": 50000,
      "latency_p99": 10.0,
      "memory_mb": 512.3
    },
    "python": {
      "throughput": 2000,
      "latency_p99": 150.0,
      "memory_mb": 2048.5
    },
    "speedup": 25.0,
    "memory_saving": 0.75
  }
}
```

## 🐛 故障排除

### 问题 1: Qdrant 连接失败

```bash
# 检查 Qdrant 是否运行
docker ps | grep qdrant

# 重启 Qdrant
docker-compose restart qdrant
```

### 问题 2: Python 依赖安装失败

```bash
# 使用虚拟环境
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 问题 3: Rust 编译错误

```bash
# 清理并重新编译
cargo clean
cargo build --release
```

## 📚 参考资料

- [Criterion.rs 文档](https://bheisler.github.io/criterion.rs/book/)
- [Mem0 文档](https://docs.mem0.ai/)
- [Qdrant 文档](https://qdrant.tech/documentation/)

## 🤝 贡献

欢迎贡献新的基准测试和改进建议！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/new-benchmark`)
3. 提交更改 (`git commit -am 'Add new benchmark'`)
4. 推送到分支 (`git push origin feature/new-benchmark`)
5. 创建 Pull Request

## 📄 许可证

MIT License

---

**最后更新**: 2025-10-14
