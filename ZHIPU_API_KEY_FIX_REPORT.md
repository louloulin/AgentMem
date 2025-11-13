# Zhipu API Key 配置问题修复报告

**日期**: 2025-11-13  
**问题**: 配置了 ZHIPU_API_KEY 但系统仍然提示 "未找到 LLM API Key 环境变量"  
**状态**: ✅ 已修复

---

## 🐛 问题描述

### 错误信息
```
2025-11-13T01:16:33.294543Z  WARN 未找到 LLM API Key 环境变量 (OPENAI_API_KEY, ANTHROPIC_API_KEY, LLM_API_KEY)
2025-11-13T01:16:33.294544Z  WARN LLM API Key 未配置，LLM Provider 将不可用
2025-11-13T01:16:33.294545Z  WARN LLM Provider 未配置，Intelligence 组件将不可用
```

### 问题根因

在 `agentmen/crates/agent-mem/src/orchestrator.rs` 的 `create_llm_provider` 函数中，API Key 检查逻辑**只检查了以下环境变量**：
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `LLM_API_KEY`

**但是缺少了对 `ZHIPU_API_KEY` 的检查**！

虽然 `auto_config.rs` 中的 `detect_llm_provider()` 函数能够检测到 `ZHIPU_API_KEY` 并设置 provider 为 "zhipu"，但在获取 API Key 时，代码没有根据 provider 类型检查对应的环境变量。

---

## 🔍 问题分析

### 代码流程

1. **Provider 检测** (`auto_config.rs:detect_llm_provider`)
   - ✅ 正确检测到 `ZHIPU_API_KEY` 环境变量
   - ✅ 设置 provider = "zhipu"

2. **API Key 获取** (`orchestrator.rs:create_llm_provider`)
   - ❌ **问题所在**：只检查 `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `LLM_API_KEY`
   - ❌ 没有检查 `ZHIPU_API_KEY`
   - ❌ 没有根据 provider 类型检查对应的 API Key

### 影响范围

- ✅ 配置了 `ZHIPU_API_KEY` 但系统无法识别
- ✅ LLM Provider 无法创建
- ✅ Intelligence 组件不可用
- ✅ 所有依赖 LLM 的智能功能都无法使用

---

## ✅ 修复方案

### 修复内容

修改 `agentmen/crates/agent-mem/src/orchestrator.rs` 中的 API Key 检查逻辑，**根据 provider 类型检查对应的环境变量**：

```rust
// 修复前（只检查固定的几个环境变量）
let api_key = match std::env::var("OPENAI_API_KEY")
    .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
    .or_else(|_| std::env::var("LLM_API_KEY"))
{
    Ok(key) => Some(key),
    Err(_) => {
        warn!("未找到 LLM API Key 环境变量 (OPENAI_API_KEY, ANTHROPIC_API_KEY, LLM_API_KEY)");
        None
    }
};

// 修复后（根据 provider 类型检查对应的环境变量）
let api_key = match provider.to_lowercase().as_str() {
    "zhipu" => std::env::var("ZHIPU_API_KEY")
        .or_else(|_| std::env::var("LLM_API_KEY"))
        .ok(),
    "openai" => std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("LLM_API_KEY"))
        .ok(),
    "anthropic" => std::env::var("ANTHROPIC_API_KEY")
        .or_else(|_| std::env::var("LLM_API_KEY"))
        .ok(),
    "deepseek" => std::env::var("DEEPSEEK_API_KEY")
        .or_else(|_| std::env::var("LLM_API_KEY"))
        .ok(),
    _ => {
        // 对于未知的 provider，尝试所有常见的 API Key 环境变量
        std::env::var("ZHIPU_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| std::env::var("DEEPSEEK_API_KEY"))
            .or_else(|_| std::env::var("LLM_API_KEY"))
            .ok()
    }
};
```

### 修复优势

1. ✅ **支持所有主流 LLM Provider**
   - Zhipu (`ZHIPU_API_KEY`)
   - OpenAI (`OPENAI_API_KEY`)
   - Anthropic (`ANTHROPIC_API_KEY`)
   - DeepSeek (`DEEPSEEK_API_KEY`)
   - 通用 (`LLM_API_KEY`)

2. ✅ **智能回退机制**
   - 优先检查 provider 对应的 API Key
   - 如果不存在，回退到通用的 `LLM_API_KEY`
   - 对于未知 provider，尝试所有常见的 API Key

3. ✅ **更清晰的错误提示**
   - 根据 provider 类型显示需要配置的环境变量
   - 例如：`未找到 LLM API Key 环境变量 (provider: zhipu, 需要: ZHIPU_API_KEY 或 LLM_API_KEY)`

---

## 🎯 验证步骤

### 1. 设置环境变量

```bash
export ZHIPU_API_KEY="your-zhipu-api-key"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"
```

### 2. 重启服务

```bash
cd agentmen
# 停止旧进程
pkill -f "agent-mem-server"

# 重新启动
./start_server_no_auth.sh
```

### 3. 验证日志

启动后应该看到：
```
✅ 成功创建 LLM Provider: zhipu (glm-4-plus)
```

而不是：
```
❌ 未找到 LLM API Key 环境变量
❌ LLM API Key 未配置，LLM Provider 将不可用
```

---

## 📋 支持的 LLM Provider 和对应的环境变量

| Provider | 环境变量 | 回退变量 |
|----------|---------|---------|
| Zhipu | `ZHIPU_API_KEY` | `LLM_API_KEY` |
| OpenAI | `OPENAI_API_KEY` | `LLM_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` | `LLM_API_KEY` |
| DeepSeek | `DEEPSEEK_API_KEY` | `LLM_API_KEY` |
| 其他/未知 | 尝试所有上述变量 | `LLM_API_KEY` |

---

## 🔧 相关文件

- **修复文件**: `agentmen/crates/agent-mem/src/orchestrator.rs`
- **检测逻辑**: `agentmen/crates/agent-mem/src/auto_config.rs`
- **配置文件**: `agentmen/config.toml`
- **启动脚本**: `agentmen/start_server_no_auth.sh`

---

## ✅ 修复完成

- ✅ 修复了 API Key 检查逻辑
- ✅ 支持 Zhipu、OpenAI、Anthropic、DeepSeek 等所有主流 Provider
- ✅ 添加了智能回退机制
- ✅ 改进了错误提示信息
- ✅ 代码已通过编译检查

**现在配置了 `ZHIPU_API_KEY` 后，系统应该能够正确识别并创建 LLM Provider！**

