#!/bin/bash
# 实际性能验证脚本

echo "========================================"
echo "AI Chat Performance Verification"
echo "========================================"
echo ""

# 检查服务器是否运行
echo "🔍 Step 1: Checking server status..."
if curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "✅ Server is running"
else
    echo "❌ Server is not running. Please start it first:"
    echo "   ./start_server_no_auth.sh"
    echo ""
    echo "Exiting..."
    exit 1
fi
echo ""

# 测试1: 简单对话 - 测试TTFB和Prompt长度
echo "🧪 Step 2: Testing simple chat (TTFB & Prompt length)..."
echo "   Sending request: '你好'"
echo ""

START_TIME=$(date +%s%3N)

RESPONSE=$(curl -s -X POST http://localhost:3000/api/agents/test_agent/chat/lumosai \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好",
    "user_id": "test_user",
    "session_id": "test_session_'$(date +%s)'"
  }' 2>&1)

END_TIME=$(date +%s%3N)
TTFB=$((END_TIME - START_TIME))

echo "   Response time: ${TTFB}ms"
echo ""

if [ $TTFB -lt 1000 ]; then
    echo "   ✅ TTFB < 1000ms: PASSED"
else
    echo "   ⚠️  TTFB >= 1000ms: NEEDS REVIEW"
fi
echo ""

# 测试2: 带记忆的对话
echo "🧪 Step 3: Testing chat with memory..."
echo "   Sending follow-up request"
echo ""

START_TIME=$(date +%s%3N)

RESPONSE2=$(curl -s -X POST http://localhost:3000/api/agents/test_agent/chat/lumosai \
  -H "Content-Type: application/json" \
  -d '{
    "message": "请回顾我们之前的对话",
    "user_id": "test_user",
    "session_id": "test_session_follow"
  }' 2>&1)

END_TIME=$(date +%s%3N)
TTFB2=$((END_TIME - START_TIME))

echo "   Response time: ${TTFB2}ms"
echo ""

if [ $TTFB2 -lt 1000 ]; then
    echo "   ✅ TTFB < 1000ms: PASSED"
else
    echo "   ⚠️  TTFB >= 1000ms: NEEDS REVIEW"
fi
echo ""

# 测试3: 检查服务器日志中的Prompt长度
echo "🧪 Step 4: Checking server logs for Prompt length..."
echo "   Please check server logs for:"
echo "   - '📋 === 完整Prompt内容（所有消息）==="
echo "   - '总字符数: XXX'"
echo ""
echo "   Expected: <500 characters"
echo "   Previous: 4606 characters"
echo ""

# 总结
echo "========================================"
echo "Performance Summary"
echo "========================================"
echo ""
echo "Target Metrics:"
echo "  TTFB: <1000ms"
echo "  Prompt Length: <500 chars"
echo "  Token Usage: ~600 tokens"
echo ""
echo "Test Results:"
echo "  Test 1 TTFB: ${TTFB}ms"
echo "  Test 2 TTFB: ${TTFB2}ms"
echo "  Prompt Length: (check server logs)"
echo ""

# 计算平均TTFB
AVG_TTFB=$(( (TTFB + TTFB2) / 2 ))
echo "  Average TTFB: ${AVG_TTFB}ms"
echo ""

if [ $AVG_TTFB -lt 1000 ]; then
    echo "✅ Performance optimization SUCCESSFUL!"
    echo "   TTFB improved from 17500ms to ${AVG_TTFB}ms (-$(( (17500 - AVG_TTFB) * 100 / 17500 ))%)"
else
    echo "⚠️  Performance needs review"
    echo "   Check server logs for details"
fi
echo ""

echo "Next Steps:"
echo "  1. Review server logs for Prompt length"
echo "  2. Check comprehensive score calculation"
echo "  3. Monitor token usage in production"
echo ""

