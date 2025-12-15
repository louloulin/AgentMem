# AgentMem LOCOMO 测试实现状态

## ✅ 已完成

### 1. 核心功能实现
- ✅ **LLM 集成模块** (`src/llm_integration.rs`)
  - 实现 OpenAI/OpenAI 兼容接口的答案生成
  - 实现 LLM-as-a-Judge 评估功能
  - 支持离线模式（无 API Key 时自动退化）
  - 支持自定义 base_url（兼容自建服务）

- ✅ **测试用例更新** (`src/test_cases.rs`)
  - Single-hop 测试：集成 LLM 生成和评估
  - Multi-hop 测试：支持跨会话推理
  - Temporal 测试：时间推理（复用单跳逻辑）
  - Open-domain 测试：开放域知识（复用单跳逻辑）
  - Adversarial 测试：对抗性问题测试
  - 所有测试都支持 LLM 和离线两种模式

- ✅ **测试框架** (`src/framework.rs`)
  - 支持 LLM 配置注入
  - 自动内存清理（每个类别测试前重置）
  - 性能指标收集（延迟、Token 消耗）

- ✅ **CLI 支持** (`src/main.rs`)
  - 支持命令行参数配置 LLM
  - 支持环境变量配置（CLI 优先）
  - 自动退化到离线模式

### 2. 数据转换
- ✅ **转换脚本** (`scripts/convert_locomo.py`)
  - 将官方 `locomo10.json` 转换为测试格式
  - 按类别（single_hop, multi_hop, temporal, open_domain, adversarial）组织
  - 每个 QA 对生成独立的会话文件

- ✅ **数据验证**
  - 数据目录已存在并包含转换后的文件
  - 各类别数据文件数量：
    - single_hop: 282 个会话
    - multi_hop: 100+ 个会话
    - temporal: 321 个会话
    - open_domain: 96 个会话
    - adversarial: 446 个会话

### 3. 工具脚本
- ✅ **运行脚本** (`scripts/run_locomo_test.sh`)
  - 自动检查数据并转换（如需要）
  - 支持离线模式和 LLM 模式
  - 自动构建项目
  - 显示测试结果和报告位置

- ✅ **环境验证脚本** (`scripts/verify_setup.sh`)
  - 检查 Rust 工具链
  - 检查数据文件
  - 检查 Python 环境
  - 检查 LLM 配置

### 4. 文档
- ✅ **README 更新**
  - 添加快速开始指南
  - 添加 LLM 配置说明
  - 添加环境变量说明
  - 添加运行脚本使用说明

## 🚧 待优化

### 1. 性能优化
- [ ] 实现真实的 Token 统计（从 LLM 响应中提取）
- [ ] 优化多跳推理的上下文构建策略
- [ ] 添加批量处理支持（减少 API 调用次数）

### 2. 功能增强
- [ ] 实现 Temporal 测试的时间衰减逻辑
- [ ] 实现 Open-domain 测试的外部知识融合
- [ ] 添加更多 LLM 提供商支持（Anthropic, Gemini 等）
- [ ] 实现流式响应支持

### 3. 测试与验证
- [ ] 运行完整测试并记录基准结果
- [ ] 对比不同 LLM 模型的表现
- [ ] 验证离线模式与 LLM 模式的得分差异
- [ ] 性能基准测试（延迟、吞吐量）

### 4. 报告增强
- [ ] 添加可视化图表
- [ ] 添加错误案例分析
- [ ] 添加与其他平台的详细对比
- [ ] 添加改进建议

## 📋 使用指南

### 快速开始

1. **验证环境**
   ```bash
   cd examples/locomo-benchmark
   ./scripts/verify_setup.sh
   ```

2. **运行离线测试**
   ```bash
   ./scripts/run_locomo_test.sh
   ```

3. **运行 LLM 测试**
   ```bash
   OPENAI_API_KEY=sk-xxxxx \
   LOCOMO_LLM_PROVIDER=openai \
   LOCOMO_LLM_MODEL=gpt-4o-mini \
   ./scripts/run_locomo_test.sh --with-llm
   ```

4. **查看报告**
   ```bash
   ls -lt results/reports/*.md | head -1
   ```

### 配置选项

**环境变量：**
- `OPENAI_API_KEY`: OpenAI API 密钥
- `LOCOMO_LLM_PROVIDER`: LLM 提供商（默认: openai）
- `LOCOMO_LLM_MODEL`: LLM 模型（默认: gpt-4o-mini）
- `LOCOMO_LLM_BASE_URL`: 自定义 API 基址（可选）

**CLI 参数：**
- `--dataset-path`: 数据集路径（默认: data）
- `--llm-provider`: LLM 提供商
- `--llm-model`: LLM 模型
- `--llm-api-key`: LLM API 密钥
- `--llm-base-url`: 自定义 API 基址

## 🔍 技术细节

### 测试流程

1. **数据加载**：从 `data/{category}/` 加载会话文件
2. **记忆存储**：将对话消息存储到 AgentMem
3. **问题检索**：使用 AgentMem 搜索相关记忆
4. **答案生成**：
   - LLM 模式：使用 LLM 基于检索结果生成答案
   - 离线模式：直接拼接检索结果
5. **答案评估**：
   - 计算 F1、BLEU-1、ROUGE-L、Cosine Similarity
   - LLM 模式：额外使用 LLM-as-a-Judge
6. **性能统计**：记录延迟、Token 消耗等指标

### 评估指标

- **准确性指标**：
  - F1 Score
  - BLEU-1 Score
  - ROUGE-L Score
  - Cosine Similarity
  - LLM-as-a-Judge Score（可选）

- **性能指标**：
  - 平均搜索延迟
  - 平均生成延迟
  - P95 搜索延迟
  - P95 总延迟
  - 平均 Token 消耗

## 📊 预期结果

测试完成后，将在 `results/reports/` 目录生成：
- Markdown 报告：包含详细的分析和表格
- JSON 报告：包含原始数据，便于进一步分析

报告内容包括：
- 总体得分和分类得分
- 性能指标
- 与其他平台的对比
- 错误案例分析

## 🎯 下一步

1. **运行完整测试**：使用真实 LLM 运行一次完整测试
2. **记录基准结果**：保存首次测试结果作为基准
3. **性能优化**：根据测试结果优化检索和生成策略
4. **扩展功能**：实现 Temporal 和 Open-domain 的专门逻辑
