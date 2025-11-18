#!/bin/bash

# LumosAI Chat 功能 HTTP 测试脚本

set -e

BASE_URL="http://localhost:8080"
ORG_ID="test_org"
AGENT_ID="test_agent_$(date +%s)"
USER_ID="test_user"

echo "🧪 测试 LumosAI Chat 功能"
echo "================================"
echo ""

# 1. 创建测试 Agent
echo "📝 1. 创建测试 Agent..."
CREATE_AGENT_RESPONSE=$(curl -s -X POST \
  "${BASE_URL}/api/v1/agents" \
  -H "Content-Type: application/json" \
  -d "{
    \"id\": \"${AGENT_ID}\",
    \"name\": \"LumosAI Test Agent\",
    \"system\": \"You are a helpful AI assistant for testing LumosAI integration.\",
    \"llm_config\": {
      \"provider\": \"zhipu\",
      \"model\": \"glm-4-flash\",
      \"api_key\": \"99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k\"
    },
    \"organization_id\": \"${ORG_ID}\"
  }")

echo "创建结果: $CREATE_AGENT_RESPONSE"
echo ""

# 验证Agent是否创建成功并提取实际的agent_id
if echo "$CREATE_AGENT_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    AGENT_ID=$(echo "$CREATE_AGENT_RESPONSE" | jq -r '.data.id')
    echo "✅ Agent创建成功，ID: $AGENT_ID"
else
    echo "❌ Agent创建失败"
    exit 1
fi

# 等待agent创建
sleep 2

# 2. 测试 LumosAI Chat
echo "💬 2. 测试 LumosAI Chat..."
CHAT_RESPONSE=$(curl -s -X POST \
  "${BASE_URL}/api/v1/agents/${AGENT_ID}/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"Hello, this is a test message for LumosAI integration. Please respond briefly.\",
    \"user_id\": \"${USER_ID}\"
  }")

echo "Chat响应: $CHAT_RESPONSE"
echo ""

# 3. 验证响应
echo "✅ 3. 验证响应..."
if echo "$CHAT_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    echo "✅ Chat成功！"
    echo "响应内容: $(echo "$CHAT_RESPONSE" | jq -r '.data.content')"
    echo "消息ID: $(echo "$CHAT_RESPONSE" | jq -r '.data.message_id')"
    echo "记忆已更新: $(echo "$CHAT_RESPONSE" | jq -r '.data.memories_updated')"
    echo "处理时间: $(echo "$CHAT_RESPONSE" | jq -r '.data.processing_time_ms')ms"
else
    echo "❌ Chat失败"
    echo "错误信息: $CHAT_RESPONSE"
    exit 1
fi

echo ""
echo "================================"
echo "🎉 LumosAI Chat 测试完成！"
