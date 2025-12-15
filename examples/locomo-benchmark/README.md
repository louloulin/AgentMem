# LOCOMO基准测试

AgentMem的LOCOMO (Long Conversation Memory) 基准测试实现，用于评估长期对话记忆能力。

## 📋 概述

LOCOMO是评估AI系统长期对话记忆能力的标准基准测试，包括5个推理类别：

1. **Single-hop reasoning**: 单跳推理
2. **Multi-hop reasoning**: 多跳推理
3. **Temporal reasoning**: 时间推理
4. **Open-domain knowledge**: 开放域知识
5. **Adversarial questions**: 对抗性问题

## 🚀 快速开始

### 方式一：使用运行脚本（推荐）

```bash
cd examples/locomo-benchmark

# 离线模式（无LLM，基于检索结果）
./scripts/run_locomo_test.sh

# 使用LLM模式（需要API Key）
OPENAI_API_KEY=sk-xxxxx \
LOCOMO_LLM_PROVIDER=openai \
LOCOMO_LLM_MODEL=gpt-4o-mini \
./scripts/run_locomo_test.sh --with-llm
```

### 方式二：直接运行

```bash
cd examples/locomo-benchmark

# 离线模式
cargo run --release -- --dataset-path data

# 使用LLM（通过环境变量）
OPENAI_API_KEY=sk-xxxxx \
LOCOMO_LLM_PROVIDER=openai \
LOCOMO_LLM_MODEL=gpt-4o-mini \
cargo run --release -- --dataset-path data

# 或通过CLI参数
cargo run --release -- \
  --dataset-path data \
  --llm-provider openai \
  --llm-model gpt-4o-mini \
  --llm-api-key sk-xxxxx
```

### 查看报告

测试完成后，报告将保存在 `results/reports/` 目录下：
- Markdown报告: `locomo_report_YYYYMMDD_HHMMSS.md`
- JSON报告: `locomo_report_YYYYMMDD_HHMMSS.json`

## 📊 测试结果

测试报告包含：

- **总体得分**: 所有类别的平均得分
- **分类得分**: 每个推理类别的详细得分
- **性能指标**: 延迟、Token消耗等
- **平台对比**: 与Mem0、MemOS、LangMem的对比
- **错误分析**: 失败案例的详细分析

## 🔧 配置

可以通过修改 `TestConfig` 来配置测试：

```rust
let config = TestConfig {
    dataset_path: "data".to_string(),
    verbose: true,
    llm_config: Some(LlmConfig {
        provider: "openai".to_string(),
        api_key: Some("your-api-key".to_string()),
        model: "gpt-4".to_string(),
        base_url: None, // 可选：兼容自建OpenAI接口
    }),
};
```

- 如未提供 `api_key`，测试将自动退化为基于检索结果的本地答案拼接，便于离线验证。
- 也可以使用 CLI/环境变量直接传入：`LOCOMO_LLM_PROVIDER`、`LOCOMO_LLM_MODEL`、`OPENAI_API_KEY`、`LOCOMO_LLM_BASE_URL`。

## 📝 数据集格式

测试数据集应按照以下格式组织：

```
data/
├── single_hop/
│   └── session_001.json
├── multi_hop/
│   └── session_001.json
├── temporal/
│   └── session_001.json
├── open_domain/
│   └── session_001.json
└── adversarial/
    └── session_001.json
```

每个JSON文件包含：

```json
{
  "session_id": "session_1",
  "timestamp": "2025-01-01T10:00:00Z",
  "messages": [
    {
      "role": "user",
      "content": "I love pizza."
    }
  ],
  "questions": [
    {
      "question_id": "q_001",
      "category": "single_hop",
      "question": "What do I love?",
      "expected_answer": "pizza",
      "session_references": ["session_1"]
    }
  ]
}
```

## 🎯 评估指标

### 准确性指标

- **F1 Score**: 精确率和召回率的调和平均
- **BLEU-1 Score**: 单字精确度
- **ROUGE-L Score**: 最长公共子序列
- **Cosine Similarity**: 语义嵌入的余弦相似度
- **LLM-as-a-Judge**: LLM评估生成响应的质量（可选）

### 性能指标

- **搜索延迟**: 从查询到检索完成的时间
- **总响应时间**: 从查询到答案生成完成的时间
- **Token消耗**: LLM调用的token使用量

## 📈 平台对比

当前测试结果与其他平台的对比：

| 平台 | Single-Hop | Multi-Hop | Open-Domain | Temporal | Overall |
|------|-----------|-----------|-------------|----------|---------|
| AgentMem | - | - | - | - | - |
| Mem0 | 67.13% | 51.15% | 72.93% | 55.51% | 66.88% |
| MemOS | 78.44% | 64.30% | 55.21% | 73.21% | 73.31% |
| LangMem | 62.23% | 47.92% | 71.12% | 23.43% | 58.10% |

## 🔄 后续优化

- [ ] 实现完整的LLM-as-a-Judge评估
- [ ] 添加更多测试数据集
- [ ] 优化多跳推理和时间推理逻辑
- [ ] 实现外部知识融合
- [ ] 添加P95/P99延迟计算

## 📚 参考

- [LOCOMO论文](https://snap-research.github.io/locomo/)
- [Mem0 LOCOMO结果](https://mem0.ai/research)
- [MemOS LOCOMO结果](https://docs.mirix.io/advanced/performance/)
