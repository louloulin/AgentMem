#!/bin/bash

# ====================================
# 记忆冲突解决方案验证测试
# 测试时间衰减、用户隔离、session优先级
# ====================================

set -e

BASE_URL="http://localhost:8080"
API_KEY="${AGENTMEM_API_KEY:-test-api-key-12345}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${PURPLE}╔════════════════════════════════════════════╗
║  记忆冲突解决方案 - 综合验证测试        ║
╚════════════════════════════════════════════╝${NC}\n"

# 使用现有的agent
AGENT_ID="agent-7bd801e2-c8da-42e4-b10f-c2ef7f610235"
SESSION_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')

echo -e "${BLUE}测试配置：${NC}"
echo "  Agent ID: $AGENT_ID"
echo "  Session ID: $SESSION_ID"
echo ""

echo -e "${YELLOW}═══════════════════════════════════════════${NC}"
echo -e "${YELLOW}测试1: Session隔离 - 用户身份识别${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════${NC}\n"

echo -e "${GREEN}[Step 1] 用户自我介绍: '我是Alex'${NC}"
RESP1=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"我是Alex\",
    \"session_id\": \"$SESSION_ID\"
  }")

echo "$RESP1" | jq -r '.content' | head -10
echo ""

sleep 2

echo -e "${GREEN}[Step 2] 测试Agent是否记得: '你还记得我是谁吗？'${NC}"
RESP2=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"你还记得我是谁吗？\",
    \"session_id\": \"$SESSION_ID\"
  }")

CONTENT2=$(echo "$RESP2" | jq -r '.content')
echo "$CONTENT2" | head -15
echo ""

# 验证结果
echo -e "${YELLOW}【验证结果 - Session隔离】${NC}"
if echo "$CONTENT2" | grep -qi "Alex"; then
    echo -e "  ${GREEN}✅ PASS: Agent正确记得当前session的用户名 (Alex)${NC}"
    TEST1_PASS=1
else
    echo -e "  ${RED}❌ FAIL: Agent没有记住当前用户名${NC}"
    TEST1_PASS=0
fi

if echo "$CONTENT2" | grep -qi "张三\|lin\|吕洁"; then
    echo -e "  ${RED}❌ FAIL: Agent混入了历史记忆 (张三/lin/吕洁)${NC}"
    TEST1_PASS=0
else
    echo -e "  ${GREEN}✅ PASS: Agent没有混入其他session的历史记忆${NC}"
fi

echo ""
echo -e "${YELLOW}═══════════════════════════════════════════${NC}"
echo -e "${YELLOW}测试2: 多轮对话上下文保持${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════${NC}\n"

echo -e "${GREEN}[Step 3] 提供个人信息: '我喜欢编程和AI'${NC}"
RESP3=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"我喜欢编程和AI\",
    \"session_id\": \"$SESSION_ID\"
  }")

echo "$RESP3" | jq -r '.content' | head -10
echo ""

sleep 2

echo -e "${GREEN}[Step 4] 测试上下文: '根据你对我的了解，推荐一个适合我的项目'${NC}"
RESP4=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"根据你对我的了解，推荐一个适合我的项目\",
    \"session_id\": \"$SESSION_ID\"
  }")

CONTENT4=$(echo "$RESP4" | jq -r '.content')
echo "$CONTENT4" | head -20
echo ""

# 验证上下文
echo -e "${YELLOW}【验证结果 - 上下文保持】${NC}"
if echo "$CONTENT4" | grep -Eqi "(编程|AI|Alex|人工智能|代码)"; then
    echo -e "  ${GREEN}✅ PASS: Agent使用了当前session的上下文信息${NC}"
    TEST2_PASS=1
else
    echo -e "  ${RED}❌ FAIL: Agent没有使用当前session的上下文${NC}"
    TEST2_PASS=0
fi

echo ""
echo -e "${YELLOW}═══════════════════════════════════════════${NC}"
echo -e "${YELLOW}测试3: 检查Working Memory${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════${NC}\n"

WM_DATA=$(curl -s "$BASE_URL/api/v1/agents/$AGENT_ID/working-memory/$SESSION_ID" \
  -H "X-API-Key: $API_KEY")

WM_COUNT=$(echo "$WM_DATA" | jq -r '.data | length')
echo -e "${BLUE}Working Memory项数: $WM_COUNT${NC}"

if [ "$WM_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅ Working Memory正常记录${NC}"
    echo ""
    echo -e "${BLUE}最近5条记录：${NC}"
    echo "$WM_DATA" | jq -r '.data[0:5] | .[] | "  [\(.created_at | split("T")[1] | split(".")[0])] \(.content[:60])..."'
    TEST3_PASS=1
else
    echo -e "${RED}❌ Working Memory未记录${NC}"
    TEST3_PASS=0
fi

echo ""
echo -e "${YELLOW}═══════════════════════════════════════════${NC}"
echo -e "${YELLOW}测试4: 检查后端日志（时间衰减和权重）${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════${NC}\n"

echo -e "${BLUE}最近的记忆权重计算日志：${NC}"
tail -50 /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/backend-onnx-fixed.log | \
  grep "Memory: user=" | tail -5 || echo "  未找到权重计算日志"

echo ""
echo -e "${YELLOW}═══════════════════════════════════════════${NC}"
echo -e "${YELLOW}测试5: 新Session测试（验证Session隔离）${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════${NC}\n"

NEW_SESSION=$(uuidgen | tr '[:upper:]' '[:lower:]')
echo -e "${BLUE}新Session ID: $NEW_SESSION${NC}\n"

echo -e "${GREEN}[Step 5] 新session，新用户: '我是Bob'${NC}"
RESP5=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"我是Bob，我喜欢音乐\",
    \"session_id\": \"$NEW_SESSION\"
  }")

echo "$RESP5" | jq -r '.content' | head -10
echo ""

sleep 2

echo -e "${GREEN}[Step 6] 在新session询问: '你知道我是谁吗？'${NC}"
RESP6=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d "{
    \"message\": \"你知道我是谁吗？\",
    \"session_id\": \"$NEW_SESSION\"
  }")

CONTENT6=$(echo "$RESP6" | jq -r '.content')
echo "$CONTENT6" | head -15
echo ""

# 验证新session隔离
echo -e "${YELLOW}【验证结果 - 新Session隔离】${NC}"
if echo "$CONTENT6" | grep -qi "Bob"; then
    echo -e "  ${GREEN}✅ PASS: 新session正确识别新用户 (Bob)${NC}"
    TEST5_PASS=1
else
    echo -e "  ${RED}❌ FAIL: 新session没有识别新用户${NC}"
    TEST5_PASS=0
fi

if echo "$CONTENT6" | grep -qi "Alex"; then
    echo -e "  ${RED}❌ FAIL: 新session混入了旧session的信息 (Alex)${NC}"
    TEST5_PASS=0
else
    echo -e "  ${GREEN}✅ PASS: 新session没有混入旧session的信息${NC}"
fi

echo ""
echo -e "${PURPLE}╔════════════════════════════════════════════╗
║              测试结果汇总                  ║
╚════════════════════════════════════════════╝${NC}\n"

TOTAL_TESTS=5
PASSED_TESTS=0
[ "$TEST1_PASS" = "1" ] && ((PASSED_TESTS++))
[ "$TEST2_PASS" = "1" ] && ((PASSED_TESTS++))
[ "$TEST3_PASS" = "1" ] && ((PASSED_TESTS++))
[ "$TEST5_PASS" = "1" ] && ((PASSED_TESTS++))

echo -e "测试1 - Session隔离:        $([ "$TEST1_PASS" = "1" ] && echo "${GREEN}✅ PASS${NC}" || echo "${RED}❌ FAIL${NC}")"
echo -e "测试2 - 上下文保持:         $([ "$TEST2_PASS" = "1" ] && echo "${GREEN}✅ PASS${NC}" || echo "${RED}❌ FAIL${NC}")"
echo -e "测试3 - Working Memory:     $([ "$TEST3_PASS" = "1" ] && echo "${GREEN}✅ PASS${NC}" || echo "${RED}❌ FAIL${NC}")"
echo -e "测试5 - 新Session隔离:      $([ "$TEST5_PASS" = "1" ] && echo "${GREEN}✅ PASS${NC}" || echo "${RED}❌ FAIL${NC}")"
echo ""
echo -e "${BLUE}通过: $PASSED_TESTS / $TOTAL_TESTS${NC}"

if [ "$PASSED_TESTS" = "$TOTAL_TESTS" ]; then
    echo -e "\n${GREEN}🎉 所有测试通过！记忆冲突解决方案工作正常！${NC}"
else
    echo -e "\n${YELLOW}⚠️  部分测试未通过，需要进一步调试${NC}"
fi

echo ""
echo -e "${BLUE}测试Session IDs:${NC}"
echo "  Session 1 (Alex): $SESSION_ID"
echo "  Session 2 (Bob):  $NEW_SESSION"

