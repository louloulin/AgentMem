#!/bin/bash
# AgentMem API Testing Script
# Tests all major API endpoints

set -e

BASE_URL="http://localhost:8080"
API_V1="$BASE_URL/api/v1"

echo "🧪 AgentMem API Testing Suite"
echo "=============================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

test_count=0
pass_count=0
fail_count=0

test_api() {
    local name=$1
    local method=$2
    local url=$3
    local data=$4
    
    test_count=$((test_count + 1))
    echo -n "Testing $name... "
    
    if [ -z "$data" ]; then
        response=$(curl -s -X $method "$url" -w "\n%{http_code}")
    else
        response=$(curl -s -X $method "$url" \
            -H "Content-Type: application/json" \
            -d "$data" \
            -w "\n%{http_code}")
    fi
    
    http_code=$(echo "$response" | tail -n 1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $http_code)"
        pass_count=$((pass_count + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $http_code)"
        echo "  Response: $body"
        fail_count=$((fail_count + 1))
        return 1
    fi
}

echo "1. Health Check"
echo "---------------"
test_api "Health endpoint" GET "$BASE_URL/health"
echo ""

echo "2. Agent APIs"
echo "-------------"
test_api "List agents" GET "$API_V1/agents"
test_api "Create agent" POST "$API_V1/agents" '{"name":"Test Agent","description":"Created by test script"}'

# Get first agent ID
AGENT_ID=$(curl -s "$API_V1/agents" | jq -r '.data[0].id')
if [ ! -z "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    test_api "Get agent by ID" GET "$API_V1/agents/$AGENT_ID"
    test_api "Get agent state" GET "$API_V1/agents/$AGENT_ID/state"
    test_api "Update agent state" PUT "$API_V1/agents/$AGENT_ID/state" '{"state":"thinking"}'
fi
echo ""

echo "3. Memory APIs"
echo "--------------"
if [ ! -z "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    test_api "List memories" GET "$API_V1/agents/$AGENT_ID/memories"
    test_api "Create memory" POST "$API_V1/memories" \
        "{\"agent_id\":\"$AGENT_ID\",\"memory_type\":\"episodic\",\"content\":\"Test memory from script\",\"importance\":0.8}"
fi
echo ""

echo "4. Chat APIs"
echo "------------"
if [ ! -z "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    test_api "Send chat message" POST "$API_V1/agents/$AGENT_ID/chat" \
        '{"message":"Hello, this is a test message"}'
    test_api "Get chat history" GET "$API_V1/agents/$AGENT_ID/chat/history"
fi
echo ""

echo "=============================="
echo "Test Results:"
echo "  Total:  $test_count"
echo -e "  ${GREEN}Passed: $pass_count${NC}"
if [ $fail_count -gt 0 ]; then
    echo -e "  ${RED}Failed: $fail_count${NC}"
else
    echo -e "  ${GREEN}Failed: $fail_count${NC}"
fi
echo ""

if [ $fail_count -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi

