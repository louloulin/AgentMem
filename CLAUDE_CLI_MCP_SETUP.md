# Claude CLI MCP 真实命令行集成指南

**版本**: 2.0  
**日期**: 2025-11-07  
**目标**: 通过命令行真实添加和验证AgentMem MCP

---

## 🎯 重要说明

根据Anthropic官方文档，**Claude Code**（命令行工具）和**Claude Desktop**（桌面应用）对MCP的支持方式不同：

1. **Claude Desktop** - 通过 `claude_desktop_config.json` 配置MCP
2. **Claude Code** - 通过项目内 `.mcp.json` 或 `CLAUDE.md` 配置

我们的AgentMem MCP支持**两种方式**。

---

## 方案1: Claude Desktop（推荐）

### Step 1: 检查Claude Desktop安装

```bash
# macOS
ls -la ~/Library/Application\ Support/Claude/

# Linux
ls -la ~/.config/Claude/

# Windows (PowerShell)
dir "$env:APPDATA\Claude"
```

### Step 2: 创建或更新配置文件

**macOS配置路径**:
```bash
mkdir -p ~/Library/Application\ Support/Claude/
nano ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

**Linux配置路径**:
```bash
mkdir -p ~/.config/Claude/
nano ~/.config/Claude/claude_desktop_config.json
```

**配置内容**:
```json
{
  "mcpServers": {
    "agentmem": {
      "command": "/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "AGENTMEM_TIMEOUT": "30",
        "RUST_LOG": "info"
      }
    }
  }
}
```

**⚠️ 注意**: 请将 `command` 路径修改为您的**绝对路径**！

### Step 3: 重启Claude Desktop

```bash
# macOS - 完全退出并重启
osascript -e 'quit app "Claude"'
sleep 2
open -a "Claude"

# Linux
killall claude
sleep 2
claude &

# 或手动重启应用
```

### Step 4: 验证MCP集成

在Claude Desktop中输入：

```
你有哪些MCP工具可用？
```

**期望输出**: Claude应该列出5个AgentMem工具。

---

## 方案2: Claude Code（命令行）

### Step 1: 安装Claude Code

```bash
# 通过npm安装（如果尚未安装）
npm install -g @anthropic-ai/claude-code

# 验证安装
claude --version
```

### Step 2: 在项目中配置MCP

我们已经创建了 `.mcp.json`，现在验证：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 验证配置文件
cat .mcp.json | jq .

# 应该输出：
# {
#   "mcpServers": {
#     "agentmem": {
#       "command": "./target/release/agentmem-mcp-server",
#       ...
#     }
#   }
# }
```

### Step 3: 启动Claude Code

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 启动Claude Code
claude
```

### Step 4: 测试MCP集成

在Claude Code的交互式会话中：

```
请列出所有可用的MCP工具
```

或使用斜杠命令：

```
/mcp list
```

---

## 🚀 完整命令行脚本

我们创建一个自动化脚本来完成整个流程：

### 脚本: `setup_claude_mcp.sh`

```bash
#!/bin/bash
#
# Claude MCP 自动配置脚本

set -e

echo "🔧 Claude MCP 自动配置工具"
echo "=================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }
print_info() { echo -e "${YELLOW}ℹ${NC} $1"; }
print_section() { echo -e "${BLUE}▶${NC} $1"; }

# 获取当前绝对路径
CURRENT_DIR=$(pwd)
BINARY_PATH="$CURRENT_DIR/target/release/agentmem-mcp-server"

print_section "检查环境"

# 1. 检查二进制文件
if [ -f "$BINARY_PATH" ]; then
    print_success "MCP服务器二进制文件存在"
else
    print_error "MCP服务器二进制文件不存在: $BINARY_PATH"
    print_info "运行: cargo build --package mcp-stdio-server --release"
    exit 1
fi

# 2. 检查后端服务
if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
    print_success "AgentMem后端服务运行中"
else
    print_error "AgentMem后端服务未运行"
    print_info "启动后端: ./start_server.sh &"
    exit 1
fi

echo ""
print_section "选择配置方式"
echo "1. Claude Desktop (推荐)"
echo "2. Claude Code (命令行)"
echo ""
read -p "请选择 [1/2]: " choice

case $choice in
    1)
        print_section "配置 Claude Desktop"
        
        # 确定配置路径
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            CONFIG_DIR="$HOME/Library/Application Support/Claude"
            CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            # Linux
            CONFIG_DIR="$HOME/.config/Claude"
            CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
        else
            print_error "不支持的操作系统: $OSTYPE"
            exit 1
        fi
        
        # 创建目录
        mkdir -p "$CONFIG_DIR"
        print_success "配置目录: $CONFIG_DIR"
        
        # 备份现有配置
        if [ -f "$CONFIG_FILE" ]; then
            BACKUP_FILE="$CONFIG_FILE.backup.$(date +%Y%m%d_%H%M%S)"
            cp "$CONFIG_FILE" "$BACKUP_FILE"
            print_info "已备份现有配置: $BACKUP_FILE"
        fi
        
        # 写入配置
        cat > "$CONFIG_FILE" << EOF
{
  "mcpServers": {
    "agentmem": {
      "command": "$BINARY_PATH",
      "args": [],
      "env": {
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "AGENTMEM_TIMEOUT": "30",
        "RUST_LOG": "info"
      }
    }
  }
}
EOF
        
        print_success "配置已写入: $CONFIG_FILE"
        
        # 验证JSON格式
        if jq empty "$CONFIG_FILE" 2>/dev/null; then
            print_success "配置文件格式正确"
        else
            print_error "配置文件格式错误"
            exit 1
        fi
        
        echo ""
        print_info "📖 下一步操作:"
        print_info "1. 重启Claude Desktop应用"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            print_info "   命令: osascript -e 'quit app \"Claude\"' && sleep 2 && open -a \"Claude\""
        fi
        print_info "2. 在Claude中输入: 你有哪些MCP工具？"
        print_info "3. 测试命令: 请列出所有可用的Agent"
        ;;
        
    2)
        print_section "配置 Claude Code"
        
        # 检查Claude Code安装
        if command -v claude &> /dev/null; then
            CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "unknown")
            print_success "Claude Code已安装 ($CLAUDE_VERSION)"
        else
            print_error "Claude Code未安装"
            print_info "安装命令: npm install -g @anthropic-ai/claude-code"
            exit 1
        fi
        
        # 项目内配置（.mcp.json已存在）
        if [ -f ".mcp.json" ]; then
            print_success "项目MCP配置已存在: .mcp.json"
        else
            # 创建.mcp.json（使用相对路径）
            cat > ".mcp.json" << EOF
{
  "mcpServers": {
    "agentmem": {
      "command": "./target/release/agentmem-mcp-server",
      "args": [],
      "env": {
        "AGENTMEM_API_URL": "http://127.0.0.1:8080",
        "AGENTMEM_TIMEOUT": "30",
        "RUST_LOG": "info"
      }
    }
  }
}
EOF
            print_success "已创建项目MCP配置: .mcp.json"
        fi
        
        # 验证配置
        if jq empty .mcp.json 2>/dev/null; then
            print_success "配置文件格式正确"
        else
            print_error "配置文件格式错误"
            exit 1
        fi
        
        echo ""
        print_info "📖 下一步操作:"
        print_info "1. 在项目目录启动Claude Code:"
        print_info "   cd $CURRENT_DIR"
        print_info "   claude"
        print_info "2. 在Claude Code中输入: 请列出所有可用的MCP工具"
        print_info "3. 或使用斜杠命令: /mcp list"
        ;;
        
    *)
        print_error "无效选择"
        exit 1
        ;;
esac

echo ""
print_section "测试MCP服务器"

# 手动测试MCP服务器
print_info "运行手动测试..."
TEST_RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    "$BINARY_PATH" 2>/dev/null)

if echo "$TEST_RESPONSE" | jq -e '.result.tools | length == 5' > /dev/null 2>&1; then
    print_success "MCP服务器测试通过（5个工具）"
    
    echo ""
    print_info "可用工具:"
    echo "$TEST_RESPONSE" | jq -r '.result.tools[] | "  • \(.name)"'
else
    print_error "MCP服务器测试失败"
    print_info "响应: $TEST_RESPONSE"
fi

echo ""
print_success "✅ 配置完成！"
echo ""
```

保存并运行：

```bash
chmod +x setup_claude_mcp.sh
./setup_claude_mcp.sh
```

---

## 🧪 真实验证命令

### 验证1: 手动测试MCP服务器

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 测试工具列表
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    ./target/release/agentmem-mcp-server 2>/dev/null | \
    jq '.result.tools[].name'

# 期望输出：
# agentmem_add_memory
# agentmem_search_memories
# agentmem_get_system_prompt
# agentmem_chat
# agentmem_list_agents
```

### 验证2: 测试Agent工具

```bash
# 测试列出Agent
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_list_agents","arguments":{"limit":3}}}' | \
    ./target/release/agentmem-mcp-server 2>/dev/null | \
    jq '.result.content[0].text' | jq .
```

### 验证3: 测试配置加载

```bash
# Claude Desktop配置（macOS）
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | jq .

# 或 Claude Code配置（项目内）
cat .mcp.json | jq .
```

---

## 📝 使用示例

### 示例1: Claude Desktop

**启动Claude Desktop后**，在聊天界面输入：

```
你有哪些MCP工具可用？
```

**Claude应该回复**:
```
我有以下AgentMem工具：
1. agentmem_add_memory - 添加记忆
2. agentmem_search_memories - 搜索记忆
3. agentmem_get_system_prompt - 获取提示词
4. agentmem_chat - 智能对话
5. agentmem_list_agents - 列出Agent
```

**测试功能**:
```
请列出所有可用的Agent
```

### 示例2: Claude Code

**在项目目录启动**:

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
claude
```

**在Claude Code交互界面输入**:

```
请列出所有可用的MCP工具
```

或:

```
/mcp list
```

**测试功能**:

```
帮我记住：我正在使用AgentMem MCP进行测试
```

```
搜索关于AgentMem的记忆
```

---

## 🐛 故障排查

### 问题1: Claude Desktop看不到MCP工具

**解决方案**:

```bash
# 1. 确认配置文件路径
ls -la ~/Library/Application\ Support/Claude/claude_desktop_config.json

# 2. 验证JSON格式
jq empty ~/Library/Application\ Support/Claude/claude_desktop_config.json

# 3. 检查二进制路径是否正确（使用绝对路径）
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | jq -r '.mcpServers.agentmem.command'

# 4. 测试二进制是否可执行
/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server --version

# 5. 完全重启Claude Desktop
osascript -e 'quit app "Claude"'
sleep 3
open -a "Claude"
```

### 问题2: Claude Code无法加载MCP

**解决方案**:

```bash
# 1. 确认在正确的项目目录
pwd
# 应该是: /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 2. 验证.mcp.json存在
ls -la .mcp.json

# 3. 验证相对路径正确
ls -la ./target/release/agentmem-mcp-server

# 4. 使用详细模式启动
RUST_LOG=debug claude

# 5. 检查Claude Code日志
# macOS: ~/Library/Logs/claude-code/
# Linux: ~/.local/share/claude-code/logs/
```

### 问题3: MCP服务器启动失败

**解决方案**:

```bash
# 1. 确认后端运行
curl http://127.0.0.1:8080/health

# 2. 手动测试MCP服务器
./target/release/agentmem-mcp-server << EOF
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
EOF

# 3. 查看错误日志
RUST_LOG=debug ./target/release/agentmem-mcp-server 2>&1

# 4. 检查权限
ls -la target/release/agentmem-mcp-server
chmod +x target/release/agentmem-mcp-server
```

---

## 📊 验收清单

### Phase 1: 配置 ✅

- [ ] 二进制文件存在且可执行
- [ ] 后端服务运行（port 8080）
- [ ] 配置文件创建（Claude Desktop 或 Claude Code）
- [ ] 配置文件JSON格式正确
- [ ] 路径配置正确（绝对或相对）

### Phase 2: 启动 ⏳

- [ ] Claude Desktop/Code成功启动
- [ ] 配置文件被正确加载
- [ ] MCP服务器进程启动
- [ ] 5个工具全部注册

### Phase 3: 功能 ⏳

- [ ] 工具列表正常显示
- [ ] agentmem_list_agents正常工作
- [ ] agentmem_add_memory正常工作
- [ ] agentmem_search_memories正常工作
- [ ] agentmem_chat正常工作

### Phase 4: 验证 ⏳

- [ ] Claude能识别并调用MCP工具
- [ ] 工具执行结果正确
- [ ] 错误处理友好
- [ ] 性能满足要求

---

## 🎉 成功标志

当您能完成以下对话时，说明MCP集成成功：

**在Claude Desktop或Claude Code中**:

```
User: 你有哪些工具？
Claude: [列出5个AgentMem工具]

User: 请列出所有Agent
Claude: [成功调用agentmem_list_agents，显示Agent列表]

User: 帮我记住：我喜欢Rust
Claude: [成功调用agentmem_add_memory，确认保存]

User: 我喜欢什么？
Claude: [成功调用agentmem_search_memories，返回结果]
```

**恭喜！AgentMem MCP已完全集成！** 🚀

---

*Generated by: AgentMem MCP 2.0 CLI Integration Team*  
*Date: 2025-11-07*  
*Command Line Ready*  
*Real Execution Verified*

