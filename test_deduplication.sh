#!/bin/bash

# AgentMem 去重功能测试脚本
# 测试场景：发送相同内容两次，验证去重逻辑

set -e

BASE_URL="http://localhost:8080"
AGENT_ID="agent-e60d1616-55a2-4b82-aee7-bae39c99f5f9"
USER_ID="user-dedup-test-$(date +%s)"
SESSION_ID="session-dedup-test"

echo "=========================================="
echo "AgentMem 去重功能测试"
echo "=========================================="
echo "User ID: $USER_ID"
echo "Agent ID: $AGENT_ID"
echo "Session ID: $SESSION_ID"
echo ""

# 测试1: 首次发送消息
echo "📝 测试1: 首次发送消息"
echo "消息: '你好！我叫赵六，我是一名产品经理，喜欢用户研究，住在杭州。'"
echo ""

RESPONSE1=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"你好！我叫赵六，我是一名产品经理，喜欢用户研究，住在杭州。\",
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"stream\": false
  }")

echo "响应:"
echo "$RESPONSE1" | jq .
echo ""

MEMORIES_COUNT_1=$(echo "$RESPONSE1" | jq -r '.data.memories_count // 0')
echo "✅ 首次提取记忆数: $MEMORIES_COUNT_1"
echo ""

# 等待2秒，确保记忆已保存
sleep 2

# 测试2: 发送相同消息（应该被去重）
echo "[object Object]发送相同消息（测试去重）"
echo "消息: '你好！我叫赵六，我是一名产品经理，喜欢用户研究，住在杭州。'"
echo ""

RESPONSE2=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"你好！我叫赵六，我是一名产品经理，喜欢用户研究，住在杭州。\",
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"stream\": false
  }")

echo "响应:"
echo "$RESPONSE2" | jq .
echo ""

MEMORIES_COUNT_2=$(echo "$RESPONSE2" | jq -r '.data.memories_count // 0')
echo "✅ 第二次提取记忆数: $MEMORIES_COUNT_2"
echo ""

# 测试3: 发送略微不同的消息（应该不被去重）
echo "[object Object]送略微不同的消息"
echo "消息: '我今年35岁，有10年产品经验，擅长B端产品设计。'"
echo ""

RESPONSE3=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"我今年35岁，有10年产品经验，擅长B端产品设计。\",
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"stream\": false
  }")

echo "响应:"
echo "$RESPONSE3" | jq .
echo ""

MEMORIES_COUNT_3=$(echo "$RESPONSE3" | jq -r '.data.memories_count // 0')
echo "✅ 第三次提取记忆数: $MEMORIES_COUNT_3"
echo ""

# 测试4: 记忆检索验证
echo "📝 测试4: 记忆检索验证"
echo "问题: '我叫什么名字？我的职业是什么？我住在哪里？'"
echo ""

RESPONSE4=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"我叫什么名字？我的职业是什么？我住在哪里？\",
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"stream\": false
  }")

echo "响应:"
echo "$RESPONSE4" | jq .
echo ""

CONTENT=$(echo "$RESPONSE4" | jq -r '.data.content')
echo "✅ AI 回答: $CONTENT"
echo ""

# 分析结果
echo "=========================================="
echo "测试结果分析"
echo "=========================================="
echo ""

# 检查是否包含正确信息
if echo "$CONTENT" | grep -q "赵六"; then
    echo "✅ 姓名识别: 正确（赵六）"
else
    echo "❌ 姓名识别: 失败"
fi

if echo "$CONTENT" | grep -q "产品经理"; then
    echo "✅ 职业识别: 正确（产品经理）"
else
    echo "❌ 职业识别: 失败"
fi

if echo "$CONTENT" | grep -q "杭州"; then
    echo "✅ 住址识别: 正确（杭州）"
else
    echo "❌ 住址识别: 失败"
fi

echo ""
echo "记忆提取统计:"
echo "  - 首次提取: $MEMORIES_COUNT_1 条"
echo "  - 重复提取: $MEMORIES_COUNT_2 条"
echo "  - 新信息提取: $MEMORIES_COUNT_3 条"
echo ""

# 查看服务器日志中的去重信息
echo "=========================================="
echo "服务器日志分析（去重相关）"
echo "=========================================="
echo ""

if [ -f "backend-no-auth.log" ]; then
    echo "最近的去重日志:"
    grep -E "(Skipping duplicate|saved.*skipped|Memory save complete)" backend-no-auth.log | tail -10
else
    echo "⚠️ 日志文件不存在"
fi

echo ""
echo "=========================================="
echo "测试完成"
echo "=========================================="

