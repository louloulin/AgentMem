#!/bin/bash
# 简化的TTFB测试

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "🚀 简化TTFB测试"
echo ""

# 获取Agent ID
AGENT_ID=$(curl -s "$API_BASE/api/v1/agents" -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')
echo "Agent: $AGENT_ID"

# 验证模型
MODEL=$(curl -s "$API_BASE/api/v1/agents/$AGENT_ID" -H "Authorization: Bearer $TOKEN" | jq -r '.data.llm_config.model')
echo "模型: $MODEL"
echo ""

# 简短测试
echo "📝 测试: 简短问题"
echo "请求: 你好"
echo ""

START=$(date +%s%N)
echo "开始时间: $(date +%H:%M:%S.%N)"

curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message": "你好", "user_id": "test"}' 2>/dev/null | {
    
    FIRST_CONTENT=""
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$TYPE" == "content" ] && [ -z "$FIRST_CONTENT" ]; then
                FIRST_CONTENT="1"
                NOW=$(date +%s%N)
                TTFB=$(( (NOW - START) / 1000000 ))
                
                echo ""
                echo "⚡ 首个content chunk到达"
                echo "⏱️  TTFB: ${TTFB}ms ($(echo "scale=2; $TTFB/1000" | bc)秒)"
                echo ""
                
                if [ "$TTFB" -lt 2000 ]; then
                    echo "🎉 优秀！< 2秒"
                elif [ "$TTFB" -lt 5000 ]; then
                    echo "✅ 良好！< 5秒"
                elif [ "$TTFB" -lt 10000 ]; then
                    echo "⚠️  一般，< 10秒"
                else
                    echo "❌ 较慢，> 10秒"
                fi
                
                echo ""
                echo "📤 响应:"
            fi
            
            if [ "$TYPE" == "content" ]; then
                CONTENT=$(echo "$JSON" | jq -r '.content // empty' 2>/dev/null)
                echo -n "$CONTENT"
            elif [ "$TYPE" == "done" ]; then
                echo ""
                echo ""
                echo "✅ 完成"
                break
            fi
        fi
    done
}

echo ""
echo "测试完成"
