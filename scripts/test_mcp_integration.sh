#!/bin/bash

# AgentMem MCP 集成验证脚本

set -e

MCP_BINARY="/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗][${NC} $1"
}

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║  AgentMem MCP 集成验证                                      ║"
echo "║  Model Context Protocol Integration Test                    ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 测试1: MCP 二进制文件存在性
echo "[TEST 1] 检查 MCP 服务器二进制文件"
if [ -f "$MCP_BINARY" ]; then
    log_success "MCP 服务器二进制文件存在"
    ls -lh "$MCP_BINARY"
else
    log_error "MCP 服务器二进制文件不存在"
    echo "预期路径: $MCP_BINARY"
    exit 1
fi

echo ""

# 测试2: MCP 二进制文件可执行性
echo "[TEST 2] 检查 MCP 服务器可执行性"
if [ -x "$MCP_BINARY" ]; then
    log_success "MCP 服务器可执行"
else
    log_error "MCP 服务器不可执行"
    exit 1
fi

echo ""

# 测试3: MCP 服务器版本信息
echo "[TEST 3] 获取 MCP 服务器版本"
if version_output=$("$MCP_BINARY" --version 2>&1); then
    log_success "版本信息:"
    echo "  $version_output"
else
    log_error "无法获取版本信息"
fi

echo ""

# 测试4: 检查 MCP 配置
echo "[TEST 4] 检查 MCP 配置文件"
MCP_CONFIG="$HOME/.config/claude/config.json"
if [ -f "$MCP_CONFIG" ]; then
    log_success "MCP 配置文件存在: $MCP_CONFIG"
    if grep -q "agentmem" "$MCP_CONFIG"; then
        log_success "配置中包含 AgentMem MCP 配置"
    else
        echo -e "${YELLOW}[!]${NC} 配置中未找到 AgentMem"
        echo "请运行以下命令配置:"
        echo "  claude mcp add agentmem $MCP_BINARY"
    fi
else
    echo -e "${YELLOW}[!]${NC} MCP 配置文件不存在"
    echo "这是正常的，如果您还未配置 Claude Code"
fi

echo ""

# 测试5: 测试 MCP 工具列表
echo "[TEST 5] 测试 MCP 工具发现"
echo "MCP 提供的工具应包括:"
echo "  - agentmem_add_memory"
echo "  - agentmem_search_memories"  
echo "  - agentmem_list_agents"
echo "  - agentmem_get_agent"
echo "  - agentmem_create_agent"

echo ""
log_info "手动验证（如果已配置 Claude Code）:"
echo "  claude mcp list"
echo "  claude mcp tools agentmem"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                     MCP 验证完成                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "MCP 集成状态:"
echo "  ✅ MCP 服务器已编译"
echo "  ✅ MCP 服务器可执行"
echo "  ⏳ 待配置到 Claude Code (手动步骤)"
echo ""
echo "配置步骤:"
echo "  1. 确保安装了 Claude Code CLI"
echo "  2. 运行: claude mcp add agentmem $MCP_BINARY"
echo "  3. 运行: claude mcp list 验证"
echo ""

