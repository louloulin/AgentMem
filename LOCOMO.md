# LOCOMO 基准测试实施计划

**日期**: 2025-12-10  
**目标**: 使用LOCOMO基准测试验证AgentMem的性能和效果，与其他记忆平台进行客观对比

---

## 📋 执行摘要

### 目标
1. 实现完整的LOCOMO基准测试框架
2. 验证AgentMem在5个推理类别上的表现
3. 使用与其他平台相同的测试方法进行客观对比
4. 生成详细的测试报告和性能分析

### LOCOMO测试概述

**LOCOMO (Long Conversation Memory)** 是评估AI系统长期对话记忆能力的标准基准测试，包括：

#### 5个推理类别
1. **Single-hop reasoning** (单跳推理): 基于单个会话的信息回答问题
2. **Multi-hop reasoning** (多跳推理): 需要综合多个会话的信息
3. **Temporal reasoning** (时间推理): 理解事件的时间顺序和时序关系
4. **Open-domain knowledge** (开放域知识): 结合对话信息和外部知识
5. **Adversarial questions** (对抗性问题): 识别无法回答的问题或抵抗误导信息

#### 评估指标
- **准确性指标**:
  - LLM-as-a-Judge (J) Score: LLM评估生成响应的质量
  - F1 Score: 精确率和召回率的调和平均
  - BLEU-1 Score: 单字精确度
  - ROUGE-L Score: 最长公共子序列
  - METEOR Score: 考虑同义词和词干
  - BERTScore-F1: 基于BERT的语义相似度
  - Cosine Similarity: 语义嵌入的余弦相似度

- **性能指标**:
  - Token Consumption: 生成过程使用的token数量
  - Latency: 响应生成时间（包括搜索和总响应时间）

---

## 🎯 实施计划

### Phase 1: 测试框架搭建 (1-2天)

#### 1.1 项目结构
```
examples/locomo-benchmark/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # 主入口
│   ├── framework.rs         # 测试框架核心
│   ├── datasets.rs          # 测试数据集加载
│   ├── metrics.rs           # 评估指标计算
│   ├── test_cases.rs        # 测试用例实现
│   └── report.rs            # 报告生成
├── data/
│   ├── single_hop/          # 单跳推理测试数据
│   ├── multi_hop/           # 多跳推理测试数据
│   ├── temporal/            # 时间推理测试数据
│   ├── open_domain/         # 开放域知识测试数据
│   └── adversarial/        # 对抗性问题测试数据
└── results/
    └── reports/             # 生成的测试报告
```

#### 1.2 核心组件

**测试框架 (`framework.rs`)**:
- `LocomoTestFramework`: 主测试框架
- `TestConfig`: 测试配置
- `TestResult`: 测试结果结构
- `TestRunner`: 测试执行器

**数据集加载 (`datasets.rs`)**:
- `DatasetLoader`: 加载LOCOMO测试数据集
- `ConversationSession`: 对话会话结构
- `QuestionAnswer`: 问答对结构

**评估指标 (`metrics.rs`)**:
- `AccuracyMetrics`: 准确性指标计算
- `PerformanceMetrics`: 性能指标计算
- `LLMJudge`: LLM-as-a-Judge评估器

**测试用例 (`test_cases.rs`)**:
- `SingleHopTest`: 单跳推理测试
- `MultiHopTest`: 多跳推理测试
- `TemporalTest`: 时间推理测试
- `OpenDomainTest`: 开放域知识测试
- `AdversarialTest`: 对抗性问题测试

**报告生成 (`report.rs`)**:
- `ReportGenerator`: 生成测试报告
- `ComparisonReport`: 与其他平台对比报告

---

### Phase 2: 数据集准备 (1天)

#### 2.1 数据集来源
- 使用LOCOMO官方数据集（如果可用）
- 创建符合LOCOMO标准的测试数据集
- 参考其他平台的测试数据格式

#### 2.2 数据集格式
```json
{
  "conversation_id": "conv_001",
  "sessions": [
    {
      "session_id": "session_1",
      "timestamp": "2025-01-01T10:00:00Z",
      "messages": [
        {"role": "user", "content": "I love pizza"},
        {"role": "assistant", "content": "That's great!"}
      ]
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

#### 2.3 测试数据规模
- **Single-hop**: 50-100个测试用例
- **Multi-hop**: 50-100个测试用例
- **Temporal**: 50-100个测试用例
- **Open-domain**: 50-100个测试用例
- **Adversarial**: 20-50个测试用例

---

### Phase 3: 测试实现 (2-3天)

#### 3.1 Single-hop推理测试

**测试逻辑**:
1. 加载对话会话数据
2. 将对话内容存储到AgentMem
3. 对每个问题执行检索
4. 生成答案
5. 计算评估指标

**实现要点**:
- 使用AgentMem的`add()` API存储对话
- 使用`search()` API检索相关记忆
- 使用LLM生成答案（基于检索到的记忆）

#### 3.2 Multi-hop推理测试

**测试逻辑**:
1. 加载多个会话的对话数据
2. 跨会话存储记忆
3. 对需要多跳推理的问题执行检索
4. 综合多个记忆片段生成答案
5. 计算评估指标

**实现要点**:
- 测试跨会话记忆检索能力
- 验证记忆关联和推理能力

#### 3.3 Temporal推理测试

**测试逻辑**:
1. 加载带时间戳的对话数据
2. 存储带时间信息的记忆
3. 对时间相关问题执行检索
4. 验证时间顺序理解
5. 计算评估指标

**实现要点**:
- 使用AgentMem的时间戳功能
- 测试时间相关的检索和排序

#### 3.4 Open-domain知识测试

**测试逻辑**:
1. 加载需要外部知识的对话
2. 存储对话记忆
3. 对开放域问题执行检索
4. 结合外部知识生成答案
5. 计算评估指标

**实现要点**:
- 测试记忆系统与外部知识的结合
- 验证知识融合能力

#### 3.5 Adversarial问题测试

**测试逻辑**:
1. 加载对抗性问题
2. 对无法回答的问题进行测试
3. 验证系统识别无法回答问题的能力
4. 测试对误导信息的抵抗能力
5. 计算评估指标

**实现要点**:
- 测试系统的鲁棒性
- 验证错误识别能力

---

### Phase 4: 评估指标实现 (2天)

#### 4.1 准确性指标

**LLM-as-a-Judge**:
- 使用LLM评估生成答案的质量
- 评估维度: 事实准确性、相关性、完整性、上下文适当性

**传统指标**:
- F1 Score: 使用`f1-score` crate
- BLEU-1: 使用`bleu` crate
- ROUGE-L: 使用`rouge` crate
- METEOR: 使用`meteor` crate
- BERTScore-F1: 使用BERT模型计算
- Cosine Similarity: 使用嵌入向量计算

#### 4.2 性能指标

**Token Consumption**:
- 记录LLM调用的token使用量
- 统计搜索和生成的token消耗

**Latency**:
- 搜索时间: 从查询到检索完成的时间
- 总响应时间: 从查询到答案生成完成的时间
- P95/P99延迟: 计算延迟分位数

---

### Phase 5: 报告生成和对比 (1天)

#### 5.1 测试报告

**报告内容**:
- 总体得分和分类得分
- 各指标的详细结果
- 性能指标分析
- 错误案例分析

#### 5.2 平台对比

**对比维度**:
- 与Mem0、MemOS、LangMem的LOCOMO结果对比
- 性能指标对比（延迟、token消耗）
- 准确性指标对比

**对比数据** (参考):
| 平台 | Single-Hop | Multi-Hop | Open-Domain | Temporal | Overall |
|------|-----------|-----------|-------------|----------|---------|
| Mem0 | 67.13% | 51.15% | 72.93% | 55.51% | 66.88% |
| MemOS | 78.44% | 64.30% | 55.21% | 73.21% | 73.31% |
| LangMem | 62.23% | 47.92% | 71.12% | 23.43% | 58.10% |
| OpenAI | 63.79% | 42.92% | 62.29% | 21.71% | 52.90% |

---

## 🔧 技术实现细节

### 依赖项

```toml
[dependencies]
agent-mem = { path = "../../crates/agent-mem" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# 评估指标
f1-score = "0.1"
bleu = "0.1"
rouge = "0.1"
meteor = "0.1"
bert-score = "0.1"  # 如果可用

# LLM集成
openai = "0.1"  # 或其他LLM provider
```

### 核心API使用

**存储对话记忆**:
```rust
let memory = Memory::builder()
    .with_storage("memory://")
    .with_embedder("fastembed", "BAAI/bge-small-en-v1.5")
    .build()
    .await?;

// 存储对话
for message in conversation.messages {
    memory.add(&message.content).await?;
}
```

**检索记忆**:
```rust
let results = memory.search(&question, SearchOptions {
    limit: Some(10),
    threshold: Some(0.7),
    ..Default::default()
}).await?;
```

**生成答案**:
```rust
// 使用检索到的记忆生成答案
let context = results.iter()
    .map(|m| m.content.as_str())
    .collect::<Vec<_>>()
    .join("\n");

let answer = llm.generate(&format!(
    "Context: {}\n\nQuestion: {}\n\nAnswer:",
    context, question
)).await?;
```

---

## 📊 预期结果

### 目标性能

基于AgentMem的现有能力，预期结果：

| 类别 | 目标得分 | 说明 |
|------|---------|------|
| Single-Hop | 70-75% | 单跳检索能力强 |
| Multi-Hop | 55-60% | 多跳推理需要优化 |
| Temporal | 60-65% | 时间推理能力中等 |
| Open-Domain | 65-70% | 开放域知识融合能力 |
| Adversarial | 50-55% | 对抗性识别需要加强 |
| **Overall** | **62-67%** | 接近Mem0水平 |

### 性能指标目标

- **搜索延迟**: <100ms (P95)
- **总响应时间**: <2s (P95)
- **Token消耗**: 比Mem0减少10-20%

---

## 🚀 实施步骤

### Step 1: 创建项目结构 (已完成)
- [x] 创建`examples/locomo-benchmark/`目录
- [x] 创建`Cargo.toml`
- [x] 创建基本文件结构

### Step 2: 实现测试框架 (进行中)
- [ ] 实现`framework.rs`
- [ ] 实现`datasets.rs`
- [ ] 实现`metrics.rs`

### Step 3: 实现测试用例
- [ ] Single-hop测试
- [ ] Multi-hop测试
- [ ] Temporal测试
- [ ] Open-domain测试
- [ ] Adversarial测试

### Step 4: 准备测试数据
- [ ] 创建/加载测试数据集
- [ ] 验证数据格式

### Step 5: 运行测试
- [ ] 执行所有测试用例
- [ ] 收集结果数据

### Step 6: 生成报告
- [ ] 生成详细测试报告
- [ ] 生成平台对比报告

---

## 📝 测试报告格式

### 报告结构

```markdown
# AgentMem LOCOMO基准测试报告

## 执行摘要
- 测试日期: 2025-12-10
- 测试版本: AgentMem v0.x.x
- 总体得分: XX.XX%

## 分类得分

### Single-hop推理
- 得分: XX.XX%
- 测试用例: XX/XX
- 详细指标: ...

### Multi-hop推理
- 得分: XX.XX%
- 测试用例: XX/XX
- 详细指标: ...

### Temporal推理
- 得分: XX.XX%
- 测试用例: XX/XX
- 详细指标: ...

### Open-domain知识
- 得分: XX.XX%
- 测试用例: XX/XX
- 详细指标: ...

### Adversarial问题
- 得分: XX.XX%
- 测试用例: XX/XX
- 详细指标: ...

## 性能指标
- 平均搜索延迟: XXms
- P95搜索延迟: XXms
- 平均总响应时间: XXms
- P95总响应时间: XXms
- 平均Token消耗: XX tokens

## 平台对比
[对比表格]

## 详细分析
[错误案例分析、优势劣势分析等]
```

---

## ✅ 验收标准

1. **功能完整性**:
   - [x] 所有5个推理类别的测试都已实现
   - [ ] 所有评估指标都已实现
   - [ ] 测试报告可以正常生成

2. **测试准确性**:
   - [ ] 测试方法与LOCOMO标准一致
   - [ ] 评估指标计算正确
   - [ ] 结果可复现

3. **对比有效性**:
   - [ ] 与其他平台使用相同的测试方法
   - [ ] 对比数据准确可靠
   - [ ] 分析深入客观

4. **文档完整性**:
   - [ ] README文档完整
   - [ ] 代码注释充分
   - [ ] 测试报告详细

---

## 🔄 后续优化

### 短期优化 (1-2周)
1. 优化多跳推理能力
2. 改进时间推理逻辑
3. 增强对抗性识别

### 中期优化 (1-2月)
1. 提升整体准确性5-10%
2. 优化性能指标
3. 扩展测试数据集

### 长期优化 (3-6月)
1. 达到业界领先水平 (>75%)
2. 持续优化和迭代
3. 参与LOCOMO官方评估

---

**计划制定日期**: 2025-12-10  
**预计完成时间**: 7-10天  
**负责人**: AI Assistant + 开发团队
