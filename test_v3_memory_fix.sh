#!/bin/bash
# V3测试 - Memory优化验证

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "=========================================="
echo "🎯 V3测试: Memory配置优化"
echo "=========================================="
echo ""

AGENT_ID=$(curl -s "$API_BASE/api/v1/agents" -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')

echo "Agent ID: $AGENT_ID"
echo "优化项:"
echo "  ✅ executor.rs: last_messages 10 → 3"
echo "  ✅ memory_adapter.rs: 已是3"
echo "  ✅ 预期: Prompt tokens从2327降到~900"
echo ""

echo "📝 测试消息: 机器学习是什么？"
echo ""

START=$(date +%s%N)

curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message": "机器学习是什么？", "user_id": "v3-test"}' 2>/dev/null | {
    
    FIRST_CONTENT=""
    CONTENT_COUNT=0
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$TYPE" == "content" ]; then
                if [ -z "$FIRST_CONTENT" ]; then
                    FIRST_CONTENT="1"
                    NOW=$(date +%s%N)
                    TTFB=$(( (NOW - START) / 1000000 ))
                    
                    echo "⚡ TTFB: ${TTFB}ms"
                    
                    if [ "$TTFB" -lt 5000 ]; then
                        echo "✅ 目标达成！< 5秒"
                    elif [ "$TTFB" -lt 10000 ]; then
                        echo "⚠️  接近目标，< 10秒"
                    else
                        echo "❌ 仍需优化，> 10秒"
                    fi
                    
                    echo ""
                    echo "响应内容:"
                fi
                
                CONTENT=$(echo "$JSON" | jq -r '.content // empty' 2>/dev/null)
                echo -n "$CONTENT"
                CONTENT_COUNT=$((CONTENT_COUNT + 1))
                
            elif [ "$TYPE" == "done" ]; then
                NOW=$(date +%s%N)
                TOTAL=$(( (NOW - START) / 1000000 ))
                
                echo ""
                echo ""
                echo "✅ 完成"
                echo ""
                echo "统计:"
                echo "  TTFB: ${TTFB}ms ($(echo "scale=1; $TTFB/1000" | bc)秒)"
                echo "  总耗时: ${TOTAL}ms ($(echo "scale=1; $TOTAL/1000" | bc)秒)"
                echo "  Chunk数: $CONTENT_COUNT"
                echo ""
                
                # 对比评估
                echo "对比基线:"
                echo "  V1 (glm-4.6, mem=10): 28.8秒"
                echo "  V2 (glm-4-flash, mem=10): 15.6秒"
                echo "  V3 (glm-4-flash, mem=3): ${TTFB}ms"
                
                if [ -n "$TTFB" ] && [ "$TTFB" -gt 0 ]; then
                    V1_IMPROVE=$((28800 / TTFB))
                    V2_IMPROVE=$((15600 / TTFB))
                    echo ""
                    echo "  vs V1提升: ${V1_IMPROVE}倍"
                    echo "  vs V2提升: ${V2_IMPROVE}倍"
                fi
                
                break
            fi
        fi
    done
}

echo ""
echo "=========================================="
echo "检查服务器日志"
echo "=========================================="
echo ""

echo "期望看到:"
echo "  ✅ Retrieved 3 memories (不是10)"
echo "  ✅ Prompt tokens < 1000"
echo "  ✅ 只有1次Zhipu API调用(streaming)"
echo ""

echo "查看最新日志:"
tail -30 server-v3.log | grep -E "Retrieved.*memories|Token 使用|Zhipu API" | tail -10

echo ""
echo "✅ V3测试完成"
