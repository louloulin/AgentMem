#!/bin/bash
# 真实Streaming性能测试脚本
# 对比真实streaming vs legacy模式的性能

set -e

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "=========================================="
echo "🚀 LumosAI 真实Streaming性能测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 1. 获取Agent
echo -e "${BLUE}📋 步骤 1: 获取测试Agent${NC}"
AGENT_RESPONSE=$(curl -s -X GET "$API_BASE/api/v1/agents" \
  -H "Authorization: Bearer $TOKEN")

AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
  echo -e "${RED}❌ 没有找到Agent，请先创建${NC}"
  exit 1
fi

echo -e "${GREEN}✅ Agent ID: $AGENT_ID${NC}"
echo ""

# 2. 准备测试消息
TEST_MESSAGE="请详细介绍一下人工智能的发展历史，包括重要的里程碑事件。"

echo "=========================================="
echo -e "${YELLOW}🌊 测试 1: 真实Streaming模式 (StreamingAgent)${NC}"
echo "=========================================="
echo ""
echo -e "📤 发送请求到: ${BLUE}/api/v1/agents/$AGENT_ID/chat/lumosai/stream${NC}"
echo -e "💬 消息: $TEST_MESSAGE"
echo ""

# 记录开始时间
START_TIME=$(date +%s%N)
FIRST_CHUNK_TIME=""
CHUNK_COUNT=0

echo -e "${GREEN}--- Streaming响应开始 ---${NC}"
echo ""

# 使用curl接收SSE流
curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"$TEST_MESSAGE\",
    \"user_id\": \"test-user\"
  }" 2>/dev/null | while IFS= read -r line; do
    # 解析SSE数据
    if [[ $line == data:* ]]; then
      JSON_DATA="${line#data: }"
      CHUNK_TYPE=$(echo "$JSON_DATA" | jq -r '.chunk_type // empty')
      
      # 记录首字节时间(TTFB)
      if [ -z "$FIRST_CHUNK_TIME" ] && [ "$CHUNK_TYPE" == "content" ]; then
        FIRST_CHUNK_TIME=$(date +%s%N)
        TTFB=$(( (FIRST_CHUNK_TIME - START_TIME) / 1000000 ))
        echo -e "${YELLOW}⚡ TTFB (首字节时间): ${TTFB}ms${NC}"
        echo ""
      fi
      
      case "$CHUNK_TYPE" in
        start)
          AGENT_EVENT_ID=$(echo "$JSON_DATA" | jq -r '.agent_id // empty')
          echo -e "${BLUE}🎬 [开始] Agent $AGENT_EVENT_ID 开始响应${NC}"
          ;;
        content)
          CONTENT=$(echo "$JSON_DATA" | jq -r '.content // empty')
          echo -n "$CONTENT"
          CHUNK_COUNT=$((CHUNK_COUNT + 1))
          ;;
        tool_call_start)
          TOOL_NAME=$(echo "$JSON_DATA" | jq -r '.tool_name // empty')
          echo ""
          echo -e "${YELLOW}🔧 [工具调用] $TOOL_NAME${NC}"
          ;;
        tool_call_complete)
          echo -e "${GREEN}✅ [工具完成]${NC}"
          ;;
        step_complete)
          STEP_TYPE=$(echo "$JSON_DATA" | jq -r '.step_type // empty')
          echo ""
          echo -e "${BLUE}✨ [步骤完成] $STEP_TYPE${NC}"
          ;;
        done)
          END_TIME=$(date +%s%N)
          TOTAL_TIME=$(( (END_TIME - START_TIME) / 1000000 ))
          TOTAL_STEPS=$(echo "$JSON_DATA" | jq -r '.total_steps // 0')
          ELAPSED_MS=$(echo "$JSON_DATA" | jq -r '.elapsed_ms // 0')
          
          echo ""
          echo ""
          echo -e "${GREEN}✅ [完成] 生成完成${NC}"
          echo -e "   ${BLUE}📊 统计信息:${NC}"
          echo -e "      - 总步骤: $TOTAL_STEPS"
          echo -e "      - 接收到的chunk数: $CHUNK_COUNT"
          echo -e "      - 总耗时: ${TOTAL_TIME}ms"
          echo -e "      - 服务器报告耗时: ${ELAPSED_MS}ms"
          if [ -n "$FIRST_CHUNK_TIME" ]; then
            echo -e "      - TTFB (首字节): ${TTFB}ms"
          fi
          ;;
        error)
          ERROR_MSG=$(echo "$JSON_DATA" | jq -r '.content // empty')
          echo ""
          echo -e "${RED}❌ [错误] $ERROR_MSG${NC}"
          ;;
        metadata)
          KEY=$(echo "$JSON_DATA" | jq -r '.key // empty')
          VALUE=$(echo "$JSON_DATA" | jq -r '.value // empty')
          echo -e "${BLUE}ℹ️  [元数据] $KEY: $VALUE${NC}"
          ;;
      esac
    fi
done

echo ""
echo -e "${GREEN}--- Streaming响应结束 ---${NC}"
echo ""

# 3. 对比测试：非流式端点
echo "=========================================="
echo -e "${YELLOW}📦 测试 2: 非Streaming模式 (对比)${NC}"
echo "=========================================="
echo ""

NON_STREAM_START=$(date +%s%N)

RESPONSE=$(curl -s -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"请简单介绍一下机器学习\",
    \"user_id\": \"test-user\"
  }")

NON_STREAM_END=$(date +%s%N)
NON_STREAM_DURATION=$(( (NON_STREAM_END - NON_STREAM_START) / 1000000 ))

echo -e "💬 ${BLUE}问题:${NC} 请简单介绍一下机器学习"
CONTENT=$(echo "$RESPONSE" | jq -r '.data.content // "无回复"' | head -c 200)
echo -e "🤖 ${BLUE}回复:${NC} ${CONTENT}..."
echo -e "⏱️  ${BLUE}响应时间:${NC} ${NON_STREAM_DURATION}ms"
echo -e "   ${YELLOW}(注意: 非streaming模式的TTFB = 总时间)${NC}"
echo ""

# 4. 性能总结
echo "=========================================="
echo -e "${GREEN}📊 性能对比总结${NC}"
echo "=========================================="
echo ""

if [ -n "$FIRST_CHUNK_TIME" ]; then
  echo -e "${BLUE}真实Streaming模式:${NC}"
  echo -e "  ⚡ TTFB: ${TTFB}ms ${GREEN}(首字节到达时间)${NC}"
  echo -e "  📦 Chunk数: $CHUNK_COUNT"
  echo -e "  ⏱️  总耗时: ${TOTAL_TIME}ms"
  echo ""
  echo -e "${BLUE}非Streaming模式:${NC}"
  echo -e "  ⏱️  响应时间: ${NON_STREAM_DURATION}ms ${YELLOW}(完整生成后才返回)${NC}"
  echo ""
  
  if [ "$TTFB" -lt 5000 ]; then
    IMPROVEMENT=$((NON_STREAM_DURATION / TTFB))
    echo -e "${GREEN}✅ 性能提升: TTFB降低了约 ${IMPROVEMENT}倍!${NC}"
    echo -e "   ${YELLOW}用户体验从等待${NON_STREAM_DURATION}ms到${TTFB}ms就能看到首个响应${NC}"
  fi
else
  echo -e "${RED}⚠️  未能测量到TTFB，可能streaming配置有问题${NC}"
fi

echo ""
echo "=========================================="
echo -e "${GREEN}🎉 性能测试完成！${NC}"
echo "=========================================="
echo ""

# 5. 日志分析提示
echo -e "${BLUE}💡 查看详细日志:${NC}"
echo -e "   tail -f backend-streaming.log | grep -E 'REAL-STREAMING|SSE|Token|TTFB'"
echo ""
echo -e "${BLUE}🔍 分析streaming事件:${NC}"
echo -e "   tail -f backend-streaming.log | grep 'AgentEvent'"
echo ""
