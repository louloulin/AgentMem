#!/bin/bash

MCP_SERVER="./target/release/agentmem-mcp-server"
USER_ID="default"  # ← 使用默认值
AGENT_ID="agent-a23bfd10-b1be-4848-8b4e-f3d34f4aae0e"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== 使用默认 User ID 测试 ===${NC}"
echo "User ID: $USER_ID"
echo ""

# 添加记忆
echo -e "${BLUE}1. 添加记忆...${NC}"
ADD='{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"使用默认UserID测试搜索功能 default user test search","user_id":"'$USER_ID'","agent_id":"'$AGENT_ID'"}}}'

ADD_RESP=$(echo "$ADD" | $MCP_SERVER 2>/dev/null)
MEMORY_ID=$(echo "$ADD_RESP" | jq -r '.result.content[0].text | fromjson.memory_id // "N/A"')
echo "记忆ID: $MEMORY_ID"
echo ""

# 等待索引
echo -e "${YELLOW}2. 等待索引（2秒）...${NC}"
sleep 2
echo ""

# 搜索记忆
echo -e "${BLUE}3. 搜索记忆...${NC}"
SEARCH='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"default user test search","user_id":"'$USER_ID'","limit":5}}}'

SEARCH_RESP=$(echo "$SEARCH" | $MCP_SERVER 2>/dev/null)
RESULT_COUNT=$(echo "$SEARCH_RESP" | jq -r '.result.content[0].text | fromjson.total_results // 0')

echo "找到: $RESULT_COUNT 条记忆"
echo ""

if [ "$RESULT_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ 测试成功！搜索功能正常工作${NC}"
    echo ""
    echo "搜索结果:"
    echo "$SEARCH_RESP" | jq '.result.content[0].text | fromjson.results[0]'
else
    echo -e "${YELLOW}✗ 仍未找到记忆${NC}"
    echo ""
    echo "可能原因:"
    echo "  1. 向量索引需要更长时间"
    echo "  2. 搜索查询与内容不够匹配"
    echo "  3. 向量数据库配置问题"
fi

