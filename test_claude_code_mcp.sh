#!/bin/bash
#
# Claude Code MCP 完整测试脚本

set -e

echo "🧪 Claude Code MCP 完整测试"
echo "=================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }
print_info() { echo -e "${YELLOW}ℹ${NC} $1"; }
print_section() { echo -e "${BLUE}▶${NC} $1"; }

# 1. 检查当前目录
print_section "检查当前目录"
CURRENT_DIR=$(pwd)
EXPECTED_DIR="/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen"

if [ "$CURRENT_DIR" = "$EXPECTED_DIR" ]; then
    print_success "当前目录正确"
    print_info "  $CURRENT_DIR"
else
    print_error "当前目录错误"
    print_info "  当前: $CURRENT_DIR"
    print_info "  期望: $EXPECTED_DIR"
    
    read -p "是否切换到正确目录? [Y/n]: " switch_dir
    if [[ ! "$switch_dir" =~ ^[Nn]$ ]]; then
        cd "$EXPECTED_DIR"
        print_success "已切换到: $(pwd)"
    else
        exit 1
    fi
fi

# 2. 检查.mcp.json
echo ""
print_section "检查MCP配置"
if [ -f ".mcp.json" ]; then
    print_success ".mcp.json存在"
    
    # 验证JSON格式
    if jq empty .mcp.json 2>/dev/null; then
        print_success "JSON格式正确"
    else
        print_error "JSON格式错误"
        cat .mcp.json
        exit 1
    fi
    
    # 显示配置
    echo ""
    print_info "配置内容:"
    jq . .mcp.json | sed 's/^/  /'
    
    # 验证命令路径
    CONFIGURED_CMD=$(jq -r '.mcpServers.agentmem.command' .mcp.json)
    print_info "  配置的命令: $CONFIGURED_CMD"
else
    print_error ".mcp.json不存在"
    print_info "创建.mcp.json..."
    
    cat > .mcp.json << 'EOF'
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
    print_success ".mcp.json已创建"
fi

# 3. 检查二进制文件
echo ""
print_section "检查MCP服务器"
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
    print_info "需要编译..."
    
    read -p "是否现在编译? [Y/n]: " compile_now
    if [[ ! "$compile_now" =~ ^[Nn]$ ]]; then
        cargo build --package mcp-stdio-server --release
        print_success "编译完成"
    else
        exit 1
    fi
fi

# 4. 测试MCP服务器
echo ""
print_section "测试MCP服务器"
print_info "发送tools/list请求..."

TEST_RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | \
    $BINARY 2>/dev/null)

if [ -z "$TEST_RESPONSE" ]; then
    print_error "MCP服务器无响应"
    exit 1
fi

TOOLS_COUNT=$(echo "$TEST_RESPONSE" | jq -r '.result.tools | length' 2>/dev/null || echo "0")

if [ "$TOOLS_COUNT" = "5" ]; then
    print_success "5个工具正常注册"
    
    echo ""
    print_info "工具列表:"
    echo "$TEST_RESPONSE" | jq -r '.result.tools[] | "  \(.name): \(.description)"' | head -5
else
    print_error "工具数量异常: $TOOLS_COUNT"
    print_info "响应内容:"
    echo "$TEST_RESPONSE" | jq . | sed 's/^/  /'
    exit 1
fi

# 5. 检查后端
echo ""
print_section "检查后端服务"
if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
    print_success "后端服务运行中"
    
    # 测试健康状态
    HEALTH_STATUS=$(curl -s http://127.0.0.1:8080/health | jq -r '.status' 2>/dev/null || echo "unknown")
    print_info "  状态: $HEALTH_STATUS"
else
    print_error "后端服务未运行"
    
    read -p "是否现在启动后端? [Y/n]: " start_backend
    if [[ ! "$start_backend" =~ ^[Nn]$ ]]; then
        if [ -f "./start_server.sh" ]; then
            ./start_server.sh &
            print_info "等待后端启动..."
            sleep 5
            
            if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
                print_success "后端启动成功"
            else
                print_error "后端启动失败"
            fi
        else
            print_error "找不到启动脚本: ./start_server.sh"
        fi
    fi
fi

# 6. 检查Claude Code
echo ""
print_section "检查Claude Code"
if command -v claude &> /dev/null; then
    CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "installed")
    print_success "Claude Code已安装"
    print_info "  版本: $CLAUDE_VERSION"
else
    print_error "Claude Code未安装"
    print_info "安装命令: npm install -g @anthropic-ai/claude-code"
    
    read -p "是否现在安装? [y/N]: " install_claude
    if [[ "$install_claude" =~ ^[Yy]$ ]]; then
        if command -v npm &> /dev/null; then
            npm install -g @anthropic-ai/claude-code
            print_success "Claude Code安装完成"
        else
            print_error "npm未安装"
            exit 1
        fi
    else
        exit 1
    fi
fi

# 7. 测试完整流程
echo ""
print_section "测试Agent工具"
AGENT_RESPONSE=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_list_agents","arguments":{"limit":3}}}' | \
    $BINARY 2>/dev/null)

AGENT_SUCCESS=$(echo "$AGENT_RESPONSE" | jq -r '.result.content[0].text' | jq -r '.success' 2>/dev/null || echo "false")

if [ "$AGENT_SUCCESS" = "true" ]; then
    AGENT_COUNT=$(echo "$AGENT_RESPONSE" | jq -r '.result.content[0].text' | jq -r '.total')
    print_success "Agent工具正常 (共$AGENT_COUNT个Agent)"
else
    print_error "Agent工具异常"
    print_info "响应:"
    echo "$AGENT_RESPONSE" | jq . | sed 's/^/  /'
fi

echo ""
echo "=================================="
print_success "✅ 所有检查完成！"
echo "=================================="
echo ""

print_info "🚀 启动Claude Code:"
echo ""
echo "  cd $(pwd)"
echo "  claude"
echo ""

print_info "📖 在Claude Code中测试:"
echo ""
echo "  1. 输入: /mcp list"
echo "     期望: 看到 agentmem 服务器"
echo ""
echo "  2. 输入: 你有哪些工具？"
echo "     期望: 看到5个AgentMem工具"
echo ""
echo "  3. 输入: 请列出所有Agent"
echo "     期望: 成功列出Agent列表"
echo ""
echo "  4. 输入: 帮我记住：测试Claude Code MCP"
echo "     期望: 成功添加记忆"
echo ""

print_info "🐛 如果看不到MCP工具:"
echo ""
echo "  1. 确认在正确目录: $(pwd)"
echo "  2. 确认.mcp.json存在"
echo "  3. 重启Claude Code: 退出后重新运行 claude"
echo "  4. 使用调试模式: RUST_LOG=debug claude"
echo ""

