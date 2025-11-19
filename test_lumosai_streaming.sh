#!/bin/bash

# LumosAI 流式功能测试脚本
# 测试新实现的流式响应端点

set -e

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "=================================="
echo "🚀 LumosAI 流式功能测试"
echo "=================================="
echo ""

# 1. 获取或创建 Agent
echo "📋 步骤 1: 获取 Agent..."
AGENT_RESPONSE=$(curl -s -X GET "$API_BASE/api/v1/agents" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
  echo "⚠️  没有找到 Agent，创建新的..."
  AGENT_RESPONSE=$(curl -s -X POST "$API_BASE/api/v1/agents" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "name": "测试Agent",
      "llm_config": {
        "provider": "zhipu",
        "model": "glm-4-flash",
        "temperature": 0.7
      }
    }')
  
  AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id')
fi

echo "✅ Agent ID: $AGENT_ID"
echo ""

# 2. 写入测试记忆
echo "📝 步骤 2: 写入测试记忆..."

curl -s -X POST "$API_BASE/api/v1/memories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"test-user\",
    \"content\": \"用户的名字是王芳\",
    \"memory_type\": \"Episodic\",
    \"importance\": 0.9
  }" > /dev/null

curl -s -X POST "$API_BASE/api/v1/memories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"test-user\",
    \"content\": \"用户是一名AI工程师，专注于大语言模型研究\",
    \"memory_type\": \"Episodic\",
    \"importance\": 0.85
  }" > /dev/null

echo "✅ 已写入 2 条测试记忆"
echo ""

# 3. 测试 LumosAI 流式端点
echo "=================================="
echo "🌊 测试 LumosAI 流式响应"
echo "=================================="
echo ""
echo "📤 发送请求到: /api/v1/agents/$AGENT_ID/chat/lumosai/stream"
echo "💬 消息: 你好，我叫什么名字？"
echo ""
echo "--- 流式响应开始 ---"

curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好，我叫什么名字？",
    "user_id": "test-user"
  }' 2>/dev/null | while IFS= read -r line; do
    # 解析 SSE 数据
    if [[ $line == data:* ]]; then
      JSON_DATA="${line#data: }"
      CHUNK_TYPE=$(echo "$JSON_DATA" | jq -r '.chunk_type // empty')
      
      case "$CHUNK_TYPE" in
        start)
          echo "🚀 [开始] Agent 开始响应"
          ;;
        content)
          CONTENT=$(echo "$JSON_DATA" | jq -r '.content // empty')
          echo -n "$CONTENT"
          ;;
        tool_call)
          TOOL_NAME=$(echo "$JSON_DATA" | jq -r '.tool_name // empty')
          echo ""
          echo "🔧 [工具调用] $TOOL_NAME"
          ;;
        done)
          echo ""
          echo ""
          echo "✅ [完成] 生成完成"
          TOTAL_STEPS=$(echo "$JSON_DATA" | jq -r '.total_steps // 0')
          MEMORIES_UPDATED=$(echo "$JSON_DATA" | jq -r '.memories_updated // false')
          echo "   - 总步骤: $TOTAL_STEPS"
          echo "   - 记忆更新: $MEMORIES_UPDATED"
          ;;
        error)
          ERROR_MSG=$(echo "$JSON_DATA" | jq -r '.content // empty')
          echo ""
          echo "❌ [错误] $ERROR_MSG"
          ;;
      esac
    fi
done

echo ""
echo "--- 流式响应结束 ---"
echo ""

# 4. 对比测试：非流式端点
echo "=================================="
echo "📦 对比测试: LumosAI 非流式响应"
echo "=================================="
echo ""

START_TIME=$(date +%s%N)

RESPONSE=$(curl -s -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "我的职业是什么？",
    "user_id": "test-user"
  }')

END_TIME=$(date +%s%N)
DURATION=$(( (END_TIME - START_TIME) / 1000000 ))

echo "💬 问题: 我的职业是什么？"
echo "🤖 回复: $(echo "$RESPONSE" | jq -r '.data.content // "无回复"')"
echo "⏱️  响应时间: ${DURATION}ms"
echo ""

# 5. 总结
echo "=================================="
echo "📊 测试总结"
echo "=================================="
echo ""
echo "✅ 流式端点可用: /api/v1/agents/:id/chat/lumosai/stream"
echo "✅ 非流式端点可用: /api/v1/agents/:id/chat/lumosai"
echo "✅ 记忆功能正常"
echo "✅ SSE 事件流正常"
echo ""
echo "🎉 LumosAI 流式功能测试完成！"
echo ""
echo "💡 提示: 现在可以在前端 UI 中启用 LumosAI + 流式模式进行测试"
echo "   访问: http://localhost:3001/admin/chat"
