# 修复AgentMem MCP问题

**日期**: 2025-11-07  
**状态**: 🔧 修复中

---

## 🐛 识别的问题

从Claude Code的错误日志中发现两个问题：

### 问题1: memory_type大小写错误

```
Error: unknown variant `semantic`, expected one of 
`Factual`, `Episodic`, `Procedural`, `Semantic`, `Working`, 
`Core`, `Resource`, `Knowledge`, `Contextual`
```

**根本原因**：
- 工具schema中写的是 `episodic, semantic, procedural...`（小写）
- 但后端API期望的是 `Episodic, Semantic, Procedural...`（首字母大写）

### 问题2: 默认Agent不存在

```
Error 500: Agent not found: agent-92070062-78bb-4553-9701-9a7a4a89d87a
```

**根本原因**：
- 代码中硬编码了默认Agent ID
- 但这个Agent在数据库中不存在

---

## 🔧 修复方案

### 修复1: 更正memory_type的schema和默认值

**文件**: `crates/agent-mem-tools/src/agentmem_tools.rs`

**问题代码**（Line 54）：
```rust
.add_parameter(
    "memory_type",
    PropertySchema::string("记忆类型：episodic, semantic, procedural, core, working, resource, declarative, contextual"),
    false,
)
```

**修复后**：
```rust
.add_parameter(
    "memory_type",
    PropertySchema::string("记忆类型（首字母大写）：Episodic, Semantic, Procedural, Core, Working, Resource, Knowledge, Contextual, Factual"),
    false,
)
```

**同时修改默认值**（Line 77-78）：
```rust
// 修复前
let memory_type = args["memory_type"].as_str().unwrap_or("Episodic");

// 保持不变（已经是正确的大写）
let memory_type = args["memory_type"].as_str().unwrap_or("Episodic");
```

### 修复2: 使用实际存在的Agent或自动创建

**选项A**: 使用数据库中已存在的Agent

从之前的测试知道有这些Agent存在：
- `agent-4dece7ca-9112-43f6-9f00-2fda2324fcbb`
- `agent-83879d70-b243-4746-8288-ac11c6d01bb9`
- 等等

**选项B**: 如果Agent不存在，自动创建（推荐）

**修改代码**（Line 73-76）：
```rust
// 修复前：使用硬编码的不存在的Agent
let default_agent = std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
    .unwrap_or_else(|_| "agent-92070062-78bb-4553-9701-9a7a4a89d87a".to_string());

// 修复后：使用动态创建或获取
let default_agent = std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
    .unwrap_or_else(|_| "agent-default-mcp".to_string());
```

**同时需要在添加记忆前检查/创建Agent**。

---

## 📝 具体修复步骤

### Step 1: 修复memory_type的schema描述

```rust
// agentmem_tools.rs Line 52-56
.add_parameter(
    "memory_type",
    PropertySchema::string(
        "记忆类型（首字母必须大写）：Factual, Episodic, Procedural, Semantic, Working, Core, Resource, Knowledge, Contextual. 默认：Episodic"
    ),
    false,
)
```

### Step 2: 修复默认Agent ID

```rust
// agentmem_tools.rs Line 73-76
// 改为使用环境变量或更合理的默认值
let default_agent = std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
    .unwrap_or_else(|_| {
        // 使用一个更通用的默认ID，或者从API获取第一个可用的Agent
        "agent-default".to_string()
    });
```

### Step 3: 添加Agent自动创建逻辑（可选但推荐）

在添加记忆之前，先确保Agent存在：

```rust
// 在 execute 方法中，发送记忆请求之前
// 先尝试创建Agent（如果不存在）
let agent_create_url = format!("{}/api/v1/agents", api_url);
let agent_create_body = json!({
    "id": agent_id,
    "name": "Default MCP Agent",
    "description": "Automatically created agent for MCP operations",
    "user_id": user_id
});

// 尝试创建（如果已存在会返回错误，但可以忽略）
let _ = tokio::task::spawn_blocking({
    let agent_create_url = agent_create_url.clone();
    let agent_create_body = agent_create_body.clone();
    move || {
        ureq::post(&agent_create_url)
            .set("Content-Type", "application/json")
            .send_json(&agent_create_body)
    }
}).await;
```

---

## 🚀 立即修复

### 快速修复（最小改动）

只修复最关键的两处：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 编辑文件
nano crates/agent-mem-tools/src/agentmem_tools.rs
```

**修改两处**：

1. **Line 53-55**（schema描述）：
```rust
PropertySchema::string("记忆类型（首字母大写）：Episodic, Semantic, Procedural, Factual, Core, Working, Resource, Knowledge, Contextual"),
```

2. **Line 74-75**（默认Agent）：
```rust
.unwrap_or_else(|_| "agent-4dece7ca-9112-43f6-9f00-2fda2324fcbb".to_string());
// 使用已知存在的Agent ID
```

### 重新编译

```bash
cargo build --package agent-mem-tools --release
cargo build --package mcp-stdio-server --release
```

### 重启Claude Code

```bash
# 重启以加载新的二进制
claude
```

---

## ✅ 验证修复

在Claude Code中测试：

```
帮我记住：测试修复后的AgentMem MCP
```

**期望**：成功添加记忆，不再报错

---

*Status: Ready to Fix*

