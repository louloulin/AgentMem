#!/bin/bash
# AgentMem 核心 API 功能验证脚本
# 用途：验证服务器启动后的核心 API 端点是否正常工作

set -e

# 配置
BASE_URL="${AGENT_MEM_URL:-http://localhost:8080}"
API_VERSION="v1"
API_BASE="$BASE_URL/api/$API_VERSION"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# 测试函数
test_endpoint() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local data="$4"
    local expected_status="${5:-200}"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "测试 #$TOTAL_TESTS: $name"
    echo "  方法: $method"
    echo "  端点: $endpoint"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$endpoint")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$endpoint")
    fi
    
    # 分离响应体和状态码
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    echo "  状态码: $http_code (期望: $expected_status)"
    
    if [ "$http_code" = "$expected_status" ]; then
        log_info "✅ 通过"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "  响应: $(echo "$body" | jq -C '.' 2>/dev/null || echo "$body" | head -c 200)"
        return 0
    else
        log_error "❌ 失败"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "  响应: $body"
        return 1
    fi
}

# 主测试流程
main() {
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║         AgentMem 核心 API 功能验证                         ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    log_info "目标服务器: $BASE_URL"
    echo ""
    
    # 1. Health Check
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "第一部分: Health & Monitoring (3个测试)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    test_endpoint "Health Check" "GET" "$BASE_URL/health" "" "200"
    test_endpoint "Liveness Check" "GET" "$BASE_URL/health/live" "" "200"
    test_endpoint "Readiness Check" "GET" "$BASE_URL/health/ready" "" "200"
    
    # 2. Metrics
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "第二部分: Metrics (2个测试)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    test_endpoint "Get Metrics" "GET" "$BASE_URL/metrics" "" "200"
    test_endpoint "Prometheus Metrics" "GET" "$BASE_URL/metrics/prometheus" "" "200"
    
    # 3. OpenAPI Documentation
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "第三部分: API Documentation (1个测试)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    test_endpoint "OpenAPI Spec" "GET" "$BASE_URL/api-docs/openapi.json" "" "200"
    
    # 4. User Management (需要认证的测试跳过)
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "第四部分: User Management (跳过 - 需要认证)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_warn "用户管理 API 需要认证，在集成测试中验证"
    
    # 5. Memory Management (核心功能)
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "第五部分: Memory Management - 核心功能 (4个测试)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 创建 Memory
    memory_data='{
        "content": "测试记忆：AgentMem 是一个企业级 AI Agent 记忆管理平台",
        "metadata": {
            "source": "api_test",
            "category": "test"
        }
    }'
    
    test_endpoint "Create Memory" "POST" "$API_BASE/memories" "$memory_data" "201" || true
    
    # 搜索 Memory
    search_data='{
        "query": "AgentMem",
        "limit": 10
    }'
    
    test_endpoint "Search Memories" "POST" "$API_BASE/memories/search" "$search_data" "200" || true
    
    # 批量创建
    batch_data='{
        "memories": [
            {
                "content": "批量测试记忆 1",
                "metadata": {"batch": "test"}
            },
            {
                "content": "批量测试记忆 2",
                "metadata": {"batch": "test"}
            }
        ]
    }'
    
    test_endpoint "Batch Create Memories" "POST" "$API_BASE/memories/batch" "$batch_data" "201" || true
    
    # 6. Stats & Dashboard
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "第六部分: Statistics & Dashboard (3个测试)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    test_endpoint "Dashboard Stats" "GET" "$API_BASE/stats/dashboard" "" "200" || true
    test_endpoint "Memory Growth" "GET" "$API_BASE/stats/memories/growth" "" "200" || true
    test_endpoint "Agent Activity" "GET" "$API_BASE/stats/agents/activity" "" "200" || true
    
    # 7. MCP Server
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_info "第七部分: MCP Server (2个测试)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    test_endpoint "MCP Server Info" "GET" "$API_BASE/mcp/info" "" "200" || true
    test_endpoint "MCP Health Check" "GET" "$API_BASE/mcp/health" "" "200" || true
    
    # 最终报告
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                    测试结果汇总                             ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  总测试数: $TOTAL_TESTS"
    echo "  ✅ 通过: $PASSED_TESTS"
    echo "  ❌ 失败: $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        log_info "🎉 所有测试通过！"
        echo ""
        echo "下一步建议："
        echo "  1. 访问 Swagger UI: $BASE_URL/swagger-ui"
        echo "  2. 查看 API 文档: $BASE_URL/api-docs/openapi.json"
        echo "  3. 运行集成测试: cargo test --workspace"
        exit 0
    else
        log_error "⚠️  有 $FAILED_TESTS 个测试失败"
        echo ""
        echo "故障排查建议："
        echo "  1. 检查服务器是否正在运行: curl $BASE_URL/health"
        echo "  2. 查看服务器日志"
        echo "  3. 验证数据库连接"
        exit 1
    fi
}

# 运行主函数
main

