#!/bin/bash
# 测试修复后的搜索功能 - 使用 default 作为默认用户

set -e

API_URL="${API_URL:-http://localhost:8080}"
USER_ID="default"
AGENT_ID="${AGENT_ID:-default-agent}"

echo "=========================================="
echo "测试搜索功能 - 使用 default 用户ID"
echo "=========================================="
echo "API URL: $API_URL"
echo "User ID: $USER_ID"
echo "Agent ID: $AGENT_ID"
echo ""

# 1. 测试搜索记忆（使用默认用户ID）
echo "1. 测试搜索记忆..."
SEARCH_QUERY="test"
SEARCH_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: default-org" \
  -d "{
    \"query\": \"$SEARCH_QUERY\",
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"$USER_ID\",
    \"limit\": 10
  }")

echo "搜索响应:"
echo "$SEARCH_RESPONSE" | jq '.' 2>/dev/null || echo "$SEARCH_RESPONSE"
echo ""

# 2. 检查返回的记忆数量
RESULT_COUNT=$(echo "$SEARCH_RESPONSE" | jq '.data | length' 2>/dev/null || echo "0")
echo "找到 $RESULT_COUNT 条记忆"
echo ""

# 3. 验证返回的记忆 user_id 是否为 default
if [ "$RESULT_COUNT" -gt 0 ]; then
  echo "2. 验证返回记忆的 user_id..."
  FIRST_USER_ID=$(echo "$SEARCH_RESPONSE" | jq -r '.data[0].user_id' 2>/dev/null || echo "")
  if [ "$FIRST_USER_ID" = "$USER_ID" ]; then
    echo "✅ 返回的记忆 user_id 正确: $FIRST_USER_ID"
  else
    echo "❌ 返回的记忆 user_id 不匹配: 期望 '$USER_ID', 实际 '$FIRST_USER_ID'"
  fi
  echo ""
fi

# 4. 检查数据库中的默认用户记忆数量
echo "3. 检查数据库中的默认用户记忆..."
DB_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE user_id = '$USER_ID' AND is_deleted = 0;" 2>/dev/null || echo "0")
echo "数据库中 user_id='$USER_ID' 的记忆数量: $DB_COUNT"
echo ""

# 5. 验证搜索结果是否匹配数据库
if [ "$RESULT_COUNT" -gt 0 ] && [ "$DB_COUNT" -gt 0 ]; then
  echo "4. 验证搜索结果匹配数据库..."
  if [ "$RESULT_COUNT" -le "$DB_COUNT" ]; then
    echo "✅ 搜索结果数量合理（搜索返回 $RESULT_COUNT，数据库有 $DB_COUNT）"
  else
    echo "⚠️  搜索结果数量超过数据库记录，可能有问题"
  fi
  echo ""
fi

# 6. 测试不传递 user_id 的情况（应该使用默认值）
echo "5. 测试不传递 user_id 的搜索..."
SEARCH_RESPONSE_NO_USER=$(curl -s -X POST "$API_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: default-org" \
  -d "{
    \"query\": \"$SEARCH_QUERY\",
    \"agent_id\": \"$AGENT_ID\",
    \"limit\": 10
  }")

RESULT_COUNT_NO_USER=$(echo "$SEARCH_RESPONSE_NO_USER" | jq '.data | length' 2>/dev/null || echo "0")
echo "不传递 user_id 时找到 $RESULT_COUNT_NO_USER 条记忆"
echo ""

echo "=========================================="
echo "测试完成"
echo "=========================================="
echo "总结:"
echo "- 数据库中有 $DB_COUNT 条 user_id='$USER_ID' 的记忆"
echo "- 搜索返回 $RESULT_COUNT 条记忆"
echo "- 不传递 user_id 时返回 $RESULT_COUNT_NO_USER 条记忆"
echo ""

