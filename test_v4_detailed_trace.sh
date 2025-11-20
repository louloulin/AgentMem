#!/bin/bash
# V4详细Trace测试

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "=========================================="
echo "🔬 V4详细Trace测试"
echo "=========================================="
echo ""

AGENT_ID=$(curl -s "$API_BASE/api/v1/agents" -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')

echo "Agent: $AGENT_ID"
echo "测试: 什么是AI？"
echo ""

# 标记日志
echo "=== V4-TEST-START $(date +%H:%M:%S.%N) ===" >> server-v4-traced.log

START=$(date +%s%N)

curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message": "什么是AI？", "user_id": "v4-test"}' 2>/dev/null | {
    
    FIRST=""
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$TYPE" == "content" ] && [ -z "$FIRST" ]; then
                FIRST="1"
                NOW=$(date +%s%N)
                TTFB=$(( (NOW - START) / 1000000 ))
                echo "⚡ TTFB: ${TTFB}ms"
                break
            fi
        fi
    done
}

echo ""
echo "📊 分析trace日志："
echo ""

tail -200 server-v4-traced.log | grep -A 100 "V4-TEST-START" | grep -E "\[REAL-STREAMING\]|\[EXECUTOR\]|\[+[0-9]+ms\]|Zhipu API|HTTP 响应|Retrieved.*memories" | head -30

echo ""
echo "=========================================="
echo "🎯 时间分布分析"
echo "=========================================="
