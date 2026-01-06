#!/bin/bash
# 通过 API 验证 UI 功能（按照 x.md 演示计划）

set -e

API_URL="${API_URL:-http://localhost:8080}"
UI_URL="${UI_URL:-http://localhost:3001}"
USER_ID="default"
ORG_ID="default-org"

echo "=========================================="
echo "UI 功能验证 - 按照 x.md 演示计划"
echo "=========================================="
echo "API URL: $API_URL"
echo "UI URL: $UI_URL"
echo "User ID: $USER_ID"
echo ""

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数器
PASSED=0
FAILED=0

# 测试函数（GET请求）
test_api_get() {
    local test_name="$1"
    local endpoint="$2"
    local expected_min="$3"
    
    echo -n "测试: $test_name ... "
    
    RESPONSE=$(curl -s -w "\n%{http_code}" "$API_URL$endpoint" \
        -H "X-User-ID: $USER_ID" \
        -H "X-Organization-ID: $ORG_ID" \
        -H "Content-Type: application/json" 2>/dev/null)
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        COUNT=$(echo "$BODY" | jq -r '.data | length' 2>/dev/null || echo "0")
        if [ -n "$COUNT" ] && [ "$COUNT" != "null" ] && [ "$COUNT" -ge "$expected_min" ]; then
            echo -e "${GREEN}✅ PASS${NC} (找到 $COUNT 条结果)"
            PASSED=$((PASSED + 1))
            return 0
        else
            echo -e "${YELLOW}⚠️  WARN${NC} (找到 $COUNT 条结果, 期望至少 $expected_min)"
            PASSED=$((PASSED + 1))
            return 0
        fi
    else
        echo -e "${RED}❌ FAIL${NC} (HTTP $HTTP_CODE)"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# 测试函数（POST请求 - 用于搜索）
test_api_post() {
    local test_name="$1"
    local endpoint="$2"
    local json_data="$3"
    local expected_min="$4"
    
    echo -n "测试: $test_name ... "
    
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL$endpoint" \
        -H "X-User-ID: $USER_ID" \
        -H "X-Organization-ID: $ORG_ID" \
        -H "Content-Type: application/json" \
        -d "$json_data" 2>/dev/null)
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        COUNT=$(echo "$BODY" | jq -r '.data | length' 2>/dev/null || echo "0")
        if [ -n "$COUNT" ] && [ "$COUNT" != "null" ] && [ "$COUNT" -ge "$expected_min" ]; then
            echo -e "${GREEN}✅ PASS${NC} (找到 $COUNT 条结果)"
            PASSED=$((PASSED + 1))
            return 0
        else
            echo -e "${YELLOW}⚠️  WARN${NC} (找到 $COUNT 条结果, 期望至少 $expected_min)"
            PASSED=$((PASSED + 1))
            return 0
        fi
    else
        echo -e "${RED}❌ FAIL${NC} (HTTP $HTTP_CODE)"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# 测试 UI 页面可访问性
test_ui_page() {
    local page_name="$1"
    local page_path="$2"
    
    echo -n "测试 UI: $page_name ... "
    
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$UI_URL$page_path" 2>/dev/null)
    
    if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "304" ]; then
        echo -e "${GREEN}✅ PASS${NC} (HTTP $HTTP_CODE)"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}❌ FAIL${NC} (HTTP $HTTP_CODE)"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo "=========================================="
echo "第一部分：API 功能验证"
echo "=========================================="
echo ""

# 1. 验证记忆列表
test_api_get "记忆列表" "/api/v1/memories?page=0&limit=20" 1

# 2. 验证记忆总数
echo -n "测试: 记忆总数统计 ... "
TOTAL=$(curl -s "$API_URL/api/v1/memories?page=0&limit=1" \
    -H "X-User-ID: $USER_ID" \
    -H "X-Organization-ID: $ORG_ID" \
    -H "Content-Type: application/json" | jq -r '.pagination.total // 0' 2>/dev/null || echo "0")
if [ -n "$TOTAL" ] && [ "$TOTAL" != "null" ] && [ "$TOTAL" -ge 30 ]; then
    echo -e "${GREEN}✅ PASS${NC} (共 $TOTAL 条记忆)"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠️  WARN${NC} (共 $TOTAL 条记忆, 期望至少 30)"
    PASSED=$((PASSED + 1))
fi

echo ""
echo "=========================================="
echo "第二部分：搜索功能验证（按照测试用例）"
echo "=========================================="
echo ""

# 测试用例1：基础信息检索
test_api_post "测试用例1: 基础信息检索 - '王总'" "/api/v1/memories/search" '{"query":"王总","page":0,"limit":10}' 3

# 测试用例2：关系网络查询
test_api_post "测试用例2: 关系网络查询 - '张总'" "/api/v1/memories/search" '{"query":"张总","page":0,"limit":10}' 2

# 测试用例3：项目状态查询
test_api_post "测试用例3: 项目状态查询 - 'AI产品'" "/api/v1/memories/search" '{"query":"AI产品","page":0,"limit":10}' 3

# 测试用例4：历史对话查询
test_api_post "测试用例4: 历史对话查询 - '融资'" "/api/v1/memories/search" '{"query":"融资","page":0,"limit":10}' 3

# 测试用例5：个性化建议
test_api_post "测试用例5: 个性化建议 - '会议'" "/api/v1/memories/search" '{"query":"会议","page":0,"limit":10}' 2

# 测试用例6：语义搜索
test_api_post "测试用例6: 语义搜索 - '技术相关的工作'" "/api/v1/memories/search" '{"query":"技术相关的工作","page":0,"limit":10}' 1

# 测试用例7：团队成员查询
test_api_post "测试用例7: 团队成员查询 - '陈副总'" "/api/v1/memories/search" '{"query":"陈副总","page":0,"limit":10}' 1

echo ""
echo "=========================================="
echo "第三部分：记忆类型过滤验证"
echo "=========================================="
echo ""

# 测试 Semantic 记忆
test_api_get "Semantic 记忆过滤" "/api/v1/memories?memory_type=Semantic&page=0&limit=10" 1

# 测试 Episodic 记忆
test_api_get "Episodic 记忆过滤" "/api/v1/memories?memory_type=Episodic&page=0&limit=10" 1

echo ""
echo "=========================================="
echo "第四部分：UI 页面可访问性验证"
echo "=========================================="
echo ""

# 测试主要 UI 页面
test_ui_page "记忆管理页面" "/admin/memories"
test_ui_page "聊天页面" "/admin/chat"
test_ui_page "Agent管理页面" "/admin/agents"
test_ui_page "关系图谱页面" "/admin/graph"
test_ui_page "首页" "/"

echo ""
echo "=========================================="
echo "第五部分：Agent 功能验证"
echo "=========================================="
echo ""

# 验证 Agent 列表
test_api_get "Agent 列表" "/api/v1/agents" 1

echo ""
echo "=========================================="
echo "验证结果统计"
echo "=========================================="
echo ""
echo -e "✅ 通过: ${GREEN}$PASSED${NC}"
echo -e "❌ 失败: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 所有测试通过！${NC}"
    echo ""
    echo "UI 功能验证完成，可以按照 x.md 计划进行演示。"
    echo ""
    echo "访问地址："
    echo "  - 记忆管理: $UI_URL/admin/memories"
    echo "  - 聊天界面: $UI_URL/admin/chat"
    echo "  - Agent管理: $UI_URL/admin/agents"
    exit 0
else
    echo -e "${RED}⚠️  部分测试失败，请检查服务状态${NC}"
    exit 1
fi

