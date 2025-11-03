#!/bin/bash

echo "=========================================="
echo "  AgentMem 修复验证测试"
echo "=========================================="
echo ""

BASE_URL="http://localhost:8080"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "📋 测试 Fix 1: 全局memories列表API"
echo "----------------------------------------"

# 测试1: GET /api/v1/memories (无参数)
echo -n "1️⃣  GET /api/v1/memories (全局列表): "
response=$(curl -s "$BASE_URL/api/v1/memories")
success=$(echo "$response" | jq -r '.success')
total=$(echo "$response" | jq -r '.data.pagination.total')
if [ "$success" = "true" ]; then
    echo -e "${GREEN}✅ 成功${NC} (total: $total)"
else
    echo -e "${RED}❌ 失败${NC}"
fi

# 测试2: 带分页参数
echo -n "2️⃣  GET /api/v1/memories?limit=5&page=0: "
response=$(curl -s "$BASE_URL/api/v1/memories?limit=5&page=0")
success=$(echo "$response" | jq -r '.success')
count=$(echo "$response" | jq -r '.data.memories | length')
if [ "$success" = "true" ] && [ "$count" -le "5" ]; then
    echo -e "${GREEN}✅ 成功${NC} (returned: $count)"
else
    echo -e "${RED}❌ 失败${NC}"
fi

echo ""
echo "🔍 测试 Fix 2: QueryOptimizer集成"
echo "----------------------------------------"

# 测试3: 搜索API（会触发QueryOptimizer）
echo -n "3️⃣  POST /api/v1/memories/search (测试优化器): "
search_response=$(curl -s -X POST "$BASE_URL/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "test query", "limit": 10}')
search_success=$(echo "$search_response" | jq -r '.success')
if [ "$search_success" = "true" ]; then
    echo -e "${GREEN}✅ 成功${NC}"
    # 检查日志中是否有优化器日志
    echo -n "   检查优化器日志: "
    if tail -50 backend-onnx-fixed.log | grep -q "Query optimized"; then
        echo -e "${GREEN}✅ 优化器已激活${NC}"
    else
        echo -e "${YELLOW}⚠️  未找到优化器日志${NC}"
    fi
else
    echo -e "${RED}❌ 失败${NC}"
fi

echo ""
echo "💾 测试 Fix 3: 历史记录数据库"
echo "----------------------------------------"

echo -n "4️⃣  检查 HistoryManager 初始化: "
if tail -100 backend-onnx-fixed.log | grep -q "✅ HistoryManager 创建成功"; then
    echo -e "${GREEN}✅ 成功${NC}"
else
    echo -e "${RED}❌ 失败${NC}"
fi

echo -n "5️⃣  检查 history.db 文件: "
if [ -f "./data/history.db" ]; then
    size=$(ls -lh ./data/history.db | awk '{print $5}')
    echo -e "${GREEN}✅ 存在${NC} (size: $size)"
else
    echo -e "${RED}❌ 不存在${NC}"
fi

echo ""
echo "🏥 健康检查"
echo "----------------------------------------"

echo -n "6️⃣  GET /health: "
health=$(curl -s "$BASE_URL/health")
health_status=$(echo "$health" | jq -r '.status')
if [ "$health_status" = "healthy" ]; then
    echo -e "${GREEN}✅ 健康${NC}"
else
    echo -e "${RED}❌ 不健康${NC}"
fi

echo ""
echo "=========================================="
echo "  测试完成"
echo "=========================================="
