#!/bin/bash
# V2性能测试 - 验证glm-4-flash + 优化配置

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "=========================================="
echo "🚀 V2性能测试 (glm-4-flash + 优化)"
echo "=========================================="
echo ""

# 获取agent
AGENT_ID=$(curl -s -X GET "$API_BASE/api/v1/agents" \
  -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
  echo "❌ 没有找到Agent"
  exit 1
fi

echo "✅ Agent ID: $AGENT_ID"
echo ""

# 测试消息
TEST_MESSAGE="请用一句话介绍人工智能"

echo "🎯 测试配置:"
echo "  模型: glm-4-flash"
echo "  Buffer Size: 1字符"
echo "  Emit Metadata: false"
echo ""
echo "📤 发送请求: $TEST_MESSAGE"
echo ""

# 记录开始时间（纳秒）
START_NS=$(date +%s%N)
FIRST_CHUNK_NS=""
CHUNK_COUNT=0

echo "--- Streaming响应 ---"
echo ""

# 使用curl接收SSE
curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"$TEST_MESSAGE\", \"user_id\": \"test-user\"}" \
  2>/dev/null | while IFS= read -r line; do
    
    if [[ $line == data:* ]]; then
      JSON_DATA="${line#data: }"
      CHUNK_TYPE=$(echo "$JSON_DATA" | jq -r '.chunk_type // empty')
      
      # 记录首chunk时间
      if [ -z "$FIRST_CHUNK_NS" ] && [ "$CHUNK_TYPE" == "content" ]; then
        FIRST_CHUNK_NS=$(date +%s%N)
        TTFB_MS=$(( (FIRST_CHUNK_NS - START_NS) / 1000000 ))
        echo ""
        echo "⚡ TTFB: ${TTFB_MS}ms"
        echo ""
        
        # 判断是否达标
        if [ "$TTFB_MS" -lt 5000 ]; then
          echo "✅ TTFB < 5秒，性能达标！"
        elif [ "$TTFB_MS" -lt 10000 ]; then
          echo "⚠️  TTFB < 10秒，还有优化空间"
        else
          echo "❌ TTFB > 10秒，需要进一步优化"
        fi
        echo ""
      fi
      
      case "$CHUNK_TYPE" in
        content)
          CONTENT=$(echo "$JSON_DATA" | jq -r '.content // empty')
          echo -n "$CONTENT"
          CHUNK_COUNT=$((CHUNK_COUNT + 1))
          ;;
        done)
          END_NS=$(date +%s%N)
          TOTAL_MS=$(( (END_NS - START_NS) / 1000000 ))
          echo ""
          echo ""
          echo "✅ 完成"
          echo ""
          echo "📊 性能统计:"
          echo "  - TTFB: ${TTFB_MS}ms"
          echo "  - 总耗时: ${TOTAL_MS}ms"
          echo "  - Chunk数: $CHUNK_COUNT"
          
          if [ -n "$FIRST_CHUNK_NS" ]; then
            STREAM_DURATION=$(( (END_NS - FIRST_CHUNK_NS) / 1000000 ))
            if [ "$CHUNK_COUNT" -gt 0 ]; then
              AVG_CHUNK_MS=$(( STREAM_DURATION / CHUNK_COUNT ))
              echo "  - 平均chunk间隔: ${AVG_CHUNK_MS}ms"
            fi
          fi
          ;;
        error)
          ERROR=$(echo "$JSON_DATA" | jq -r '.content // empty')
          echo ""
          echo "❌ 错误: $ERROR"
          ;;
      esac
    fi
done

echo ""
echo "--- 响应结束 ---"
echo ""

# 性能评估
if [ -n "$FIRST_CHUNK_NS" ]; then
  TTFB_MS=$(( (FIRST_CHUNK_NS - START_NS) / 1000000 ))
  
  echo "=========================================="
  echo "📊 性能评估"
  echo "=========================================="
  echo ""
  
  if [ "$TTFB_MS" -lt 2000 ]; then
    echo "🎉 优秀！TTFB ${TTFB_MS}ms < 2秒"
    echo "   性能提升: $(( 28800 / TTFB_MS ))倍 (vs 28.8秒基线)"
  elif [ "$TTFB_MS" -lt 5000 ]; then
    echo "✅ 良好！TTFB ${TTFB_MS}ms < 5秒"
    echo "   性能提升: $(( 28800 / TTFB_MS ))倍 (vs 28.8秒基线)"
  elif [ "$TTFB_MS" -lt 10000 ]; then
    echo "⚠️  一般。TTFB ${TTFB_MS}ms < 10秒"
    echo "   性能提升: $(( 28800 / TTFB_MS ))倍"
  else
    echo "❌ 未改善。TTFB ${TTFB_MS}ms > 10秒"
    echo "   可能原因: 模型未生效或网络问题"
  fi
  
  echo ""
fi

echo "🎯 V2优化项:"
echo "  ✅ 模型: glm-4.6 → glm-4-flash"
echo "  ✅ Buffer: 10字符 → 1字符"
echo "  ✅ Metadata: 已禁用"
echo ""
