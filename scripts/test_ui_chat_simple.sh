#!/bin/bash

# 简化的 UI Chat 功能测试
# 日期: 2025-11-03

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "======================================"
echo "  UI Chat 功能测试"
echo "======================================"
echo ""

# 1. 测试 UI 页面
echo -e "${BLUE}[测试]${NC} UI 页面可访问性..."
echo ""

pages=(
    "http://localhost:3001"
    "http://localhost:3001/admin"
    "http://localhost:3001/admin/chat"
    "http://localhost:3001/admin/memories"
    "http://localhost:3001/admin/agents"
)

for url in "${pages[@]}"; do
    if curl -s -f "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅${NC} $url"
    else
        echo "❌ $url"
    fi
done

echo ""

# 2. 测试 Chat API
echo -e "${BLUE}[测试]${NC} Chat API 功能..."
echo ""

# 使用已存在的 Agent
AGENT_ID="agent-6812f152-16c0-4637-8fc0-714efee147f3"
SESSION_ID="ui-test-$(date +%s)"

echo "Agent ID: $AGENT_ID"
echo "Session ID: $SESSION_ID"
echo ""

# 发送测试消息
echo "发送测试消息..."
RESPONSE=$(curl -s -X POST "http://localhost:8080/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"你好，这是 UI Chat 测试\",
    \"session_id\": \"$SESSION_ID\",
    \"user_id\": \"ui-test-user\",
    \"stream\": false
  }")

MESSAGE_ID=$(echo "$RESPONSE" | jq -r '.data.message_id')
CONTENT=$(echo "$RESPONSE" | jq -r '.data.content')

if [ "$MESSAGE_ID" != "null" ] && [ -n "$MESSAGE_ID" ]; then
    echo -e "${GREEN}✅${NC} Chat API 响应成功"
    echo "消息 ID: $MESSAGE_ID"
    echo "AI 回复: $CONTENT"
else
    echo "❌ Chat API 响应失败"
    echo "$RESPONSE" | jq .
fi

echo ""

# 3. 验证 Working Memory
echo -e "${BLUE}[测试]${NC} Working Memory 写入..."
echo ""

sleep 2

if [ -f "data/agentmem.db" ]; then
    COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE session_id='$SESSION_ID' AND memory_type='working';")
    
    if [ "$COUNT" -gt 0 ]; then
        echo -e "${GREEN}✅${NC} Working Memory 已写入: $COUNT 条记录"
    else
        echo "❌ Working Memory 未写入"
    fi
else
    echo "❌ 数据库文件不存在"
fi

echo ""
echo "======================================"
echo "  测试完成"
echo "======================================"

