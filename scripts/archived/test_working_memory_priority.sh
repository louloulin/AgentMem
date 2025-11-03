#!/bin/bash

# ====================================
# Working Memory优先级测试脚本
# 测试当前会话记忆是否优先于历史长期记忆
# ====================================

set -e

BASE_URL="http://localhost:8080"
API_KEY="${AGENTMEM_API_KEY:-test-api-key-12345}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}================================
Working Memory 优先级测试
================================${NC}\n"

# 使用现有的agent
AGENT_ID="agent-7bd801e2-c8da-42e4-b10f-c2ef7f610235"
SESSION_ID=$(uuidgen)

echo -e "${YELLOW}测试场景：${NC}"
echo "1. 用户说：我是lin"
echo "2. 用户问：你是谁"
echo "3. 验证Agent是否只使用当前会话上下文，不混入历史记忆"
echo ""

echo -e "${BLUE}Session ID: ${SESSION_ID}${NC}"
echo ""

# 第一轮对话：建立当前会话上下文
echo -e "${YELLOW}[1/2] 发送消息: '我是lin'${NC}"
RESPONSE1=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"我是lin\",
    \"session_id\": \"$SESSION_ID\"
  }")

echo "Response:"
echo "$RESPONSE1" | jq -r '.content' | head -20
echo ""

sleep 2

# 第二轮对话：测试Agent是否记得当前会话
echo -e "${YELLOW}[2/2] 发送消息: '你是谁'${NC}"
RESPONSE2=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"你是谁\",
    \"session_id\": \"$SESSION_ID\"
  }")

echo "Response:"
CONTENT=$(echo "$RESPONSE2" | jq -r '.content')
echo "$CONTENT" | head -30
echo ""

# 验证结果
echo -e "${YELLOW}================================
验证结果
================================${NC}"

# 检查是否提到了"lin"
if echo "$CONTENT" | grep -qi "lin"; then
    echo -e "${GREEN}✅ 正确：回复中提到了'lin'（当前会话的用户名）${NC}"
else
    echo -e "${RED}❌ 错误：回复中没有提到'lin'${NC}"
fi

# 检查是否错误地提到了"张三"
if echo "$CONTENT" | grep -qi "张三"; then
    echo -e "${RED}❌ 错误：回复中提到了'张三'（这是历史记忆，不应该出现）${NC}"
else
    echo -e "${GREEN}✅ 正确：回复中没有提到'张三'（历史记忆被正确过滤）${NC}"
fi

# 检查Working Memory
echo ""
echo -e "${YELLOW}检查Working Memory:${NC}"
WM_ITEMS=$(curl -s "$BASE_URL/api/v1/agents/$AGENT_ID/working-memory/$SESSION_ID" \
  -H "X-API-Key: $API_KEY" | jq -r '.data | length')

echo -e "Working memory items: ${BLUE}${WM_ITEMS}${NC}"

if [ "$WM_ITEMS" -gt 0 ]; then
    echo -e "${GREEN}✅ Working memory正常记录${NC}"
    curl -s "$BASE_URL/api/v1/agents/$AGENT_ID/working-memory/$SESSION_ID" \
      -H "X-API-Key: $API_KEY" | jq -r '.data[] | "- [\(.created_at)] \(.content[:80])..."' | head -10
else
    echo -e "${RED}❌ Working memory为空${NC}"
fi

echo ""
echo -e "${GREEN}================================
测试完成
================================${NC}"
echo "Session ID: $SESSION_ID"
echo "Agent ID: $AGENT_ID"

