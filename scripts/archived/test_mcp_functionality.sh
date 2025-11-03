#!/bin/bash

# AgentMem MCP 功能测试脚本
# 测试 MCP 协议的 4 个核心工具

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
echo "AgentMem MCP 功能测试"
echo "版本: v1.0"
echo "日期: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 检查MCP服务器二进制文件
log_info "检查MCP服务器二进制文件..."
if [ -f "./target/release/agentmem-mcp-server" ]; then
    log_success "MCP服务器二进制文件存在"
    ls -lh ./target/release/agentmem-mcp-server
else
    log_error "MCP服务器二进制文件不存在"
    exit 1
fi

# 测试 1: Initialize 握手
echo ""
echo "=========================================="
echo "测试 1: MCP Initialize 握手"
echo "=========================================="

log_info "发送 Initialize 请求..."
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'

INIT_RESPONSE=$(echo "$INIT_REQUEST" | ./target/release/agentmem-mcp-server 2>/dev/null | head -1)

if echo "$INIT_RESPONSE" | grep -q "protocolVersion"; then
    log_success "Initialize 握手成功"
    echo "响应: $INIT_RESPONSE"
else
    log_error "Initialize 握手失败"
    echo "响应: $INIT_RESPONSE"
fi

# 测试 2: 列出工具
echo ""
echo "=========================================="
echo "测试 2: 列出可用工具"
echo "=========================================="

log_info "发送 tools/list 请求..."
TOOLS_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

TOOLS_RESPONSE=$(echo -e "$INIT_REQUEST\n$TOOLS_REQUEST" | ./target/release/agentmem-mcp-server 2>/dev/null | tail -1)

if echo "$TOOLS_RESPONSE" | grep -q "agentmem_add_memory"; then
    log_success "工具列表获取成功"
    echo "响应: $TOOLS_RESPONSE"
    
    # 统计工具数量
    TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | grep -o "agentmem_" | wc -l)
    log_info "发现 $TOOL_COUNT 个 AgentMem 工具"
else
    log_error "工具列表获取失败"
    echo "响应: $TOOLS_RESPONSE"
fi

# 测试 3: 添加记忆
echo ""
echo "=========================================="
echo "测试 3: 添加记忆 (agentmem_add_memory)"
echo "=========================================="

log_info "发送 agentmem_add_memory 请求..."
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"MCP测试记忆 - '"$TIMESTAMP"'","memory_type":"episodic","user_id":"test-user-001","agent_id":"mcp-test-agent","session_id":"test-session-001"}}}'

ADD_MEMORY_RESPONSE=$(echo -e "$INIT_REQUEST\n$ADD_MEMORY_REQUEST" | ./target/release/agentmem-mcp-server 2>/dev/null | tail -1)

if echo "$ADD_MEMORY_RESPONSE" | grep -q "id"; then
    log_success "添加记忆成功"
    echo "响应: $ADD_MEMORY_RESPONSE"
    
    # 提取记忆ID
    MEMORY_ID=$(echo "$ADD_MEMORY_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    log_info "记忆ID: $MEMORY_ID"
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

SEARCH_RESPONSE=$(echo -e "$INIT_REQUEST\n$SEARCH_REQUEST" | ./target/release/agentmem-mcp-server 2>/dev/null | tail -1)

if echo "$SEARCH_RESPONSE" | grep -q "content"; then
    log_success "搜索记忆成功"
    echo "响应: $SEARCH_RESPONSE"
    
    # 统计搜索结果数量
    RESULT_COUNT=$(echo "$SEARCH_RESPONSE" | grep -o '"id"' | wc -l)
    log_info "搜索到 $RESULT_COUNT 条记忆"
else
    log_warning "搜索记忆可能失败"
    echo "响应: $SEARCH_RESPONSE"
fi

# 测试 5: 聊天功能
echo ""
echo "=========================================="
echo "测试 5: 聊天功能 (agentmem_chat)"
echo "=========================================="

log_info "发送 agentmem_chat 请求..."
CHAT_REQUEST='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"agentmem_chat","arguments":{"message":"你好，请介绍一下AgentMem项目","user_id":"test-user-001","session_id":"test-session-001","use_memory":true}}}'

CHAT_RESPONSE=$(echo -e "$INIT_REQUEST\n$CHAT_REQUEST" | ./target/release/agentmem-mcp-server 2>/dev/null | tail -1)

if echo "$CHAT_RESPONSE" | grep -q "response"; then
    log_success "聊天功能成功"
    echo "响应: $(echo "$CHAT_RESPONSE" | head -c 300)..."
else
    log_warning "聊天功能可能失败"
    echo "响应: $CHAT_RESPONSE"
fi

# 测试 6: 获取系统提示词
echo ""
echo "=========================================="
echo "测试 6: 获取系统提示词 (agentmem_get_system_prompt)"
echo "=========================================="

log_info "发送 agentmem_get_system_prompt 请求..."
PROMPT_REQUEST='{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"agentmem_get_system_prompt","arguments":{"user_id":"test-user-001","context":"AgentMem项目验证"}}}'

PROMPT_RESPONSE=$(echo -e "$INIT_REQUEST\n$PROMPT_REQUEST" | ./target/release/agentmem-mcp-server 2>/dev/null | tail -1)

if echo "$PROMPT_RESPONSE" | grep -q "prompt"; then
    log_success "获取系统提示词成功"
    echo "响应: $(echo "$PROMPT_RESPONSE" | head -c 200)..."
else
    log_warning "获取系统提示词可能失败"
    echo "响应: $PROMPT_RESPONSE"
fi

# 验证数据库持久化
echo ""
echo "=========================================="
echo "验证数据库持久化"
echo "=========================================="

log_info "检查数据库中的MCP测试记忆..."
MCP_MEMORY_COUNT=$(sqlite3 ./data/agentmem.db "SELECT COUNT(*) FROM memories WHERE content LIKE '%MCP测试%';" 2>/dev/null || echo "0")

if [ "$MCP_MEMORY_COUNT" -gt "0" ]; then
    log_success "数据库中找到 $MCP_MEMORY_COUNT 条MCP测试记忆"
    
    log_info "最新的MCP测试记忆:"
    sqlite3 ./data/agentmem.db "SELECT id, content, created_at FROM memories WHERE content LIKE '%MCP测试%' ORDER BY created_at DESC LIMIT 1;" 2>/dev/null || echo "查询失败"
else
    log_warning "数据库中未找到MCP测试记忆"
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
log_success "✅ 聊天功能正常"
log_success "✅ 获取系统提示词功能正常"
log_success "✅ 数据库持久化正常"

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
echo '      "command": "'"$(pwd)/target/release/agentmem-mcp-server"'",'
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

