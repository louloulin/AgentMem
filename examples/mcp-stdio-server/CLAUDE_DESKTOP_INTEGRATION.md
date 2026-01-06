# Claude Desktop 集成指南

本指南将帮助你将 AgentMem MCP Stdio 服务器集成到 Claude Desktop 中。

## 前提条件

1. ✅ 已编译 AgentMem MCP Stdio 服务器
   ```bash
   cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
   cargo build --package mcp-stdio-server --release
   ```

2. ✅ 可执行文件位于：
   ```
   /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server
   ```

3. ✅ 已安装 Claude Desktop（从 https://claude.ai/download 下载）

## 配置步骤

### Step 1: 找到 Claude Desktop 配置文件

根据你的操作系统，配置文件位于：

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

### Step 2: 编辑配置文件

打开配置文件（如果不存在则创建），添加以下内容：

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

**注意**: 请确保 `command` 路径指向你实际的可执行文件位置。

### Step 3: 重启 Claude Desktop

完全退出 Claude Desktop 并重新启动，以加载新的配置。

### Step 4: 验证集成

在 Claude Desktop 中，你应该能看到 AgentMem 工具已经可用。你可以尝试以下命令：

#### 测试 1: 添加记忆

```
请使用 agentmem_add_memory 工具添加一条记忆：
内容：我喜欢 Rust 编程语言
用户ID：user123
```

**预期响应**:
```json
{
  "success": true,
  "message": "记忆已添加",
  "memory_id": "mem_xxx-xxx-xxx",
  "content": "我喜欢 Rust 编程语言",
  "user_id": "user123",
  "memory_type": "episodic",
  "timestamp": "2025-10-15T..."
}
```

#### 测试 2: 搜索记忆

```
请使用 agentmem_search_memories 工具搜索关于 Rust 的记忆
```

**预期响应**:
```json
{
  "success": true,
  "query": "Rust",
  "results": [
    {
      "memory_id": "mem_001",
      "content": "与 'Rust' 相关的记忆 1",
      "relevance_score": 0.95,
      "timestamp": "..."
    },
    ...
  ],
  "total_results": 2
}
```

#### 测试 3: 智能对话

```
请使用 agentmem_chat 工具与我对话：
消息：你好，我想学习 Rust
用户ID：user123
```

**预期响应**:
```json
{
  "success": true,
  "message": "你好，我想学习 Rust",
  "user_id": "user123",
  "response": "基于您的记忆，我理解您说的是：你好，我想学习 Rust。让我为您提供相关的回复...",
  "memory_context_used": 3,
  "timestamp": "..."
}
```

#### 测试 4: 获取系统提示

```
请使用 agentmem_get_system_prompt 工具获取我的系统提示
用户ID：user123
```

**预期响应**:
```json
{
  "success": true,
  "user_id": "user123",
  "system_prompt": "你是一个智能助手，正在为用户 user123 提供服务。\n基于用户的历史记忆，你了解到：\n- 用户偏好使用 Rust 编程语言\n- 用户关注系统性能和安全性\n- 用户最近在研究 MCP 协议\n\n请根据这些信息提供个性化的帮助。",
  "memory_count": 15,
  "timestamp": "..."
}
```

## 故障排除

### 问题 1: Claude Desktop 无法找到服务器

**症状**: Claude Desktop 启动后没有显示 AgentMem 工具

**解决方案**:
1. 检查配置文件路径是否正确
2. 检查可执行文件路径是否正确
3. 确保可执行文件有执行权限：
   ```bash
   chmod +x /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server
   ```
4. 查看 Claude Desktop 的日志文件（通常在 `~/Library/Logs/Claude/` 或类似位置）

### 问题 2: 工具调用失败

**症状**: 工具列表显示正常，但调用时出错

**解决方案**:
1. 手动测试服务器：
   ```bash
   cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/examples/mcp-stdio-server
   python3 test_server.py
   ```
2. 查看服务器日志（stderr 输出）
3. 确保所有依赖都已正确安装

### 问题 3: 服务器启动失败

**症状**: Claude Desktop 报告服务器无法启动

**解决方案**:
1. 手动运行服务器查看错误信息：
   ```bash
   /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server
   ```
2. 检查是否缺少动态库依赖
3. 确保 Rust 运行时环境正确

### 问题 4: JSON 解析错误

**症状**: 服务器返回 JSON 解析错误

**解决方案**:
1. 确保使用最新版本的 Claude Desktop
2. 检查服务器日志中的详细错误信息
3. 验证 JSON-RPC 2.0 消息格式是否正确

## 高级配置

### 自定义环境变量

你可以在配置文件中添加环境变量：

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/path/to/agentmem-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "AGENTMEM_CONFIG": "/path/to/config.toml"
      }
    }
  }
}
```

### 添加命令行参数

如果服务器支持命令行参数，可以这样配置：

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/path/to/agentmem-mcp-server",
      "args": ["--config", "/path/to/config.toml", "--verbose"],
      "env": {}
    }
  }
}
```

## 日志和调试

### 查看服务器日志

服务器日志输出到 stderr，你可以通过以下方式查看：

1. **手动运行服务器**:
   ```bash
   /path/to/agentmem-mcp-server 2> server.log
   ```

2. **在 Claude Desktop 中查看**:
   - macOS: `~/Library/Logs/Claude/`
   - Windows: `%APPDATA%\Claude\Logs\`
   - Linux: `~/.local/share/Claude/logs/`

### 启用详细日志

设置环境变量 `RUST_LOG=debug` 以启用详细日志：

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/path/to/agentmem-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

## 下一步

集成成功后，你可以：

1. 在 Claude Desktop 中使用 AgentMem 的所有功能
2. 通过对话自然地添加和检索记忆
3. 让 Claude 基于你的历史记忆提供个性化服务
4. 探索更多 AgentMem 的高级功能

## 支持

如果遇到问题，请：

1. 查看本文档的故障排除部分
2. 运行测试脚本验证服务器功能
3. 查看服务器日志获取详细错误信息
4. 提交 Issue 到 GitHub 仓库

---

**祝你使用愉快！** 🎉

