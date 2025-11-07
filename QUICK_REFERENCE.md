# AgentMem + Claude Code 快速参考

## 🚀 启动

```bash
# 1. 确保Backend运行
curl http://127.0.0.1:8080/health

# 2. 启动Claude Code
claude
```

## ⚡ 常用命令

| 命令 | 说明 |
|------|------|
| `/mcp` | 查看MCP服务器连接状态 |
| `/help` | 查看帮助 |
| `/clear` | 清除当前对话 |

## 📝 快速测试（复制使用）

### 1️⃣ 验证连接
```
/mcp
```

### 2️⃣ User Scope - 个人记忆
```
帮我添加记忆：我喜欢喝咖啡看书。
user_id是"alice"，scope_type是"user"
```

### 3️⃣ Agent Scope - 工作助手
```
帮我添加记忆：明天下午2点开会。
user_id是"alice"，agent_id是"work_agent"，scope_type是"agent"
```

### 4️⃣ Run Scope - 临时笔记
```
帮我添加临时笔记：实验效果不错。
user_id是"alice"，run_id是"exp-001"，scope_type是"run"
```

### 5️⃣ 搜索记忆
```
帮我搜索alice的记忆，关键词"咖啡"
```

### 6️⃣ 列出Agents
```
帮我列出所有agents
```

## 🎯 Scope类型速查

| Scope | 用途 | 必需参数 |
|-------|------|----------|
| `user` | 个人知识库 | `user_id` |
| `agent` | 多Agent系统 | `user_id`, `agent_id` |
| `run` | 临时会话 | `user_id`, `run_id` |
| `session` | 对话隔离 | `user_id`, `session_id` |
| `organization` | 企业多租户 | `org_id` |

## 📊 可用工具

| 工具名称 | 功能 |
|----------|------|
| `agentmem_add_memory` | 添加记忆 |
| `agentmem_search_memories` | 搜索记忆 |
| `agentmem_chat` | 对话 |
| `agentmem_list_agents` | 列出Agents |
| `agentmem_get_system_prompt` | 获取系统提示 |

## 🔧 故障排除

### Backend未运行
```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
./start_server_no_auth.sh
```

### MCP未连接
```bash
# 检查配置
cat ~/.claude.json | jq '.mcpServers.agentmem'

# 重启Claude Code
```

## 📚 完整文档

- **使用指南**: `CLAUDE_CODE_USAGE_GUIDE.md`
- **测试提示词**: `claude_code_test_prompts.md`
- **技术方案**: `agentmem60.md`
- **实施报告**: `SCOPE_IMPLEMENTATION_COMPLETE.md`

---

**开始使用**: 运行 `claude`，然后输入 `/mcp` 验证连接！ 🎉
