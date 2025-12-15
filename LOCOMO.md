# LOCOMO 基准测试实施与验证计划

**日期**: 2025-12-15  
**目标**: 使用真实 LoCoMo 数据对 AgentMem 进行端到端基准验证，并以相同流程对比其他记忆平台（优先 Mem0、LangMem/MemOS），产出可复现报告。

---

## 🔎 现状速览
- 代码完成度：框架、数据加载、基础指标、测试用例和报告模块均已实现，可编译。
- 数据现状：`examples/locomo-benchmark/LoCoMo/data/` 下已有大体量原始数据（如 `msc_personas_all.json`, `locomo10.json`），但当前加载逻辑仍依赖分类目录（`single_hop` 等），缺失时会回退到示例数据，尚未接入真实数据。
- 评估缺口：LLM-as-a-Judge 评分与 BLEU/ROUGE/METEOR/BERTScore 等指标的真实计算未落地；对比平台适配与统一流程未完成。
- 运行验证：尚未在真实 LoCoMo 数据上完成 AgentMem 跑分与对比报告。

---

## 🎯 目标
1) 接入官方/真实 LoCoMo 数据，并按官方类别切分到 `data/{single_hop,multi_hop,temporal,open_domain,adversarial}`。
2) 在 AgentMem 上跑通端到端评测，生成分项/总体得分、性能指标与错误案例。
3) 使用同一流程复现 1-2 个对比平台（Mem0、LangMem/MemOS），生成客观对比表。
4) 形成可复现的脚本与报告（JSON/Markdown），便于回归与共享。

---

## 🛠️ 行动计划（更新版）

### Phase 0: 数据与目录落地（今天完成）
- 将原始 LoCoMo 数据放入 `examples/locomo-benchmark/data/raw/`（已有文件可直接使用）。
- 编写转换脚本，将 `raw/*.json` 解析并按类别写入 `data/{single_hop,...}/**/*.json`，格式对齐 `datasets.rs` 中的 `ConversationSession` 结构；输出 `data/stats.json` 汇总计数。
- 在转换过程中校验：类别标注、问题数、session 引用完整性。

### Phase 1: AgentMem 端到端基准（明天完成）
- 修改测试入口：默认 `--dataset-path examples/locomo-benchmark/data`，若分类目录缺失则报错提示运行转换脚本（取消示例数据回退）。
- 跑通 `cargo run -p locomo-benchmark -- --dataset-path <path> --verbose` 基于真实数据生成报告 `results/reports/agentmem-{date}.json|md`。
- 记录性能：搜索延迟、总延迟、P95，以及 token 消耗（检索+生成）。

### Phase 2: 评估指标对齐与 LLM Judge（1 天）
- 落地 BLEU/ROUGE/METEOR/F1/BERTScore 计算，版本固定以便复现；在报告中列出依赖版本。
- 实现 LLM-as-a-Judge：支持 OpenAI/Anthropic provider，评分维度对齐官方定义；裁决结果可缓存复用。
- 报告同时输出自动指标与 LLM 评分，并附关键错误案例。

### Phase 3: 对比平台复现（1-2 天）
- 选 1-2 个平台（优先 Mem0、LangMem 或 MemOS），为每个平台写适配器：统一接口 `store(conversation) -> retrieve(question) -> generate/return`。
- 复用相同数据、提示模板、随机种子和温度/`top_p` 参数；保证对比公平。
- 生成对比报告 `results/reports/comparison-{date}.md`，含总体/分项表格、性能对比、误差条和失败案例对照。

### Phase 4: 稳定性与复现（0.5 天）
- 提供 `make` 或 `cargo xtask` 脚本：`prepare-data`、`run-agentmem`、`run-platform --name mem0` 等。
- 在 README/本计划中添加最小可复现命令与依赖说明（API Key、模型、硬件）。
- 增加小样本回归集（每类 10 条）用于 CI 快速校验。

---

## ✅ 立即可执行的 Top 5 任务
1. **数据转换脚本**：在 `examples/locomo-benchmark/scripts/convert_raw.rs`（或 Python）实现 raw→分类目录转换，并生成 `data/stats.json`。
2. **严格数据加载**：更新 `datasets.rs`，分类目录缺失时报错提示运行转换脚本，取消示例数据回退。
3. **AgentMem 跑分**：使用真实数据运行基准，生成 `results/reports/agentmem-{date}.*`，记录性能与 token。
4. **指标与 LLM Judge**：补全 BLEU/ROUGE/METEOR/BERTScore/F1，并接入 LLM-as-a-Judge（可通过环境变量注入 Key/模型）。
5. **对比平台接入**：先接入 Mem0（或 LangMem），用同一流程生成对比报告。

---

## 📝 运行与验证（目标状态）
- 准备数据：`cargo xtask prepare-data --input examples/locomo-benchmark/data/raw --out examples/locomo-benchmark/data`
- 运行 AgentMem 基准：`cargo run -p locomo-benchmark -- --dataset-path examples/locomo-benchmark/data --verbose`
- 运行对比平台：`cargo run -p locomo-benchmark -- --dataset-path ... --platform mem0 --config configs/mem0.toml`
- 查看报告：`results/reports/agentmem-{date}.md` / `comparison-{date}.md`

---

## 📊 预期结果与基准
- 目标准确性（AgentMem 初版）：Single-Hop 70-75%、Multi-Hop 55-60%、Temporal 60-65%、Open-Domain 65-70%、Adversarial 50-55%，Overall 62-67%。
- 目标性能：搜索延迟 P95 <100ms，总响应 P95 <2s，token 消耗较 Mem0 下降 10-20%。

---

## 🔄 进度检查点
- Phase 0 完成标志：`data/{single_hop,...}` 生成且 `data/stats.json` 通过校验。
- Phase 1 完成标志：真实数据跑通，生成首版 AgentMem 报告（含性能）。
- Phase 2 完成标志：自动指标与 LLM Judge 分数写入报告。
- Phase 3 完成标志：至少一个对比平台报告生成。
- Phase 4 完成标志：脚本与最小复现命令可在全新环境一次跑通。

---

## ⚠️ 风险与缓解
- 数据标注/类别缺失：转换时校验并输出缺陷列表；必要时回退到人工标注子集。
- API 速率/成本：LLM Judge 支持批量/缓存；提供可选小样本模式。
- 指标版本不一致：在报告中写明指标库与模型版本；固定随机种子。
- 对比公平性：统一温度/`top_p`、上下文长度、提示模板与检索 top-k。
