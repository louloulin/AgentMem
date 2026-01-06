#!/bin/bash
# Test Chat UI in Browser - Working Memory Verification
# 通过浏览器打开Chat UI，手动测试working memory功能

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Chat UI Browser Test Guide${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if UI is running
UI_PORT=3001
UI_URL="http://localhost:${UI_PORT}"

echo -e "${YELLOW}Checking if UI is running...${NC}"
if curl -s "${UI_URL}" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ UI is running on port ${UI_PORT}${NC}"
else
    echo -e "${YELLOW}⚠ UI may not be running on port ${UI_PORT}${NC}"
    echo ""
    echo "Starting UI server..."
    cd agentmem-ui
    npm run dev &
    UI_PID=$!
    echo "UI started with PID: $UI_PID"
    sleep 5
fi

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Opening Chat UI in Browser...${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Open browser
CHAT_URL="${UI_URL}/admin/chat"
echo -e "Opening: ${CHAT_URL}"
echo ""

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    open "${CHAT_URL}"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    xdg-open "${CHAT_URL}" 2>/dev/null || echo "Please open ${CHAT_URL} manually"
else
    echo "Please open ${CHAT_URL} in your browser"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Manual Test Instructions${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}Step 1: Select an Agent${NC}"
echo "  - Choose '智谱AI助手' or 'Working Memory Test Agent' from the dropdown"
echo ""

echo -e "${YELLOW}Step 2: Test Working Memory${NC}"
echo "  1. Send first message:"
echo "     '我叫张三，我是一名软件工程师。'"
echo ""
echo "  2. Wait for response (should acknowledge your introduction)"
echo ""
echo "  3. Send second message:"
echo "     '你还记得我的名字吗？我的职业是什么？'"
echo ""
echo "  4. Check if agent recalls:"
echo "     ✓ Should say: '张三'"
echo "     ✓ Should say: '软件工程师'"
echo ""

echo -e "${YELLOW}Step 3: Verify Working Memory via API${NC}"
echo "  While chatting, open a new terminal and run:"
echo ""
echo "  # Get your session_id from browser DevTools (Network tab)"
echo "  # Or check the chat request payload"
echo ""
echo "  curl http://localhost:8080/api/v1/working-memory?session_id=YOUR_SESSION_ID | python3 -m json.tool"
echo ""

echo -e "${YELLOW}Step 4: Check Database${NC}"
echo "  sqlite3 data/agentmem.db"
echo "  SELECT COUNT(*) FROM memories WHERE memory_type='working';"
echo "  SELECT session_id, SUBSTR(content, 1, 50) FROM memories WHERE memory_type='working' ORDER BY created_at DESC LIMIT 5;"
echo ""

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Expected Results${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "✓ Agent should remember your name (张三)"
echo "✓ Agent should remember your profession (软件工程师)"
echo "✓ Working memory API should return conversation pairs"
echo "✓ Database should contain working memory records"
echo ""

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Browser Debugging Tips${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "1. Open Browser DevTools (F12 or Cmd+Option+I)"
echo "2. Go to Network tab"
echo "3. Send chat messages"
echo "4. Look for:"
echo "   - POST /api/v1/agents/{agent_id}/chat"
echo "   - Check request payload for session_id"
echo "   - Check response for memories_count"
echo ""
echo "5. Go to Console tab"
echo "6. Check for any JavaScript errors"
echo ""

echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Press Ctrl+C when done testing${NC}"
echo -e "${YELLOW}========================================${NC}"

# Keep script running
wait

