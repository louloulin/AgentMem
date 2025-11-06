#!/bin/bash

# AgentMem MCP 集成测试脚本
# 
# 此脚本用于验证 AgentMem 的 MCP (Model Context Protocol) 功能
# 通过 JSON-RPC 2.0 协议测试各项核心功能

set -e

# 颜色输出
RED='\033[0.31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}AgentMem MCP 集成测试${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# 检查编译产物
MCP_SERVER_PATH="target/release/agentmem-mcp-server"

if [ ! -f "$MCP_SERVER_PATH" ]; then
    echo -e "${RED}错误: MCP 服务器未找到${NC}"
    echo -e "${YELLOW}正在编译 MCP 服务器...${NC}"
    cargo build --package mcp-stdio-server --release
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}编译失败${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✓ MCP 服务器已就绪: $MCP_SERVER_PATH${NC}"
echo ""

# 测试1: Initialize
echo -e "${BLUE}[测试 1/5] Initialize - 初始化 MCP 连接${NC}"
echo ""

INITIALIZE_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'

echo "$INITIALIZE_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/init_response.json

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Initialize 成功${NC}"
    echo "响应:"
    cat /tmp/init_response.json | jq .
    echo ""
else
    echo -e "${RED}✗ Initialize 失败${NC}"
    exit 1
fi

# 测试2: List Tools
echo -e "${BLUE}[测试 2/5] Tools/List - 列出可用工具${NC}"
echo ""

TOOLS_LIST_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

echo "$TOOLS_LIST_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/tools_response.json

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Tools/List 成功${NC}"
    echo "可用工具:"
    cat /tmp/tools_response.json | jq '.result.tools[] | {name, description}'
    echo ""
    
    TOOL_COUNT=$(cat /tmp/tools_response.json | jq '.result.tools | length')
    echo -e "${GREEN}共 $TOOL_COUNT 个可用工具${NC}"
    echo ""
else
    echo -e "${RED}✗ Tools/List 失败${NC}"
    exit 1
fi

# 测试3: Add Memory
echo -e "${BLUE}[测试 3/5] Tools/Call - 添加记忆${NC}"
echo ""

ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"AgentMem is a high-performance memory management platform written in Rust","user_id":"test_user_001","memory_type":"semantic","tags":["rust","memory","platform"]}}}'

echo "$ADD_MEMORY_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/add_memory_response.json

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Add Memory 成功${NC}"
    echo "响应:"
    cat /tmp/add_memory_response.json | jq .
    echo ""
    
    # 提取memory_id
    MEMORY_ID=$(cat /tmp/add_memory_response.json | jq -r '.result.content[0].text' | jq -r '.memory_id // empty')
    if [ -n "$MEMORY_ID" ]; then
        echo -e "${GREEN}记忆ID: $MEMORY_ID${NC}"
        echo "$MEMORY_ID" > /tmp/memory_id.txt
    fi
    echo ""
else
    echo -e "${RED}✗ Add Memory 失败${NC}"
    cat /tmp/add_memory_response.json
    exit 1
fi

# 测试4: Search Memories
echo -e "${BLUE}[测试 4/5] Tools/Call - 搜索记忆${NC}"
echo ""

SEARCH_MEMORIES_REQUEST='{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"Rust memory platform","user_id":"test_user_001","limit":5}}}'

echo "$SEARCH_MEMORIES_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/search_response.json

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Search Memories 成功${NC}"
    echo "搜索结果:"
    cat /tmp/search_response.json | jq .
    echo ""
    
    RESULTS_COUNT=$(cat /tmp/search_response.json | jq -r '.result.content[0].text' | jq -r '.total_results // 0')
    echo -e "${GREEN}找到 $RESULTS_COUNT 条记忆${NC}"
    echo ""
else
    echo -e "${RED}✗ Search Memories 失败${NC}"
    cat /tmp/search_response.json
    exit 1
fi

# 测试5: Chat
echo -e "${BLUE}[测试 5/5] Tools/Call - 智能对话${NC}"
echo ""

CHAT_REQUEST='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"agentmem_chat","arguments":{"message":"What do you know about AgentMem?","user_id":"test_user_001","agent_id":"agent_001"}}}'

echo "$CHAT_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/chat_response.json

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Chat 成功${NC}"
    echo "对话响应:"
    cat /tmp/chat_response.json | jq .
    echo ""
else
    echo -e "${RED}✗ Chat 失败${NC}"
    cat /tmp/chat_response.json
    exit 1
fi

# 测试总结
echo -e "${BLUE}================================${NC}"
echo -e "${GREEN}✓ 所有测试通过！${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

echo -e "${GREEN}MCP 功能验证完成：${NC}"
echo ""
echo "1. ✓ Initialize - MCP 协议初始化"
echo "2. ✓ Tools/List - 工具列表获取"
echo "3. ✓ Add Memory - 记忆添加功能"
echo "4. ✓ Search Memories - 记忆搜索功能"
echo "5. ✓ Chat - 智能对话功能"
echo ""

echo -e "${BLUE}Claude Desktop 集成配置：${NC}"
echo ""
echo "将以下内容添加到 Claude Desktop 配置文件中："
echo ""
echo -e "${YELLOW}macOS: ~/Library/Application Support/Claude/claude_desktop_config.json${NC}"
echo -e "${YELLOW}Windows: %APPDATA%\\Claude\\claude_desktop_config.json${NC}"
echo -e "${YELLOW}Linux: ~/.config/Claude/claude_desktop_config.json${NC}"
echo ""
echo '{'
echo '  "mcpServers": {'
echo '    "agentmem": {'
echo "      \"command\": \"$(pwd)/$MCP_SERVER_PATH\","
echo '      "args": [],'
echo '      "env": {}'
echo '    }'
echo '  }'
echo '}'
echo ""

echo -e "${BLUE}使用示例：${NC}"
echo ""
echo "在 Claude Desktop 中："
echo ""
echo -e "${GREEN}1. 添加记忆：${NC}"
echo '   "请使用 agentmem_add_memory 添加：我喜欢使用Rust编程"'
echo ""
echo -e "${GREEN}2. 搜索记忆：${NC}"
echo '   "请使用 agentmem_search_memories 搜索关于Rust的记忆"'
echo ""
echo -e "${GREEN}3. 智能对话：${NC}"
echo '   "请使用 agentmem_chat 与我对话：你了解我的编程偏好吗？"'
echo ""

echo -e "${GREEN}测试完成！ 🎉${NC}"

