#!/bin/bash
# 仅测试LumosAI性能（无需API Key）

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "=========================================="
echo "🧪 LumosAI性能测试（带详细trace）"
echo "=========================================="
echo ""

# 获取Agent ID
AGENT_ID=$(curl -s "$API_BASE/api/v1/agents" -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')

if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" == "null" ]; then
    echo "❌ 无法获取Agent ID"
    exit 1
fi

echo "Agent: $AGENT_ID"
echo "测试消息: 什么是机器学习？"
echo ""

# 标记日志
echo "=== BENCHMARK-START $(date +%H:%M:%S.%N) ===" >> server-v4-traced.log

# 记录开始时间
START=$(date +%s%N)
echo "⏱️  [0ms] 发送请求"

# 发送请求并测量
curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message": "什么是机器学习？", "user_id": "benchmark"}' 2>/dev/null | {
    
    FIRST_CONTENT=""
    CHUNK_COUNT=0
    RESPONSE=""
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$TYPE" == "content" ]; then
                if [ -z "$FIRST_CONTENT" ]; then
                    FIRST_CONTENT="1"
                    NOW=$(date +%s%N)
                    TTFB=$(( (NOW - START) / 1000000 ))
                    
                    echo ""
                    echo "⚡ TTFB: ${TTFB}ms ($(echo "scale=2; $TTFB/1000" | bc)秒)"
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
                    echo "📤 响应内容:"
                fi
                
                CONTENT=$(echo "$JSON" | jq -r '.content // empty' 2>/dev/null)
                if [ -n "$CONTENT" ]; then
                    echo -n "$CONTENT"
                    RESPONSE="${RESPONSE}${CONTENT}"
                    CHUNK_COUNT=$((CHUNK_COUNT + 1))
                fi
                
            elif [ "$TYPE" == "done" ]; then
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
                echo "  - 响应长度: ${#RESPONSE}字符"
                
                if [ "$CHUNK_COUNT" -gt 0 ]; then
                    AVG=$(( TOTAL / CHUNK_COUNT ))
                    echo "  - 平均chunk间隔: ${AVG}ms"
                fi
                
                break
            fi
        fi
    done
}

echo ""
echo "=========================================="
echo "📊 详细时间分解（从服务器日志）"
echo "=========================================="
echo ""

# 分析服务器日志
echo "提取trace时间点："
tail -200 server-v4-traced.log | grep -A 100 "BENCHMARK-START" | grep "⏱️" | head -20

echo ""
echo "=========================================="
echo "🎯 框架开销分析"
echo "=========================================="
echo ""

cat << 'EOF'
预期时间分布:
┌─────────────────────────────────────────┐
│ 0-20ms:    HTTP路由 + 验证               │
│ 20-50ms:   Agent Factory创建            │
│ 50-300ms:  Memory retrieve ← 最大开销   │
│ 300-320ms: Streaming初始化               │
│ 320ms+:    等待LLM API首token           │
└─────────────────────────────────────────┘

如果TTFB明显高于预期:
1. 查看Memory retrieve耗时
2. 检查数据库查询性能
3. 检查LLM API网络延迟

优化建议:
- Memory retrieve > 200ms → 异步化
- Agent Factory > 50ms → 缓存优化
- 总开销 > 500ms → 深度优化
EOF

echo ""
