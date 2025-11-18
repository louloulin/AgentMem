#!/bin/bash

echo "🧪 独立测试 AgentMem Backend 功能"
echo "=========================================="

BASE_URL="http://localhost:8080"

# 1. 测试直接保存记忆到AgentMem
echo -e "\n📝 1. 测试直接保存记忆..."
AGENT_ID="test-agent-001"
USER_ID="test-user-001"

# 创建测试agent
echo "创建测试Agent..."
AGENT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Memory Backend Test\",
    \"type\": \"chat\",
    \"system\": \"Test agent\",
    \"llm_config\": {
      \"provider\": \"zhipu\",
      \"model\": \"glm-4-flash\",
      \"temperature\": 0.7
    }
  }")

AGENT_ID=$(echo $AGENT_RESPONSE | jq -r '.data.id')
echo "✅ Agent ID: $AGENT_ID"

# 2. 第一次对话 - 触发memory store
echo -e "\n💬 2. 第一次对话（保存记忆）..."
CHAT1=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"我叫测试用户，我的ID是12345\", \"user_id\": \"$USER_ID\"}")

echo "Response:"
echo $CHAT1 | jq '{content: .data.content, memories_updated: .data.memories_updated, memories_count: .data.memories_count}'

# 3. 直接查询数据库中的记忆
echo -e "\n🔍 3. 查询数据库中的记忆..."
SEARCH_RESULT=$(curl -s "$BASE_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "", "limit": 20}')

TOTAL_MEMORIES=$(echo $SEARCH_RESULT | jq '.data | length')
echo "数据库中总共有 $TOTAL_MEMORIES 条记忆"

if [ "$TOTAL_MEMORIES" -gt 0 ]; then
    echo -e "\n最近的记忆:"
    echo $SEARCH_RESULT | jq '.data[0:3] | .[] | {content: .content, user_id: .user_id, created_at: .created_at}'
fi

# 4. 按user_id查询
echo -e "\n🔍 4. 查询特定用户的记忆..."
USER_MEMORIES=$(curl -s "$BASE_URL/api/v1/memories?user_id=$USER_ID&limit=10" \
  | jq '.data | length')
echo "用户 $USER_ID 的记忆数量: $USER_MEMORIES"

# 5. 第二次对话 - 测试memory retrieve
echo -e "\n💬 5. 第二次对话（测试记忆检索）..."
sleep 2
CHAT2=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"我的ID是多少？\", \"user_id\": \"$USER_ID\"}")

echo "Response:"
echo $CHAT2 | jq '{content: .data.content, memories_count: .data.memories_count}'

# 6. 总结
echo -e "\n=========================================="
echo "📊 测试总结"
echo "=========================================="
echo "✅ Agent ID: $AGENT_ID"
echo "✅ User ID: $USER_ID"
echo "✅ 数据库记忆总数: $TOTAL_MEMORIES"
echo "✅ 用户记忆数: $USER_MEMORIES"

if [ "$USER_MEMORIES" -gt 0 ]; then
    echo -e "\n✅ Memory store 功能正常"
else
    echo -e "\n❌ Memory store 功能异常"
fi

# 检查第二次对话的memories_count
MEMORIES_USED=$(echo $CHAT2 | jq -r '.data.memories_count')
if [ "$MEMORIES_USED" -gt 0 ]; then
    echo "✅ Memory retrieve 功能正常 (使用了 $MEMORIES_USED 条记忆)"
else
    echo "❌ Memory retrieve 功能异常 (未使用任何记忆)"
fi
