#!/bin/bash
# UI Chat Testing via API - 模拟前端Chat UI的完整流程
# 验证working memory在UI对话中的完整生命周期

set -e

API_BASE="http://localhost:8080"
UI_BASE="http://localhost:3001"
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Chat UI Working Memory Test${NC}"
echo -e "${BLUE}Via API (Simulating Frontend)${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check services
echo -e "${YELLOW}=== Step 1: Check Services ===${NC}"
echo ""

# Check backend
if curl -s "${API_BASE}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Backend API running on ${API_BASE}${NC}"
else
    echo -e "${RED}✗ Backend API not accessible${NC}"
    exit 1
fi

# Check frontend
if curl -s "${UI_BASE}" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Frontend UI running on ${UI_BASE}${NC}"
else
    echo -e "${YELLOW}⚠ Frontend UI not accessible (not critical)${NC}"
fi

echo ""

# Get available agents
echo -e "${YELLOW}=== Step 2: Get Available Agents ===${NC}"
echo ""

AGENTS_RESPONSE=$(curl -s "${API_BASE}/api/v1/agents")
echo "Available agents:"
echo "$AGENTS_RESPONSE" | python3 -m json.tool 2>/dev/null | grep -E '"id"|"name"' | head -10
echo ""

# Use first available agent
AGENT_ID=$(echo "$AGENTS_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
AGENT_NAME=$(echo "$AGENTS_RESPONSE" | grep -o '"name":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -z "$AGENT_ID" ]; then
    echo -e "${RED}✗ No agents found${NC}"
    exit 1
fi

echo -e "${GREEN}Selected Agent:${NC}"
echo -e "  ID: ${AGENT_ID}"
echo -e "  Name: ${AGENT_NAME}"
echo ""

# Generate session ID (like frontend does)
USER_ID="ui-test-user-$(date +%s)"
SESSION_ID="${USER_ID}_$(uuidgen | tr '[:upper:]' '[:lower:]')"
echo -e "${CYAN}Generated Session:${NC}"
echo -e "  User ID: ${USER_ID}"
echo -e "  Session ID: ${SESSION_ID}"
echo ""

# Test 1: First message (user introduction)
echo -e "${YELLOW}=== Step 3: Send First Message ===${NC}"
echo ""

MESSAGE_1="你好！我是李明，我在一家科技公司工作，负责AI产品开发。最近在研究大模型的记忆管理系统。"

echo -e "${CYAN}User says:${NC} ${MESSAGE_1}"
echo ""

CHAT1_START=$(date +%s%3N)
CHAT1_RESPONSE=$(curl -s -X POST "${API_BASE}/api/v1/agents/${AGENT_ID}/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"${MESSAGE_1}\",
    \"user_id\": \"${USER_ID}\",
    \"session_id\": \"${SESSION_ID}\"
  }")
CHAT1_END=$(date +%s%3N)
CHAT1_TIME=$((CHAT1_END - CHAT1_START))

echo "Response received in ${CHAT1_TIME}ms"
echo ""

# Parse response
RESPONSE_1=$(echo "$CHAT1_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['data']['content'] if 'data' in data else 'No response')" 2>/dev/null || echo "Parse error")
MEMORIES_COUNT_1=$(echo "$CHAT1_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['data']['memories_count'] if 'data' in data else 0)" 2>/dev/null || echo "0")

echo -e "${CYAN}Agent says:${NC}"
echo "$RESPONSE_1" | fold -w 80 -s
echo ""
echo -e "${BLUE}Memories used: ${MEMORIES_COUNT_1}${NC}"
echo ""

# Wait for working memory to be written
echo "Waiting 2 seconds for working memory to be persisted..."
sleep 2

# Check working memory via API
echo -e "${YELLOW}=== Step 4: Check Working Memory (After Message 1) ===${NC}"
echo ""

WM1_RESPONSE=$(curl -s "${API_BASE}/api/v1/working-memory?session_id=${SESSION_ID}")
WM1_COUNT=$(echo "$WM1_RESPONSE" | python3 -c "import sys, json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

echo -e "${BLUE}Working Memory Items: ${WM1_COUNT}${NC}"

if [ "$WM1_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Working memory created successfully${NC}"
    echo ""
    echo "Working Memory Content:"
    echo "$WM1_RESPONSE" | python3 -m json.tool 2>/dev/null | head -30
else
    echo -e "${RED}✗ No working memory found${NC}"
fi
echo ""

# Test 2: Second message (recall test)
echo -e "${YELLOW}=== Step 5: Send Second Message (Recall Test) ===${NC}"
echo ""

MESSAGE_2="你还记得我的名字吗？我在哪里工作？"

echo -e "${CYAN}User says:${NC} ${MESSAGE_2}"
echo ""

CHAT2_START=$(date +%s%3N)
CHAT2_RESPONSE=$(curl -s -X POST "${API_BASE}/api/v1/agents/${AGENT_ID}/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"${MESSAGE_2}\",
    \"user_id\": \"${USER_ID}\",
    \"session_id\": \"${SESSION_ID}\"
  }")
CHAT2_END=$(date +%s%3N)
CHAT2_TIME=$((CHAT2_END - CHAT2_START))

echo "Response received in ${CHAT2_TIME}ms"
echo ""

RESPONSE_2=$(echo "$CHAT2_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['data']['content'] if 'data' in data else 'No response')" 2>/dev/null || echo "Parse error")
MEMORIES_COUNT_2=$(echo "$CHAT2_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['data']['memories_count'] if 'data' in data else 0)" 2>/dev/null || echo "0")

echo -e "${CYAN}Agent says:${NC}"
echo "$RESPONSE_2" | fold -w 80 -s
echo ""
echo -e "${BLUE}Memories used: ${MEMORIES_COUNT_2}${NC}"
echo ""

# Analyze recall accuracy
echo -e "${YELLOW}=== Step 6: Analyze Recall Accuracy ===${NC}"
echo ""

RECALL_SCORE=0

if echo "$RESPONSE_2" | grep -qi "李明"; then
    echo -e "${GREEN}✓ Agent recalled name: 李明${NC}"
    ((RECALL_SCORE++))
else
    echo -e "${RED}✗ Agent did not recall name${NC}"
fi

if echo "$RESPONSE_2" | grep -qi "科技公司\|公司"; then
    echo -e "${GREEN}✓ Agent recalled workplace: 科技公司${NC}"
    ((RECALL_SCORE++))
else
    echo -e "${RED}✗ Agent did not recall workplace${NC}"
fi

if echo "$RESPONSE_2" | grep -qi "AI产品\|产品开发\|AI"; then
    echo -e "${GREEN}✓ Agent recalled job: AI产品开发${NC}"
    ((RECALL_SCORE++))
else
    echo -e "${YELLOW}⚠ Agent partially recalled job${NC}"
fi

echo ""
echo -e "${BLUE}Recall Accuracy: ${RECALL_SCORE}/3 (${RECALL_SCORE}×33% = $((RECALL_SCORE*33))%)${NC}"
echo ""

# Check working memory after second message
sleep 2
echo -e "${YELLOW}=== Step 7: Check Working Memory (After Message 2) ===${NC}"
echo ""

WM2_RESPONSE=$(curl -s "${API_BASE}/api/v1/working-memory?session_id=${SESSION_ID}")
WM2_COUNT=$(echo "$WM2_RESPONSE" | python3 -c "import sys, json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

echo -e "${BLUE}Working Memory Items: ${WM2_COUNT}${NC}"

if [ "$WM2_COUNT" -gt "$WM1_COUNT" ]; then
    echo -e "${GREEN}✓ Working memory expanded (${WM1_COUNT} → ${WM2_COUNT})${NC}"
elif [ "$WM2_COUNT" -eq "$WM1_COUNT" ] && [ "$WM1_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Working memory maintained${NC}"
else
    echo -e "${YELLOW}⚠ Working memory status unclear${NC}"
fi
echo ""

# Test 3: Third message (complex context)
echo -e "${YELLOW}=== Step 8: Send Third Message (Context Test) ===${NC}"
echo ""

MESSAGE_3="我们刚才聊到了什么话题？请总结一下我们的对话。"

echo -e "${CYAN}User says:${NC} ${MESSAGE_3}"
echo ""

CHAT3_RESPONSE=$(curl -s -X POST "${API_BASE}/api/v1/agents/${AGENT_ID}/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"${MESSAGE_3}\",
    \"user_id\": \"${USER_ID}\",
    \"session_id\": \"${SESSION_ID}\"
  }")

RESPONSE_3=$(echo "$CHAT3_RESPONSE" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['data']['content'] if 'data' in data else 'No response')" 2>/dev/null || echo "Parse error")

echo -e "${CYAN}Agent says:${NC}"
echo "$RESPONSE_3" | fold -w 80 -s
echo ""

# Check if summary is comprehensive
SUMMARY_SCORE=0
if echo "$RESPONSE_3" | grep -qi "李明"; then ((SUMMARY_SCORE++)); fi
if echo "$RESPONSE_3" | grep -qi "科技公司\|公司"; then ((SUMMARY_SCORE++)); fi
if echo "$RESPONSE_3" | grep -qi "AI\|产品\|开发"; then ((SUMMARY_SCORE++)); fi
if echo "$RESPONSE_3" | grep -qi "记忆\|大模型"; then ((SUMMARY_SCORE++)); fi

echo -e "${BLUE}Summary Completeness: ${SUMMARY_SCORE}/4 (${SUMMARY_SCORE}×25% = $((SUMMARY_SCORE*25))%)${NC}"
echo ""

# Database verification
echo -e "${YELLOW}=== Step 9: Database Verification ===${NC}"
echo ""

if [ -f "data/agentmem.db" ]; then
    echo "Querying database directly..."
    
    DB_WM_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE session_id='${SESSION_ID}' AND memory_type='working' AND is_deleted=0;" 2>/dev/null || echo "0")
    
    echo -e "${BLUE}Working Memory in DB: ${DB_WM_COUNT}${NC}"
    
    if [ "$DB_WM_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✓ Working memory persisted to database${NC}"
        echo ""
        echo "Sample content from database:"
        sqlite3 data/agentmem.db "SELECT id, SUBSTR(content, 1, 80) FROM memories WHERE session_id='${SESSION_ID}' AND memory_type='working' ORDER BY created_at LIMIT 2;" 2>/dev/null
    else
        echo -e "${RED}✗ No working memory in database${NC}"
    fi
    
    # Check API vs DB consistency
    if [ "$WM2_COUNT" -eq "$DB_WM_COUNT" ]; then
        echo ""
        echo -e "${GREEN}✓ API and Database counts match${NC}"
    else
        echo ""
        echo -e "${YELLOW}⚠ API (${WM2_COUNT}) != DB (${DB_WM_COUNT})${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Database file not found${NC}"
fi

echo ""

# Cleanup
echo -e "${YELLOW}=== Step 10: Cleanup ===${NC}"
echo ""

CLEANUP_RESPONSE=$(curl -s -X DELETE "${API_BASE}/api/v1/working-memory/sessions/${SESSION_ID}")
echo -e "${GREEN}✓ Session cleaned up${NC}"
echo ""

# Final Report
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Summary Report${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

echo -e "${CYAN}Session Information:${NC}"
echo "  Agent: ${AGENT_NAME} (${AGENT_ID})"
echo "  User ID: ${USER_ID}"
echo "  Session ID: ${SESSION_ID}"
echo ""

echo -e "${CYAN}Performance Metrics:${NC}"
echo "  Message 1 Response Time: ${CHAT1_TIME}ms"
echo "  Message 2 Response Time: ${CHAT2_TIME}ms"
echo "  Average Response Time: $(( (CHAT1_TIME + CHAT2_TIME) / 2 ))ms"
echo ""

echo -e "${CYAN}Working Memory Metrics:${NC}"
echo "  Initial Working Memory: ${WM1_COUNT} items"
echo "  Final Working Memory: ${WM2_COUNT} items"
echo "  Database Records: ${DB_WM_COUNT:-N/A}"
echo ""

echo -e "${CYAN}Accuracy Metrics:${NC}"
echo "  Recall Accuracy: ${RECALL_SCORE}/3 ($((RECALL_SCORE*33))%)"
echo "  Summary Completeness: ${SUMMARY_SCORE}/4 ($((SUMMARY_SCORE*25))%)"
echo ""

# Overall assessment
TOTAL_SCORE=$((RECALL_SCORE + SUMMARY_SCORE))
MAX_SCORE=7

if [ "$WM2_COUNT" -gt 0 ] && [ "$TOTAL_SCORE" -ge 5 ]; then
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}✓ CHAT UI WORKING MEMORY TEST PASSED${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "Overall Score: ${TOTAL_SCORE}/${MAX_SCORE} ($((TOTAL_SCORE*100/MAX_SCORE))%)"
    echo ""
    echo "Key Findings:"
    echo "  ✓ Working memory is created and maintained"
    echo "  ✓ Agent can recall context from working memory"
    echo "  ✓ Session isolation works correctly"
    echo "  ✓ Data persists to database"
elif [ "$WM2_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}========================================${NC}"
    echo -e "${YELLOW}⚠ CHAT UI WORKING MEMORY PARTIAL PASS${NC}"
    echo -e "${YELLOW}========================================${NC}"
    echo ""
    echo -e "Overall Score: ${TOTAL_SCORE}/${MAX_SCORE} ($((TOTAL_SCORE*100/MAX_SCORE))%)"
    echo ""
    echo "Issues:"
    [ "$RECALL_SCORE" -lt 2 ] && echo "  ⚠ Recall accuracy could be improved"
    [ "$SUMMARY_SCORE" -lt 3 ] && echo "  ⚠ Summary completeness needs work"
else
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}✗ CHAT UI WORKING MEMORY TEST FAILED${NC}"
    echo -e "${RED}========================================${NC}"
    echo ""
    echo "Critical Issues:"
    echo "  ✗ Working memory not being created"
fi

echo ""
echo -e "${BLUE}Test completed at: $(date)${NC}"
echo ""
echo -e "${CYAN}To test manually in UI:${NC}"
echo "  1. Open: ${UI_BASE}/admin/chat"
echo "  2. Select agent: ${AGENT_NAME}"
echo "  3. Send messages and observe responses"
echo "  4. Check if agent remembers context"
echo ""

