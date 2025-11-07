#!/bin/bash
#
# Claude MCP 自动配置脚本
# 用途: 自动配置Claude Desktop或Claude Code的MCP集成

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
    SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
    print_success "MCP服务器二进制文件存在 ($SIZE)"
    
    # 检查执行权限
    if [ -x "$BINARY_PATH" ]; then
        print_success "二进制文件有执行权限"
    else
        print_error "二进制文件无执行权限"
        print_info "添加执行权限..."
        chmod +x "$BINARY_PATH"
        print_success "执行权限已添加"
    fi
else
    print_error "MCP服务器二进制文件不存在: $BINARY_PATH"
    print_info "运行: cargo build --package mcp-stdio-server --release"
    exit 1
fi

# 2. 检查后端服务
echo ""
print_info "检查AgentMem后端服务..."
if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
    print_success "AgentMem后端服务运行中"
else
    print_error "AgentMem后端服务未运行"
    print_info "请先启动后端: ./start_server.sh &"
    
    read -p "是否现在启动后端? [y/N]: " start_backend
    if [[ "$start_backend" =~ ^[Yy]$ ]]; then
        if [ -f "./start_server.sh" ]; then
            ./start_server.sh &
            print_info "等待后端启动..."
            sleep 5
            
            if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
                print_success "后端启动成功"
            else
                print_error "后端启动失败"
                exit 1
            fi
        else
            print_error "找不到启动脚本: ./start_server.sh"
            exit 1
        fi
    else
        exit 1
    fi
fi

echo ""
print_section "选择配置方式"
echo ""
echo "1. Claude Desktop (桌面应用) - 推荐"
echo "2. Claude Code (命令行工具)"
echo "3. 两者都配置"
echo ""
read -p "请选择 [1/2/3]: " choice

configure_desktop() {
    print_section "配置 Claude Desktop"
    
    # 确定配置路径
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        CONFIG_DIR="$HOME/Library/Application Support/Claude"
        CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
        print_info "平台: macOS"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        CONFIG_DIR="$HOME/.config/Claude"
        CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
        print_info "平台: Linux"
    else
        print_error "不支持的操作系统: $OSTYPE"
        return 1
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
    if command -v jq &> /dev/null; then
        if jq empty "$CONFIG_FILE" 2>/dev/null; then
            print_success "配置文件格式正确"
            
            echo ""
            print_info "配置内容:"
            jq . "$CONFIG_FILE"
        else
            print_error "配置文件格式错误"
            return 1
        fi
    else
        print_info "未安装jq，跳过JSON验证"
    fi
    
    echo ""
    print_info "📖 Claude Desktop 下一步:"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        print_info "1. 重启Claude Desktop:"
        print_info "   osascript -e 'quit app \"Claude\"' && sleep 2 && open -a \"Claude\""
    else
        print_info "1. 手动重启Claude Desktop应用"
    fi
    print_info "2. 在Claude中输入: 你有哪些MCP工具？"
    print_info "3. 测试命令: 请列出所有可用的Agent"
}

configure_code() {
    print_section "配置 Claude Code"
    
    # 检查Claude Code安装
    if command -v claude &> /dev/null; then
        CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "installed")
        print_success "Claude Code已安装 ($CLAUDE_VERSION)"
    else
        print_error "Claude Code未安装"
        print_info "安装命令: npm install -g @anthropic-ai/claude-code"
        
        read -p "是否现在安装Claude Code? [y/N]: " install_claude
        if [[ "$install_claude" =~ ^[Yy]$ ]]; then
            if command -v npm &> /dev/null; then
                npm install -g @anthropic-ai/claude-code
                print_success "Claude Code安装完成"
            else
                print_error "未安装npm，无法自动安装Claude Code"
                return 1
            fi
        else
            return 1
        fi
    fi
    
    # 项目内配置
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
    if command -v jq &> /dev/null; then
        if jq empty .mcp.json 2>/dev/null; then
            print_success "配置文件格式正确"
            
            echo ""
            print_info "配置内容:"
            jq . .mcp.json
        else
            print_error "配置文件格式错误"
            return 1
        fi
    fi
    
    echo ""
    print_info "📖 Claude Code 下一步:"
    print_info "1. 在项目目录启动Claude Code:"
    print_info "   cd $CURRENT_DIR"
    print_info "   claude"
    print_info "2. 在Claude Code中输入: 请列出所有可用的MCP工具"
    print_info "3. 或使用斜杠命令: /mcp list"
}

case $choice in
    1)
        configure_desktop
        ;;
    2)
        configure_code
        ;;
    3)
        configure_desktop
        echo ""
        configure_code
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

if command -v jq &> /dev/null; then
    if echo "$TEST_RESPONSE" | jq -e '.result.tools | length == 5' > /dev/null 2>&1; then
        print_success "MCP服务器测试通过（5个工具）"
        
        echo ""
        print_info "可用工具:"
        echo "$TEST_RESPONSE" | jq -r '.result.tools[] | "  • \(.name): \(.description)"'
    else
        print_error "MCP服务器测试失败"
        print_info "响应: $TEST_RESPONSE"
    fi
else
    print_info "未安装jq，跳过详细验证"
    if echo "$TEST_RESPONSE" | grep -q "agentmem"; then
        print_success "MCP服务器响应正常"
    else
        print_error "MCP服务器响应异常"
    fi
fi

echo ""
echo "=================================="
print_success "✅ 配置完成！"
echo "=================================="
echo ""

print_info "🎯 快速测试命令:"
echo ""
echo "# 手动测试MCP服务器"
echo "echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}' | $BINARY_PATH | jq ."
echo ""
echo "# 测试Agent工具"
echo "echo '{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"agentmem_list_agents\",\"arguments\":{\"limit\":3}}}' | $BINARY_PATH | jq ."
echo ""

print_info "📖 完整文档: CLAUDE_CLI_MCP_SETUP.md"
echo ""

