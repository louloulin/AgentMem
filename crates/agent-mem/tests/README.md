# AgentMem Intelligence 组件测试指南

## 测试文件说明

### 1. `orchestrator_intelligence_test.rs`
**占位测试**，用于验证架构集成，不需要真实的 LLM Provider。

**运行方式**:
```bash
cargo test --package agent-mem --test orchestrator_intelligence_test
```

**测试内容**:
- 类型转换测试（4 个）
- 智能添加测试（4 个）
- 混合搜索测试（4 个，需要 postgres 特性）
- 智能决策测试（4 个）
- 集成测试（5 个，包括 infer 参数测试）
- 性能测试（3 个，使用 `--ignored` 运行）

---

### 2. `intelligence_real_test.rs` ⭐
**真实测试**，测试 Intelligence 组件的实际功能，需要配置 LLM Provider。

**运行方式**:
```bash
# 使用 OpenAI
export OPENAI_API_KEY=your_key
cargo test --package agent-mem --test intelligence_real_test -- --ignored --nocapture

# 使用 Anthropic
export ANTHROPIC_API_KEY=your_key
cargo test --package agent-mem --test intelligence_real_test -- --ignored --nocapture

# 使用 Ollama (本地)
# 确保 Ollama 服务运行在 http://localhost:11434
cargo test --package agent-mem --test intelligence_real_test -- --ignored --nocapture
```

**测试内容**:
- `test_fact_extractor_real` - 测试事实提取功能
- `test_advanced_fact_extractor_real` - 测试结构化事实提取功能
- `test_importance_evaluator_real` - 测试重要性评估功能
- `test_full_intelligence_pipeline` - 测试完整 Intelligence 流水线

---

## LLM Provider 配置

### 选项 1: OpenAI (推荐 ⭐⭐⭐)

**优点**:
- 稳定可靠
- 响应速度快
- 质量高

**配置方式**:
```bash
export OPENAI_API_KEY=sk-...
```

**支持的模型**:
- `gpt-4` (最佳质量)
- `gpt-3.5-turbo` (默认，性价比高)

---

### 选项 2: Anthropic (推荐 ⭐⭐)

**优点**:
- 质量高
- 上下文窗口大
- 安全性好

**配置方式**:
```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

**支持的模型**:
- `claude-3-opus-20240229` (最佳质量)
- `claude-3-sonnet-20240229` (平衡)
- `claude-3-haiku-20240307` (默认，速度快)

---

### 选项 3: Ollama (本地) (推荐 ⭐)

**优点**:
- 完全免费
- 数据隐私
- 离线可用

**配置方式**:
```bash
# 1. 安装 Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 2. 下载模型
ollama pull llama2

# 3. 启动服务（默认在 http://localhost:11434）
ollama serve
```

**支持的模型**:
- `llama2` (默认)
- `llama3`
- `mistral`
- `qwen`

---

## 测试示例

### 示例 1: 测试事实提取

```bash
export OPENAI_API_KEY=your_key
cargo test --package agent-mem --test intelligence_real_test test_fact_extractor_real -- --ignored --nocapture
```

**预期输出**:
```
========== 测试 FactExtractor ==========

🔧 使用 OpenAI Provider
📝 提取事实中...
✅ 成功提取 3 个事实:

  1. 用户名字是张三
     类别: Personal
     置信度: 0.95

  2. 用户年龄是25岁
     类别: Personal
     置信度: 0.90

  3. 用户住在北京
     类别: Location
     置信度: 0.92
```

---

### 示例 2: 测试重要性评估

```bash
export ANTHROPIC_API_KEY=your_key
cargo test --package agent-mem --test intelligence_real_test test_importance_evaluator_real -- --ignored --nocapture
```

**预期输出**:
```
========== 测试 EnhancedImportanceEvaluator ==========

🔧 使用 Anthropic Provider
📝 评估重要性中...
✅ 重要性评估成功:

  重要性分数: 0.85
  置信度: 0.90
  理由: 生日是重要的个人信息，对于个性化服务很有价值
  因素: ["personal_info", "long_term_value"]
```

---

### 示例 3: 测试完整流水线

```bash
# 使用本地 Ollama
ollama serve &
cargo test --package agent-mem --test intelligence_real_test test_full_intelligence_pipeline -- --ignored --nocapture
```

**预期输出**:
```
========== 测试完整 Intelligence 流水线 ==========

🔧 尝试使用 Ollama Provider (本地)
📝 Step 1: 事实提取
   ✅ 提取了 2 个事实

📝 Step 2: 结构化事实提取
   ✅ 提取了 2 个结构化事实

📝 Step 3: 重要性评估
   ✅ 重要性评估完成

========== 完整流水线测试成功 ==========
```

---

## 故障排除

### 问题 1: "无法创建 LLM Provider"

**原因**: 未设置 API Key 或 Ollama 服务未启动

**解决方案**:
```bash
# 检查环境变量
echo $OPENAI_API_KEY
echo $ANTHROPIC_API_KEY

# 或启动 Ollama
ollama serve
```

---

### 问题 2: "Request failed: connection refused"

**原因**: Ollama 服务未启动或端口不正确

**解决方案**:
```bash
# 启动 Ollama
ollama serve

# 检查服务状态
curl http://localhost:11434/api/tags
```

---

### 问题 3: "API error 401: Unauthorized"

**原因**: API Key 无效或过期

**解决方案**:
```bash
# 重新设置正确的 API Key
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
```

---

## 性能测试

运行性能测试（需要 LLM Provider）:

```bash
export OPENAI_API_KEY=your_key
cargo test --package agent-mem --test orchestrator_intelligence_test performance -- --ignored --nocapture
```

**预期输出**:
```
========== 性能对比测试 ==========

📊 测试 1: 简单模式添加性能 (infer=false)
   总耗时: 235.083µs
   平均每条: 4.701µs
   吞吐量: 212,690 条/秒

📊 测试 2: 智能模式添加性能 (infer=true)
   总耗时: 185.583µs
   平均每条: 3.711µs
   吞吐量: 269,421 条/秒

📈 添加性能对比:
   简单模式: 4.701µs (基准)
   智能模式: 3.711µs
   性能差异: -21.1% (智能模式更快)
```

---

## 下一步

1. ✅ 配置 LLM Provider
2. ✅ 运行真实测试
3. ⏳ 验证 Intelligence 组件功能
4. ⏳ 实现 PostgreSQL Managers 初始化
5. ⏳ 开始 Phase 2: 多模态支持

---

## 相关文档

- [agentmem30.md](../../../agentmem30.md) - AgentMem 3.0 改造计划
- [Intelligence 模块文档](../../agent-mem-intelligence/README.md)
- [LLM Provider 文档](../../agent-mem-llm/README.md)

