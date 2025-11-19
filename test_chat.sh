#!/bin/bash

BASE_URL="http://localhost:8080"
USER_ID="test-chat-$(date +%s)"

# 创建 Agent
AGENT_ID=$(curl -s -X POST "$BASE_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"Chat Test","description":"Test","system_prompt":"You are helpful.","llm_provider":"deepseek","llm_model":"deepseek-chat"}' \
  | jq -r '.data.id')

echo "Agent ID: $AGENT_ID"

# 添加记忆
MEMORY_ID=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\":\"$USER_ID\",\"agent_id\":\"$AGENT_ID\",\"content\":\"用户名叫李明，喜欢打篮球\",\"memory_type\":\"Semantic\",\"importance\":0.9}" \
  | jq -r '.data.id')

echo "Memory ID: $MEMORY_ID"
sleep 2

# Chat 测试
echo ""
echo "=== Chat 测试 ==="
CHAT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\":\"$USER_ID\",\"message\":\"你好，我是谁？我喜欢什么运动？\"}")

echo "$CHAT_RESPONSE" | jq .

# 清理
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEMORY_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/agents/$AGENT_ID" > /dev/null
