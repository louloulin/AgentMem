# LOCOMO 实测计划（AgentMem）

目标：基于官方 LoCoMo 数据集，以相同评测方式验证 AgentMem 的长期对话记忆效果，并可对比其他记忆系统（Mem0/MemOS/LangMem 等）。

## 数据与素材
- 已有：`LoCoMo/data/locomo10.json`、`msc_personas_all.json`、多模态示例等（本仓库已包含）。
- 官方资源：<https://snap-research.github.io/locomo/>（含论文与数据下载入口）。
- 数据组织需求：转换为当前基准期望的目录结构：
  - `data/single_hop/*.json`
  - `data/multi_hop/*.json`
  - `data/temporal/*.json`
  - `data/open_domain/*.json`
  - `data/adversarial/*.json`

## 测试框架现状
- `src/framework.rs`: 已实现 5 类推理测试和性能指标收集。
- `src/datasets.rs`: 默认从 `data/` 目录按类别读取，不存在则生成示例数据。
- `src/report.rs`: 生成 Markdown/JSON 报告。
- 缺口：需要将真实 LoCoMo 数据转换为基准所需的 per-category JSON，会话需包含 `messages` 与 `questions`。

## 实施步骤（可按顺序执行）
1) 数据准备  
   - 确认 `LoCoMo/data` 中的 JSON 结构；从官方入口下载缺失部分（如更完整的会话或多模态子集）。  
   - 编写转换脚本（建议 `scripts/convert_locomo.py`）将 `locomo10.json`/`msc_personas_all.json` 拆分为会话文件，并为每条问题生成 `questions` 列表，填充 `expected_answer`（可用 GT/summary 字段）。
   - 生成后的文件落地到 `examples/locomo-benchmark/data/{category}/session_xxx.json`。

2) AgentMem 评测  
   - 在 `TestConfig` 中指定 `dataset_path = "data"`（默认即可），确保转换后目录存在。  
   - 运行：`cd examples/locomo-benchmark && cargo run --release`。  
   - 核对输出：`results/reports/locomo_report_*.md/json`。

3) 平台对比（同一测试逻辑）  
   - 复用测试框架，抽象出 Memory 适配层：允许注入不同的记忆后端（Mem0/MemOS/LangMem）。  
   - 对每个平台运行相同数据与流程，记录总体得分、各类得分、P95 延迟、Token 消耗。  
   - 在报告中加入平台对比表格（当前表格已预留，补充真实数值）。

4) 真实性能验证  
   - 打开 LLM 调用：在 `TestConfig.llm_config` 中填写 provider、API key、model，替换 `generate_answer` 与 `judge_answer` 的真实调用。  
   - 统计 Token：在 LLM 客户端中累积 prompt/completion tokens，填入 `PerformanceMetrics.avg_tokens`。  
   - 采集延迟：记录检索耗时与生成耗时，计算 P95。

5) 交付与复现  
   - 报告：`results/reports/` 产出 Markdown+JSON；额外在本文件更新结果摘要。  
   - 代码：提交转换脚本与配置变更，确保 `cargo test` / `cargo run --release` 可复现。  
   - 数据：若官方数据需手动下载，在仓库外存放并在 README 中注明获取方式与路径。

## 近期行动清单
- [x] 审阅 `LoCoMo/data/*.json` 字段，定义拆分/标注规则。
- [x] 编写转换脚本，生成五类目录和会话文件。
- [x] 跑通 AgentMem 基准（真实数据），生成首版报告。
- [ ] 抽象 Memory 适配层，跑通至少 1 个对比平台。
- [x] 启用真实 LLM（API Key 注入），补齐 Token/延迟统计。
- [x] 更新本文件与 README，沉淀复现场景与结果。

## ✅ 已完成工作

### 核心实现
1. **LLM 集成**：完整实现 OpenAI 兼容接口，支持答案生成和 LLM-as-a-Judge 评估
2. **测试用例**：所有 5 类测试（single_hop, multi_hop, temporal, open_domain, adversarial）均已集成 LLM 支持
3. **CLI 工具**：支持命令行参数和环境变量配置
4. **运行脚本**：提供便捷的运行和验证脚本

### 数据准备
- ✅ 数据转换脚本已实现并验证
- ✅ 数据已转换完成（共 1986 个会话文件）
- ✅ 数据格式符合测试框架要求

### 工具与文档
- ✅ 运行脚本：`scripts/run_locomo_test.sh`
- ✅ 验证脚本：`scripts/verify_setup.sh`
- ✅ README 更新：包含完整使用指南
- ✅ 实现状态文档：`IMPLEMENTATION_STATUS.md`

## 🚀 快速使用

```bash
# 1. 验证环境
./scripts/verify_setup.sh

# 2. 运行离线测试
./scripts/run_locomo_test.sh

# 3. 运行 LLM 测试（需要 API Key）
OPENAI_API_KEY=sk-xxx ./scripts/run_locomo_test.sh --with-llm
```

## 数据下载提示
- 官方链接：<https://snap-research.github.io/locomo/>。  
- 如需 CLI：可使用 `git submodule` 或直接 `wget` 官方发布包；建议放置于 `examples/locomo-benchmark/LoCoMo/data_raw/`，再由转换脚本输出到 `data/`。  
- 注意版权与使用条款，避免将大体量数据直接提交仓库。
