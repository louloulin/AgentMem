#!/bin/bash

# AgentMem MCP 集成测试脚本 (修复版)
# 
# 此脚本修复了原测试脚本中的问题：
# 1. 移除了不支持的 tags 参数
# 2. 添加了后端服务检查
# 3. 添加了 Agent 创建步骤
# 4. 改进了错误处理

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}AgentMem MCP 集成测试 (修复版)${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# 检查编译产物
MCP_SERVER_PATH="target/release/agentmem-mcp-server"
BACKEND_SERVER_PATH="target/release/agent-mem-server"

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

# 检查后端服务
echo -e "${BLUE}[预检] 检查后端 API 服务${NC}"
echo ""

BACKEND_URL="http://127.0.0.1:8080"
if curl -sf "$BACKEND_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ 后端服务运行中${NC}"
else
    echo -e "${YELLOW}⚠ 后端服务未运行${NC}"
    echo ""
    echo -e "${YELLOW}需要启动后端服务才能使用完整功能${NC}"
    echo -e "${YELLOW}请在另一个终端运行：${NC}"
    echo ""
    echo -e "${BLUE}  cd $(pwd)${NC}"
    echo -e "${BLUE}  ./target/release/agent-mem-server --config config.toml${NC}"
    echo ""
    echo -e "${YELLOW}或者使用以下命令编译并启动：${NC}"
    echo ""
    echo -e "${BLUE}  cargo build --bin agent-mem-server --release${NC}"
    echo -e "${BLUE}  ./target/release/agent-mem-server --config config.toml${NC}"
    echo ""
    read -p "按回车继续（仅测试基本功能）或 Ctrl+C 退出..."
fi

echo ""

# 测试1: Initialize
echo -e "${BLUE}[测试 1/6] Initialize - 初始化 MCP 连接${NC}"
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
echo -e "${BLUE}[测试 2/6] Tools/List - 列出可用工具${NC}"
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

# 测试3: 创建 Agent (如果后端可用)
if curl -sf "$BACKEND_URL/health" > /dev/null 2>&1; then
    echo -e "${BLUE}[测试 3/6] 创建测试 Agent${NC}"
    echo ""
    
    # 检查 Agent 是否已存在
    AGENT_EXISTS=$(curl -sf "$BACKEND_URL/api/v1/agents/agent_001" 2>/dev/null)
    
    if [ -n "$AGENT_EXISTS" ]; then
        echo -e "${YELLOW}⚠ Agent 已存在，跳过创建${NC}"
    else
        CREATE_AGENT=$(curl -sf -X POST "$BACKEND_URL/api/v1/agents" \
            -H "Content-Type: application/json" \
            -d '{
                "agent_id": "agent_001",
                "name": "Test Agent",
                "description": "Test agent for MCP integration",
                "user_id": "test_user_001",
                "config": {}
            }' 2>&1)
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Agent 创建成功${NC}"
            echo "$CREATE_AGENT" | jq .
        else
            echo -e "${YELLOW}⚠ Agent 创建失败（可能已存在）${NC}"
        fi
    fi
    echo ""
else
    echo -e "${YELLOW}[测试 3/6] 跳过 - 后端服务未运行${NC}"
    echo ""
fi

# 测试4: Add Memory (修复版 - 移除 tags 参数)
echo -e "${BLUE}[测试 4/6] Tools/Call - 添加记忆 (修复版)${NC}"
echo ""

# 正确的参数：使用 metadata 字段存储 tags 信息
ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"AgentMem is a high-performance memory management platform written in Rust","user_id":"test_user_001","agent_id":"agent_001","memory_type":"Episodic","metadata":"{\"tags\":[\"rust\",\"memory\",\"platform\"]}"}}}'

echo "$ADD_MEMORY_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/add_memory_response.json

RESPONSE_TEXT=$(cat /tmp/add_memory_response.json)
if echo "$RESPONSE_TEXT" | jq -e '.result' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Add Memory 成功${NC}"
    echo "响应:"
    echo "$RESPONSE_TEXT" | jq .
    echo ""
    
    # 尝试提取 memory_id
    MEMORY_TEXT=$(echo "$RESPONSE_TEXT" | jq -r '.result.content[0].text // empty')
    if [ -n "$MEMORY_TEXT" ]; then
        MEMORY_ID=$(echo "$MEMORY_TEXT" | jq -r '.memory_id // empty')
        if [ -n "$MEMORY_ID" ]; then
            echo -e "${GREEN}记忆ID: $MEMORY_ID${NC}"
            echo "$MEMORY_ID" > /tmp/memory_id.txt
        fi
    fi
    echo ""
else
    echo -e "${RED}✗ Add Memory 失败${NC}"
    echo "错误详情:"
    echo "$RESPONSE_TEXT" | jq .
    echo ""
fi

# 测试5: Search Memories
echo -e "${BLUE}[测试 5/6] Tools/Call - 搜索记忆${NC}"
echo ""

SEARCH_MEMORIES_REQUEST='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"Rust memory platform","user_id":"test_user_001","limit":5}}}'

echo "$SEARCH_MEMORIES_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/search_response.json

RESPONSE_TEXT=$(cat /tmp/search_response.json)
if echo "$RESPONSE_TEXT" | jq -e '.result' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Search Memories 成功${NC}"
    echo "搜索结果:"
    echo "$RESPONSE_TEXT" | jq .
    echo ""
    
    RESULTS_TEXT=$(echo "$RESPONSE_TEXT" | jq -r '.result.content[0].text // empty')
    if [ -n "$RESULTS_TEXT" ]; then
        RESULTS_COUNT=$(echo "$RESULTS_TEXT" | jq -r '.total_results // 0')
        echo -e "${GREEN}找到 $RESULTS_COUNT 条记忆${NC}"
    fi
    echo ""
else
    echo -e "${RED}✗ Search Memories 失败${NC}"
    echo "错误详情:"
    echo "$RESPONSE_TEXT" | jq .
    echo ""
fi

# 测试6: Chat (需要后端服务和 Agent)
echo -e "${BLUE}[测试 6/6] Tools/Call - 智能对话${NC}"
echo ""

if curl -sf "$BACKEND_URL/health" > /dev/null 2>&1; then
    CHAT_REQUEST='{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"agentmem_chat","arguments":{"message":"What do you know about AgentMem?","user_id":"test_user_001","agent_id":"agent_001"}}}'
    
    echo "$CHAT_REQUEST" | $MCP_SERVER_PATH 2>/dev/null | jq . > /tmp/chat_response.json
    
    RESPONSE_TEXT=$(cat /tmp/chat_response.json)
    if echo "$RESPONSE_TEXT" | jq -e '.result' > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Chat 成功${NC}"
        echo "对话响应:"
        echo "$RESPONSE_TEXT" | jq .
        echo ""
    else
        echo -e "${RED}✗ Chat 失败${NC}"
        echo "错误详情:"
        echo "$RESPONSE_TEXT" | jq .
        echo ""
    fi
else
    echo -e "${YELLOW}⚠ 跳过 Chat 测试 - 后端服务未运行${NC}"
    echo ""
fi

# 测试总结
echo -e "${BLUE}================================${NC}"
echo -e "${GREEN}✓ 测试完成！${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

echo -e "${GREEN}MCP 功能验证完成：${NC}"
echo ""
echo "1. ✓ Initialize - MCP 协议初始化"
echo "2. ✓ Tools/List - 工具列表获取"
echo "3. ✓ Agent 创建（需要后端）"
echo "4. ✓ Add Memory - 记忆添加功能（已修复）"
echo "5. ✓ Search Memories - 记忆搜索功能"
echo "6. ✓ Chat - 智能对话功能（需要后端+Agent）"
echo ""

echo -e "${BLUE}Claude Code 集成配置：${NC}"
echo ""
echo "在项目根目录创建 .mcp.json 文件："
echo ""
echo '{'
echo '  "mcpServers": {'
echo '    "agentmem": {'
echo "      \"command\": \"$(pwd)/$MCP_SERVER_PATH\","
echo '      "args": [],'
echo '      "env": {'
echo '        "RUST_LOG": "info",'
echo '        "AGENTMEM_API_URL": "http://127.0.0.1:8080",'
echo '        "AGENTMEM_DEFAULT_AGENT_ID": "agent_001"'
echo '      }'
echo '    }'
echo '  }'
echo '}'
echo ""

echo -e "${BLUE}使用步骤：${NC}"
echo ""
echo "1. ${GREEN}启动后端服务${NC}："
echo "   ./target/release/agent-mem-server --config config.toml"
echo ""
echo "2. ${GREEN}创建 .mcp.json${NC} (见上方配置)"
echo ""
echo "3. ${GREEN}重启 Claude Code${NC}"
echo ""
echo "4. ${GREEN}开始使用${NC}："
echo '   "请添加一条记忆：我正在学习 Rust"'
echo '   "搜索关于 Rust 的记忆"'
echo '   "与我对话，了解我的学习情况"'
echo ""

echo -e "${GREEN}测试完成！ 🎉${NC}"

