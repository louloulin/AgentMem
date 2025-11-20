#!/bin/bash
# 智能对比测试 - 根据API Key可用性决定测试范围

API_KEY="${ZHIPU_API_KEY}"
LUMOSAI_API="http://localhost:8080"
LUMOSAI_TOKEN="test-token"
TEST_MESSAGE="什么是深度学习？"

echo "=========================================="
echo "🔬 智能对比测试"
echo "=========================================="
echo ""

# 检查API Key
if [ -z "$API_KEY" ]; then
    echo "⚠️  ZHIPU_API_KEY未设置"
    echo ""
    echo "将仅测试LumosAI性能（无法对比框架开销）"
    echo ""
    read -p "是否继续？(y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "测试取消"
        exit 0
    fi
    COMPARE_MODE="lumosai-only"
else
    echo "✅ API Key已设置"
    echo "将执行完整对比测试"
    COMPARE_MODE="full"
fi

echo ""
echo "测试消息: $TEST_MESSAGE"
echo ""

# ===== 测试1: 直接API (如果可用) =====
if [ "$COMPARE_MODE" == "full" ]; then
    echo "=========================================="
    echo "📝 测试1: 直接调用智谱AI API"
    echo "=========================================="
    echo ""
    
    START1=$(date +%s%N)
    echo "⏱️  发送直接API请求..."
    
    DIRECT_OUTPUT=$(curl -N -X POST "https://open.bigmodel.cn/api/paas/v4/chat/completions" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d "{
        \"model\": \"glm-4-flash\",
        \"messages\": [{\"role\": \"user\", \"content\": \"$TEST_MESSAGE\"}],
        \"stream\": true
      }" 2>/dev/null | {
        
        FIRST=""
        CHUNK_COUNT=0
        
        while IFS= read -r line; do
            if [[ $line == data:* ]]; then
                DATA="${line#data: }"
                
                if [[ "$DATA" == "[DONE]" ]]; then
                    break
                fi
                
                if [ -z "$FIRST" ]; then
                    FIRST="1"
                    NOW=$(date +%s%N)
                    TTFB=$(( (NOW - START1) / 1000000 ))
                    echo "$TTFB" > /tmp/direct_ttfb
                    echo ""
                    echo "⚡ 直接API TTFB: ${TTFB}ms ($(echo "scale=2; $TTFB/1000" | bc)秒)"
                fi
                
                CONTENT=$(echo "$DATA" | jq -r '.choices[0].delta.content // empty' 2>/dev/null)
                if [ -n "$CONTENT" ]; then
                    CHUNK_COUNT=$((CHUNK_COUNT + 1))
                fi
            fi
        done
        
        END1=$(date +%s%N)
        TOTAL1=$(( (END1 - START1) / 1000000 ))
        echo "$TOTAL1" > /tmp/direct_total
        echo "$CHUNK_COUNT" > /tmp/direct_chunks
        
        echo "📊 Chunk数: $CHUNK_COUNT"
        echo "⏱️  总耗时: ${TOTAL1}ms"
    })
    
    echo ""
    sleep 2  # 避免API限流
fi

# ===== 测试2: LumosAI =====
echo "=========================================="
echo "📝 测试2: 通过LumosAI调用"
echo "=========================================="
echo ""

AGENT_ID=$(curl -s "$LUMOSAI_API/api/v1/agents" -H "Authorization: Bearer $LUMOSAI_TOKEN" | jq -r '.data[0].id')

if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" == "null" ]; then
    echo "❌ 无法获取Agent ID"
    exit 1
fi

echo "Agent: $AGENT_ID"
echo ""

# 标记日志
echo "=== COMPARISON-TEST $(date +%H:%M:%S.%N) ===" >> server-v4-traced.log

START2=$(date +%s%N)
echo "⏱️  发送LumosAI请求..."

curl -N -X POST "$LUMOSAI_API/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $LUMOSAI_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"$TEST_MESSAGE\", \"user_id\": \"benchmark\"}" 2>/dev/null | {
    
    FIRST=""
    CHUNK_COUNT=0
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$TYPE" == "content" ]; then
                if [ -z "$FIRST" ]; then
                    FIRST="1"
                    NOW=$(date +%s%N)
                    TTFB=$(( (NOW - START2) / 1000000 ))
                    echo "$TTFB" > /tmp/lumosai_ttfb
                    echo ""
                    echo "⚡ LumosAI TTFB: ${TTFB}ms ($(echo "scale=2; $TTFB/1000" | bc)秒)"
                fi
                CHUNK_COUNT=$((CHUNK_COUNT + 1))
            elif [ "$TYPE" == "done" ]; then
                break
            fi
        fi
    done
    
    END2=$(date +%s%N)
    TOTAL2=$(( (END2 - START2) / 1000000 ))
    echo "$TOTAL2" > /tmp/lumosai_total
    echo "$CHUNK_COUNT" > /tmp/lumosai_chunks
    
    echo "📊 Chunk数: $CHUNK_COUNT"
    echo "⏱️  总耗时: ${TOTAL2}ms"
}

echo ""

# ===== 结果分析 =====
echo "=========================================="
echo "📊 性能对比分析"
echo "=========================================="
echo ""

if [ -f /tmp/lumosai_ttfb ]; then
    LUMOSAI_TTFB=$(cat /tmp/lumosai_ttfb)
    LUMOSAI_TOTAL=$(cat /tmp/lumosai_total)
    LUMOSAI_CHUNKS=$(cat /tmp/lumosai_chunks)
    
    if [ "$COMPARE_MODE" == "full" ] && [ -f /tmp/direct_ttfb ]; then
        DIRECT_TTFB=$(cat /tmp/direct_ttfb)
        DIRECT_TOTAL=$(cat /tmp/direct_total)
        DIRECT_CHUNKS=$(cat /tmp/direct_chunks)
        
        echo "┌──────────────────────────────────────┐"
        echo "│         完整性能对比                  │"
        echo "├──────────────────────────────────────┤"
        printf "│ %-20s │ %-12s │\n" "指标" "直接API" "LumosAI"
        echo "├──────────────────────────────────────┤"
        printf "│ %-20s │ %8sms │ %8sms │\n" "TTFB" "$DIRECT_TTFB" "$LUMOSAI_TTFB"
        printf "│ %-20s │ %8sms │ %8sms │\n" "总耗时" "$DIRECT_TOTAL" "$LUMOSAI_TOTAL"
        printf "│ %-20s │ %10s │ %10s │\n" "Chunk数" "$DIRECT_CHUNKS" "$LUMOSAI_CHUNKS"
        echo "└──────────────────────────────────────┘"
        echo ""
        
        # 计算开销
        OVERHEAD=$(( LUMOSAI_TTFB - DIRECT_TTFB ))
        OVERHEAD_PCT=$(awk "BEGIN {printf \"%.1f\", ($OVERHEAD * 100.0 / $DIRECT_TTFB)}")
        
        echo "🎯 框架开销分析:"
        echo "  绝对开销: ${OVERHEAD}ms"
        echo "  相对开销: ${OVERHEAD_PCT}%"
        echo ""
        
        if [ "$OVERHEAD" -lt 0 ]; then
            echo "  ⚠️  负开销（可能是测量误差或缓存效应）"
        elif [ "$OVERHEAD" -lt 300 ]; then
            echo "  ✅ 开销极小 (<300ms)"
            echo "  框架性能优秀！"
        elif [ "$OVERHEAD" -lt 500 ]; then
            echo "  ✅ 开销可接受 (300-500ms)"
            echo "  在合理范围内"
        elif [ "$OVERHEAD" -lt 1000 ]; then
            echo "  ⚠️  开销偏大 (500-1000ms)"
            echo "  建议: 优化Memory retrieve和Agent初始化"
        else
            echo "  ❌ 开销过大 (>1000ms)"
            echo "  需要: 深度性能优化"
        fi
        
        echo ""
        echo "📋 开销来源推测:"
        echo "  1. HTTP路由 + 验证: ~10-20ms"
        echo "  2. Agent Factory: ~30-50ms"
        echo "  3. Memory retrieve: ~50-300ms ⚠️"
        echo "  4. Streaming包装: ~10-20ms"
        echo "  5. SSE转换: ~10-20ms"
        echo "  ────────────────────────────"
        echo "  预期总开销: ~110-410ms"
        echo ""
        
        if [ "$OVERHEAD" -gt 500 ]; then
            echo "⚠️  实际开销超出预期，可能原因:"
            echo "  - Memory retrieve耗时过长"
            echo "  - 数据库查询慢"
            echo "  - Agent初始化开销大"
            echo ""
            echo "建议查看详细trace:"
            echo "  tail -100 server-v4-traced.log | grep '⏱️'"
        fi
        
    else
        echo "LumosAI性能:"
        echo "  TTFB: ${LUMOSAI_TTFB}ms ($(echo "scale=2; $LUMOSAI_TTFB/1000" | bc)秒)"
        echo "  总耗时: ${LUMOSAI_TOTAL}ms"
        echo "  Chunk数: $LUMOSAI_CHUNKS"
        echo ""
        echo "ℹ️  未进行直接API对比（无API Key）"
        echo "   无法计算框架开销"
    fi
else
    echo "❌ 测试失败，请检查服务状态"
fi

# 清理
rm -f /tmp/direct_* /tmp/lumosai_*

echo ""
echo "=========================================="
echo "💡 优化建议"
echo "=========================================="
echo ""

if [ "$COMPARE_MODE" == "full" ] && [ -n "$OVERHEAD" ]; then
    if [ "$OVERHEAD" -lt 500 ]; then
        cat << 'EOF'
✅ 当前性能已经很好

建议:
1. 部署到生产环境
2. 监控P95/P99指标
3. 收集真实用户反馈

可选优化 (V4):
- Memory异步化 (-200ms)
- HTTP连接池 (-50ms)
EOF
    else
        cat << 'EOF'
⚠️  建议进行V4优化

优先级:
1. Memory异步化 (高) - 预计-200ms
2. 减少memory检索数量 (中)
3. 添加memory缓存 (中)
4. HTTP连接池 (低) - 预计-50ms

预期效果:
- 开销从XXXms降到200-300ms
- 更稳定的性能表现
EOF
    fi
else
    cat << 'EOF'
ℹ️  当前仅测试了LumosAI

要进行完整对比:
1. 设置 export ZHIPU_API_KEY='your-key'
2. 重新运行此脚本

当前建议:
- 基于历史数据，框架开销约200-400ms
- 如果TTFB稳定在3-5秒范围，性能良好
- 如果经常>10秒，主要是API波动问题
EOF
fi

echo ""
