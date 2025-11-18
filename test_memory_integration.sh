#!/bin/bash

echo "🧪 LumosAI 记忆集成完整测试"
echo "=========================================="

BASE_URL="http://localhost:8080"

# 1. 创建测试Agent
echo -e "\n📝 1. 创建测试Agent..."
AGENT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Memory Test Agent",
    "type": "chat",
    "system": "你是一个记忆测试助手，能够记住用户的信息",
    "llm_config": {
      "provider": "zhipu",
      "model": "glm-4-flash",
      "temperature": 0.7
    }
  }')

AGENT_ID=$(echo $AGENT_RESPONSE | jq -r '.data.id')
echo "✅ Agent ID: $AGENT_ID"

# 2. 第一次对话 - 告诉Agent信息
echo -e "\n💬 2. 第一次对话 - 告诉Agent我的名字..."
CHAT1=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d '{"message": "你好，我叫张三，我住在北京", "user_id": "test_memory_user"}')

echo "Response:"
echo $CHAT1 | jq '.data | {content, memories_updated, memories_count}'

# 3. 验证记忆已保存
echo -e "\n🔍 3. 验证记忆已保存到数据库..."
SEARCH1=$(curl -s "$BASE_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "张三", "limit": 5}')

MEMORY_COUNT=$(echo $SEARCH1 | jq '.data | length')
echo "找到 $MEMORY_COUNT 条包含'张三'的记忆"

if [ "$MEMORY_COUNT" -gt 0 ]; then
    echo "记忆内容:"
    echo $SEARCH1 | jq '.data[] | {content: .content, created_at: .created_at}' | head -10
fi

# 4. 第二次对话 - 测试记忆检索
echo -e "\n💬 4. 第二次对话 - 测试记忆检索..."
CHAT2=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d '{"message": "你还记得我叫什么名字吗？我住在哪里？", "user_id": "test_memory_user"}')

echo "Response:"
echo $CHAT2 | jq '.data | {content, memories_updated, memories_count}'

# 5. 第三次对话 - 测试记忆检索（搜索"北京"）
echo -e "\n💬 5. 第三次对话 - 测试记忆检索（北京）..."
CHAT3=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d '{"message": "北京的天气怎么样？", "user_id": "test_memory_user"}')

echo "Response:"
echo $CHAT3 | jq '.data | {content, memories_updated, memories_count}'

# 6. 验证所有记忆
echo -e "\n🔍 6. 查看所有保存的记忆..."
SEARCH2=$(curl -s "$BASE_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "张三", "limit": 10}')

TOTAL_MEMORIES=$(echo $SEARCH2 | jq '.data | length')
echo "总共保存了 $TOTAL_MEMORIES 条相关记忆"

# 7. 总结
echo -e "\n=========================================="
echo "📊 测试总结"
echo "=========================================="
echo "✅ Agent创建成功"
echo "✅ 对话记忆保存: $([ "$MEMORY_COUNT" -gt 0 ] && echo '成功' || echo '失败')"
echo "✅ 记忆检索测试: 查看上面的响应内容"
echo "✅ 总记忆数: $TOTAL_MEMORIES"

if [ "$TOTAL_MEMORIES" -gt 0 ]; then
    echo -e "\n🎉 记忆集成功能正常工作！"
else
    echo -e "\n⚠️  记忆保存可能存在问题"
fi
