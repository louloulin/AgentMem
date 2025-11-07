# 如何在Claude Code中调用AgentMem MCP

**日期**: 2025-11-07  
**状态**: ✅ 完整指南

---

## 🎯 核心方法

根据官方文档，Claude Code会**自动发现和使用**配置的MCP服务器。虽然`claude mcp list`健康检查中可能不显示，但**只要配置正确，就可以直接使用**。

---

## ✅ 确认配置正确

### Step 1: 检查配置文件

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cat .mcp.json
```

**应该看到**：
```json
{
  "mcpServers": {
    "agentmem": {
      "type": "stdio",
      "command": "/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Step 2: 确认后端运行

```bash
curl http://127.0.0.1:8080/health
# 应该返回: {"status":"healthy"}
```

---

## 🚀 启动Claude Code

### 方法1: 普通启动

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
claude
```

### 方法2: 调试模式（推荐）

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
claude --mcp-debug
```

这会显示MCP加载的详细信息，帮助诊断问题。

---

## 💬 在Claude中调用AgentMem

启动Claude Code后，**直接在对话中使用**，无需特殊命令！

### 测试1: 让Claude发现工具

**输入**：
```
你能使用哪些工具来管理记忆？
```

**期望**：Claude会自动发现并列出AgentMem的5个工具

### 测试2: 直接请求使用工具

**输入**：
```
请使用agentmem_list_agents工具列出所有可用的Agent
```

**期望**：Claude会调用`agentmem_list_agents`工具并显示结果

### 测试3: 自然语言请求

**输入**：
```
帮我列出所有的Agent
```

**期望**：Claude会自动选择合适的工具（agentmem_list_agents）并执行

### 测试4: 添加记忆

**输入**：
```
帮我记住：我正在测试Claude Code与AgentMem的MCP集成，目前进展顺利
```

**期望**：Claude会调用`agentmem_add_memory`工具

### 测试5: 搜索记忆

**输入**：
```
搜索关于AgentMem的记忆
```

**期望**：Claude会调用`agentmem_search_memories`工具

---

## 🔍 如果Claude看不到工具

### 方案1: 使用user级别配置

```bash
# 删除project配置
claude mcp remove agentmem

# 添加到user级别（全局）
claude mcp add agentmem \
  /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server \
  -s user \
  -e AGENTMEM_API_URL=http://127.0.0.1:8080 \
  -e RUST_LOG=info

# 验证
claude mcp list
```

### 方案2: 直接编辑全局配置

```bash
# 编辑用户配置文件
nano ~/.claude.json
```

添加以下内容：

```json
{
  "mcpServers": {
    "agentmem": {
      "type": "stdio",
      "command": "/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 方案3: 使用allowedTools权限

如果工具被加载但无法使用，添加权限配置：

```bash
# 编辑配置文件，添加allowedTools
nano ~/.claude.json
```

```json
{
  "mcpServers": {
    "agentmem": { ... }
  },
  "allowedTools": [
    "mcp__agentmem"
  ]
}
```

---

## 🧪 完整测试脚本

```bash
#!/bin/bash
# 完整测试AgentMem MCP调用

cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

echo "🔍 检查配置..."
cat .mcp.json | jq .

echo ""
echo "🔍 检查后端..."
curl -s http://127.0.0.1:8080/health | jq .

echo ""
echo "🔍 手动测试MCP服务器..."
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
  ./target/release/agentmem-mcp-server 2>/dev/null | \
  jq -r '.result.tools[].name'

echo ""
echo "✅ 准备就绪！现在启动Claude Code："
echo ""
echo "  claude --mcp-debug"
echo ""
echo "📝 在Claude中测试："
echo "  1. 你有哪些工具？"
echo "  2. 请列出所有Agent"
echo "  3. 帮我记住：测试成功"
echo ""
```

---

## 📊 预期行为

### 正常情况

当配置正确时，Claude Code会：

1. **启动时**自动加载`.mcp.json`中的MCP服务器
2. **自动发现**5个AgentMem工具
3. **根据对话内容**自动选择和调用合适的工具
4. **显示工具调用**和结果

### 示例对话

```
User: 你好！你能帮我管理记忆吗？

Claude: 当然可以！我有以下AgentMem工具可以帮你管理记忆：

1. agentmem_add_memory - 添加新记忆
2. agentmem_search_memories - 搜索已有记忆
3. agentmem_get_system_prompt - 获取系统提示词
4. agentmem_chat - 智能对话
5. agentmem_list_agents - 列出Agent

你想使用哪个功能？

User: 请列出所有Agent

Claude: [调用 agentmem_list_agents 工具]

我找到了以下Agent：
1. Fixed Test Agent (agent-4dece7ca-...)
2. Complete Verification Agent (agent-248396d0-...)
...

User: 帮我记住：今天学会了使用Claude Code的MCP功能

Claude: [调用 agentmem_add_memory 工具]

好的，我已经帮你记住了这条信息！
```

---

## 🐛 故障排查

### 问题1: Claude启动时报错

**解决**：
```bash
# 使用调试模式查看详细错误
claude --mcp-debug

# 检查日志
# macOS: ~/Library/Logs/claude-code/
# Linux: ~/.local/share/claude-code/logs/
```

### 问题2: Claude看不到AgentMem工具

**解决**：
```bash
# 1. 确认配置存在
cat .mcp.json

# 2. 手动测试MCP服务器
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
  ./target/release/agentmem-mcp-server 2>/dev/null | jq .

# 3. 尝试user级别配置
claude mcp remove agentmem
claude mcp add agentmem $(pwd)/target/release/agentmem-mcp-server -s user \
  -e AGENTMEM_API_URL=http://127.0.0.1:8080

# 4. 重启Claude
```

### 问题3: 工具调用失败

**解决**：
```bash
# 确认后端运行
curl http://127.0.0.1:8080/health

# 如果未运行，启动
./start_server.sh &
sleep 5
curl http://127.0.0.1:8080/health
```

---

## ✅ 快速开始（一键命令）

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 1. 确认后端运行
curl http://127.0.0.1:8080/health || ./start_server.sh &

# 2. 重新配置（user级别）
claude mcp remove agentmem 2>/dev/null
claude mcp add agentmem $(pwd)/target/release/agentmem-mcp-server -s user \
  -e AGENTMEM_API_URL=http://127.0.0.1:8080

# 3. 启动Claude Code
claude --mcp-debug
```

然后在Claude中直接说：
```
你好！请列出所有可用的Agent
```

---

## 🎉 成功标志

当您在Claude Code中看到类似这样的响应时，说明成功：

```
User: 你有哪些工具？

Claude: 我有以下工具可用：
[显示包括agentmem_*在内的工具列表]

User: 列出所有Agent

Claude: [使用 agentmem_list_agents 工具]
我找到了10个Agent：
1. Fixed Test Agent (agent-4dece7ca-...)
2. Complete Verification Agent (agent-248396d0-...)
...
```

---

*Last Updated: 2025-11-07*  
*Status: ✅ Ready to Use*  
*Just say what you want to do!*

