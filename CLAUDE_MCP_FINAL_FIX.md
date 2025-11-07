# Claude Code MCP 最终修复方案

**问题**: `claude mcp list` 看不到agentmem服务器  
**日期**: 2025-11-07

---

## 🔍 问题诊断

### 当前状态

✅ **已执行**:
```bash
claude mcp add agentmem /path/to/agentmem-mcp-server -s project -e ...
# 输出: Added stdio MCP server agentmem...
```

❌ **问题**:
```bash
claude mcp list
# agentmem不在列表中，只显示其他MCP服务器
```

### 可能原因

1. **健康检查失败** - agentmem服务器无法通过健康检查
2. **启动超时** - 服务器启动时间过长
3. **stdio通信问题** - JSON-RPC通信异常
4. **环境变量问题** - AGENTMEM_API_URL等未正确传递

---

## 🔧 解决方案

### 方案1: 修改MCP服务器支持健康检查

Claude Code的健康检查可能需要特定的响应格式。修改MCP服务器代码：

**文件**: `examples/mcp-stdio-server/src/main.rs`

添加健康检查支持：

```rust
// 在 handle_request 函数中添加 ping 方法
"ping" => {
    // Claude Code健康检查
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request_id,
        result: Some(serde_json::json!({
            "status": "ok"
        })),
        error: None,
    }
}
```

### 方案2: 使用HTTP传输替代STDIO

由于STDIO可能有问题，改用HTTP传输：

#### Step 1: 启动HTTP MCP服务器

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 启动HTTP服务器（后台运行）
./target/release/agentmem-mcp-server --http --port 8081 &
```

#### Step 2: 添加HTTP MCP服务器

```bash
claude mcp add agentmem http://localhost:8081 -s project -t http
```

### 方案3: 创建包装脚本

创建一个包装脚本确保服务器正确启动：

**文件**: `start_mcp_server.sh`

```bash
#!/bin/bash
# MCP服务器包装脚本

# 设置工作目录
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 设置环境变量
export AGENTMEM_API_URL="http://127.0.0.1:8080"
export RUST_LOG="info"

# 启动服务器
exec ./target/release/agentmem-mcp-server "$@"
```

然后使用包装脚本：

```bash
chmod +x start_mcp_server.sh

claude mcp remove agentmem
claude mcp add agentmem /path/to/start_mcp_server.sh -s project
```

### 方案4: 检查实际问题

手动测试MCP服务器是否正常：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 测试initialize请求
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' | \
    ./target/release/agentmem-mcp-server 2>&1

# 应该返回成功的initialize响应
```

如果这个测试失败，说明MCP服务器本身有问题。

---

## 🚀 推荐方案：添加ping方法

### Step 1: 修改代码

编辑 `examples/mcp-stdio-server/src/main.rs`:

```rust
async fn handle_request(
    server: &Arc<McpServer>,
    _client: &Arc<AgentMemClient>,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let request_id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => {
            // 现有代码...
        }
        "ping" => {
            // 🆕 添加ping支持（Claude Code健康检查）
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request_id,
                result: Some(serde_json::json!({
                    "status": "ok",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
                error: None,
            }
        }
        "tools/list" => {
            // 现有代码...
        }
        // ... 其他方法
    }
}
```

### Step 2: 重新编译

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

cargo build --package mcp-stdio-server --release
```

### Step 3: 重新添加到Claude Code

```bash
# 删除旧的
claude mcp remove agentmem

# 添加新的
claude mcp add agentmem \
    /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server \
    -s project \
    -e AGENTMEM_API_URL=http://127.0.0.1:8080 \
    -e RUST_LOG=info

# 验证
claude mcp list
```

---

## 📋 测试清单

执行以下测试确认问题：

### 测试1: 手动stdio测试

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 测试initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
    ./target/release/agentmem-mcp-server 2>&1

# 测试tools/list
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | \
    ./target/release/agentmem-mcp-server 2>&1

# 测试ping（如果已添加）
echo '{"jsonrpc":"2.0","id":3,"method":"ping"}' | \
    ./target/release/agentmem-mcp-server 2>&1
```

### 测试2: 检查Claude Code配置

```bash
# 查看实际配置
cat ~/.config/claude/config.json

# 或项目配置
cat .mcp.json
```

### 测试3: Claude Code日志

```bash
# 查看Claude Code日志
# macOS
cat ~/Library/Logs/claude-code/mcp.log

# Linux
cat ~/.local/share/claude-code/logs/mcp.log
```

---

## 💡 临时解决方案

如果以上方案都不行，可以使用**user-level**配置：

```bash
# 使用user级别而不是project级别
claude mcp add agentmem \
    /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server \
    -s user \
    -e AGENTMEM_API_URL=http://127.0.0.1:8080

# 这样配置会保存到 ~/.config/claude/config.json
```

---

## 🎯 最终建议

**立即执行**：

1. 添加ping方法到MCP服务器
2. 重新编译
3. 使用绝对路径重新添加

**完整命令**：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 1. 编译
cargo build --package mcp-stdio-server --release

# 2. 测试
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    ./target/release/agentmem-mcp-server 2>/dev/null | jq .

# 3. 添加到Claude Code
claude mcp remove agentmem 2>/dev/null || true
claude mcp add agentmem \
    $(pwd)/target/release/agentmem-mcp-server \
    -s user \
    -e AGENTMEM_API_URL=http://127.0.0.1:8080

# 4. 验证
claude mcp list

# 5. 如果看到agentmem，测试使用
claude
# 然后输入: 你有哪些工具？
```

---

*Last Updated: 2025-11-07*  
*Status: Investigating*

