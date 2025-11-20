#!/bin/bash

# 测试Memory优化效果
# 通过直接调用API并计算TTFB来验证性能

AGENT_ID=$(curl -s http://localhost:8080/api/v1/agents -H "Authorization: Bearer test-token" | jq -r '.data[0].id')

echo "=== Memory优化性能测试 ==="
echo "Agent ID: $AGENT_ID"
echo "配置: 检索1条历史消息（已修改memory_adapter.rs）"
echo ""

# 测试1: 测量TTFB
echo "📊 测试1: 首Token时间（TTFB）"
echo "---"

START_TIME=$(ruby -e 'puts (Time.now.to_f * 1000).to_i')

curl -N -X POST "http://localhost:8080/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer test-token" \
  -H "Content-Type: application/json" \
  -d '{"message":"什么是卷积神经网络？","user_id":"opt-test"}' \
  2>&1 | while IFS= read -r line; do
    if [[ "$line" == data:*content* ]]; then
      END_TIME=$(ruby -e 'puts (Time.now.to_f * 1000).to_i')
      TTFB=$((END_TIME - START_TIME))
      echo ""
      echo "⚡ TTFB: ${TTFB}ms ($(echo "scale=2; $TTFB/1000" | bc)秒)"
      
      if [ "$TTFB" -lt 3000 ]; then
        echo "✅ 性能优秀: TTFB < 3秒"
      elif [ "$TTFB" -lt 5000 ]; then
        echo "⚠️  性能一般: 3秒 < TTFB < 5秒"
      else
        echo "❌ 性能较差: TTFB > 5秒"
      fi
      
      break
    fi
  done

echo ""
echo "---"
echo "📋 查看日志中的Memory检索信息..."
sleep 2
tail -50 logs/server-current.log | grep -E "MEMORY-RETRIEVE|Returned.*messages" | tail -3

echo ""
echo "=== 测试完成 ==="

