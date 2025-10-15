# AgentMem MCP Stdio 服务器

这是一个通过标准输入输出（stdio）与 Claude Desktop 集成的 MCP 服务器。

## 功能特性

- ✅ 完整的 JSON-RPC 2.0 协议支持
- ✅ MCP 协议 2024-11-05 版本
- ✅ 支持 4 个核心工具：
  - `agentmem_add_memory` - 添加记忆
  - `agentmem_search_memories` - 搜索记忆
  - `agentmem_chat` - 智能对话
  - `agentmem_get_system_prompt` - 获取系统提示
- ✅ 与 Claude Desktop 无缝集成

## 编译

```bash
cd agentmen
cargo build --package mcp-stdio-server --release
```

编译后的可执行文件位于：
```
agentmen/target/release/agentmem-mcp-server
```

## 配置 Claude Desktop

1. 找到 Claude Desktop 配置文件位置：
   - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
   - Linux: `~/.config/Claude/claude_desktop_config.json`

2. 编辑配置文件，添加 AgentMem MCP 服务器：

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/path/to/agentmen/target/release/agentmem-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

**注意**: 请将 `/path/to/agentmen` 替换为实际的路径。

3. 重启 Claude Desktop

## 使用示例

在 Claude Desktop 中，你可以使用以下工具：

### 1. 添加记忆

```
请使用 agentmem_add_memory 工具添加一条记忆：
内容：我喜欢 Rust 编程语言
用户ID：user123
```

### 2. 搜索记忆

```
请使用 agentmem_search_memories 工具搜索关于 Rust 的记忆
```

### 3. 智能对话

```
请使用 agentmem_chat 工具与我对话，询问我的编程偏好
```

### 4. 获取系统提示

```
请使用 agentmem_get_system_prompt 工具获取与编程相关的系统提示
```

## 测试

你可以使用以下命令手动测试服务器：

```bash
# 启动服务器
./target/release/agentmem-mcp-server

# 发送 initialize 请求（在另一个终端）
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/agentmem-mcp-server

# 列出工具
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | ./target/release/agentmem-mcp-server
```

## 日志

服务器日志输出到 stderr，不会干扰 stdio 通信。你可以查看日志以调试问题：

```bash
./target/release/agentmem-mcp-server 2> server.log
```

## 故障排除

### 问题：Claude Desktop 无法连接到服务器

**解决方案**:
1. 检查可执行文件路径是否正确
2. 确保可执行文件有执行权限：`chmod +x target/release/agentmem-mcp-server`
3. 查看 Claude Desktop 的日志文件
4. 尝试手动运行服务器，检查是否有错误

### 问题：工具调用失败

**解决方案**:
1. 检查服务器日志（stderr）
2. 确保 JSON-RPC 请求格式正确
3. 验证工具参数是否符合 schema

## 开发

如果你想修改服务器代码：

1. 编辑 `src/main.rs`
2. 重新编译：`cargo build --package mcp-stdio-server --release`
3. 重启 Claude Desktop

## 相关文档

- [MCP 协议规范](https://modelcontextprotocol.io/specification/2025-03-26/index)
- [AgentMem 文档](../../README.md)
- [mem19.md - 生产 MVP 计划](../../../doc/technical-design/memory-systems/mem19.md)

## 许可证

与 AgentMem 项目相同的许可证。

