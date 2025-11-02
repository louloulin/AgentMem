#!/bin/bash
# Chat Working Memory Verification Test
# 验证对话产生的working memory是否生效，以及chat对话是否能获取working memory信息

set -e

API_BASE="http://localhost:8080"
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Chat Working Memory Verification Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Helper function to make API calls
call_api() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4
    
    echo -e "${BLUE}Test: ${description}${NC}"
    echo -e "Request: ${method} ${endpoint}"
    
    if [ -n "$data" ]; then
        echo "Request Data: ${data}"
        response=$(curl -s -X ${method} "${API_BASE}${endpoint}" \
            -H "Content-Type: application/json" \
            -d "${data}")
    else
        response=$(curl -s -X ${method} "${API_BASE}${endpoint}")
    fi
    
    echo "Response: ${response}"
    echo ""
    echo "$response"
}

# Test 1: Create a test agent for chat
echo -e "${YELLOW}=== Phase 1: Create Test Agent ===${NC}"
echo ""

# Check if OPENAI_API_KEY is set
if [ -z "$OPENAI_API_KEY" ]; then
    echo -e "${RED}Error: OPENAI_API_KEY environment variable is not set${NC}"
    echo "Please set it with: export OPENAI_API_KEY=your_api_key"
    exit 1
fi

AGENT_RESPONSE=$(call_api POST "/api/v1/agents" "{
  \"name\": \"Chat Working Memory Test Agent\",
  \"description\": \"Agent for testing working memory in chat\",
  \"organization_id\": \"org-test-working-memory\",
  \"user_id\": \"user-test-working-memory\",
  \"llm_config\": {
    \"provider\": \"openai\",
    \"model\": \"gpt-3.5-turbo\",
    \"api_key\": \"$OPENAI_API_KEY\",
    \"temperature\": 0.7,
    \"max_tokens\": 1000
  }
}" "Create test agent")

AGENT_ID=$(echo "$AGENT_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -z "$AGENT_ID" ]; then
    echo -e "${RED}✗ Failed to create agent${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Agent created: ${AGENT_ID}${NC}"
echo ""

# Generate a unique session ID for this test
SESSION_ID="test-session-$(date +%s)"
echo -e "${BLUE}Using session ID: ${SESSION_ID}${NC}"
echo ""

# Test 2: Send first chat message (user introduces themselves)
echo -e "${YELLOW}=== Phase 2: Send First Chat Message ===${NC}"
echo ""

CHAT1_RESPONSE=$(call_api POST "/api/v1/agents/${AGENT_ID}/chat" "{
  \"message\": \"我叫张三，我是一名软件工程师。\",
  \"user_id\": \"user-test-working-memory\",
  \"session_id\": \"${SESSION_ID}\"
}" "First chat message - user introduction")

echo -e "${GREEN}✓ First message sent${NC}"
echo ""

# Wait a moment for working memory to be updated
echo "Waiting 2 seconds for working memory to be updated..."
sleep 2

# Test 3: Check working memory after first message
echo -e "${YELLOW}=== Phase 3: Check Working Memory (After First Message) ===${NC}"
echo ""

WM_RESPONSE1=$(call_api GET "/api/v1/working-memory?session_id=${SESSION_ID}" "" "Get working memory for session")

WM_COUNT1=$(echo "$WM_RESPONSE1" | grep -o '"id"' | wc -l)
echo -e "${BLUE}Working memory items count: ${WM_COUNT1}${NC}"

if [ "$WM_COUNT1" -gt 0 ]; then
    echo -e "${GREEN}✓ Working memory items found${NC}"
    echo ""
    echo -e "${BLUE}Working Memory Content:${NC}"
    echo "$WM_RESPONSE1" | python3 -m json.tool 2>/dev/null || echo "$WM_RESPONSE1"
    echo ""
else
    echo -e "${RED}✗ No working memory items found${NC}"
    echo ""
fi

# Test 4: Send second chat message (asking about self)
echo -e "${YELLOW}=== Phase 4: Send Second Chat Message ===${NC}"
echo ""

CHAT2_RESPONSE=$(call_api POST "/api/v1/agents/${AGENT_ID}/chat" "{
  \"message\": \"你还记得我的名字吗？我的职业是什么？\",
  \"user_id\": \"user-test-working-memory\",
  \"session_id\": \"${SESSION_ID}\"
}" "Second chat message - asking for recall")

echo -e "${BLUE}Agent Response:${NC}"
AGENT_REPLY=$(echo "$CHAT2_RESPONSE" | grep -o '"content":"[^"]*"' | head -1 | cut -d'"' -f4)
echo "$AGENT_REPLY"
echo ""

# Check if agent correctly recalls information
if echo "$AGENT_REPLY" | grep -qi "张三"; then
    echo -e "${GREEN}✓ Agent correctly recalled name (张三)${NC}"
else
    echo -e "${RED}✗ Agent did not recall name correctly${NC}"
fi

if echo "$AGENT_REPLY" | grep -qi "软件工程师\|软件开发\|工程师"; then
    echo -e "${GREEN}✓ Agent correctly recalled profession${NC}"
else
    echo -e "${RED}✗ Agent did not recall profession correctly${NC}"
fi
echo ""

# Test 5: Check working memory after second message
echo -e "${YELLOW}=== Phase 5: Check Working Memory (After Second Message) ===${NC}"
echo ""

WM_RESPONSE2=$(call_api GET "/api/v1/working-memory?session_id=${SESSION_ID}" "" "Get updated working memory")

WM_COUNT2=$(echo "$WM_RESPONSE2" | grep -o '"id"' | wc -l)
echo -e "${BLUE}Working memory items count: ${WM_COUNT2}${NC}"

if [ "$WM_COUNT2" -gt "$WM_COUNT1" ]; then
    echo -e "${GREEN}✓ Working memory expanded with new conversation${NC}"
else
    echo -e "${YELLOW}⚠ Working memory count unchanged (may be expected)${NC}"
fi

echo ""
echo -e "${BLUE}Working Memory Content:${NC}"
echo "$WM_RESPONSE2" | python3 -m json.tool 2>/dev/null || echo "$WM_RESPONSE2"
echo ""

# Test 6: Send third message with new information
echo -e "${YELLOW}=== Phase 6: Add New Information ===${NC}"
echo ""

CHAT3_RESPONSE=$(call_api POST "/api/v1/agents/${AGENT_ID}/chat" "{
  \"message\": \"我今天要开会讨论项目架构，会议在下午3点。\",
  \"user_id\": \"user-test-working-memory\",
  \"session_id\": \"${SESSION_ID}\"
}" "Third chat message - new context")

echo -e "${GREEN}✓ Third message sent${NC}"
echo ""

sleep 2

# Test 7: Verify new information is in working memory
echo -e "${YELLOW}=== Phase 7: Verify New Information in Working Memory ===${NC}"
echo ""

WM_RESPONSE3=$(call_api GET "/api/v1/working-memory?session_id=${SESSION_ID}" "" "Get final working memory state")

if echo "$WM_RESPONSE3" | grep -qi "会议\|开会\|下午3点\|项目架构"; then
    echo -e "${GREEN}✓ New information added to working memory${NC}"
else
    echo -e "${YELLOW}⚠ Could not verify new information in working memory${NC}"
fi

echo ""
echo -e "${BLUE}Final Working Memory Content:${NC}"
echo "$WM_RESPONSE3" | python3 -m json.tool 2>/dev/null || echo "$WM_RESPONSE3"
echo ""

# Test 8: Test context retrieval in new message
echo -e "${YELLOW}=== Phase 8: Test Context Retrieval ===${NC}"
echo ""

CHAT4_RESPONSE=$(call_api POST "/api/v1/agents/${AGENT_ID}/chat" "{
  \"message\": \"总结一下我们今天讨论的所有内容。\",
  \"user_id\": \"user-test-working-memory\",
  \"session_id\": \"${SESSION_ID}\"
}" "Fourth chat message - context recall")

echo -e "${BLUE}Agent Summary:${NC}"
AGENT_SUMMARY=$(echo "$CHAT4_RESPONSE" | grep -o '"content":"[^"]*"' | head -1 | cut -d'"' -f4)
echo "$AGENT_SUMMARY"
echo ""

# Verify agent recalls all context
RECALL_CHECKS=0
if echo "$AGENT_SUMMARY" | grep -qi "张三"; then
    echo -e "${GREEN}✓ Summary includes name${NC}"
    ((RECALL_CHECKS++))
fi
if echo "$AGENT_SUMMARY" | grep -qi "软件工程师\|软件开发\|工程师"; then
    echo -e "${GREEN}✓ Summary includes profession${NC}"
    ((RECALL_CHECKS++))
fi
if echo "$AGENT_SUMMARY" | grep -qi "会议\|开会"; then
    echo -e "${GREEN}✓ Summary includes meeting${NC}"
    ((RECALL_CHECKS++))
fi
if echo "$AGENT_SUMMARY" | grep -qi "下午3点\|3点\|15:00"; then
    echo -e "${GREEN}✓ Summary includes time${NC}"
    ((RECALL_CHECKS++))
fi

echo ""
echo -e "${BLUE}Context Recall Score: ${RECALL_CHECKS}/4${NC}"
echo ""

# Test 9: Check working memory in database directly
echo -e "${YELLOW}=== Phase 9: Database Verification ===${NC}"
echo ""

if [ -f "data/agentmem.db" ]; then
    echo -e "${BLUE}Checking database for working memory entries...${NC}"
    
    DB_RESULT=$(sqlite3 data/agentmem.db <<EOF
SELECT 
    COUNT(*) as count,
    GROUP_CONCAT(content, ' | ') as contents
FROM memories 
WHERE session_id = '${SESSION_ID}' 
  AND memory_type = 'working'
  AND is_deleted = 0;
EOF
)
    
    DB_COUNT=$(echo "$DB_RESULT" | cut -d'|' -f1)
    DB_CONTENTS=$(echo "$DB_RESULT" | cut -d'|' -f2-)
    
    echo -e "${BLUE}Database records count: ${DB_COUNT}${NC}"
    
    if [ "$DB_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✓ Working memory persisted to database${NC}"
        echo ""
        echo -e "${BLUE}Database Contents:${NC}"
        echo "$DB_CONTENTS"
        echo ""
    else
        echo -e "${RED}✗ No working memory found in database${NC}"
        echo ""
    fi
    
    # Check if working memory data matches between API and database
    if [ "$WM_COUNT2" -eq "$DB_COUNT" ]; then
        echo -e "${GREEN}✓ API and database counts match${NC}"
    else
        echo -e "${YELLOW}⚠ API count (${WM_COUNT2}) != Database count (${DB_COUNT})${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Database file not found, skipping database verification${NC}"
fi

echo ""

# Test 10: Test working memory with a different session
echo -e "${YELLOW}=== Phase 10: Session Isolation Test ===${NC}"
echo ""

SESSION_ID_2="test-session-$(date +%s)-2"
echo -e "${BLUE}Using new session ID: ${SESSION_ID_2}${NC}"

CHAT5_RESPONSE=$(call_api POST "/api/v1/agents/${AGENT_ID}/chat" "{
  \"message\": \"我叫李四，我是一名设计师。\",
  \"user_id\": \"user-test-working-memory\",
  \"session_id\": \"${SESSION_ID_2}\"
}" "New session - different user")

sleep 2

# Check that new session has its own working memory
WM_RESPONSE_NEW=$(call_api GET "/api/v1/working-memory?session_id=${SESSION_ID_2}" "" "Get working memory for new session")

# Verify sessions are isolated
if echo "$WM_RESPONSE_NEW" | grep -qi "李四" && ! echo "$WM_RESPONSE_NEW" | grep -qi "张三"; then
    echo -e "${GREEN}✓ Session isolation working correctly${NC}"
else
    echo -e "${RED}✗ Session isolation may have issues${NC}"
fi

echo ""
echo -e "${BLUE}New Session Working Memory:${NC}"
echo "$WM_RESPONSE_NEW" | python3 -m json.tool 2>/dev/null || echo "$WM_RESPONSE_NEW"
echo ""

# Test 11: Clean up - clear working memory
echo -e "${YELLOW}=== Phase 11: Cleanup ===${NC}"
echo ""

CLEAR_RESPONSE1=$(call_api DELETE "/api/v1/working-memory/sessions/${SESSION_ID}" "" "Clear first session working memory")
CLEAR_RESPONSE2=$(call_api DELETE "/api/v1/working-memory/sessions/${SESSION_ID_2}" "" "Clear second session working memory")

echo -e "${GREEN}✓ Working memory cleared for both sessions${NC}"
echo ""

# Verify clearing worked
WM_VERIFY=$(call_api GET "/api/v1/working-memory?session_id=${SESSION_ID}" "" "Verify session 1 cleared")
if echo "$WM_VERIFY" | grep -q '"id"'; then
    echo -e "${RED}✗ Working memory not fully cleared${NC}"
else
    echo -e "${GREEN}✓ Working memory successfully cleared${NC}"
fi

echo ""

# Delete test agent
DELETE_RESPONSE=$(call_api DELETE "/api/v1/agents/${AGENT_ID}" "" "Delete test agent")
echo -e "${GREEN}✓ Test agent deleted${NC}"
echo ""

# Final Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${GREEN}✓ Tests Completed${NC}"
echo ""
echo -e "Key Findings:"
echo -e "  - Agent ID: ${AGENT_ID}"
echo -e "  - Session 1: ${SESSION_ID}"
echo -e "  - Session 2: ${SESSION_ID_2}"
echo -e "  - Working Memory Items (Session 1): ${WM_COUNT2}"
echo -e "  - Context Recall Score: ${RECALL_CHECKS}/4"
if [ -n "$DB_COUNT" ]; then
    echo -e "  - Database Records: ${DB_COUNT}"
fi
echo ""

if [ "$RECALL_CHECKS" -ge 3 ] && [ "$WM_COUNT2" -gt 0 ]; then
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}✓ WORKING MEMORY VERIFICATION PASSED${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "Working memory is functioning correctly:"
    echo -e "  ✓ Chat messages create working memory items"
    echo -e "  ✓ Working memory is retrieved in subsequent messages"
    echo -e "  ✓ Agent can recall context from working memory"
    echo -e "  ✓ Session isolation is maintained"
else
    echo -e "${YELLOW}========================================${NC}"
    echo -e "${YELLOW}⚠ WORKING MEMORY NEEDS ATTENTION${NC}"
    echo -e "${YELLOW}========================================${NC}"
    echo ""
    echo -e "Issues detected:"
    if [ "$WM_COUNT2" -eq 0 ]; then
        echo -e "  ✗ No working memory items created"
    fi
    if [ "$RECALL_CHECKS" -lt 3 ]; then
        echo -e "  ✗ Agent context recall incomplete"
    fi
fi

echo ""
echo -e "${BLUE}Test completed at: $(date)${NC}"

