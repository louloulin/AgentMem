#!/bin/bash
# 测试 AgentMem 记忆功能 API

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║         测试 AgentMem 记忆功能 API                              ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

BASE_URL="http://localhost:8080"

echo "1️⃣  测试添加记忆（默认智能功能 infer=true）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -X POST "$BASE_URL/v1/memories" \
  -H "Content-Type: application/json" \
  -d '{
    "messages": "我喜欢吃披萨和意大利面",
    "user_id": "alice"
  }' | jq '.' 2>/dev/null || echo "添加记忆 1"
echo ""
echo ""

echo "2️⃣  测试添加记忆（组织级 Organization Scope）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -X POST "$BASE_URL/v1/memories" \
  -H "Content-Type: application/json" \
  -d '{
    "messages": "公司政策：每周五远程办公",
    "metadata": {"org_id": "acme-corp"}
  }' | jq '.' 2>/dev/null || echo "添加记忆 2"
echo ""
echo ""

echo "3️⃣  测试搜索记忆"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -X POST "$BASE_URL/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "我喜欢什么食物？",
    "user_id": "alice",
    "limit": 5
  }' | jq '.results[0:3]' 2>/dev/null || echo "搜索记忆"
echo ""
echo ""

echo "4️⃣  测试获取所有记忆"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -X GET "$BASE_URL/v1/memories?user_id=alice" | jq '.memories | length' 2>/dev/null || echo "获取记忆数量"
echo ""
echo ""

echo "✅ API 测试完成！"
echo ""
echo "🌐 打开 UI: open http://localhost:3001"
