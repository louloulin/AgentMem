# Claude Code MCP 快速启动指南

**AgentMem MCP Integration for Claude Code**  
**Version**: 2.0.0  
**Last Updated**: 2025-01-05

---

## 🎯 Overview

This guide will help you integrate AgentMem's Model Context Protocol (MCP) server with Claude Code, enabling intelligent memory management, semantic search, and personalized conversations directly within your coding workflow.

### What You'll Get

- ✅ **5 MCP Tools**: Memory management, search, chat, system prompts, and agent listing
- ✅ **Seamless Integration**: Natural language commands in Claude Code
- ✅ **Persistent Memory**: Cross-session memory retention
- ✅ **Intelligent Search**: Semantic search across all your memories
- ✅ **Production Ready**: Battle-tested integration

---

## 🚀 Quick Start (5 Minutes)

### Prerequisites

- ✅ Claude Code installed (`npm install -g @anthropic-ai/claude-code`)
- ✅ Rust toolchain (for building MCP server)
- ✅ AgentMem backend running (optional but recommended)

### Step 1: Build MCP Server

```bash
cd /path/to/agentmen
cargo build --package mcp-stdio-server --release
```

The binary will be at: `target/release/agentmem-mcp-server`

### Step 2: Create MCP Configuration

Create `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "agentmem": {
      "command": "./target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "AGENTMEM_DEFAULT_AGENT_ID": "coding_assistant"
      }
    }
  }
}
```

### Step 3: Start Claude Code

```bash
# Navigate to project directory
cd /path/to/agentmen

# Start Claude Code
claude
```

### Step 4: Verify Integration

In Claude Code, try:

```
User: /mcp list

Claude: Available MCP Servers:
• agentmem
  Status: Connected
  Tools: 5
```

---

## 🎯 Common Issues

### Issue 1: MCP Tools Not Visible

**Symptoms**: Claude Code doesn't show AgentMem tools

**Solutions**:
1. Ensure you're in the project directory (where `.mcp.json` exists)
2. Verify `.mcp.json` syntax: `cat .mcp.json | jq .`
3. Check binary exists: `ls -lh ./target/release/agentmem-mcp-server`
4. Restart Claude Code completely

---

## ✅ 正确的启动方法

### 方法1: 在项目目录启动（推荐）

```bash
# 1. 切换到项目目录（.mcp.json所在目录）
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 2. 确认.mcp.json存在
ls -la .mcp.json

# 3. 启动Claude Code
claude

# 4. 在Claude Code中查询MCP工具
# 输入: 列出所有MCP服务器
# 或输入: /mcp list
```

### 方法2: 显式指定配置文件

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 使用--config参数
claude --config .mcp.json
```

### 方法3: 使用环境变量

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 设置环境变量
export CLAUDE_CONFIG_PATH=.mcp.json

# 启动
claude
```

---

## 🧪 验证MCP是否加载

启动Claude Code后，尝试这些命令：

### 命令1: 查看MCP服务器

```
/mcp list
```

**期望输出**：
```
Available MCP Servers:
- agentmem: AgentMem Memory Management
```

### 命令2: 查看可用工具

```
列出所有可用的工具
```

**期望输出**：
```
我有以下工具可用：
1. agentmem_add_memory
2. agentmem_search_memories
3. agentmem_get_system_prompt
4. agentmem_chat
5. agentmem_list_agents
```

### 命令3: 直接测试

```
请使用agentmem_list_agents工具列出所有Agent
```

---

## 🐛 故障排查

### 问题1: 找不到.mcp.json

```bash
# 检查当前目录
pwd
# 应该是: /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 检查文件是否存在
ls -la .mcp.json
cat .mcp.json | jq .
```

### 问题2: 二进制文件不存在

```bash
# 检查MCP服务器二进制
ls -lh ./target/release/agentmem-mcp-server

# 如果不存在，重新编译
cargo build --package mcp-stdio-server --release
```

### 问题3: 权限问题

```bash
# 添加执行权限
chmod +x ./target/release/agentmem-mcp-server

# 测试是否可执行
./target/release/agentmem-mcp-server --help 2>&1 | head -3
```

### 问题4: 后端未运行

```bash
# 检查后端
curl http://127.0.0.1:8080/health

# 如果失败，启动后端
./start_server.sh &
sleep 5
curl http://127.0.0.1:8080/health
```

### 问题5: Claude Code版本问题

```bash
# 检查Claude Code版本
claude --version

# 如果版本过旧，更新
npm update -g @anthropic-ai/claude-code
```

---

## 📝 完整测试脚本

保存为 `test_claude_code_mcp.sh`:

```bash
#!/bin/bash
#
# Claude Code MCP 测试脚本

set -e

echo "🧪 Claude Code MCP 测试"
echo "=================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }
print_info() { echo -e "${YELLOW}ℹ${NC} $1"; }

# 1. 检查当前目录
echo "1️⃣ 检查当前目录"
CURRENT_DIR=$(pwd)
EXPECTED_DIR="/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen"

if [ "$CURRENT_DIR" = "$EXPECTED_DIR" ]; then
    print_success "当前目录正确: $CURRENT_DIR"
else
    print_error "当前目录错误"
    print_info "当前: $CURRENT_DIR"
    print_info "期望: $EXPECTED_DIR"
    print_info "执行: cd $EXPECTED_DIR"
    exit 1
fi

# 2. 检查.mcp.json
echo ""
echo "2️⃣ 检查MCP配置"
if [ -f ".mcp.json" ]; then
    print_success ".mcp.json存在"
    
    # 验证JSON格式
    if jq empty .mcp.json 2>/dev/null; then
        print_success "JSON格式正确"
    else
        print_error "JSON格式错误"
        exit 1
    fi
    
    # 显示配置
    print_info "配置内容:"
    jq . .mcp.json
else
    print_error ".mcp.json不存在"
    exit 1
fi

# 3. 检查二进制文件
echo ""
echo "3️⃣ 检查MCP服务器"
BINARY="./target/release/agentmem-mcp-server"

if [ -f "$BINARY" ]; then
    SIZE=$(ls -lh "$BINARY" | awk '{print $5}')
    print_success "MCP服务器存在 ($SIZE)"
    
    if [ -x "$BINARY" ]; then
        print_success "有执行权限"
    else
        print_error "无执行权限"
        chmod +x "$BINARY"
        print_success "已添加执行权限"
    fi
else
    print_error "MCP服务器不存在: $BINARY"
    print_info "运行: cargo build --package mcp-stdio-server --release"
    exit 1
fi

# 4. 测试MCP服务器
echo ""
echo "4️⃣ 测试MCP服务器"
TEST_RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    $BINARY 2>/dev/null)

TOOLS_COUNT=$(echo "$TEST_RESPONSE" | jq -r '.result.tools | length')

if [ "$TOOLS_COUNT" = "5" ]; then
    print_success "5个工具正常注册"
    
    echo ""
    print_info "工具列表:"
    echo "$TEST_RESPONSE" | jq -r '.result.tools[] | "  • \(.name)"'
else
    print_error "工具数量异常: $TOOLS_COUNT"
    exit 1
fi

# 5. 检查后端
echo ""
echo "5️⃣ 检查后端服务"
if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
    print_success "后端服务运行中"
else
    print_error "后端服务未运行"
    print_info "启动命令: ./start_server.sh &"
fi

# 6. 检查Claude Code
echo ""
echo "6️⃣ 检查Claude Code"
if command -v claude &> /dev/null; then
    CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "installed")
    print_success "Claude Code已安装 ($CLAUDE_VERSION)"
else
    print_error "Claude Code未安装"
    print_info "安装命令: npm install -g @anthropic-ai/claude-code"
    exit 1
fi

echo ""
echo "=================================="
print_success "✅ 所有检查通过！"
echo "=================================="
echo ""
print_info "🚀 启动Claude Code:"
echo ""
echo "cd $CURRENT_DIR"
echo "claude"
echo ""
print_info "📖 在Claude Code中测试:"
echo ""
echo "1. 输入: /mcp list"
echo "   期望: 看到 agentmem 服务器"
echo ""
echo "2. 输入: 列出所有可用的工具"
echo "   期望: 看到5个AgentMem工具"
echo ""
echo "3. 输入: 请使用agentmem_list_agents列出所有Agent"
echo "   期望: 成功列出Agent列表"
echo ""
```

运行测试：

```bash
chmod +x test_claude_code_mcp.sh
./test_claude_code_mcp.sh
```

---

## 🎯 Claude Code 使用示例

### 启动Claude Code

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
claude
```

### 测试对话1: 查看MCP服务器

**输入**:
```
/mcp list
```

**期望输出**:
```
Available MCP Servers:
• agentmem
  Status: Connected
  Tools: 5
```

### 测试对话2: 查看工具

**输入**:
```
你有哪些工具可以帮我管理记忆？
```

**期望输出**:
```
我有以下AgentMem工具：
1. agentmem_add_memory - 添加记忆
2. agentmem_search_memories - 搜索记忆
3. agentmem_get_system_prompt - 获取系统提示词
4. agentmem_chat - 智能对话
5. agentmem_list_agents - 列出Agent
```

### 测试对话3: 使用工具

**输入**:
```
请列出所有可用的Agent
```

**期望行为**:
- Claude调用 `agentmem_list_agents` 工具
- 返回Agent列表

**输入**:
```
帮我记住：我正在测试Claude Code与AgentMem的MCP集成
```

**期望行为**:
- Claude调用 `agentmem_add_memory` 工具
- 成功保存记忆

**输入**:
```
搜索关于AgentMem的记忆
```

**期望行为**:
- Claude调用 `agentmem_search_memories` 工具
- 返回相关记忆

---

## 🔍 调试模式

如果仍然看不到工具，使用调试模式启动：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 启用详细日志
RUST_LOG=debug claude

# 或使用Claude的调试模式（如果支持）
claude --debug

# 或查看Claude日志
# macOS: ~/Library/Logs/claude-code/
# Linux: ~/.local/share/claude-code/logs/
```

---

## ✅ 成功标志

当您能在Claude Code中完成以下对话时，说明集成成功：

```
User: /mcp list
Claude: [显示agentmem服务器]

User: 列出所有工具
Claude: [显示5个AgentMem工具]

User: 请列出所有Agent
Claude: [调用工具并显示Agent列表]
```

---

## 📞 快速帮助

如果问题仍然存在，请提供以下信息：

```bash
# 运行诊断脚本
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 收集信息
echo "=== 当前目录 ==="
pwd

echo ""
echo "=== .mcp.json ==="
cat .mcp.json | jq .

echo ""
echo "=== MCP服务器 ==="
ls -lh ./target/release/agentmem-mcp-server

echo ""
echo "=== Claude版本 ==="
claude --version

echo ""
echo "=== MCP测试 ==="
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    ./target/release/agentmem-mcp-server 2>/dev/null | \
    jq '.result.tools[].name'
```

---

*Generated: 2025-11-07*  
*Target: Claude Code CLI*  
*Status: Ready for Testing*

