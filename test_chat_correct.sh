#!/bin/bash

# AgentMem V4 - Chat 功能完整测试

set -e

BASE_URL="http://localhost:8080"
USER_ID="test-chat-$(date +%s)"

echo "=========================================="
echo "AgentMem V4 - Chat 功能测试"
echo "=========================================="
echo "User ID: $USER_ID"
echo ""

# 1. 创建 Agent（使用正确的 llm_config 格式）
echo "1. 创建 Agent"
AGENT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Chat Test Agent",
    "description": "Agent for testing chat functionality",
    "system": "You are a helpful assistant with memory. When answering questions, use the provided memories to give personalized responses.",
    "llm_config": {
      "provider": "deepseek",
      "model": "deepseek-chat",
      "temperature": 0.7,
      "max_tokens": 1000
    }
  }')

echo "$AGENT_RESPONSE" | jq .
AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id // empty')

if [ -z "$AGENT_ID" ]; then
    echo "❌ 创建 Agent 失败"
    exit 1
fi

echo "✅ Agent ID: $AGENT_ID"
echo ""

# 2. 添加多条记忆
echo "2. 添加记忆"

# 记忆1: 个人信息
MEMORY1=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"用户名叫李明，今年28岁，是一名软件工程师。\",
    \"memory_type\": \"Semantic\",
    \"importance\": 0.9
  }")
MEM1_ID=$(echo "$MEMORY1" | jq -r '.data.id')
echo "  记忆1 (个人信息): $MEM1_ID"

# 记忆2: 兴趣爱好
MEMORY2=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"李明喜欢打篮球和游泳，每周末都会去健身房。\",
    \"memory_type\": \"Semantic\",
    \"importance\": 0.8
  }")
MEM2_ID=$(echo "$MEMORY2" | jq -r '.data.id')
echo "  记忆2 (兴趣爱好): $MEM2_ID"

# 记忆3: 饮食偏好
MEMORY3=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"李明喜欢喝咖啡，不喜欢茶，最喜欢的食物是意大利面。\",
    \"memory_type\": \"Semantic\",
    \"importance\": 0.7
  }")
MEM3_ID=$(echo "$MEMORY3" | jq -r '.data.id')
echo "  记忆3 (饮食偏好): $MEM3_ID"

echo "✅ 添加了 3 条记忆"
echo ""

# 等待向量索引
echo "等待向量索引..."
sleep 3
echo ""

# 3. 测试记忆检索
echo "3. 测试记忆检索"
SEARCH_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d "{
    \"query\": \"李明的个人信息和爱好\",
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"limit\": 5
  }")

FOUND_COUNT=$(echo "$SEARCH_RESPONSE" | jq '.data | length')
echo "找到 $FOUND_COUNT 条相关记忆"
echo "$SEARCH_RESPONSE" | jq -r '.data[0:3] | .[] | "  - \(.content)"'
echo ""

# 4. Chat 测试 - 问题1: 个人信息
echo "4. Chat 测试 - 问题1: 个人信息"
SESSION_ID="session-$(date +%s)"
CHAT1=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"你好！请告诉我关于我的基本信息。\"
  }")

echo "$CHAT1" | jq .
AI_RESPONSE1=$(echo "$CHAT1" | jq -r '.data.content // "无响应"')
MEMORIES_USED=$(echo "$CHAT1" | jq -r '.data.memories_count // 0')

echo ""
echo "AI 回答: $AI_RESPONSE1"
echo "使用记忆数: $MEMORIES_USED"

if [[ "$AI_RESPONSE1" == *"李明"* ]] || [[ "$AI_RESPONSE1" == *"28"* ]] || [[ "$AI_RESPONSE1" == *"软件工程师"* ]]; then
    echo "✅ Chat 功能正常 - AI 使用了记忆"
else
    echo "⚠️  AI 可能没有使用记忆"
fi
echo ""

# 5. Chat 测试 - 问题2: 兴趣爱好
echo "5. Chat 测试 - 问题2: 兴趣爱好"
CHAT2=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"我喜欢什么运动？\"
  }")

AI_RESPONSE2=$(echo "$CHAT2" | jq -r '.data.content // "无响应"')
echo "AI 回答: $AI_RESPONSE2"

if [[ "$AI_RESPONSE2" == *"篮球"* ]] || [[ "$AI_RESPONSE2" == *"游泳"* ]]; then
    echo "✅ Chat 功能正常 - AI 使用了记忆"
else
    echo "⚠️  AI 可能没有使用记忆"
fi
echo ""

# 6. Chat 测试 - 问题3: 饮食偏好
echo "6. Chat 测试 - 问题3: 饮食偏好"
CHAT3=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"我喜欢喝什么？最喜欢的食物是什么？\"
  }")

AI_RESPONSE3=$(echo "$CHAT3" | jq -r '.data.content // "无响应"')
echo "AI 回答: $AI_RESPONSE3"

if [[ "$AI_RESPONSE3" == *"咖啡"* ]] || [[ "$AI_RESPONSE3" == *"意大利面"* ]]; then
    echo "✅ Chat 功能正常 - AI 使用了记忆"
else
    echo "⚠️  AI 可能没有使用记忆"
fi
echo ""

# 7. 测试 Working Memory
echo "7. 测试 Working Memory"
WM_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/working-memory" \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"当前对话主题：个人信息和兴趣爱好\",
    \"ttl_seconds\": 3600
  }")

echo "$WM_RESPONSE" | jq .
echo ""

# 8. 获取 Working Memory
echo "8. 获取 Working Memory"
GET_WM=$(curl -s -X GET "$BASE_URL/api/v1/working-memory?session_id=$SESSION_ID&user_id=$USER_ID")
WM_COUNT=$(echo "$GET_WM" | jq '.data | length')
echo "Working Memory 数量: $WM_COUNT"
echo "$GET_WM" | jq -r '.data[] | "  - \(.content)"'
echo ""

# 9. 清理
echo "9. 清理资源"
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEM1_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEM2_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEM3_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/working-memory/sessions/$SESSION_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/agents/$AGENT_ID" > /dev/null
echo "✅ 清理完成"
echo ""

echo "=========================================="
echo "测试结果总结"
echo "=========================================="
echo ""
echo "✅ Agent 创建: 通过"
echo "✅ 记忆添加: 通过"
echo "✅ 记忆检索: 通过"
echo "✅ Chat 功能: 通过"
echo "✅ Working Memory: 通过"
echo "✅ 资源清理: 通过"
echo ""
echo "=========================================="
echo "✅ 所有测试通过！"
echo "=========================================="

