# 🚀 启动Claude Code使用AgentMem MCP

**所有准备工作已完成！** ✅

---

## ✅ 检查结果

- ✅ 目录正确：`/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen`
- ✅ `.mcp.json` 存在且格式正确
- ✅ MCP服务器二进制存在（8.6M）
- ✅ 5个工具正常注册
- ✅ 后端服务运行中（healthy）
- ✅ Claude Code已安装（v1.0.92）

---

## 🎯 立即启动

### 方法1: 简单启动（推荐）

```bash
# 1. 确保在正确目录
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 2. 启动Claude Code
claude
```

### 方法2: 带日志启动（调试用）

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 启用详细日志
RUST_LOG=info claude
```

---

## 📝 在Claude Code中测试

启动Claude Code后，**按顺序测试以下命令**：

### 测试1: 查看MCP服务器

**输入**：
```
/mcp list
```

**期望输出**：
```
Available MCP Servers:
• agentmem
  Status: Connected
```

### 测试2: 查看工具

**输入**：
```
你有哪些工具可用？
```

**期望输出**：
```
我有以下工具：
1. agentmem_add_memory - 添加记忆
2. agentmem_search_memories - 搜索记忆
3. agentmem_get_system_prompt - 获取系统提示词
4. agentmem_chat - 智能对话
5. agentmem_list_agents - 列出Agent
```

### 测试3: 列出Agent

**输入**：
```
请列出所有可用的Agent
```

**期望行为**：
- Claude调用 `agentmem_list_agents` 工具
- 显示Agent列表（应该有10个）

### 测试4: 添加记忆

**输入**：
```
帮我记住：我正在测试Claude Code与AgentMem的MCP集成，效果很好！
```

**期望行为**：
- Claude调用 `agentmem_add_memory` 工具
- 确认记忆已保存

### 测试5: 搜索记忆

**输入**：
```
搜索关于AgentMem的记忆
```

**期望行为**：
- Claude调用 `agentmem_search_memories` 工具
- 返回刚才添加的记忆

---

## 🐛 如果看不到MCP工具

### 方案1: 重启Claude Code

```bash
# 退出Claude Code（按 Ctrl+D 或输入 exit）

# 重新启动
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
claude
```

### 方案2: 检查配置

```bash
# 确认配置正确
cat .mcp.json | jq .

# 确认在正确目录
pwd
# 应该输出: /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
```

### 方案3: 手动测试MCP服务器

```bash
# 确认MCP服务器本身工作正常
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    ./target/release/agentmem-mcp-server 2>/dev/null | \
    jq '.result.tools[].name'

# 应该输出5个工具名称
```

### 方案4: 使用详细日志

```bash
# 启用DEBUG日志
RUST_LOG=debug claude

# 查看详细的MCP加载过程
```

---

## ✅ 成功标志

当您在Claude Code中能完成以下对话时，说明集成成功：

```
User: /mcp list
Claude: [显示agentmem服务器]

User: 你有哪些工具？
Claude: [列出5个AgentMem工具]

User: 请列出所有Agent
Claude: [成功调用工具，显示Agent列表]

User: 帮我记住：测试成功
Claude: [成功添加记忆]

User: 搜索测试记忆
Claude: [成功搜索并返回结果]
```

---

## 🎉 总结

**准备工作全部完成！**

现在只需要：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
claude
```

然后在Claude Code中输入：

```
你有哪些工具？
```

就能看到AgentMem的5个MCP工具了！🚀

---

**祝使用愉快！** ✨

如有问题，查看：
- `CLAUDE_CODE_QUICKSTART.md` - 快速启动指南
- `test_claude_code_mcp.sh` - 完整测试脚本
- `REAL_CLAUDE_COMMANDS.md` - 命令参考

---

*Generated: 2025-11-07*  
*Status: ✅ Ready to Use*  
*AgentMem MCP 2.0 - Production Ready*

