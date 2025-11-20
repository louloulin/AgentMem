#!/bin/bash

# 测试真实SSE流式传输

echo "🧪 测试AgentMem真实SSE流式传输"
echo "======================================"

# 获取agent_id
AGENT_ID=$(curl -s http://localhost:8080/api/v1/agents | jq -r '.[0].agent_id' 2>/dev/null)

if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" = "null" ]; then
    echo "❌ 无法获取agent_id，请先创建agent"
    exit 1
fi

echo "✅ 使用Agent: $AGENT_ID"
echo ""

# 测试Standard流式
echo "📡 测试Standard流式 (agentmem)"
echo "--------------------------------------"
curl -N -X POST "http://localhost:8080/api/v1/agents/$AGENT_ID/chat/stream" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好，请介绍一下你自己",
    "user_id": "test-user",
    "stream": true
  }' \
  2>&1 | while IFS= read -r line; do
    echo "[$(date +%H:%M:%S.%3N)] $line"
  done

echo ""
echo ""

# 测试LumosAI流式  
echo "📡 测试LumosAI流式"
echo "--------------------------------------"
curl -N -X POST "http://localhost:8080/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好，请用一句话介绍自己",
    "user_id": "test-user",
    "stream": true
  }' \
  2>&1 | while IFS= read -r line; do
    echo "[$(date +%H:%M:%S.%3N)] $line"
  done

echo ""
echo "✅ 测试完成"

