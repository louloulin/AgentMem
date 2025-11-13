# Claude Code + AgentMem MCP 使用指南

**日期**: 2025-11-07  
**状态**: ✅ 已配置并可用

---

## ✅ 当前配置状态

### 1. MCP Server配置
- **配置文件**: `~/.claude.json`
- **Server名称**: `agentmem`
- **可执行文件**: `/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server`
- **API URL**: `http://127.0.0.1:8080`
- **状态**: ✅ 已配置

### 2. Backend Server状态
- **URL**: `http://127.0.0.1:8080`
- **健康状态**: ✅ 运行正常
- **数据库**: ✅ 连接成功
- **内存系统**: ✅ 运行正常

### 3. MCP Tools可用
- ✅ `agentmem_add_memory` - 添加记忆
- ✅ `agentmem_search_memories` - 搜索记忆
- ✅ `agentmem_chat` - 对话
- ✅ `agentmem_get_system_prompt` - 获取系统提示
- ✅ `agentmem_list_agents` - 列出Agents

---

## 🚀 快速开始

### Step 1: 启动Claude Code

```bash
# 在终端中运行
claude
```

### Step 2: 验证MCP Server连接

在Claude Code中输入：
```
/mcp
```

您应该看到 `agentmem` server已连接。

---

## 💡 使用示例

### 示例1: 添加User Scope记忆

**提示词**:
```
帮我添加一条用户级记忆：我喜欢吃pizza，使用agentmem_add_memory工具，
scope_type设置为"user"，user_id设置为"alice"
```

**Claude会调用**:
```json
{
  "name": "agentmem_add_memory",
  "arguments": {
    "content": "我喜欢吃pizza",
    "scope_type": "user",
    "user_id": "alice"
  }
}
```

### 示例2: 添加Agent Scope记忆

**提示词**:
```
帮我添加一条Agent级记忆：明天下午2点开会，
user_id是"alice"，agent_id是"work_assistant"，
scope_type设置为"agent"
```

**Claude会调用**:
```json
{
  "name": "agentmem_add_memory",
  "arguments": {
    "content": "明天下午2点开会",
    "scope_type": "agent",
    "user_id": "alice",
    "agent_id": "work_assistant"
  }
}
```

### 示例3: 添加Run Scope记忆（临时会话）

**提示词**:
```
帮我添加一条临时会话记忆：这是一个测试笔记，
user_id是"alice"，run_id是"temp-session-123"，
scope_type设置为"run"
```

**Claude会调用**:
```json
{
  "name": "agentmem_add_memory",
  "arguments": {
    "content": "这是一个测试笔记",
    "scope_type": "run",
    "user_id": "alice",
    "run_id": "temp-session-123"
  }
}
```

### 示例4: 搜索记忆

**提示词**:
```
帮我搜索alice的记忆，关键词是"pizza"
```

**Claude会调用**:
```json
{
  "name": "agentmem_search_memories",
  "arguments": {
    "query": "pizza",
    "user_id": "alice",
    "limit": 10
  }
}
```

### 示例5: 列出所有Agents

**提示词**:
```
帮我列出系统中所有的agents
```

**Claude会调用**:
```json
{
  "name": "agentmem_list_agents",
  "arguments": {
    "limit": 20
  }
}
```

---

## 🎯 完整测试流程

### 测试1: User Scope隔离

```
1. 添加alice的记忆：
   "帮我添加记忆：alice喜欢pizza，user_id是alice，scope_type是user"

2. 添加bob的记忆：
   "帮我添加记忆：bob喜欢sushi，user_id是bob，scope_type是user"

3. 搜索alice的记忆：
   "帮我搜索alice的记忆，关键词是食物"
   
   应该只返回alice的pizza记忆，不会返回bob的sushi
```

### 测试2: Agent Scope隔离

```
1. 添加work_assistant的记忆：
   "帮我添加记忆：明天开会，user_id是alice，agent_id是work_assistant，scope_type是agent"

2. 添加life_assistant的记忆：
   "帮我添加记忆：买菜，user_id是alice，agent_id是life_assistant，scope_type是agent"

3. 验证隔离：
   搜索work_assistant的记忆应该只看到"开会"，不会看到"买菜"
```

### 测试3: Run Scope（临时会话）

```
1. 创建临时会话记忆：
   "帮我添加临时记忆：实验笔记，user_id是alice，run_id是experiment-1，scope_type是run"

2. 验证临时性：
   这条记忆只在experiment-1这个run中可见
```

---

## 🧪 自动化验证脚本

如果您想快速验证所有功能，可以运行：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
./test_server_scope_support.sh
```

---

## 📊 常见场景

### 场景1: 个人知识管理

```
提示: 帮我记住我的偏好：
- 喜欢喝咖啡
- 周末喜欢爬山
- 正在学习Rust编程

使用user scope，user_id是我的名字
```

### 场景2: 多Agent系统

```
提示: 我有两个AI助手：
- 工作助手（work_agent）：帮我管理工作任务
- 生活助手（life_agent）：帮我管理个人事务

帮我在work_agent中添加记忆：项目deadline是12月1日
帮我在life_agent中添加记忆：周末去超市买菜
```

### 场景3: 临时实验

```
提示: 我在做一个临时实验，需要记录一些笔记。
创建一个run_id是"nlp-experiment-2025-11-07"的临时会话，
添加记忆：测试了BERT模型，效果不错
```

---

## 🔧 故障排除

### 问题1: MCP Server未连接

**症状**: `/mcp` 命令中看不到 agentmem

**解决方案**:
```bash
# 1. 检查配置
cat ~/.claude.json | jq '.mcpServers.agentmem'

# 2. 检查可执行文件
ls -lh /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server

# 3. 重新启动Claude Code
# 按Ctrl+C退出，然后重新运行: claude
```

### 问题2: Backend未运行

**症状**: 添加记忆时返回"backend unavailable"

**解决方案**:
```bash
# 1. 检查backend状态
curl http://127.0.0.1:8080/health

# 2. 如果未运行，启动backend
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
./start_server_no_auth.sh
```

### 问题3: Agent不存在

**症状**: "Agent not found"错误

**解决方案**:
AgentMem会自动创建Agent，如果仍然报错：
```bash
# 手动创建Agent
curl -X POST http://127.0.0.1:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "id": "your-agent-id",
    "name": "Your Agent Name",
    "user_id": "your-user-id"
  }'
```

---

## 📚 进阶使用

### 自定义Metadata

```
提示: 帮我添加记忆，内容是"学习笔记：Rust所有权系统"，
user_id是"alice"，scope_type是"user"，
metadata包含tags: ["Rust", "programming", "learning"]
```

### 指定Memory Type

```
提示: 帮我添加一条程序性记忆（Procedural）：
"如何编译Rust项目：运行cargo build命令"，
memory_type设置为"Procedural"，
user_id是"alice"
```

### 设置重要性

```
提示: 添加一条重要记忆：
"紧急：项目deadline提前到11月15日"，
importance设置为0.9（表示非常重要），
user_id是"alice"
```

---

## 🎉 功能清单

- [x] User Scope - 个人知识库
- [x] Agent Scope - 多Agent系统
- [x] Run Scope - 临时会话
- [x] Session Scope - 对话隔离
- [x] Organization Scope - 企业多租户（schema支持）
- [x] 自动Scope推断
- [x] Agent自动创建
- [x] Scope隔离验证
- [x] 搜索功能
- [x] 列出Agents
- [x] 健康检查

---

## 💬 示例对话

### 完整对话示例

```
User: 你好Claude！我想使用AgentMem来管理我的记忆。
      帮我添加一条记忆：我喜欢在周末喝咖啡看书。
      使用user scope，user_id设置为"alice"

Claude: 好的！我帮你添加这条记忆到AgentMem。

[Claude调用 agentmem_add_memory tool]

结果：
{
  "success": true,
  "message": "记忆已添加",
  "memory_id": "abc123...",
  "scope_type": "user",
  "user_id": "alice"
}

已成功添加你的记忆！这条记忆被存储在user scope下，
只有alice这个用户可以访问。

User: 太好了！现在帮我搜索一下我的记忆，看看有没有关于咖啡的。

Claude: 当然！让我搜索你的记忆。

[Claude调用 agentmem_search_memories tool]

找到了1条相关记忆：
- 我喜欢在周末喝咖啡看书

User: 完美！现在我想为我的工作助手添加一条记忆：
      明天下午2点要参加项目会议。
      agent_id是"work_assistant"

Claude: 好的！我帮你为work_assistant添加这条记忆。

[Claude调用 agentmem_add_memory tool with agent scope]

已成功添加！这条记忆只在work_assistant这个agent的范围内可见。
```

---

## 🚀 开始使用

现在您可以：

1. **打开终端**，运行 `claude`
2. **验证连接**: 输入 `/mcp`，确认看到 `agentmem` server
3. **开始对话**: 按照上面的示例提示词进行交互
4. **探索功能**: 尝试不同的scope类型和场景

---

**祝您使用愉快！AgentMem + Claude Code为您提供强大的多维度记忆管理能力！** 🎉

---

*更多详情请参阅:*
- *技术文档: `agentmem60.md`*
- *实施报告: `SCOPE_IMPLEMENTATION_COMPLETE.md`*
- *测试脚本: `test_server_scope_support.sh`*

