#!/bin/bash

# AgentMem V4 - Chat 功能测试（使用 Zhipu AI）

set -e

BASE_URL="http://localhost:8080"
USER_ID="test-chat-zhipu-$(date +%s)"

echo "=========================================="
echo "AgentMem V4 - Chat 功能测试 (Zhipu AI)"
echo "=========================================="
echo "User ID: $USER_ID"
echo ""

# 1. 创建 Agent（使用 Zhipu AI）
echo "1. 创建 Agent (使用 Zhipu AI)"
AGENT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Chat Test Agent (Zhipu)",
    "description": "Agent for testing chat functionality with Zhipu AI",
    "system": "你是一个有记忆的智能助手。回答问题时，请使用提供的记忆信息给出个性化的回答。",
    "llm_config": {
      "provider": "zhipu",
      "model": "glm-4-flash",
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

# 2. 添加记忆
echo "2. 添加记忆"

MEMORY1=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"用户名叫张伟，今年32岁，是一名产品经理。\",
    \"memory_type\": \"Semantic\",
    \"importance\": 0.9
  }")
MEM1_ID=$(echo "$MEMORY1" | jq -r '.data.id')
echo "  记忆1: $MEM1_ID"

MEMORY2=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"张伟喜欢阅读科幻小说，最喜欢的作家是刘慈欣。\",
    \"memory_type\": \"Semantic\",
    \"importance\": 0.8
  }")
MEM2_ID=$(echo "$MEMORY2" | jq -r '.data.id')
echo "  记忆2: $MEM2_ID"

MEMORY3=$(curl -s -X POST "$BASE_URL/api/v1/memories" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"张伟每天早上喝一杯黑咖啡，喜欢去星巴克工作。\",
    \"memory_type\": \"Semantic\",
    \"importance\": 0.7
  }")
MEM3_ID=$(echo "$MEMORY3" | jq -r '.data.id')
echo "  记忆3: $MEM3_ID"

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
    \"query\": \"张伟的个人信息\",
    \"user_id\": \"$USER_ID\",
    \"agent_id\": \"$AGENT_ID\",
    \"limit\": 5
  }")

FOUND_COUNT=$(echo "$SEARCH_RESPONSE" | jq '.data | length')
echo "找到 $FOUND_COUNT 条相关记忆"
echo "$SEARCH_RESPONSE" | jq -r '.data[0:3] | .[] | "  - \(.content)"'
echo ""

# 4. Chat 测试 - 问题1
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

if [[ "$AI_RESPONSE1" == *"张伟"* ]] || [[ "$AI_RESPONSE1" == *"32"* ]] || [[ "$AI_RESPONSE1" == *"产品经理"* ]]; then
    echo "✅ Chat 功能正常 - AI 使用了记忆"
else
    echo "⚠️  AI 回答: $AI_RESPONSE1"
fi
echo ""

# 5. Chat 测试 - 问题2
echo "5. Chat 测试 - 问题2: 兴趣爱好"
CHAT2=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"我喜欢什么书？\"
  }")

AI_RESPONSE2=$(echo "$CHAT2" | jq -r '.data.content // "无响应"')
echo "AI 回答: $AI_RESPONSE2"

if [[ "$AI_RESPONSE2" == *"科幻"* ]] || [[ "$AI_RESPONSE2" == *"刘慈欣"* ]]; then
    echo "✅ Chat 功能正常 - AI 使用了记忆"
else
    echo "⚠️  AI 回答: $AI_RESPONSE2"
fi
echo ""

# 6. Chat 测试 - 问题3
echo "6. Chat 测试 - 问题3: 生活习惯"
CHAT3=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"我早上通常喝什么？\"
  }")

AI_RESPONSE3=$(echo "$CHAT3" | jq -r '.data.content // "无响应"')
echo "AI 回答: $AI_RESPONSE3"

if [[ "$AI_RESPONSE3" == *"咖啡"* ]] || [[ "$AI_RESPONSE3" == *"黑咖啡"* ]]; then
    echo "✅ Chat 功能正常 - AI 使用了记忆"
else
    echo "⚠️  AI 回答: $AI_RESPONSE3"
fi
echo ""

# 7. 清理
echo "7. 清理资源"
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEM1_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEM2_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/memories/$MEM3_ID" > /dev/null
curl -s -X DELETE "$BASE_URL/api/v1/agents/$AGENT_ID" > /dev/null
echo "✅ 清理完成"
echo ""

echo "=========================================="
echo "测试完成"
echo "=========================================="

