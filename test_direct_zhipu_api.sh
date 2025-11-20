#!/bin/bash
# 直接测试智谱AI API性能

# 需要设置API Key
API_KEY="${ZHIPU_API_KEY}"

if [ -z "$API_KEY" ]; then
    echo "❌ 请设置环境变量 ZHIPU_API_KEY"
    echo "export ZHIPU_API_KEY='your-api-key'"
    exit 1
fi

API_URL="https://open.bigmodel.cn/api/paas/v4/chat/completions"

echo "=========================================="
echo "🧪 直接测试智谱AI API"
echo "=========================================="
echo ""
echo "模型: glm-4-flash"
echo "消息: 什么是AI？"
echo ""

# 记录开始时间
START=$(date +%s%N)

echo "⏱️  [0ms] 发送请求..."

# 发送streaming请求
curl -N -X POST "$API_URL" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "glm-4-flash",
    "messages": [
      {
        "role": "user",
        "content": "什么是AI？"
      }
    ],
    "stream": true,
    "temperature": 0.7
  }' 2>/dev/null | {
    
    FIRST_CHUNK=""
    CHUNK_COUNT=0
    
    while IFS= read -r line; do
        # SSE格式: data: {...}
        if [[ $line == data:* ]]; then
            DATA="${line#data: }"
            
            # 跳过 [DONE]
            if [[ "$DATA" == "[DONE]" ]]; then
                break
            fi
            
            # 记录首chunk时间
            if [ -z "$FIRST_CHUNK" ]; then
                FIRST_CHUNK="1"
                NOW=$(date +%s%N)
                TTFB=$(( (NOW - START) / 1000000 ))
                echo ""
                echo "⚡ 首chunk TTFB: ${TTFB}ms"
                echo ""
                echo "📤 响应内容:"
            fi
            
            # 解析content
            CONTENT=$(echo "$DATA" | jq -r '.choices[0].delta.content // empty' 2>/dev/null)
            if [ -n "$CONTENT" ]; then
                echo -n "$CONTENT"
                CHUNK_COUNT=$((CHUNK_COUNT + 1))
            fi
        fi
    done
    
    if [ -n "$FIRST_CHUNK" ]; then
        END=$(date +%s%N)
        TOTAL=$(( (END - START) / 1000000 ))
        
        echo ""
        echo ""
        echo "✅ 完成"
        echo ""
        echo "📊 性能统计:"
        echo "  - TTFB: ${TTFB}ms"
        echo "  - 总耗时: ${TOTAL}ms"
        echo "  - Chunk数: $CHUNK_COUNT"
        echo ""
        echo "🎯 这是智谱AI API的原生性能基线"
    fi
}

echo ""
