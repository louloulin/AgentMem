#!/bin/bash
# 更新Agent使用glm-4-flash模型

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "🔄 更新Agent模型配置..."
echo ""

# 获取agent ID
AGENTS=$(curl -s -X GET "$API_BASE/api/v1/agents" \
  -H "Authorization: Bearer $TOKEN")

AGENT_ID=$(echo "$AGENTS" | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
  echo "❌ 没有找到Agent"
  exit 1
fi

echo "✅ Agent ID: $AGENT_ID"
echo ""

# 获取当前配置
AGENT_DETAIL=$(curl -s -X GET "$API_BASE/api/v1/agents/$AGENT_ID" \
  -H "Authorization: Bearer $TOKEN")

CURRENT_MODEL=$(echo "$AGENT_DETAIL" | jq -r '.data.llm_config.model')
echo "📋 当前模型: $CURRENT_MODEL"
echo ""

# 更新为glm-4-flash
echo "🚀 更新为 glm-4-flash..."

UPDATE_RESULT=$(curl -s -X PATCH "$API_BASE/api/v1/agents/$AGENT_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"llm_config\": {
      \"provider\": \"zhipu\",
      \"model\": \"glm-4-flash\"
    }
  }")

if echo "$UPDATE_RESULT" | jq -e '.success' > /dev/null 2>&1; then
  echo "✅ 更新成功！"
  echo ""
  echo "📋 新配置:"
  echo "$UPDATE_RESULT" | jq '.data.llm_config'
  echo ""
  echo "🎯 预期效果："
  echo "  TTFB: 28.8秒 → 2-5秒"
  echo "  性能提升: 约6-14倍"
else
  echo "❌ 更新失败:"
  echo "$UPDATE_RESULT" | jq '.'
fi
