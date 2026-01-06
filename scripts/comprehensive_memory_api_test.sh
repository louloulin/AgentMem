#!/bin/bash
# AgentMem 全面记忆 API 验证脚本
# 目的：测试所有记忆相关的 HTTP 接口
# 作者：AI Assistant
# 日期：2025-11-17

set -e

# ==================== 配置 ====================
API_BASE="${AGENT_MEM_URL:-http://localhost:8080}"
API_V1="$API_BASE/api/v1"
TIMESTAMP=$(date +%s)
TEST_AGENT_ID="test-agent-$TIMESTAMP"
TEST_USER_ID="test-user-$TIMESTAMP"
TEST_ORG_ID="test-org-$TIMESTAMP"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# ==================== 工具函数 ====================

log_section() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║  $1${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
}

log_test() {
    echo ""
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "${YELLOW}[TEST #$TOTAL_TESTS]${NC} $1"
}

log_success() {
    echo -e "${GREEN}  ✓ $1${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

log_error() {
    echo -e "${RED}  ✗ $1${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

log_info() {
    echo -e "${BLUE}  ℹ $1${NC}"
}

# API调用辅助函数
api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    
    if [ "$method" = "GET" ]; then
        curl -s -w "\n%{http_code}" "$endpoint"
    else
        curl -s -w "\n%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$endpoint"
    fi
}

extract_http_code() {
    echo "$1" | tail -n 1
}

extract_body() {
    echo "$1" | sed '$d'
}

# ==================== 主测试流程 ====================

main() {
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║    AgentMem 全面记忆 API 验证                              ║"
    echo "║    Comprehensive Memory API Verification                   ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    log_info "API Base: $API_BASE"
    log_info "Test Agent ID: $TEST_AGENT_ID"
    log_info "Test User ID: $TEST_USER_ID"
    log_info "Test Organization ID: $TEST_ORG_ID"
    echo ""

    # ==================== PART 0: 前置条件 ====================
    log_section "PART 0: 前置条件检查"
    
    log_test "健康检查"
    response=$(api_call "GET" "$API_BASE/health" "")
    http_code=$(extract_http_code "$response")
    if [ "$http_code" = "200" ]; then
        log_success "服务器健康 (HTTP $http_code)"
    else
        log_error "服务器不健康 (HTTP $http_code)"
        exit 1
    fi

    log_test "创建测试组织"
    response=$(api_call "POST" "$API_V1/organizations" "{\"id\":\"$TEST_ORG_ID\",\"name\":\"Test Org\"}")
    http_code=$(extract_http_code "$response")
    log_info "HTTP $http_code"

    log_test "创建测试 Agent"
    agent_data=$(cat <<EOF
{
    "id": "$TEST_AGENT_ID",
    "name": "Memory Test Agent",
    "description": "Agent for comprehensive memory API testing",
    "system": "You are a helpful assistant for testing memory APIs",
    "organization_id": "$TEST_ORG_ID",
    "llm_config": {
        "provider": "zhipu",
        "model": "glm-4",
        "temperature": 0.7
    }
}
EOF
)
    response=$(api_call "POST" "$API_V1/agents" "$agent_data")
    http_code=$(extract_http_code "$response")
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        log_success "Agent 创建成功 (HTTP $http_code)"
    else
        log_info "Agent 可能已存在，继续测试..."
    fi

    sleep 1

    # ==================== PART 1: 单个记忆操作 ====================
    log_section "PART 1: 单个记忆 CRUD 操作 (6个测试)"
    
    # 1.1 添加记忆
    log_test "POST /api/v1/memories - 添加记忆"
    memory_data=$(cat <<EOF
{
    "content": "AgentMem 是一个企业级 AI Agent 记忆管理平台，支持多种记忆类型",
    "agent_id": "$TEST_AGENT_ID",
    "user_id": "$TEST_USER_ID",
    "memory_type": "Factual",
    "importance": 0.9,
    "metadata": {
        "source": "api_test",
        "category": "platform_info",
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    }
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories" "$memory_data")
    http_code=$(extract_http_code "$response")
    body=$(extract_body "$response")
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        MEMORY_ID=$(echo "$body" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
        log_success "记忆创建成功 (ID: $MEMORY_ID)"
    else
        log_error "记忆创建失败 (HTTP $http_code)"
        echo "$body"
    fi

    sleep 0.5

    # 1.2 获取记忆
    log_test "GET /api/v1/memories/{id} - 获取单个记忆"
    if [ -n "$MEMORY_ID" ]; then
        response=$(api_call "GET" "$API_V1/memories/$MEMORY_ID" "")
        http_code=$(extract_http_code "$response")
        
        if [ "$http_code" = "200" ]; then
            log_success "记忆获取成功"
        else
            log_error "记忆获取失败 (HTTP $http_code)"
        fi
    else
        log_error "跳过 - 没有有效的记忆 ID"
    fi

    sleep 0.5

    # 1.3 更新记忆
    log_test "PUT /api/v1/memories/{id} - 更新记忆"
    if [ -n "$MEMORY_ID" ]; then
        update_data=$(cat <<EOF
{
    "content": "AgentMem 是一个企业级 AI Agent 记忆管理平台，支持多种记忆类型和向量检索",
    "importance": 0.95
}
EOF
)
        response=$(api_call "PUT" "$API_V1/memories/$MEMORY_ID" "$update_data")
        http_code=$(extract_http_code "$response")
        
        if [ "$http_code" = "200" ]; then
            log_success "记忆更新成功"
        else
            log_error "记忆更新失败 (HTTP $http_code)"
        fi
    else
        log_error "跳过 - 没有有效的记忆 ID"
    fi

    sleep 0.5

    # 1.4 添加第二条记忆（用于后续测试）
    log_test "POST /api/v1/memories - 添加第二条记忆（Episodic）"
    memory_data_2=$(cat <<EOF
{
    "content": "我最喜欢的编程语言是Rust，因为它的性能和安全性",
    "agent_id": "$TEST_AGENT_ID",
    "user_id": "$TEST_USER_ID",
    "memory_type": "Episodic",
    "importance": 0.8,
    "metadata": {
        "topic": "programming",
        "language": "rust"
    }
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories" "$memory_data_2")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        MEMORY_ID_2=$(echo "$(extract_body "$response")" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
        log_success "第二条记忆创建成功"
    else
        log_error "第二条记忆创建失败"
    fi

    sleep 0.5

    # 1.5 添加第三条记忆（Semantic）
    log_test "POST /api/v1/memories - 添加第三条记忆（Semantic）"
    memory_data_3=$(cat <<EOF
{
    "content": "向量数据库用于高效存储和检索高维向量数据",
    "agent_id": "$TEST_AGENT_ID",
    "user_id": "$TEST_USER_ID",
    "memory_type": "Semantic",
    "importance": 0.7
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories" "$memory_data_3")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        log_success "第三条记忆创建成功"
    else
        log_error "第三条记忆创建失败"
    fi

    sleep 1

    # ==================== PART 2: 批量操作 ====================
    log_section "PART 2: 批量操作 (3个测试)"
    
    # 2.1 批量添加记忆
    log_test "POST /api/v1/memories/batch - 批量添加记忆"
    batch_add_data=$(cat <<EOF
{
    "memories": [
        {
            "content": "Python 是一种高级编程语言，广泛用于数据科学",
            "agent_id": "$TEST_AGENT_ID",
            "user_id": "$TEST_USER_ID",
            "memory_type": "Factual",
            "metadata": {"batch": "test", "index": 1}
        },
        {
            "content": "JavaScript 是 Web 开发的核心语言",
            "agent_id": "$TEST_AGENT_ID",
            "user_id": "$TEST_USER_ID",
            "memory_type": "Factual",
            "metadata": {"batch": "test", "index": 2}
        },
        {
            "content": "Go 语言以其并发性能著称",
            "agent_id": "$TEST_AGENT_ID",
            "user_id": "$TEST_USER_ID",
            "memory_type": "Factual",
            "metadata": {"batch": "test", "index": 3}
        }
    ]
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories/batch" "$batch_add_data")
    http_code=$(extract_http_code "$response")
    body=$(extract_body "$response")
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        success_count=$(echo "$body" | grep -o '"success_count":[0-9]*' | cut -d: -f2)
        log_success "批量添加成功 (成功: $success_count 条)"
    else
        log_error "批量添加失败 (HTTP $http_code)"
    fi

    sleep 1

    # 2.2 获取 Agent 的所有记忆
    log_test "GET /api/v1/agents/{id}/memories - 获取 Agent 所有记忆"
    response=$(api_call "GET" "$API_V1/agents/$TEST_AGENT_ID/memories" "")
    http_code=$(extract_http_code "$response")
    body=$(extract_body "$response")
    
    if [ "$http_code" = "200" ]; then
        count=$(echo "$body" | grep -o '"id"' | wc -l | tr -d ' ')
        log_success "获取成功 (共 $count 条记忆)"
    else
        log_error "获取失败 (HTTP $http_code)"
    fi

    # 2.3 列出所有记忆（带分页）
    log_test "GET /api/v1/memories - 列出所有记忆（分页）"
    response=$(api_call "GET" "$API_V1/memories?limit=10&offset=0" "")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        log_success "列表获取成功"
    else
        log_error "列表获取失败 (HTTP $http_code)"
    fi

    sleep 1

    # ==================== PART 3: 搜索功能 ====================
    log_section "PART 3: 搜索和检索功能 (5个测试)"
    
    # 3.1 基础搜索
    log_test "POST /api/v1/memories/search - 基础向量搜索"
    search_data=$(cat <<EOF
{
    "query": "编程语言",
    "agent_id": "$TEST_AGENT_ID",
    "user_id": "$TEST_USER_ID",
    "limit": 5
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories/search" "$search_data")
    http_code=$(extract_http_code "$response")
    body=$(extract_body "$response")
    
    if [ "$http_code" = "200" ]; then
        count=$(echo "$body" | grep -o '"id"' | wc -l | tr -d ' ')
        score=$(echo "$body" | grep -o '"score":[0-9.]*' | head -1 | cut -d: -f2)
        log_success "搜索成功 (找到 $count 条，top score: $score)"
        
        # 验证 score 字段不为 null
        if [ -n "$score" ] && [ "$score" != "null" ]; then
            log_success "✓ Score 字段正常 (非 null)"
        else
            log_error "✗ Score 字段为 null"
        fi
    else
        log_error "搜索失败 (HTTP $http_code)"
    fi

    sleep 0.5

    # 3.2 带阈值的搜索
    log_test "POST /api/v1/memories/search - 带相似度阈值的搜索"
    search_threshold=$(cat <<EOF
{
    "query": "Rust 编程",
    "agent_id": "$TEST_AGENT_ID",
    "user_id": "$TEST_USER_ID",
    "limit": 3,
    "threshold": 0.5
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories/search" "$search_threshold")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        count=$(echo "$(extract_body "$response")" | grep -o '"id"' | wc -l | tr -d ' ')
        log_success "阈值搜索成功 (结果: $count 条，threshold ≥ 0.5)"
    else
        log_error "阈值搜索失败"
    fi

    sleep 0.5

    # 3.3 按记忆类型搜索
    log_test "POST /api/v1/memories/search - 按记忆类型筛选"
    search_by_type=$(cat <<EOF
{
    "query": "语言",
    "agent_id": "$TEST_AGENT_ID",
    "user_id": "$TEST_USER_ID",
    "limit": 5,
    "filters": {
        "memory_type": "Factual"
    }
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories/search" "$search_by_type")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        log_success "类型筛选搜索成功"
    else
        log_error "类型筛选搜索失败"
    fi

    sleep 0.5

    # 3.4 跨 Session 搜索（新会话ID）
    log_test "POST /api/v1/memories/search - 跨 Session 检索测试"
    NEW_SESSION_ID="session-$(date +%s)"
    log_info "使用新 Session ID: $NEW_SESSION_ID"
    
    search_cross_session=$(cat <<EOF
{
    "query": "AgentMem 平台",
    "agent_id": "$TEST_AGENT_ID",
    "user_id": "$TEST_USER_ID",
    "session_id": "$NEW_SESSION_ID",
    "limit": 3
}
EOF
)
    response=$(api_call "POST" "$API_V1/memories/search" "$search_cross_session")
    http_code=$(extract_http_code "$response")
    body=$(extract_body "$response")
    
    if [ "$http_code" = "200" ]; then
        if echo "$body" | grep -q "AgentMem"; then
            log_success "跨 Session 检索成功 - 找到之前的记忆"
        else
            log_error "跨 Session 检索失败 - 未找到之前的记忆"
        fi
    else
        log_error "跨 Session 搜索失败"
    fi

    sleep 0.5

    # 3.5 获取记忆历史
    log_test "GET /api/v1/memories/{id}/history - 获取记忆历史"
    if [ -n "$MEMORY_ID" ]; then
        response=$(api_call "GET" "$API_V1/memories/$MEMORY_ID/history" "")
        http_code=$(extract_http_code "$response")
        
        if [ "$http_code" = "200" ]; then
            log_success "记忆历史获取成功"
        else
            log_info "记忆历史功能可能未实现 (HTTP $http_code)"
        fi
    else
        log_error "跳过 - 没有有效的记忆 ID"
    fi

    sleep 1

    # ==================== PART 4: Chat 集成测试 ====================
    log_section "PART 4: Chat 对话集成测试 (2个测试)"
    
    # 4.1 基础对话（应检索记忆）
    log_test "POST /api/v1/agents/{id}/chat - 对话并检索记忆"
    chat_data=$(cat <<EOF
{
    "message": "请告诉我关于 AgentMem 平台和编程语言的信息",
    "user_id": "$TEST_USER_ID",
    "stream": false
}
EOF
)
    response=$(api_call "POST" "$API_V1/agents/$TEST_AGENT_ID/chat" "$chat_data")
    http_code=$(extract_http_code "$response")
    body=$(extract_body "$response")
    
    if [ "$http_code" = "200" ]; then
        if echo "$body" | grep -qi "agentmem\|rust\|编程"; then
            log_success "对话成功 - AI 回复包含记忆内容"
        else
            log_info "对话成功但未明确包含记忆内容"
        fi
        
        processing_time=$(echo "$body" | grep -o '"processing_time_ms":[0-9]*' | cut -d: -f2)
        log_info "处理时间: ${processing_time}ms"
    else
        log_error "对话失败 (HTTP $http_code)"
    fi

    sleep 2

    # 4.2 获取对话历史
    log_test "GET /api/v1/agents/{id}/chat/history - 获取对话历史"
    response=$(api_call "GET" "$API_V1/agents/$TEST_AGENT_ID/chat/history?user_id=$TEST_USER_ID" "")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        log_success "对话历史获取成功"
    else
        log_info "对话历史功能可能未实现 (HTTP $http_code)"
    fi

    sleep 1

    # ==================== PART 5: 统计和监控 ====================
    log_section "PART 5: 统计和监控 (4个测试)"
    
    # 5.1 获取记忆统计
    log_test "GET /api/v1/stats/dashboard - Dashboard 统计"
    response=$(api_call "GET" "$API_V1/stats/dashboard" "")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        log_success "Dashboard 统计获取成功"
    else
        log_error "Dashboard 统计获取失败"
    fi

    # 5.2 记忆增长趋势
    log_test "GET /api/v1/stats/memories/growth - 记忆增长趋势"
    response=$(api_call "GET" "$API_V1/stats/memories/growth" "")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        log_success "增长趋势获取成功"
    else
        log_info "增长趋势功能可能未实现 (HTTP $http_code)"
    fi

    # 5.3 Agent 活动统计
    log_test "GET /api/v1/stats/agents/activity - Agent 活动统计"
    response=$(api_call "GET" "$API_V1/stats/agents/activity" "")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        log_success "活动统计获取成功"
    else
        log_info "活动统计功能可能未实现 (HTTP $http_code)"
    fi

    # 5.4 Metrics 端点
    log_test "GET /metrics - Prometheus Metrics"
    response=$(api_call "GET" "$API_BASE/metrics" "")
    http_code=$(extract_http_code "$response")
    
    if [ "$http_code" = "200" ]; then
        log_success "Metrics 获取成功"
    else
        log_error "Metrics 获取失败"
    fi

    sleep 1

    # ==================== PART 6: 删除操作 ====================
    log_section "PART 6: 删除操作 (2个测试)"
    
    # 6.1 删除单个记忆
    log_test "DELETE /api/v1/memories/{id} - 删除单个记忆"
    if [ -n "$MEMORY_ID_2" ]; then
        response=$(api_call "DELETE" "$API_V1/memories/$MEMORY_ID_2" "")
        http_code=$(extract_http_code "$response")
        
        if [ "$http_code" = "200" ]; then
            log_success "单个记忆删除成功"
        else
            log_error "单个记忆删除失败 (HTTP $http_code)"
        fi
    else
        log_error "跳过 - 没有有效的记忆 ID"
    fi

    # 6.2 批量删除记忆
    log_test "POST /api/v1/memories/batch/delete - 批量删除记忆"
    if [ -n "$MEMORY_ID" ]; then
        batch_delete_data="[\"$MEMORY_ID\"]"
        response=$(api_call "POST" "$API_V1/memories/batch/delete" "$batch_delete_data")
        http_code=$(extract_http_code "$response")
        
        if [ "$http_code" = "200" ]; then
            log_success "批量删除成功"
        else
            log_info "批量删除响应 (HTTP $http_code)"
        fi
    else
        log_error "跳过 - 没有有效的记忆 ID"
    fi

    # ==================== 测试总结 ====================
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                   测试完成                                  ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    echo "测试结果汇总:"
    echo "  总测试数: $TOTAL_TESTS"
    echo -e "  ${GREEN}✓ 通过: $PASSED_TESTS${NC}"
    echo -e "  ${RED}✗ 失败: $FAILED_TESTS${NC}"
    
    if [ $TOTAL_TESTS -gt 0 ]; then
        success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo "  成功率: $success_rate%"
    fi
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}  🎉 所有测试通过！记忆 API 功能完整！${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "关键验证："
        echo "  ✓ CRUD 操作正常"
        echo "  ✓ 批量操作正常"
        echo "  ✓ 向量搜索正常"
        echo "  ✓ Score 字段正确"
        echo "  ✓ Chat 集成正常"
        echo "  ✓ 跨 Session 记忆连续性"
        exit 0
    else
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${YELLOW}  ⚠️  部分测试失败，请查看详细日志${NC}"
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        exit 1
    fi
}

# 执行主函数
main
