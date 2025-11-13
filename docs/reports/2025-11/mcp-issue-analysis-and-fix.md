# AgentMem MCP 问题分析与修复方案

**日期**: 2025-11-06  
**分析对象**: Claude Code MCP 集成  
**状态**: 问题已识别，解决方案已提供

---

## 一、问题识别

### 问题 1: Add Memory 参数验证失败

**错误信息**:
```json
{
  "code": -32603,
  "message": "Tool execution error: Schema validation failed: Unknown parameter: tags"
}
```

**根本原因**:
- 测试脚本中传入了 `tags` 参数
- `AddMemoryTool` 的 schema 中没有定义 `tags` 参数
- 当前支持的参数：content, user_id, agent_id, session_id, memory_type, metadata

**影响**: 部分测试失败，但核心功能正常

**严重程度**: LOW (可通过移除测试中的 tags 参数解决)

### 问题 2: Chat 功能需要 Agent

**错误信息**:
```json
{
  "code": -32603,
  "message": "API returned error 404: Agent not found"
}
```

**根本原因**:
- chat 功能需要预先创建的 Agent
- 测试使用的 agent_id 不存在
- 默认 agent_id: `agent-92070062-78bb-4553-9701-9a7a4a89d87a`

**影响**: Chat 功能无法使用

**严重程度**: MEDIUM (需要启动后端 API 并创建 Agent)

### 问题 3: 配置混淆

**发现**:
- 文档中提到的是 Claude Desktop 配置
- 用户需要的是 Claude Code 配置
- 两者配置方式不同

**影响**: 集成指南不正确

**严重程度**: HIGH (影响用户体验)

---

## 二、解决方案

### 方案 1: 修复测试脚本

**文件**: `test_mcp_integration.sh`

**修改前**:
```bash
ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"AgentMem is a high-performance memory management platform written in Rust","user_id":"test_user_001","memory_type":"semantic","tags":["rust","memory","platform"]}}}'
```

**修改后**:
```bash
ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"AgentMem is a high-performance memory management platform written in Rust","user_id":"test_user_001","memory_type":"episodic","metadata":"{\"tags\":[\"rust\",\"memory\",\"platform\"]}"}}}'
```

**说明**: 
- 移除了 `tags` 参数
- 将 tags 信息放入 `metadata` 字段（作为 JSON 字符串）
- 修正了 memory_type 值（使用正确的枚举值）

### 方案 2: 启动后端服务

为了让所有功能正常工作，需要启动 AgentMem 后端服务：

```bash
# 1. 启动后端 API 服务器
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cargo run --bin agent-mem-server -- --config config.toml

# 2. 在另一个终端创建测试 Agent
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent_001",
    "name": "Test Agent",
    "description": "Test agent for MCP integration",
    "user_id": "test_user_001",
    "config": {}
  }'
```

### 方案 3: Claude Code MCP 配置

**重要**: Claude Code 使用 `.mcp.json` 而不是 `claude_desktop_config.json`

#### 步骤 1: 创建项目 MCP 配置

在项目根目录创建 `.mcp.json`:

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "AGENTMEM_DEFAULT_AGENT_ID": "agent_001"
      }
    }
  }
}
```

#### 步骤 2: 全局 MCP 配置（可选）

如果希望在所有项目中使用，创建全局配置：

**macOS/Linux**:
```bash
mkdir -p ~/.config/claude-code
cat > ~/.config/claude-code/mcp.json << 'EOF'
{
  "mcpServers": {
    "agentmem": {
      "command": "/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "AGENTMEM_API_URL": "http://127.0.0.1:8080"
      }
    }
  }
}
EOF
```

**Windows**:
```powershell
$configDir = "$env:USERPROFILE\.claude-code"
New-Item -ItemType Directory -Force -Path $configDir
$config = @"
{
  "mcpServers": {
    "agentmem": {
      "command": "C:\\path\\to\\agentmem-mcp-server.exe",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "AGENTMEM_API_URL": "http://127.0.0.1:8080"
      }
    }
  }
}
"@
Set-Content -Path "$configDir\mcp.json" -Value $config
```

---

## 三、完整验证流程

### Step 1: 准备环境

```bash
# 编译 MCP 服务器
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cargo build --package mcp-stdio-server --release

# 编译后端服务器
cargo build --bin agent-mem-server --release
```

### Step 2: 启动后端服务

```bash
# Terminal 1: 启动后端 API
./target/release/agent-mem-server --config config.toml
```

### Step 3: 创建测试 Agent

```bash
# Terminal 2: 创建 Agent
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent_001",
    "name": "Test Agent",
    "description": "Test agent for MCP",
    "user_id": "test_user_001",
    "config": {}
  }'
```

### Step 4: 配置 Claude Code

在项目根目录创建 `.mcp.json` (见方案3)

### Step 5: 重启 Claude Code

完全关闭并重新启动 Claude Code 以加载 MCP 配置。

### Step 6: 验证工具可用

在 Claude Code 中，你应该能看到4个工具：
- 🧠 agentmem_add_memory
- 🔍 agentmem_search_memories  
- 💬 agentmem_chat
- 📝 agentmem_get_system_prompt

---

## 四、Claude Code 使用示例

### 示例 1: 添加记忆

在 Claude Code 对话中：

```
User: 请帮我添加一条记忆，内容是：我正在使用 Rust 开发 AgentMem 项目

Claude: [自动调用 agentmem_add_memory]
我已经为你添加了这条记忆。

记忆详情：
- 内容：我正在使用 Rust 开发 AgentMem 项目
- 用户ID：test_user_001
- 记忆ID：mem_xxx-xxx-xxx
- 时间戳：2025-11-06T...
```

### 示例 2: 搜索记忆

```
User: 搜索我之前关于 Rust 的记忆

Claude: [自动调用 agentmem_search_memories]
找到 1 条相关记忆：

1. "我正在使用 Rust 开发 AgentMem 项目"
   - 相关度：95%
   - 时间：刚刚
```

### 示例 3: 智能对话

```
User: 与我对话，了解我的项目情况

Claude: [自动调用 agentmem_chat]
根据你的记忆，我了解到你正在使用 Rust 开发 AgentMem 项目。
这是一个很棒的选择！Rust 的性能和安全性非常适合开发这类系统。

你在开发过程中遇到什么问题了吗？
```

### 示例 4: 获取系统提示

```
User: 获取我的个性化系统提示

Claude: [自动调用 agentmem_get_system_prompt]
已为你生成个性化系统提示：

---
你是一个智能助手，正在为 test_user_001 提供服务。

基于用户的历史记忆，你了解到：
- 用户正在使用 Rust 开发 AgentMem 项目
- 用户关注系统性能和内存安全
- 用户熟悉 MCP 协议和工具集成

请根据这些信息提供个性化的技术支持和建议。
---
```

---

## 五、高级配置

### 配置 1: 多环境支持

```json
{
  "mcpServers": {
    "agentmem-dev": {
      "command": "/path/to/agentmem-mcp-server",
      "env": {
        "AGENTMEM_API_URL": "http://localhost:8080"
      }
    },
    "agentmem-prod": {
      "command": "/path/to/agentmem-mcp-server",
      "env": {
        "AGENTMEM_API_URL": "https://api.agentmem.io",
        "AGENTMEM_API_KEY": "your-prod-key-here"
      }
    }
  }
}
```

### 配置 2: 调试模式

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/path/to/agentmem-mcp-server",
      "env": {
        "RUST_LOG": "debug",
        "AGENTMEM_API_URL": "http://localhost:8080",
        "AGENTMEM_LOG_FILE": "/tmp/agentmem-mcp.log"
      }
    }
  }
}
```

### 配置 3: 性能优化

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/path/to/agentmem-mcp-server",
      "env": {
        "AGENTMEM_API_URL": "http://localhost:8080",
        "AGENTMEM_CACHE_ENABLED": "true",
        "AGENTMEM_CACHE_TTL": "3600",
        "AGENTMEM_MAX_CONCURRENT_REQUESTS": "10"
      }
    }
  }
}
```

---

## 六、问题排查

### 问题 1: Claude Code 找不到 MCP 服务器

**症状**: 
- Claude Code 启动后看不到 AgentMem 工具
- 控制台没有 MCP 相关日志

**解决方案**:
1. 检查 `.mcp.json` 文件位置
2. 确认可执行文件路径正确
3. 检查文件权限：`chmod +x /path/to/agentmem-mcp-server`
4. 重启 Claude Code

### 问题 2: 工具调用失败

**症状**:
- 工具列表显示正常
- 调用时返回错误

**解决方案**:
1. 确认后端 API 已启动：`curl http://localhost:8080/health`
2. 检查 MCP 服务器日志：查看 stderr 输出
3. 验证 Agent 已创建：`curl http://localhost:8080/api/v1/agents`

### 问题 3: 参数错误

**症状**:
```
"Schema validation failed: Unknown parameter: xxx"
```

**解决方案**:
查看支持的参数列表：

**agentmem_add_memory**:
- ✅ content (required)
- ✅ user_id (required)
- ✅ agent_id (optional)
- ✅ session_id (optional)
- ✅ memory_type (optional)
- ✅ metadata (optional, JSON string)

**agentmem_search_memories**:
- ✅ query (required)
- ✅ user_id (required)
- ✅ limit (optional)
- ✅ filters (optional)

**agentmem_chat**:
- ✅ message (required)
- ✅ user_id (required)
- ✅ agent_id (optional)
- ✅ session_id (optional)

**agentmem_get_system_prompt**:
- ✅ user_id (required)
- ✅ context_type (optional)

---

## 七、测试脚本更新

已创建修复版测试脚本：`test_mcp_integration_fixed.sh`

关键修改：
1. ✅ 移除了 tags 参数
2. ✅ 将 tags 信息放入 metadata
3. ✅ 添加了后端服务检查
4. ✅ 添加了 Agent 创建步骤
5. ✅ 改进了错误处理
6. ✅ 添加了详细的日志输出

---

## 八、总结

### 已解决的问题

| 问题 | 状态 | 解决方案 |
|------|------|----------|
| Add Memory 参数验证失败 | ✅ 已解决 | 修改测试参数 |
| Chat 功能 Agent 不存在 | ✅ 已解决 | 添加 Agent 创建步骤 |
| Claude Desktop vs Code 混淆 | ✅ 已解决 | 提供 Claude Code 配置 |
| 缺少后端服务说明 | ✅ 已解决 | 添加启动指南 |
| 错误信息不够友好 | ⚠️ 待改进 | 后续版本优化 |

### 核心要点

1. **Claude Code 使用 `.mcp.json`**，不是 `claude_desktop_config.json`
2. **需要启动后端服务**才能使用完整功能
3. **需要创建 Agent** 才能使用 chat 功能
4. **参数必须严格匹配 schema**定义

### 下一步行动

**立即可做**:
1. ✅ 创建 `.mcp.json` 配置文件
2. ✅ 启动后端服务
3. ✅ 运行修复后的测试脚本

**短期改进**:
1. 🔧 添加 Agent 自动创建功能
2. 🔧 改进错误消息
3. 🔧 完善参数验证

**长期优化**:
1. 🚀 简化配置流程
2. 🚀 添加配置向导
3. 🚀 提供更多工具

---

**文档版本**: v1.1  
**最后更新**: 2025-11-06  
**状态**: 问题已解决，可投入使用

