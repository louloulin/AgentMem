#!/bin/bash
# Simplified Chat Working Memory Verification Test
# 使用现有的agent测试working memory功能

set -e

API_BASE="http://localhost:8080"
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Chat Working Memory Simple Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Use existing agent
AGENT_ID="agent-7bd801e2-c8da-42e4-b10f-c2ef7f610235"
echo -e "${GREEN}Using existing agent: ${AGENT_ID}${NC}"
echo ""

# Generate a unique session ID for this test
SESSION_ID="test-session-$(date +%s)"
echo -e "${BLUE}Using session ID: ${SESSION_ID}${NC}"
echo ""

# Test 1: Send first chat message (user introduces themselves)
echo -e "${YELLOW}=== Phase 1: Send First Chat Message ===${NC}"
echo ""

echo "Sending: 我叫张三，我是一名软件工程师。"
CHAT1_RESPONSE=$(curl -s -X POST "${API_BASE}/api/v1/agents/${AGENT_ID}/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"我叫张三，我是一名软件工程师。\",
    \"user_id\": \"user-test-working-memory\",
    \"session_id\": \"${SESSION_ID}\"
  }")

echo "Response:"
echo "$CHAT1_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$CHAT1_RESPONSE"
echo ""

# Extract agent response
AGENT_REPLY1=$(echo "$CHAT1_RESPONSE" | grep -o '"content":"[^"]*"' | head -1 | cut -d'"' -f4)
echo -e "${BLUE}Agent said: ${AGENT_REPLY1}${NC}"
echo ""

# Wait for working memory to be updated
echo "Waiting 3 seconds for working memory to be updated..."
sleep 3

# Test 2: Check working memory after first message
echo -e "${YELLOW}=== Phase 2: Check Working Memory ===${NC}"
echo ""

WM_RESPONSE=$(curl -s -X GET "${API_BASE}/api/v1/working-memory?session_id=${SESSION_ID}")
echo "Working Memory Response:"
echo "$WM_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$WM_RESPONSE"
echo ""

WM_COUNT=$(echo "$WM_RESPONSE" | grep -o '"id"' | wc -l)
echo -e "${BLUE}Working memory items count: ${WM_COUNT}${NC}"

if [ "$WM_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Working memory items found!${NC}"
else
    echo -e "${YELLOW}⚠ No working memory items found yet${NC}"
fi
echo ""

# Test 3: Check database directly
echo -e "${YELLOW}=== Phase 3: Check Database ===${NC}"
echo ""

if [ -f "data/agentmem.db" ]; then
    DB_RESULT=$(sqlite3 data/agentmem.db <<EOF
SELECT 
    id,
    session_id,
    memory_type,
    SUBSTR(content, 1, 100) as content_preview,
    importance,
    created_at
FROM memories 
WHERE session_id = '${SESSION_ID}' 
  AND memory_type = 'working'
  AND is_deleted = 0
ORDER BY created_at DESC;
EOF
)
    
    if [ -n "$DB_RESULT" ]; then
        echo -e "${GREEN}✓ Working memory found in database:${NC}"
        echo "$DB_RESULT"
        echo ""
        
        DB_COUNT=$(echo "$DB_RESULT" | wc -l)
        echo -e "${BLUE}Database records count: ${DB_COUNT}${NC}"
    else
        echo -e "${RED}✗ No working memory found in database${NC}"
        
        # Show all working memory records
        echo ""
        echo "Checking for ANY working memory records..."
        ALL_WM=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE memory_type='working' AND is_deleted=0;")
        echo -e "${BLUE}Total working memory records in DB: ${ALL_WM}${NC}"
        
        # Show recent records
        echo ""
        echo "Recent memory records (any type):"
        sqlite3 data/agentmem.db "SELECT id, memory_type, session_id, SUBSTR(content, 1, 50) FROM memories WHERE is_deleted=0 ORDER BY created_at DESC LIMIT 5;"
    fi
else
    echo -e "${RED}✗ Database file not found${NC}"
fi
echo ""

# Test 4: Send second chat message (asking for recall)
echo -e "${YELLOW}=== Phase 4: Test Context Recall ===${NC}"
echo ""

echo "Sending: 你还记得我的名字吗？我的职业是什么？"
CHAT2_RESPONSE=$(curl -s -X POST "${API_BASE}/api/v1/agents/${AGENT_ID}/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"你还记得我的名字吗？我的职业是什么？\",
    \"user_id\": \"user-test-working-memory\",
    \"session_id\": \"${SESSION_ID}\"
  }")

echo "Response:"
echo "$CHAT2_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$CHAT2_RESPONSE"
echo ""

AGENT_REPLY2=$(echo "$CHAT2_RESPONSE" | grep -o '"content":"[^"]*"' | head -1 | cut -d'"' -f4)
echo -e "${BLUE}Agent said: ${AGENT_REPLY2}${NC}"
echo ""

# Check if agent recalls information
RECALL_SUCCESS=false
if echo "$AGENT_REPLY2" | grep -qi "张三"; then
    echo -e "${GREEN}✓ Agent correctly recalled name (张三)${NC}"
    RECALL_SUCCESS=true
else
    echo -e "${RED}✗ Agent did not recall name${NC}"
fi

if echo "$AGENT_REPLY2" | grep -qi "软件工程师\|软件开发\|工程师"; then
    echo -e "${GREEN}✓ Agent correctly recalled profession${NC}"
    RECALL_SUCCESS=true
else
    echo -e "${RED}✗ Agent did not recall profession${NC}"
fi
echo ""

# Test 5: Check working memory after second message
echo -e "${YELLOW}=== Phase 5: Check Updated Working Memory ===${NC}"
echo ""

sleep 2
WM_RESPONSE2=$(curl -s -X GET "${API_BASE}/api/v1/working-memory?session_id=${SESSION_ID}")
WM_COUNT2=$(echo "$WM_RESPONSE2" | grep -o '"id"' | wc -l)
echo -e "${BLUE}Working memory items count: ${WM_COUNT2}${NC}"

if [ "$WM_COUNT2" -gt "$WM_COUNT" ]; then
    echo -e "${GREEN}✓ Working memory expanded after second message${NC}"
elif [ "$WM_COUNT2" -gt 0 ]; then
    echo -e "${GREEN}✓ Working memory still active${NC}"
fi
echo ""

# Cleanup
echo -e "${YELLOW}=== Phase 6: Cleanup ===${NC}"
echo ""

CLEAR_RESPONSE=$(curl -s -X DELETE "${API_BASE}/api/v1/working-memory/sessions/${SESSION_ID}")
echo -e "${GREEN}✓ Working memory cleared for session${NC}"
echo ""

# Final Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Session ID: ${SESSION_ID}"
echo -e "Working Memory Items (Initial): ${WM_COUNT}"
echo -e "Working Memory Items (Final): ${WM_COUNT2}"
echo ""

if [ "$WM_COUNT" -gt 0 ] || [ "$WM_COUNT2" -gt 0 ]; then
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}✓ WORKING MEMORY IS FUNCTIONING${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "Working memory is being created and stored:"
    [ "$WM_COUNT" -gt 0 ] && echo -e "  ✓ Working memory created after first message"
    [ "$WM_COUNT2" -gt "$WM_COUNT" ] && echo -e "  ✓ Working memory expanded with conversation"
    [ "$RECALL_SUCCESS" = true ] && echo -e "  ✓ Agent can recall context from working memory"
else
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}✗ WORKING MEMORY NOT FUNCTIONING${NC}"
    echo -e "${RED}========================================${NC}"
    echo ""
    echo -e "Issues detected:"
    echo -e "  ✗ No working memory items created"
    echo -e ""
    echo -e "Possible causes:"
    echo -e "  1. Working memory store not properly initialized"
    echo -e "  2. Orchestrator not calling update_working_memory"
    echo -e "  3. Database write permissions issue"
fi

echo ""
echo -e "${BLUE}Test completed at: $(date)${NC}"

