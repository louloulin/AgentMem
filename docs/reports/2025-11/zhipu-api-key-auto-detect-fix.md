# Zhipu API Key 自动检测修复报告

**日期**: 2025-11-13  
**问题**: 配置了 ZHIPU_API_KEY 但系统仍然提示 "未找到 LLM API Key 环境变量"  
**状态**: ✅ 已修复（增强版）

---

## 🐛 问题分析

### 根本原因

即使修复了 API Key 检查逻辑，仍然存在以下问题：

1. **Provider 默认值问题**
   - 如果 `LLM_PROVIDER` 环境变量未设置，系统会使用默认值 `"openai"`
   - 即使配置了 `ZHIPU_API_KEY`，如果 provider 是 `"openai"`，系统会去检查 `OPENAI_API_KEY` 而不是 `ZHIPU_API_KEY`

2. **配置传递问题**
   - 如果 `OrchestratorConfig` 是通过默认配置创建的（没有调用 `AutoConfig::detect()`），`llm_provider` 就是 `None`
   - 当 `llm_provider` 是 `None` 时，代码会尝试从环境变量 `LLM_PROVIDER` 获取
   - 如果 `LLM_PROVIDER` 也没有设置，就会使用默认值 `"openai"`

3. **缺少自动检测机制**
   - 当当前 provider 的 API Key 不存在时，系统应该自动检测其他可用的 provider
   - 例如：如果 provider 是 `"openai"` 但 `OPENAI_API_KEY` 不存在，应该自动检测 `ZHIPU_API_KEY` 并切换到 `"zhipu"`

---

## ✅ 修复方案（增强版）

### 修复内容

在 `agentmen/crates/agent-mem/src/orchestrator.rs` 的 `create_llm_provider` 函数中，添加了**智能自动检测机制**：

1. **首先尝试当前 provider 的 API Key**
   - 如果找到了，直接使用

2. **如果当前 provider 的 API Key 不存在，自动检测其他可用的 provider**
   - 按优先级检测：Zhipu → OpenAI → Anthropic → DeepSeek → 通用 LLM_API_KEY
   - 如果检测到可用的 API Key，自动切换到对应的 provider

3. **添加辅助函数**
   - `create_llm_provider_with_config()`: 用于创建指定配置的 LLM Provider

### 代码逻辑

```rust
// 1. 首先尝试当前 provider 的 API Key
let api_key = match provider {
    "zhipu" => env::var("ZHIPU_API_KEY").or_else(|_| env::var("LLM_API_KEY")).ok(),
    "openai" => env::var("OPENAI_API_KEY").or_else(|_| env::var("LLM_API_KEY")).ok(),
    // ...
};

// 2. 如果找到了，直接使用
if let Some(key) = api_key {
    // 使用当前 provider
} else {
    // 3. 自动检测其他可用的 provider
    if let Ok(zhipu_key) = env::var("ZHIPU_API_KEY") {
        // 自动切换到 zhipu
        return create_llm_provider_with_config("zhipu", "glm-4.6", Some(zhipu_key));
    }
    // 继续检测其他 provider...
}
```

---

## 🎯 修复效果

### 修复前

- ❌ 如果 `LLM_PROVIDER` 未设置，默认使用 `"openai"`
- ❌ 即使配置了 `ZHIPU_API_KEY`，系统也会去检查 `OPENAI_API_KEY`
- ❌ 如果 `OPENAI_API_KEY` 不存在，系统直接失败，不会尝试其他 provider

### 修复后

- ✅ 如果当前 provider 的 API Key 不存在，自动检测其他可用的 provider
- ✅ 如果检测到 `ZHIPU_API_KEY`，自动切换到 `"zhipu"` provider
- ✅ 支持所有主流 LLM Provider 的自动检测和切换
- ✅ 更智能的错误提示，显示检测到的 provider

---

## 📋 使用场景

### 场景 1: 只配置了 ZHIPU_API_KEY，没有设置 LLM_PROVIDER

```bash
export ZHIPU_API_KEY="your-zhipu-api-key"
# 没有设置 LLM_PROVIDER
```

**修复前**: ❌ 系统使用默认值 `"openai"`，检查 `OPENAI_API_KEY`，失败  
**修复后**: ✅ 系统检测到 `ZHIPU_API_KEY`，自动切换到 `"zhipu"` provider

### 场景 2: 配置了错误的 LLM_PROVIDER

```bash
export LLM_PROVIDER="openai"
export ZHIPU_API_KEY="your-zhipu-api-key"
# 没有设置 OPENAI_API_KEY
```

**修复前**: ❌ 系统使用 `"openai"`，检查 `OPENAI_API_KEY`，失败  
**修复后**: ✅ 系统检测到 `ZHIPU_API_KEY`，自动切换到 `"zhipu"` provider

### 场景 3: 配置了多个 API Key

```bash
export ZHIPU_API_KEY="your-zhipu-api-key"
export OPENAI_API_KEY="your-openai-api-key"
export LLM_PROVIDER="openai"
```

**修复前**: ✅ 使用 `"openai"` provider  
**修复后**: ✅ 优先使用当前 provider (`"openai"`)，如果不存在则自动切换到其他可用的 provider

---

## 🔧 验证步骤

### 1. 设置环境变量（只设置 ZHIPU_API_KEY）

```bash
export ZHIPU_API_KEY="your-zhipu-api-key"
# 不设置 LLM_PROVIDER
```

### 2. 重启服务

```bash
cd agentmen
pkill -f "agent-mem-server"
./start_server_no_auth.sh
```

### 3. 验证日志

应该看到：
```
INFO 当前 provider (openai) 的 API Key 未找到，尝试自动检测其他可用的 provider
INFO ✅ 检测到 ZHIPU_API_KEY，自动切换到 zhipu provider
INFO 成功创建 LLM Provider: zhipu (glm-4.6)
```

而不是：
```
WARN 未找到 LLM API Key 环境变量
WARN LLM API Key 未配置，LLM Provider 将不可用
```

---

## 📊 支持的自动检测 Provider

| Provider | 环境变量 | 默认 Model |
|----------|---------|-----------|
| Zhipu | `ZHIPU_API_KEY` | `glm-4.6` |
| OpenAI | `OPENAI_API_KEY` | `gpt-4` |
| Anthropic | `ANTHROPIC_API_KEY` | `claude-3-5-sonnet-20241022` |
| DeepSeek | `DEEPSEEK_API_KEY` | `deepseek-chat` |
| 通用 | `LLM_API_KEY` | 使用当前 provider 的 model |

**检测优先级**: Zhipu > OpenAI > Anthropic > DeepSeek > 通用 LLM_API_KEY

---

## ✅ 修复完成

- ✅ 添加了智能自动检测机制
- ✅ 支持所有主流 LLM Provider 的自动切换
- ✅ 改进了错误提示和日志信息
- ✅ 代码已通过编译检查
- ✅ 向后兼容，不影响现有配置

**现在即使只配置了 `ZHIPU_API_KEY` 而没有设置 `LLM_PROVIDER`，系统也能自动检测并切换到 zhipu provider！**

