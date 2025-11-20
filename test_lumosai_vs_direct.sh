#!/bin/bash
# 对比测试：直接API vs LumosAI

API_KEY="${ZHIPU_API_KEY}"
LUMOSAI_API="http://localhost:8080"
LUMOSAI_TOKEN="test-token"

echo "=========================================="
echo "🔬 性能对比测试"
echo "=========================================="
echo ""

# 获取Agent ID
AGENT_ID=$(curl -s "$LUMOSAI_API/api/v1/agents" -H "Authorization: Bearer $LUMOSAI_TOKEN" | jq -r '.data[0].id')

TEST_MESSAGE="介绍机器学习"

echo "测试消息: $TEST_MESSAGE"
echo ""

# ===== 测试1: 直接调用智谱AI API =====
echo "=========================================="
echo "📝 测试1: 直接调用智谱AI API"
echo "=========================================="
echo ""

if [ -z "$API_KEY" ]; then
    echo "⚠️  未设置ZHIPU_API_KEY，跳过直接API测试"
    DIRECT_TTFB="N/A"
else
    START1=$(date +%s%N)
    
    curl -N -X POST "https://open.bigmodel.cn/api/paas/v4/chat/completions" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d "{
        \"model\": \"glm-4-flash\",
        \"messages\": [{\"role\": \"user\", \"content\": \"$TEST_MESSAGE\"}],
        \"stream\": true
      }" 2>/dev/null | {
        
        FIRST=""
        while IFS= read -r line; do
            if [[ $line == data:* ]]; then
                DATA="${line#data: }"
                if [[ "$DATA" != "[DONE]" ]] && [ -z "$FIRST" ]; then
                    FIRST="1"
                    NOW=$(date +%s%N)
                    DIRECT_TTFB=$(( (NOW - START1) / 1000000 ))
                    echo "⚡ 直接API TTFB: ${DIRECT_TTFB}ms"
                    echo "$DIRECT_TTFB" > /tmp/direct_ttfb
                    break
                fi
            fi
        done
    }
    
    if [ -f /tmp/direct_ttfb ]; then
        DIRECT_TTFB=$(cat /tmp/direct_ttfb)
    fi
fi

echo ""

# ===== 测试2: 通过LumosAI调用 =====
echo "=========================================="
echo "📝 测试2: 通过LumosAI调用"
echo "=========================================="
echo ""

START2=$(date +%s%N)

curl -N -X POST "$LUMOSAI_API/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $LUMOSAI_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"$TEST_MESSAGE\", \"user_id\": \"benchmark\"}" 2>/dev/null | {
    
    FIRST=""
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$TYPE" == "content" ] && [ -z "$FIRST" ]; then
                FIRST="1"
                NOW=$(date +%s%N)
                LUMOSAI_TTFB=$(( (NOW - START2) / 1000000 ))
                echo "⚡ LumosAI TTFB: ${LUMOSAI_TTFB}ms"
                echo "$LUMOSAI_TTFB" > /tmp/lumosai_ttfb
                break
            fi
        fi
    done
}

if [ -f /tmp/lumosai_ttfb ]; then
    LUMOSAI_TTFB=$(cat /tmp/lumosai_ttfb)
fi

echo ""

# ===== 分析对比 =====
echo "=========================================="
echo "📊 性能对比分析"
echo "=========================================="
echo ""

if [ -f /tmp/direct_ttfb ] && [ -f /tmp/lumosai_ttfb ]; then
    DIRECT_TTFB=$(cat /tmp/direct_ttfb)
    LUMOSAI_TTFB=$(cat /tmp/lumosai_ttfb)
    
    echo "结果对比:"
    echo "  直接API:   ${DIRECT_TTFB}ms"
    echo "  LumosAI:   ${LUMOSAI_TTFB}ms"
    echo ""
    
    if [ "$DIRECT_TTFB" -gt 0 ] && [ "$LUMOSAI_TTFB" -gt 0 ]; then
        OVERHEAD=$(( LUMOSAI_TTFB - DIRECT_TTFB ))
        RATIO=$(awk "BEGIN {printf \"%.2f\", $LUMOSAI_TTFB / $DIRECT_TTFB}")
        
        echo "框架开销分析:"
        echo "  绝对开销: ${OVERHEAD}ms"
        echo "  相对倍数: ${RATIO}x"
        echo ""
        
        if [ "$OVERHEAD" -lt 500 ]; then
            echo "✅ 开销可接受 (<500ms)"
        elif [ "$OVERHEAD" -lt 1000 ]; then
            echo "⚠️  开销偏大 (500-1000ms)"
        else
            echo "❌ 开销过大 (>1000ms)"
        fi
        echo ""
        
        echo "开销来源分析:"
        echo "  1. HTTP路由层: ~5-10ms"
        echo "  2. Agent Factory: ~20-50ms"
        echo "  3. Memory retrieve: ~50-300ms ⚠️"
        echo "  4. StreamingAgent包装: ~5-10ms"
        echo "  5. SSE转换: ~5-10ms"
        echo "  ----------------------------------"
        echo "  预期总开销: ~85-380ms"
        echo ""
        
        if [ "$OVERHEAD" -gt 400 ]; then
            echo "⚠️  实际开销超过预期，可能原因:"
            echo "  - Memory retrieve过慢"
            echo "  - 数据库查询问题"
            echo "  - Agent初始化开销"
        fi
    fi
elif [ -f /tmp/lumosai_ttfb ]; then
    echo "仅LumosAI测试: ${LUMOSAI_TTFB}ms"
    echo "（未测试直接API，无法对比）"
else
    echo "测试失败，请检查服务状态"
fi

# 清理
rm -f /tmp/direct_ttfb /tmp/lumosai_ttfb

echo ""
echo "=========================================="
echo "🔍 详细分析建议"
echo "=========================================="
echo ""

cat << 'EOF'
查看服务器日志分析详细耗时:

tail -100 server-v4-traced.log | grep -E "⏱️|TTFB|Memory|Zhipu"

关键时间点:
1. [+0ms] Request received
2. [+Xms] Agent verified  
3. [+Xms] Permission checked
4. [+Xms] Starting Agent Factory
5. [+Xms] BasicAgent created
6. [+Xms] StreamingAgent created
7. [+Xms] Starting memory retrieve ← 重点
8. [+Xms] Memory retrieved
9. [+Xms] Calling execute_streaming
10. [+Xms] 首个content chunk

理想时间分布:
- 路由+权限: 0-20ms
- Agent Factory: 20-50ms  
- Memory retrieve: 50-200ms
- Streaming开始: 0-10ms
- 等待LLM首token: X秒 (API固有延迟)
EOF
