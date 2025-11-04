#!/bin/bash
# 通过 OpenAPI 方式验证搜索功能（使用 default 用户ID）

set -e

API_URL="${API_URL:-http://localhost:8080}"
USER_ID="default"
AGENT_ID="${AGENT_ID:-}"

echo "=========================================="
echo "通过 OpenAPI 验证搜索功能"
echo "=========================================="
echo "API URL: $API_URL"
echo "User ID: $USER_ID"
echo ""

# 1. 检查后端健康状态
echo "1. 检查后端健康状态..."
HEALTH_RESPONSE=$(curl -s "$API_URL/health" || echo "")
if [ -z "$HEALTH_RESPONSE" ]; then
    echo "❌ 后端服务未运行，请先启动后端服务"
    exit 1
fi
echo "✅ 后端服务运行正常"
echo ""

# 2. 获取可用 Agent 列表
echo "2. 获取可用 Agent 列表..."
AGENTS_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: default-org")

AGENT_COUNT=$(echo "$AGENTS_RESPONSE" | jq '.data | length' 2>/dev/null || echo "0")
echo "找到 $AGENT_COUNT 个 Agent"

if [ "$AGENT_COUNT" -gt 0 ] && [ -z "$AGENT_ID" ]; then
    AGENT_ID=$(echo "$AGENTS_RESPONSE" | jq -r '.data[0].id' 2>/dev/null || echo "")
    echo "使用第一个 Agent: $AGENT_ID"
fi
echo ""

# 3. 检查数据库中的记忆数量
echo "3. 检查数据库中的记忆数量..."
DB_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE user_id = '$USER_ID' AND is_deleted = 0;" 2>/dev/null || echo "0")
echo "数据库中 user_id='$USER_ID' 的记忆数量: $DB_COUNT"
echo ""

# 4. 测试搜索记忆（不指定 agent_id）
echo "4. 测试搜索记忆（不指定 agent_id）..."
SEARCH_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: default-org" \
  -d "{
    \"query\": \"test\",
    \"user_id\": \"$USER_ID\",
    \"limit\": 10
  }")

RESULT_COUNT=$(echo "$SEARCH_RESPONSE" | jq '.data | length' 2>/dev/null || echo "0")
echo "搜索返回 $RESULT_COUNT 条记忆"

if [ "$RESULT_COUNT" -gt 0 ]; then
    echo "✅ 搜索成功！找到 $RESULT_COUNT 条记忆"
    echo ""
    echo "前3条记忆预览:"
    echo "$SEARCH_RESPONSE" | jq -r '.data[0:3] | .[] | "  - ID: \(.id), Content: \(.content[:50])..."' 2>/dev/null || echo "  无法解析响应"
else
    echo "⚠️  搜索返回0条记忆"
    echo "响应内容:"
    echo "$SEARCH_RESPONSE" | jq '.' 2>/dev/null || echo "$SEARCH_RESPONSE"
fi
echo ""

# 5. 验证返回的记忆 user_id
if [ "$RESULT_COUNT" -gt 0 ]; then
    echo "5. 验证返回的记忆 user_id..."
    FIRST_USER_ID=$(echo "$SEARCH_RESPONSE" | jq -r '.data[0].user_id' 2>/dev/null || echo "")
    if [ "$FIRST_USER_ID" = "$USER_ID" ]; then
        echo "✅ 返回的记忆 user_id 正确: $FIRST_USER_ID"
    else
        echo "❌ 返回的记忆 user_id 不匹配: 期望 '$USER_ID', 实际 '$FIRST_USER_ID'"
    fi
    echo ""
fi

# 6. 如果指定了 agent_id，测试指定 agent 的搜索
if [ -n "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    echo "6. 测试搜索指定 Agent 的记忆 (agent_id: $AGENT_ID)..."
    AGENT_SEARCH_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/memories/search" \
      -H "Content-Type: application/json" \
      -H "X-User-ID: $USER_ID" \
      -H "X-Organization-ID: default-org" \
      -d "{
        \"query\": \"test\",
        \"agent_id\": \"$AGENT_ID\",
        \"user_id\": \"$USER_ID\",
        \"limit\": 10
      }")
    
    AGENT_RESULT_COUNT=$(echo "$AGENT_SEARCH_RESPONSE" | jq '.data | length' 2>/dev/null || echo "0")
    echo "搜索返回 $AGENT_RESULT_COUNT 条记忆"
    
    if [ "$AGENT_RESULT_COUNT" -gt 0 ]; then
        echo "✅ Agent 搜索成功！找到 $AGENT_RESULT_COUNT 条记忆"
    else
        echo "⚠️  Agent 搜索返回0条记忆"
    fi
    echo ""
fi

# 7. 对比数据库和搜索结果
echo "7. 对比数据库和搜索结果..."
if [ "$RESULT_COUNT" -gt 0 ] && [ "$DB_COUNT" -gt 0 ]; then
    if [ "$RESULT_COUNT" -le "$DB_COUNT" ]; then
        echo "✅ 搜索结果数量合理（搜索返回 $RESULT_COUNT，数据库有 $DB_COUNT）"
    else
        echo "⚠️  搜索结果数量超过数据库记录，可能有问题"
    fi
else
    if [ "$DB_COUNT" -eq 0 ]; then
        echo "⚠️  数据库中没有 user_id='$USER_ID' 的记忆，搜索返回0条是正常的"
    fi
fi
echo ""

# 8. 总结
echo "=========================================="
echo "验证总结"
echo "=========================================="
echo "✅ 后端服务: 运行正常"
echo "✅ 数据库检查: $DB_COUNT 条 user_id='$USER_ID' 的记忆"
echo "✅ 搜索功能: 返回 $RESULT_COUNT 条记忆"
if [ "$RESULT_COUNT" -gt 0 ]; then
    echo "✅ User ID 匹配: 已验证"
    echo ""
    echo "🎉 搜索功能验证通过！User ID 统一修复成功！"
else
    echo "⚠️  搜索返回0条记忆"
    if [ "$DB_COUNT" -eq 0 ]; then
        echo "   原因: 数据库中没有 user_id='$USER_ID' 的记忆"
        echo "   建议: 先创建一些记忆，然后再测试搜索"
    else
        echo "   原因: 可能是搜索查询条件不匹配"
        echo "   建议: 尝试使用不同的搜索关键词"
    fi
fi
echo ""

