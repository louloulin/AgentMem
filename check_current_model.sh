#!/bin/bash
# 检查当前agent使用的模型

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "🔍 检查当前Agent配置..."
echo ""

# 获取agent列表
AGENTS=$(curl -s -X GET "$API_BASE/api/v1/agents" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

# 获取第一个agent的详细信息
AGENT_ID=$(echo "$AGENTS" | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
  echo "❌ 没有找到Agent"
  exit 1
fi

echo "✅ Agent ID: $AGENT_ID"
echo ""

# 获取agent详情
AGENT_DETAIL=$(curl -s -X GET "$API_BASE/api/v1/agents/$AGENT_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "📋 LLM配置:"
echo "$AGENT_DETAIL" | jq '.data.llm_config'
echo ""

MODEL=$(echo "$AGENT_DETAIL" | jq -r '.data.llm_config.model // "unknown"')
PROVIDER=$(echo "$AGENT_DETAIL" | jq -r '.data.llm_config.provider // "unknown"')

echo "🎯 当前配置:"
echo "  Provider: $PROVIDER"
echo "  Model: $MODEL"
echo ""

if [[ "$MODEL" == "glm-4-flash" ]]; then
  echo "✅ 已使用快速模型 glm-4-flash"
elif [[ "$MODEL" == *"flash"* ]] || [[ "$MODEL" == *"air"* ]]; then
  echo "✅ 使用快速模型: $MODEL"
else
  echo "⚠️  当前使用慢速模型: $MODEL"
  echo ""
  echo "建议修改为："
  echo "  glm-4-flash (推荐，最快)"
  echo "  glm-4-air (很快)"
fi
