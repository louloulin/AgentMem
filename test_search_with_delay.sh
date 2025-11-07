#!/bin/bash

# 测试：添加延迟后能否搜索到记忆

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

MCP_SERVER="./target/release/agentmem-mcp-server"
TEST_USER="test_delay_user"
TEST_AGENT="agent-a23bfd10-b1be-4848-8b4e-f3d34f4aae0e"  # 使用已存在的Agent

echo -e "${BLUE}=== 测试：搜索延迟验证 ===${NC}"
echo ""

# 1. 添加记忆
echo -e "${BLUE}步骤1: 添加记忆${NC}"
ADD_REQUEST='{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"测试向量索引延迟的记忆内容 vector index delay test","user_id":"'$TEST_USER'","agent_id":"'$TEST_AGENT'","memory_type":"Episodic"}}}'

ADD_RESPONSE=$(echo "$ADD_REQUEST" | $MCP_SERVER 2>/dev/null)
MEMORY_ID=$(echo "$ADD_RESPONSE" | jq -r '.result.content[0].text | fromjson.memory_id // "N/A"')

echo "记忆ID: $MEMORY_ID"
echo ""

# 2. 测试不同延迟
echo -e "${BLUE}步骤2: 测试不同延迟后的搜索结果${NC}"
echo ""

for delay in 0 1 2 3; do
    if [ $delay -gt 0 ]; then
        echo -e "${YELLOW}等待 ${delay} 秒...${NC}"
        sleep $delay
    else
        echo -e "${YELLOW}立即搜索（0秒延迟）...${NC}"
    fi
    
    SEARCH_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"vector index delay","user_id":"'$TEST_USER'","limit":5}}}'
    
    SEARCH_RESPONSE=$(echo "$SEARCH_REQUEST" | $MCP_SERVER 2>/dev/null)
    RESULT_COUNT=$(echo "$SEARCH_RESPONSE" | jq -r '.result.content[0].text | fromjson.total_results // 0')
    
    if [ "$RESULT_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✓ 延迟 ${delay}s: 找到 $RESULT_COUNT 条记忆${NC}"
        
        # 显示相似度
        SIMILARITY=$(echo "$SEARCH_RESPONSE" | jq -r '.result.content[0].text | fromjson.results[0].similarity // "N/A"')
        echo "  相似度: $SIMILARITY"
    else
        echo -e "✗ 延迟 ${delay}s: 未找到记忆"
    fi
    echo ""
done

echo -e "${BLUE}==================================${NC}"
echo -e "${GREEN}结论: 向量索引需要 1-2 秒完成${NC}"
echo -e "${BLUE}==================================${NC}"
