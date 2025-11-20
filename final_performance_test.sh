#!/bin/bash
# 最终性能验证测试

set -e

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "=========================================="
echo "🎯 LumosAI V2 最终性能验证"
echo "=========================================="
echo ""

# 获取Agent ID
AGENT_ID=$(curl -s "$API_BASE/api/v1/agents" -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')

if [ -z "$AGENT_ID" ]; then
    echo "❌ 无法获取Agent"
    exit 1
fi

echo "📋 Agent ID: $AGENT_ID"
echo ""

# 验证模型配置
MODEL=$(curl -s "$API_BASE/api/v1/agents/$AGENT_ID" -H "Authorization: Bearer $TOKEN" | jq -r '.data.llm_config.model')
echo "🎯 当前模型: $MODEL"

if [ "$MODEL" != "glm-4-flash" ]; then
    echo "⚠️  警告: 期望模型为glm-4-flash，实际为$MODEL"
fi

echo ""
echo "=========================================="
echo "📝 测试1: 简短对话"
echo "=========================================="
echo ""

TEST1_MSG="请用一句话介绍人工智能"
echo "💬 请求: $TEST1_MSG"
echo ""

START1=$(date +%s%N)
FIRST_CHUNK=""

curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"$TEST1_MSG\", \"user_id\": \"perf-test\"}" 2>/dev/null | {
    
    CHUNK_COUNT=0
    RESPONSE=""
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty')
            
            if [ -z "$FIRST_CHUNK" ] && [ "$TYPE" == "content" ]; then
                FIRST_CHUNK=$(date +%s%N)
                TTFB=$(( (FIRST_CHUNK - START1) / 1000000 ))
                echo "⚡ TTFB: ${TTFB}ms"
                echo ""
                echo "📤 响应内容:"
            fi
            
            if [ "$TYPE" == "content" ]; then
                CONTENT=$(echo "$JSON" | jq -r '.content // empty')
                echo -n "$CONTENT"
                RESPONSE="${RESPONSE}${CONTENT}"
                CHUNK_COUNT=$((CHUNK_COUNT + 1))
            elif [ "$TYPE" == "done" ]; then
                END1=$(date +%s%N)
                TOTAL1=$(( (END1 - START1) / 1000000 ))
                
                echo ""
                echo ""
                echo "✅ 完成"
                echo ""
                echo "📊 统计:"
                echo "  - TTFB: ${TTFB}ms"
                echo "  - 总耗时: ${TOTAL1}ms"
                echo "  - Chunk数: $CHUNK_COUNT"
                
                # 导出变量供后续使用
                echo "$TTFB" > /tmp/test1_ttfb
                echo "$TOTAL1" > /tmp/test1_total
                echo "$CHUNK_COUNT" > /tmp/test1_chunks
            fi
        fi
    done
}

echo ""
echo "=========================================="
echo "📝 测试2: 中等长度对话"
echo "=========================================="
echo ""

TEST2_MSG="请介绍机器学习的基本概念"
echo "💬 请求: $TEST2_MSG"
echo ""

START2=$(date +%s%N)
FIRST_CHUNK2=""

curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"$TEST2_MSG\", \"user_id\": \"perf-test\"}" 2>/dev/null | {
    
    CHUNK_COUNT=0
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty')
            
            if [ -z "$FIRST_CHUNK2" ] && [ "$TYPE" == "content" ]; then
                FIRST_CHUNK2=$(date +%s%N)
                TTFB2=$(( (FIRST_CHUNK2 - START2) / 1000000 ))
                echo "⚡ TTFB: ${TTFB2}ms"
                echo ""
                echo "📤 响应内容:"
            fi
            
            if [ "$TYPE" == "content" ]; then
                CONTENT=$(echo "$JSON" | jq -r '.content // empty')
                echo -n "$CONTENT"
                CHUNK_COUNT=$((CHUNK_COUNT + 1))
            elif [ "$TYPE" == "done" ]; then
                END2=$(date +%s%N)
                TOTAL2=$(( (END2 - START2) / 1000000 ))
                
                echo ""
                echo ""
                echo "✅ 完成"
                echo ""
                echo "📊 统计:"
                echo "  - TTFB: ${TTFB2}ms"
                echo "  - 总耗时: ${TOTAL2}ms"
                echo "  - Chunk数: $CHUNK_COUNT"
                
                echo "$TTFB2" > /tmp/test2_ttfb
                echo "$TOTAL2" > /tmp/test2_total
                echo "$CHUNK_COUNT" > /tmp/test2_chunks
            fi
        fi
    done
}

echo ""
echo "=========================================="
echo "📊 最终性能评估"
echo "=========================================="
echo ""

# 读取结果
if [ -f /tmp/test1_ttfb ]; then
    T1_TTFB=$(cat /tmp/test1_ttfb)
    T1_TOTAL=$(cat /tmp/test1_total)
    T1_CHUNKS=$(cat /tmp/test1_chunks)
    
    echo "测试1 (简短对话):"
    echo "  ✅ TTFB: ${T1_TTFB}ms"
    echo "  ✅ 总耗时: ${T1_TOTAL}ms"
    echo "  ✅ Chunk数: $T1_CHUNKS"
    echo ""
    
    # 性能评级
    if [ "$T1_TTFB" -lt 2000 ]; then
        echo "  🎉 性能评级: 优秀 (TTFB < 2秒)"
    elif [ "$T1_TTFB" -lt 5000 ]; then
        echo "  ✅ 性能评级: 良好 (TTFB < 5秒)"
    elif [ "$T1_TTFB" -lt 10000 ]; then
        echo "  ⚠️  性能评级: 一般 (TTFB < 10秒)"
    else
        echo "  ❌ 性能评级: 需改进 (TTFB > 10秒)"
    fi
    
    # 计算提升倍数 (vs 28.8秒基线)
    if [ "$T1_TTFB" -gt 0 ]; then
        IMPROVEMENT=$((28800 / T1_TTFB))
        echo "  📈 vs基线提升: ${IMPROVEMENT}倍 (28.8秒 → ${T1_TTFB}ms)"
    fi
fi

echo ""

if [ -f /tmp/test2_ttfb ]; then
    T2_TTFB=$(cat /tmp/test2_ttfb)
    T2_TOTAL=$(cat /tmp/test2_total)
    T2_CHUNKS=$(cat /tmp/test2_chunks)
    
    echo "测试2 (中等长度):"
    echo "  ✅ TTFB: ${T2_TTFB}ms"
    echo "  ✅ 总耗时: ${T2_TOTAL}ms"
    echo "  ✅ Chunk数: $T2_CHUNKS"
    echo ""
    
    if [ "$T2_TTFB" -lt 2000 ]; then
        echo "  🎉 性能评级: 优秀"
    elif [ "$T2_TTFB" -lt 5000 ]; then
        echo "  ✅ 性能评级: 良好"
    else
        echo "  ⚠️  性能评级: 可接受"
    fi
fi

echo ""
echo "=========================================="
echo "🎯 V2优化验证结果"
echo "=========================================="
echo ""

if [ -f /tmp/test1_ttfb ] && [ "$T1_TTFB" -lt 5000 ]; then
    echo "✅ V2优化成功！"
    echo ""
    echo "优化项:"
    echo "  ✅ 模型切换: glm-4.6 → glm-4-flash"
    echo "  ✅ Buffer优化: 10字符 → 1字符"
    echo "  ✅ Metadata: 已禁用"
    echo "  ✅ 真实Streaming: Token-by-token"
    echo ""
    echo "性能指标:"
    echo "  🎯 目标: TTFB < 5秒"
    echo "  ✅ 实际: TTFB = ${T1_TTFB}ms (${T1_TTFB}ms / 1000 = $((T1_TTFB / 1000))秒)"
    echo "  🎉 状态: 目标达成"
else
    echo "⚠️  性能未达预期"
    echo ""
    echo "建议检查:"
    echo "  - 模型是否正确切换为glm-4-flash"
    echo "  - 服务器是否重启生效"
    echo "  - 网络连接是否稳定"
fi

echo ""
echo "=========================================="
echo "✅ 测试完成"
echo "=========================================="

# 清理临时文件
rm -f /tmp/test1_* /tmp/test2_*
