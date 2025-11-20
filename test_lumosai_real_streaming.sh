#!/bin/bash
# LumosAI Real Streaming 真实测试脚本
# 测试backend的真实streaming端点

set -e

API_BASE="http://localhost:8080"
TOKEN="test-token"

echo "╔════════════════════════════════════════════════════════╗"
echo "║  🚀 LumosAI Real Streaming Test                       ║"
echo "║  Backend真实SSE测试                                   ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查服务器是否运行
echo "📡 检查服务器状态..."
if ! curl -s "$API_BASE/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ 服务器未运行！请先启动服务器${NC}"
    echo ""
    echo "启动命令:"
    echo "  cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen"
    echo "  ./start_server_no_auth.sh"
    exit 1
fi
echo -e "${GREEN}✅ 服务器运行正常${NC}"
echo ""

# 1. 获取或创建Agent
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 步骤 1: 获取Agent"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

AGENT_RESPONSE=$(curl -s -X GET "$API_BASE/api/v1/agents" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
  echo -e "${YELLOW}⚠️  没有找到Agent，创建新的...${NC}"
  AGENT_RESPONSE=$(curl -s -X POST "$API_BASE/api/v1/agents" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "name": "Streaming测试Agent",
      "llm_config": {
        "provider": "zhipu",
        "model": "glm-4-flash",
        "temperature": 0.7
      },
      "system": "你是一个AI助手，请用简短的语言回答问题。"
    }')
  
  AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id // empty')
  
  if [ -z "$AGENT_ID" ]; then
    echo -e "${RED}❌ 创建Agent失败${NC}"
    echo "响应: $AGENT_RESPONSE"
    exit 1
  fi
  echo -e "${GREEN}✅ Agent已创建${NC}"
else
  echo -e "${GREEN}✅ 找到已存在的Agent${NC}"
fi

echo "   Agent ID: $AGENT_ID"
echo ""

# 2. 测试非流式端点（对比）
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 步骤 2: 测试非流式端点（对比基准）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "💬 发送消息: 1+1等于几？"
echo ""

START_TIME=$(date +%s%N)

NON_STREAM_RESPONSE=$(curl -s -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "1+1等于几？请简短回答",
    "user_id": "test-user"
  }')

END_TIME=$(date +%s%N)
NON_STREAM_DURATION=$(( (END_TIME - START_TIME) / 1000000 ))

CONTENT=$(echo "$NON_STREAM_RESPONSE" | jq -r '.data.content // "无响应"')
echo "🤖 回复: $CONTENT"
echo "⏱️  耗时: ${NON_STREAM_DURATION}ms"
echo ""

# 3. 测试流式端点
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌊 步骤 3: 测试LumosAI流式端点"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "📤 发送流式请求到: /api/v1/agents/$AGENT_ID/chat/lumosai/stream"
echo "💬 消息: 你好，请用一句话介绍自己"
echo ""
echo "--- 流式响应开始 ---"
echo ""

STREAM_START=$(date +%s%N)
EVENT_COUNT=0
DELTA_COUNT=0
CHAR_COUNT=0
HAD_START=0
HAD_COMPLETE=0

echo -n "🤖 "

# 使用curl进行SSE streaming
curl -N -s -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好，请用一句话介绍自己",
    "user_id": "test-user"
  }' 2>/dev/null | while IFS= read -r line; do
  
  # 解析SSE数据
  if [[ $line == data:* ]]; then
    JSON_DATA="${line#data: }"
    CHUNK_TYPE=$(echo "$JSON_DATA" | jq -r '.chunk_type // empty' 2>/dev/null)
    
    EVENT_COUNT=$((EVENT_COUNT + 1))
    
    case "$CHUNK_TYPE" in
      start)
        echo ""
        echo -e "${GREEN}🚀 [启动] Agent开始响应${NC}"
        echo -n "🤖 "
        HAD_START=1
        ;;
      content)
        CONTENT=$(echo "$JSON_DATA" | jq -r '.content // empty' 2>/dev/null)
        if [ -n "$CONTENT" ]; then
          echo -n "$CONTENT"
          DELTA_COUNT=$((DELTA_COUNT + 1))
          CHAR_COUNT=$((CHAR_COUNT + ${#CONTENT}))
        fi
        ;;
      tool_call)
        TOOL_NAME=$(echo "$JSON_DATA" | jq -r '.tool_name // empty' 2>/dev/null)
        echo ""
        echo -e "${BLUE}🔧 [工具调用] $TOOL_NAME${NC}"
        ;;
      done)
        echo ""
        echo ""
        TOTAL_STEPS=$(echo "$JSON_DATA" | jq -r '.total_steps // 0' 2>/dev/null)
        MEMORIES_UPDATED=$(echo "$JSON_DATA" | jq -r '.memories_updated // false' 2>/dev/null)
        echo -e "${GREEN}✅ [完成] 生成完成${NC}"
        echo "   - 总步骤: $TOTAL_STEPS"
        echo "   - 记忆更新: $MEMORIES_UPDATED"
        HAD_COMPLETE=1
        ;;
      error)
        ERROR_MSG=$(echo "$JSON_DATA" | jq -r '.content // empty' 2>/dev/null)
        echo ""
        echo -e "${RED}❌ [错误] $ERROR_MSG${NC}"
        ;;
      metadata)
        # 静默处理元数据
        ;;
      *)
        if [ -n "$CHUNK_TYPE" ]; then
          echo ""
          echo "📊 [其他] 类型: $CHUNK_TYPE"
        fi
        ;;
    esac
  fi
done

STREAM_END=$(date +%s%N)
STREAM_DURATION=$(( (STREAM_END - STREAM_START) / 1000000 ))

echo ""
echo "--- 流式响应结束 ---"
echo ""

# 4. 统计和验证
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 测试统计"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "   ⏱️  流式总耗时: ${STREAM_DURATION}ms"
echo "   📝 输出字符数: $CHAR_COUNT"
echo "   🔢 总事件数: $EVENT_COUNT"
echo "   ✨ TextDelta事件: $DELTA_COUNT"
echo ""
echo "   📊 对比非流式:"
echo "   - 非流式耗时: ${NON_STREAM_DURATION}ms"
echo "   - 流式耗时: ${STREAM_DURATION}ms"
if [ $STREAM_DURATION -gt 0 ] && [ $NON_STREAM_DURATION -gt 0 ]; then
  RATIO=$(( NON_STREAM_DURATION * 100 / STREAM_DURATION ))
  echo "   - 性能比: ${RATIO}%"
fi
echo ""

# 5. 验证结果
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 验证结果"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

PASSED=0
TOTAL=4

# 测试1: 是否收到TextDelta事件
if [ $DELTA_COUNT -gt 0 ]; then
  echo -e "   ${GREEN}✅ 测试1: 收到TextDelta事件 ($DELTA_COUNT 个)${NC}"
  PASSED=$((PASSED + 1))
else
  echo -e "   ${RED}❌ 测试1: 未收到TextDelta事件${NC}"
fi

# 测试2: 是否输出了文本
if [ $CHAR_COUNT -gt 0 ]; then
  echo -e "   ${GREEN}✅ 测试2: 输出了文本 ($CHAR_COUNT 字符)${NC}"
  PASSED=$((PASSED + 1))
else
  echo -e "   ${RED}❌ 测试2: 未输出文本${NC}"
fi

# 测试3: 是否收到开始和完成事件
if [ $HAD_START -eq 1 ] && [ $HAD_COMPLETE -eq 1 ]; then
  echo -e "   ${GREEN}✅ 测试3: 收到开始和完成事件${NC}"
  PASSED=$((PASSED + 1))
else
  echo -e "   ${RED}❌ 测试3: 缺少开始或完成事件${NC}"
fi

# 测试4: 是否在合理时间内完成
if [ $STREAM_DURATION -lt 60000 ]; then
  echo -e "   ${GREEN}✅ 测试4: 在合理时间内完成 (${STREAM_DURATION}ms)${NC}"
  PASSED=$((PASSED + 1))
else
  echo -e "   ${RED}❌ 测试4: 耗时过长 (${STREAM_DURATION}ms)${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $PASSED -eq $TOTAL ]; then
  echo -e "${GREEN}🎉 测试结果: $PASSED/$TOTAL 通过${NC}"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo -e "${GREEN}   ✅ 恭喜！LumosAI Streaming功能正常工作！${NC}"
  echo ""
  echo "💡 后续步骤:"
  echo "   1. 可以在前端UI中测试: http://localhost:3001/admin/chat"
  echo "   2. 启用LumosAI模式和Streaming开关"
  echo "   3. 观察实时流式响应效果"
  echo ""
  exit 0
else
  echo -e "${YELLOW}⚠️  测试结果: $PASSED/$TOTAL 通过${NC}"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo -e "${YELLOW}   部分测试失败，需要检查实现${NC}"
  echo ""
  echo "🔍 调试建议:"
  echo "   1. 检查后端日志: tail -f backend-no-auth.log"
  echo "   2. 验证SSE格式是否正确"
  echo "   3. 确认streaming endpoint是否正确注册"
  echo ""
  exit 1
fi
