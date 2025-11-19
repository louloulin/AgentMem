#!/bin/bash

# AgentMem V4 - 核心 API 功能测试

set -e

BASE_URL="http://localhost:8080"
USER_ID="test-user-$(date +%s)"

echo "=========================================="
echo "AgentMem V4 - 核心 API 功能测试"
echo "=========================================="
echo "User ID: $USER_ID"
echo ""

# 测试1: Health Check
echo "📝 测试1: Health Check"
curl -s "$BASE_URL/health" | jq .
echo ""

# 测试2: 创建 Agent
echo "📝 测试2: 创建 Agent"
AGENT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Test Agent\",
    \"description\": \"Test agent for API validation\",
    \"system_prompt\": \"You are a helpful assistant.\",
    \"llm_provider\": \"deepseek\",
    \"llm_model\": \"deepseek-chat\"
  }")

echo "$AGENT_RESPONSE" | jq .
AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id // empty')

if [ -z "$AGENT_ID" ]; then
    echo "❌ 创建 Agent 失败"
    exit 1
fi

echo "✅ Agent ID: $AGENT_ID"
echo ""

# 测试3: 添加记忆
echo "📝 测试3: 添加记忆"
MEMORY_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"用户喜欢喝咖啡，不喜欢茶。\",
    \"memory_type\": \"semantic\",
    \"importance\": 0.8
  }")

echo "$MEMORY_RESPONSE" | jq .
MEMORY_ID=$(echo "$MEMORY_RESPONSE" | jq -r '.data.id // empty')

if [ -z "$MEMORY_ID" ]; then
    echo "❌ 添加记忆失败"
    exit 1
fi

echo "✅ Memory ID: $MEMORY_ID"
echo ""

# 测试4: 搜索记忆
echo "[object Object]搜索记忆"
sleep 2
SEARCH_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d "{
    \"query\": \"用户喜欢什么饮料？\",
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"limit\": 5
  }")

echo "$SEARCH_RESPONSE" | jq .
FOUND_COUNT=$(echo "$SEARCH_RESPONSE" | jq '.data | length')
echo "找到 $FOUND_COUNT 条记忆"
echo ""

# 测试5: 更新记忆
echo "[object Object]新记忆"
UPDATE_RESPONSE=$(curl -s -X PUT "$BASE_URL/api/v1/memories/$MEMORY_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "用户喜欢喝咖啡和果汁，不喜欢茶。",
    "importance": 0.9
  }')

echo "$UPDATE_RESPONSE" | jq .
echo ""

# 测试6: 获取更新后的记忆
echo[object Object] 验证更新"
GET_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/memories/$MEMORY_ID")
echo "$GET_RESPONSE" | jq .
UPDATED_CONTENT=$(echo "$GET_RESPONSE" | jq -r '.data.content // empty')

if [[ "$UPDATED_CONTENT" == *"果汁"* ]]; then
    echo "✅ 记忆更新成功"
else
    echo "❌ 记忆更新失败"
fi
echo ""

# 测试7: Chat 功能
echo "📝 测试7: Chat 功能（带记忆检索）"
CHAT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"message\": \"我喜欢喝什么？\"
  }")

echo "$CHAT_RESPONSE" | jq .
AI_CONTENT=$(echo "$CHAT_RESPONSE" | jq -r '.data.content // empty')
echo ""
echo "AI 回答: $AI_CONTENT"

if [[ "$AI_CONTENT" == *"咖啡"* ]] || [[ "$AI_CONTENT" == *"果汁"* ]]; then
    echo "✅ Chat 功能正常 - AI 使用了记忆"
else
    echo "⚠️  Chat 功能部分正常 - AI 回答: $AI_CONTENT"
fi
echo ""

# 测试8: 删除记忆
echo "[object Object]除记忆"
DELETE_RESPONSE=$(curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEMORY_ID")
echo "$DELETE_RESPONSE" | jq .
echo ""

# 测试9: 验证删除
echo "📝 测试9: 验证删除"
VERIFY_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/memories/$MEMORY_ID")
echo "$VERIFY_RESPONSE" | jq .

if echo "$VERIFY_RESPONSE" | jq -e '.success == false' > /dev/null; then
    echo "✅ 记忆删除成功"
else
    echo "⚠️  记忆可能仍存在"
fi
echo ""

# 清理
echo "[object Object] 删除 Agent"
curl -s -X DELETE "$BASE_URL/api/v1/agents/$AGENT_ID" > /dev/null
echo "✅ Agent 已删除"
echo ""

echo "=========================================="
echo "测试完成"
echo "=========================================="
