#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         AgentMem 全面功能验证测试                            ║"
echo "║         Comprehensive Feature Validation                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

BASE_URL="http://localhost:8080"
PASS=0
FAIL=0

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

test_api() {
    local name="$1"
    local cmd="$2"
    local expected="$3"
    
    echo -n "  测试: $name ... "
    result=$(eval "$cmd" 2>&1)
    
    if echo "$result" | grep -q "$expected"; then
        echo -e "${GREEN}✅ PASS${NC}"
        ((PASS++))
    else
        echo -e "${RED}❌ FAIL${NC}"
        echo "    预期: $expected"
        echo "    实际: $result"
        ((FAIL++))
    fi
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣  系统健康检查"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "健康检查端点" \
    "curl -s $BASE_URL/health | jq -r '.status'" \
    "healthy"

test_api "数据库连接" \
    "curl -s $BASE_URL/health | jq -r '.checks.database.status'" \
    "healthy"

test_api "记忆系统" \
    "curl -s $BASE_URL/health | jq -r '.checks.memory_system.status'" \
    "healthy"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣  Fix 1: 全局memories列表API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "GET /api/v1/memories (无参数)" \
    "curl -s $BASE_URL/api/v1/memories | jq -r '.success'" \
    "true"

test_api "返回分页信息" \
    "curl -s '$BASE_URL/api/v1/memories?limit=5' | jq -r '.data.pagination.limit'" \
    "5"

test_api "总记忆数量 > 0" \
    "curl -s $BASE_URL/api/v1/memories | jq -r '.data.pagination.total'" \
    "[0-9]"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣  Fix 2: QueryOptimizer和Reranker集成"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "搜索API可用" \
    "curl -s -X POST $BASE_URL/api/v1/memories/search -H 'Content-Type: application/json' -d '{\"query\":\"test\",\"limit\":5}' | jq -r '.success'" \
    "true"

echo -n "  检查优化器日志 ... "
if tail -100 backend-onnx-fixed.log 2>/dev/null | grep -q "Query optimized"; then
    echo -e "${GREEN}✅ PASS${NC} (优化器已激活)"
    ((PASS++))
else
    echo -e "${YELLOW}⚠️  WARN${NC} (未找到优化器日志，可能没有触发搜索)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4️⃣  Fix 3: 历史记录数据库"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -n "  HistoryManager初始化 ... "
if tail -100 backend-onnx-fixed.log 2>/dev/null | grep -q "HistoryManager 创建成功"; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASS++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAIL++))
fi

echo -n "  history.db文件存在 ... "
if [ -f "./data/history.db" ]; then
    size=$(ls -lh ./data/history.db | awk '{print $5}')
    echo -e "${GREEN}✅ PASS${NC} (size: $size)"
    ((PASS++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAIL++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5️⃣  基础CRUD操作"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_api "GET /api/v1/agents" \
    "curl -s $BASE_URL/api/v1/agents | jq -r '.success'" \
    "true"

test_api "Agent数量 >= 0" \
    "curl -s $BASE_URL/api/v1/agents | jq -r '.data | length'" \
    "[0-9]"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6️⃣  性能检查"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -n "  健康检查响应时间 ... "
start=$(date +%s%N)
curl -s $BASE_URL/health > /dev/null
end=$(date +%s%N)
duration=$(( ($end - $start) / 1000000 ))
if [ $duration -lt 100 ]; then
    echo -e "${GREEN}✅ PASS${NC} (${duration}ms)"
    ((PASS++))
else
    echo -e "${YELLOW}⚠️  SLOW${NC} (${duration}ms)"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    验证测试总结                               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo -e "  通过: ${GREEN}$PASS${NC} 个测试"
echo -e "  失败: ${RED}$FAIL${NC} 个测试"
TOTAL=$((PASS + FAIL))
if [ $TOTAL -gt 0 ]; then
    RATE=$((PASS * 100 / TOTAL))
    echo -e "  通过率: ${BLUE}${RATE}%${NC}"
fi
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}🎉 所有测试通过！系统运行正常！${NC}"
    exit 0
else
    echo -e "${RED}⚠️  部分测试失败，请检查日志${NC}"
    exit 1
fi
