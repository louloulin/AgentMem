#!/bin/bash
# 测试演示记忆的搜索功能

set -e

API_URL="${API_URL:-http://localhost:8080}"
USER_ID="default"

echo "=========================================="
echo "演示记忆搜索测试"
echo "=========================================="
echo ""

# 测试不同的搜索关键词
SEARCH_KEYWORDS=(
    "张明"
    "意大利"
    "编程"
    "咖啡"
    "李华"
    "日本"
    "Rust"
)

for keyword in "${SEARCH_KEYWORDS[@]}"; do
    echo "搜索关键词: '$keyword'"
    echo "----------------------------------------"
    
    RESPONSE=$(curl -s -X POST "$API_URL/api/v1/memories/search" \
      -H "Content-Type: application/json" \
      -H "X-User-ID: $USER_ID" \
      -H "X-Organization-ID: default-org" \
      -d "{
        \"query\": \"$keyword\",
        \"user_id\": \"$USER_ID\",
        \"limit\": 5
      }")
    
    COUNT=$(echo "$RESPONSE" | jq '.data | length' 2>/dev/null || echo "0")
    
    if [ "$COUNT" -gt 0 ]; then
        echo "✅ 找到 $COUNT 条相关记忆:"
        echo "$RESPONSE" | jq -r '.data[] | "  - \(.content[:70])..."' 2>/dev/null || echo "  无法解析响应"
    else
        echo "⚠️  未找到相关记忆"
    fi
    echo ""
done

echo "=========================================="
echo "测试完成"
echo "=========================================="
echo ""
echo "🎯 现在可以在 UI 中查看这些记忆："
echo "   http://localhost:3001/admin/memories"
echo ""
echo "💬 在 Chat 中测试对话："
echo "   http://localhost:3001/admin/chat"
echo ""

