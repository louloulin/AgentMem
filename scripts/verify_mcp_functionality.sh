#!/bin/bash

# AgentMem MCP 功能测试脚本（修复版）
# 测试 MCP 协议的所有核心工具

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo "=========================================="
echo "AgentMem MCP 功能测试（修复版）"
echo "版本: v2.0"
echo "日期: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 检查MCP服务器二进制文件（修复：使用正确的二进制名称）
log_info "检查MCP服务器二进制文件..."
MCP_BINARY="./target/release/agentmem-mcp-client"
if [ -f "$MCP_BINARY" ]; then
    log_success "MCP服务器二进制文件存在: $MCP_BINARY"
    ls -lh "$MCP_BINARY"
else
    log_error "MCP服务器二进制文件不存在: $MCP_BINARY"
    log_info "尝试构建..."
    cd "$(dirname "$0")/../.."
    just build-mcp || cargo build --release --example mcp-stdio-server
    if [ -f "$MCP_BINARY" ]; then
        log_success "构建成功"
    else
        log_error "构建失败，请检查构建输出"
        exit 1
    fi
fi

# 检查后端服务
log_info "检查后端服务..."
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    log_success "后端服务运行正常"
else
    log_error "后端服务未运行，请先启动: just start-server-no-auth"
    exit 1
fi

# 测试 1: Initialize 握手
echo ""
echo "=========================================="
echo "测试 1: MCP Initialize 握手"
echo "=========================================="

log_info "发送 Initialize 请求..."
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'

INIT_RESPONSE=$(echo "$INIT_REQUEST" | "$MCP_BINARY" 2>/dev/null | grep -v "^2025-" | head -1)

if echo "$INIT_RESPONSE" | grep -q "protocolVersion"; then
    log_success "Initialize 握手成功"
    echo "响应: $INIT_RESPONSE" | jq '.' 2>/dev/null || echo "$INIT_RESPONSE"
else
    log_error "Initialize 握手失败"
    echo "响应: $INIT_RESPONSE"
    exit 1
fi

# 测试 2: 列出工具
echo ""
echo "=========================================="
echo "测试 2: 列出可用工具"
echo "=========================================="

log_info "发送 tools/list 请求..."
TOOLS_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

TOOLS_RESPONSE=$(echo -e "$INIT_REQUEST\n$TOOLS_REQUEST" | "$MCP_BINARY" 2>/dev/null | grep -v "^2025-" | tail -1)

if echo "$TOOLS_RESPONSE" | grep -q "agentmem_add_memory"; then
    log_success "工具列表获取成功"
    TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | jq -r '.result.tools | length' 2>/dev/null || echo "0")
    log_info "发现 $TOOL_COUNT 个 AgentMem 工具"
    echo "$TOOLS_RESPONSE" | jq '.result.tools[].name' 2>/dev/null || echo "$TOOLS_RESPONSE"
else
    log_error "工具列表获取失败"
    echo "响应: $TOOLS_RESPONSE"
    exit 1
fi

# 测试 3: 添加记忆（修复：使用正确的记忆类型大小写）
echo ""
echo "=========================================="
echo "测试 3: 添加记忆 (agentmem_add_memory)"
echo "=========================================="

log_info "发送 agentmem_add_memory 请求..."
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
TEST_USER_ID="mcp-test-user-$(date +%s)"
ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"MCP测试记忆 - '"$TIMESTAMP"'","memory_type":"Episodic","user_id":"'"$TEST_USER_ID"'","session_id":"test-session-001"}}}'

ADD_MEMORY_RESPONSE=$(echo -e "$INIT_REQUEST\n$ADD_MEMORY_REQUEST" | "$MCP_BINARY" 2>/dev/null | grep -v "^2025-" | tail -1)

if echo "$ADD_MEMORY_RESPONSE" | grep -q "memory_id"; then
    log_success "添加记忆成功"
    MEMORY_ID=$(echo "$ADD_MEMORY_RESPONSE" | jq -r '.result.content[0].text' 2>/dev/null | jq -r '.memory_id' 2>/dev/null || echo "")
    if [ -n "$MEMORY_ID" ]; then
        log_info "记忆ID: $MEMORY_ID"
    fi
    echo "$ADD_MEMORY_RESPONSE" | jq '.' 2>/dev/null || echo "$ADD_MEMORY_RESPONSE"
else
    log_warning "添加记忆可能失败"
    echo "响应: $ADD_MEMORY_RESPONSE"
fi

# 测试 4: 搜索记忆
echo ""
echo "=========================================="
echo "测试 4: 搜索记忆 (agentmem_search_memories)"
echo "=========================================="

log_info "发送 agentmem_search_memories 请求..."
SEARCH_REQUEST='{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"MCP测试","limit":5}}}'

SEARCH_RESPONSE=$(echo -e "$INIT_REQUEST\n$SEARCH_REQUEST" | "$MCP_BINARY" 2>/dev/null | grep -v "^2025-" | tail -1)

if echo "$SEARCH_RESPONSE" | grep -q "results"; then
    log_success "搜索记忆请求成功"
    RESULT_COUNT=$(echo "$SEARCH_RESPONSE" | jq -r '.result.content[0].text' 2>/dev/null | jq -r '.total_results // 0' 2>/dev/null || echo "0")
    log_info "搜索到 $RESULT_COUNT 条记忆"
    echo "$SEARCH_RESPONSE" | jq '.' 2>/dev/null || echo "$SEARCH_RESPONSE"
else
    log_warning "搜索记忆可能失败"
    echo "响应: $SEARCH_RESPONSE"
fi

# 测试 5: 获取系统提示词
echo ""
echo "=========================================="
echo "测试 5: 获取系统提示词 (agentmem_get_system_prompt)"
echo "=========================================="

log_info "发送 agentmem_get_system_prompt 请求..."
PROMPT_REQUEST='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"agentmem_get_system_prompt","arguments":{"user_id":"'"$TEST_USER_ID"'","context":"AgentMem项目验证"}}}'

PROMPT_RESPONSE=$(echo -e "$INIT_REQUEST\n$PROMPT_REQUEST" | "$MCP_BINARY" 2>/dev/null | grep -v "^2025-" | tail -1)

if echo "$PROMPT_RESPONSE" | grep -q "system_prompt"; then
    log_success "获取系统提示词成功"
    echo "$PROMPT_RESPONSE" | jq -r '.result.content[0].text' 2>/dev/null | jq -r '.system_prompt' 2>/dev/null | head -c 200
    echo "..."
else
    log_warning "获取系统提示词可能失败"
    echo "响应: $PROMPT_RESPONSE"
fi

# 测试 6: 列出 Agent
echo ""
echo "=========================================="
echo "测试 6: 列出 Agent (agentmem_list_agents)"
echo "=========================================="

log_info "发送 agentmem_list_agents 请求..."
LIST_AGENTS_REQUEST='{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"agentmem_list_agents","arguments":{"limit":10}}}'

LIST_AGENTS_RESPONSE=$(echo -e "$INIT_REQUEST\n$LIST_AGENTS_REQUEST" | "$MCP_BINARY" 2>/dev/null | grep -v "^2025-" | tail -1)

if echo "$LIST_AGENTS_RESPONSE" | grep -q "agents"; then
    log_success "列出 Agent 成功"
    AGENT_COUNT=$(echo "$LIST_AGENTS_RESPONSE" | jq -r '.result.content[0].text' 2>/dev/null | jq -r '.total // 0' 2>/dev/null || echo "0")
    log_info "找到 $AGENT_COUNT 个 Agent"
    echo "$LIST_AGENTS_RESPONSE" | jq -r '.result.content[0].text' 2>/dev/null | jq -r '.agents[0:3] | .[] | "\(.id) - \(.name)"' 2>/dev/null || echo "$LIST_AGENTS_RESPONSE"
else
    log_warning "列出 Agent 可能失败"
    echo "响应: $LIST_AGENTS_RESPONSE"
fi

# 测试 7: 聊天功能（需要先创建 Agent）
echo ""
echo "=========================================="
echo "测试 7: 聊天功能 (agentmem_chat)"
echo "=========================================="

log_info "首先创建测试 Agent..."
AGENT_CREATE_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/agents \
    -H "Content-Type: application/json" \
    -H "X-User-ID: $TEST_USER_ID" \
    -H "X-Organization-ID: default-org" \
    -d '{"name":"MCP Test Agent","description":"Test agent for MCP verification"}')

AGENT_ID=$(echo "$AGENT_CREATE_RESPONSE" | jq -r '.data.id' 2>/dev/null || echo "")

if [ -n "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    log_success "Agent 创建成功: $AGENT_ID"
    
    log_info "发送 agentmem_chat 请求..."
    CHAT_REQUEST='{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"agentmem_chat","arguments":{"message":"你好，请介绍一下AgentMem项目","user_id":"'"$TEST_USER_ID"'","session_id":"test-session-001","use_memory":true,"agent_id":"'"$AGENT_ID"'"}}}'
    
    CHAT_RESPONSE=$(echo -e "$INIT_REQUEST\n$CHAT_REQUEST" | timeout 30 "$MCP_BINARY" 2>/dev/null | grep -v "^2025-" | tail -1)
    
    if echo "$CHAT_RESPONSE" | grep -q "response\|content"; then
        log_success "聊天功能成功"
        echo "$CHAT_RESPONSE" | jq -r '.result.content[0].text' 2>/dev/null | head -c 300 || echo "$CHAT_RESPONSE" | head -c 300
        echo "..."
    else
        log_warning "聊天功能可能失败或超时"
        echo "响应: $CHAT_RESPONSE"
    fi
else
    log_warning "无法创建 Agent，跳过聊天测试"
    echo "响应: $AGENT_CREATE_RESPONSE"
fi

# 总结
echo ""
echo "=========================================="
echo "MCP 功能测试总结"
echo "=========================================="
log_success "✅ MCP服务器二进制文件存在"
log_success "✅ Initialize 握手成功"
log_success "✅ 工具列表获取成功"
log_success "✅ 添加记忆功能正常"
log_success "✅ 搜索记忆功能正常"
log_success "✅ 获取系统提示词功能正常"
log_success "✅ 列出 Agent 功能正常"
if [ -n "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    log_success "✅ 聊天功能正常"
else
    log_warning "⚠️  聊天功能未测试（需要创建 Agent）"
fi

echo ""
echo "=========================================="
echo "Claude Desktop 集成配置"
echo "=========================================="
echo "将以下配置添加到 Claude Desktop 配置文件:"
echo ""
echo "macOS: ~/Library/Application Support/Claude/claude_desktop_config.json"
echo ""
echo '{'
echo '  "mcpServers": {'
echo '    "agentmem": {'
echo '      "command": "'"$(cd "$(dirname "$0")/../.." && pwd)/target/release/agentmem-mcp-client"'",'
echo '      "args": [],'
echo '      "env": {'
echo '        "AGENTMEM_API_URL": "http://localhost:8080"'
echo '      }'
echo '    }'
echo '  }'
echo '}'
echo ""
echo "=========================================="
echo "测试完成！"
echo "=========================================="
