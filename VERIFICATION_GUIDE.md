# AgentMem 验证指南

## 概述

本文档说明如何在真实环境中验证 AgentMem 的功能，包括 P0 优化验证。

---

## 环境要求

### 1. 代理设置（必需）

FastEmbed 需要从 HuggingFace 下载模型，因此需要配置代理：

```bash
export http_proxy=http://127.0.0.1:4780
export https_proxy=http://127.0.0.1:4780
export HTTP_PROXY=http://127.0.0.1:4780
export HTTPS_PROXY=http://127.0.0.1:4780
```

**说明**: 端口 4780 是示例，请根据实际代理配置调整。

### 2. ONNX Runtime 库（可选）

如果使用本地 ONNX Runtime 库：

```bash
export DYLD_LIBRARY_PATH="$(pwd)/lib:$(pwd)/target/release:$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib"
```

### 3. LLM Provider 配置

#### Zhipu AI（推荐）

```bash
export ZHIPU_API_KEY="your-api-key-here"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"
```

#### OpenAI

```bash
export OPENAI_API_KEY="your-api-key-here"
export LLM_PROVIDER="openai"
export LLM_MODEL="gpt-4"
```

### 4. Embedder 配置

#### FastEmbed（推荐）

```bash
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
# 或者
export EMBEDDER_MODEL="multilingual-e5-small"
```

#### OpenAI Embeddings

```bash
export EMBEDDER_PROVIDER="openai"
export EMBEDDER_MODEL="text-embedding-3-small"
```

---

## 验证步骤

### 1. 运行单元测试

```bash
# 运行所有测试
cargo test

# 运行默认行为测试（P0 相关）
cargo test --package agent-mem --test default_behavior_test -- --nocapture

# 运行智能组件测试
cargo test --package agent-mem --test intelligence_test -- --nocapture
```

**预期结果**:
```
test result: ok. 12 passed; 0 failed; 0 ignored
```

---

### 2. 测试 FastEmbed 初始化

```bash
# 设置代理
export http_proxy=http://127.0.0.1:4780
export https_proxy=http://127.0.0.1:4780

# 运行 FastEmbed 测试
cd examples/test-fastembed
cargo run
```

**预期结果**:
```
✅ FastEmbed 创建成功！
   - Provider: fastembed
   - Model: multilingual-e5-small
   - Dimension: 384

✅ 嵌入生成成功！维度: 384
```

---

### 3. 运行 P0 真实验证

```bash
# 设置所有环境变量
export http_proxy=http://127.0.0.1:4780
export https_proxy=http://127.0.0.1:4780
export ZHIPU_API_KEY="your-api-key-here"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 运行验证
cd examples/p0-real-verification
cargo run
```

**预期结果**:
```
🧪 P0 真实验证：默认启用智能功能

✅ 测试 1: AddMemoryOptions::default().infer = true
✅ 测试 2: 简单模式（infer: false）正常工作
✅ 测试 3: 默认行为（infer: true）正常工作（降级策略）
✅ 测试 4: 向后兼容性：用户可以显式设置 infer 值

🎉 P0 真实验证完成！
```

---

### 4. 运行零配置示例

```bash
# 设置环境变量
export http_proxy=http://127.0.0.1:4780
export https_proxy=http://127.0.0.1:4780
export ZHIPU_API_KEY="your-api-key-here"
export EMBEDDER_PROVIDER="fastembed"

# 运行示例
cd examples/quickstart-zero-config
cargo run
```

**预期结果**:
```
✅ Memory 初始化成功
✅ 添加记忆成功
✅ 搜索记忆成功
```

---

## 一键验证脚本

创建 `verify_p0.sh` 脚本：

```bash
#!/bin/bash

# 设置代理
export http_proxy=http://127.0.0.1:4780
export https_proxy=http://127.0.0.1:4780
export HTTP_PROXY=http://127.0.0.1:4780
export HTTPS_PROXY=http://127.0.0.1:4780

# 设置 LLM Provider
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"

# 设置 Embedder
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 设置 ONNX Runtime（可选）
export DYLD_LIBRARY_PATH="$(pwd)/lib:$(pwd)/target/release:$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib"

echo "🧪 开始 P0 真实验证..."
echo "================================"

# 1. 运行单元测试
echo "📋 步骤 1: 运行单元测试"
cargo test --package agent-mem --test default_behavior_test -- --nocapture
if [ $? -ne 0 ]; then
    echo "❌ 单元测试失败"
    exit 1
fi
echo "✅ 单元测试通过"
echo ""

# 2. 测试 FastEmbed
echo "📋 步骤 2: 测试 FastEmbed 初始化"
cd examples/test-fastembed
cargo run
if [ $? -ne 0 ]; then
    echo "❌ FastEmbed 测试失败"
    exit 1
fi
cd ../..
echo "✅ FastEmbed 测试通过"
echo ""

# 3. 运行 P0 真实验证
echo "📋 步骤 3: 运行 P0 真实验证"
cd examples/p0-real-verification
cargo run
if [ $? -ne 0 ]; then
    echo "❌ P0 真实验证失败"
    exit 1
fi
cd ../..
echo "✅ P0 真实验证通过"
echo ""

echo "================================"
echo "🎉 所有验证通过！"
```

运行脚本：

```bash
chmod +x verify_p0.sh
./verify_p0.sh
```

---

## 常见问题

### Q1: FastEmbed 下载模型失败

**错误信息**:
```
Failed to retrieve onnx/model.onnx
connecting to huggingface.co:443
```

**解决方案**:
1. 确保代理正确配置
2. 检查代理端口是否正确（常见端口：4780, 7890, 1087）
3. 测试代理连接：`curl -x http://127.0.0.1:4780 https://huggingface.co`

### Q2: ONNX Runtime 库未找到

**错误信息**:
```
dyld: Library not loaded: @rpath/libonnxruntime.1.22.0.dylib
```

**解决方案**:
```bash
export DYLD_LIBRARY_PATH="$(pwd)/lib:$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib"
```

### Q3: Embedder 未初始化

**错误信息**:
```
EmbeddingError("Embedder not initialized")
```

**解决方案**:
1. 确保 FastEmbed 模型已下载（需要代理）
2. 或者使用简单模式：`infer: false`
3. 或者使用 OpenAI embeddings

### Q4: LLM API 调用失败

**错误信息**:
```
LLM API call failed
```

**解决方案**:
1. 检查 API Key 是否正确
2. 检查 LLM Provider 配置
3. 检查网络连接

---

## 参考文档

- **P0 实施报告**: `P0_IMPLEMENTATION_REPORT.md`
- **P0 真实验证报告**: `P0_REAL_VERIFICATION_REPORT.md`
- **P0+P1 最终报告**: `P0_P1_FINAL_REPORT.md`
- **改进计划**: `agentmem71.md`
- **README**: `README.md`

---

## 联系方式

如有问题，请查看文档或提交 Issue。

