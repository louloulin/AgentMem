#!/bin/bash
set -e

BASE_URL="http://localhost:8080"
USER_ID="test-user-$(date +%s)"

echo "=== AgentMem V4 核心功能测试 ==="
echo "User ID: $USER_ID"
echo ""

# 1. Health Check
echo "1. Health Check"
curl -s "$BASE_URL/health" | jq -r '.status'
echo ""

# 2. 创建 Agent
echo "2. 创建 Agent"
AGENT_ID=$(curl -s -X POST "$BASE_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","description":"Test","system_prompt":"You are helpful.","llm_provider":"deepseek","llm_model":"deepseek-chat"}' \
  | jq -r '.data.id')
echo "Agent ID: $AGENT_ID"
echo ""

# 3. 添加记忆
echo "3. 添加记忆"
MEMORY_ID=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\":\"$USER_ID\",\"agent_id\":\"$AGENT_ID\",\"content\":\"用户喜欢咖啡\",\"memory_type\":\"Semantic\",\"importance\":0.8}" \
  | jq -r '.data.id')
echo "Memory ID: $MEMORY_ID"
sleep 2
echo ""

# 4. 搜索记忆
echo "4. 搜索记忆"
FOUND=$(curl -s -X POST "$BASE_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d "{\"query\":\"咖啡\",\"user_id\":\"$USER_ID\",\"agent_id\":\"$AGENT_ID\",\"limit\":5}" \
  | jq '.data | length')
echo "找到 $FOUND 条记忆"
echo ""

# 5. 更新记忆
echo "5. 更新记忆"
curl -s -X PUT "$BASE_URL/api/v1/memories/$MEMORY_ID" \
  -H "Content-Type: application/json" \
  -d '{"content":"用户喜欢咖啡和果汁","importance":0.9}' \
  | jq -r '.data.message'
sleep 1
echo ""

# 6. 验证更新
echo "6. 验证更新"
CONTENT=$(curl -s -X GET "$BASE_URL/api/v1/memories/$MEMORY_ID" | jq -r '.data.content')
echo "更新后: $CONTENT"
echo ""

# 7. Working Memory
echo "7. Working Memory"
SESSION_ID="session-$(date +%s)"
curl -s -X POST "$BASE_URL/api/v1/working-memory" \
  -H "Content-Type: application/json" \
  -d "{\"session_id\":\"$SESSION_ID\",\"user_id\":\"$USER_ID\",\"agent_id\":\"$AGENT_ID\",\"content\":\"当前主题：咖啡\",\"ttl_seconds\":3600}" \
  | jq -r '.data.message // "添加成功"'
echo ""

# 8. Chat 测试
echo "8. Chat 测试"
AI_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{\"user_id\":\"$USER_ID\",\"session_id\":\"$SESSION_ID\",\"message\":\"我喜欢喝什么？\"}" \
  | jq -r '.data.content')
echo "AI: $AI_RESPONSE"
echo ""

# 9. 删除记忆
echo "9. 删除记忆"
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEMORY_ID" | jq -r '.data.message'
echo ""

# 10. 清理
echo "10. 清理"
curl -s -X DELETE "$BASE_URL/api/v1/agents/$AGENT_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/working-memory/sessions/$SESSION_ID" > /dev/null
echo "清理完成"
echo ""

echo "=== 所有测试完成 ==="
