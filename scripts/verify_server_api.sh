#!/bin/bash

# AgentMem 后端 API 验证脚本
# 验证所有关键 API 端点

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
BACKEND_URL="http://localhost:8080"
TEST_USER_ID="api-test-user-$(date +%s)"
TEST_ORG_ID="default-org"

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 测试结果
PASSED=0
FAILED=0
WARNINGS=0

# 测试函数
test_endpoint() {
    local name=$1
    local method=$2
    local url=$3
    local data=$4
    local expected_status=${5:-200}
    
    log_info "测试: $name"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" -X GET "$BACKEND_URL$url" \
            -H "X-User-ID: $TEST_USER_ID" \
            -H "X-Organization-ID: $TEST_ORG_ID" \
            -H "Content-Type: application/json" 2>&1)
    elif [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "$BACKEND_URL$url" \
            -H "X-User-ID: $TEST_USER_ID" \
            -H "X-Organization-ID: $TEST_ORG_ID" \
            -H "Content-Type: application/json" \
            -d "$data" 2>&1)
    elif [ "$method" = "PUT" ]; then
        response=$(curl -s -w "\n%{http_code}" -X PUT "$BACKEND_URL$url" \
            -H "X-User-ID: $TEST_USER_ID" \
            -H "X-Organization-ID: $TEST_ORG_ID" \
            -H "Content-Type: application/json" \
            -d "$data" 2>&1)
    elif [ "$method" = "DELETE" ]; then
        response=$(curl -s -w "\n%{http_code}" -X DELETE "$BACKEND_URL$url" \
            -H "X-User-ID: $TEST_USER_ID" \
            -H "X-Organization-ID: $TEST_ORG_ID" \
            -H "Content-Type: application/json" 2>&1)
    fi
    
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected_status" ]; then
        log_success "$name - HTTP $http_code"
        ((PASSED++))
        return 0
    else
        log_error "$name - HTTP $http_code (期望 $expected_status)"
        echo "响应: $body" | head -c 200
        echo "..."
        ((FAILED++))
        return 1
    fi
}

echo "=========================================="
echo "AgentMem 后端 API 验证"
echo "版本: v1.0"
echo "日期: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 检查后端服务
log_info "检查后端服务..."
if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    log_success "后端服务运行正常"
    health_response=$(curl -s "$BACKEND_URL/health")
    echo "$health_response" | jq '.' 2>/dev/null || echo "$health_response"
    echo ""
else
    log_error "后端服务未运行，请先启动: just start-server-no-auth"
    exit 1
fi

# 1. 健康检查
test_endpoint "健康检查" "GET" "/health" "" 200

# 2. Dashboard 统计
test_endpoint "Dashboard 统计" "GET" "/api/v1/stats/dashboard" "" 200

# 3. Agent 管理
echo ""
echo "=========================================="
echo "Agent 管理 API"
echo "=========================================="

# 创建 Agent
AGENT_DATA='{"name":"API Test Agent","description":"Test agent created via API verification"}'
test_endpoint "创建 Agent" "POST" "/api/v1/agents" "$AGENT_DATA" 201

# 获取创建的 Agent ID
AGENT_LIST_RESPONSE=$(curl -s -X GET "$BACKEND_URL/api/v1/agents" \
    -H "X-User-ID: $TEST_USER_ID" \
    -H "X-Organization-ID: $TEST_ORG_ID" \
    -H "Content-Type: application/json")

AGENT_ID=$(echo "$AGENT_LIST_RESPONSE" | jq -r '.data[0].id' 2>/dev/null || echo "")

if [ -n "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    log_info "找到 Agent ID: $AGENT_ID"
    
    # 获取 Agent 详情
    test_endpoint "获取 Agent 详情" "GET" "/api/v1/agents/$AGENT_ID" "" 200
    
    # 更新 Agent
    UPDATE_DATA='{"name":"API Test Agent Updated","description":"Updated description"}'
    test_endpoint "更新 Agent" "PUT" "/api/v1/agents/$AGENT_ID" "$UPDATE_DATA" 200
else
    log_warning "无法获取 Agent ID，跳过相关测试"
    ((WARNINGS++))
fi

# 列出所有 Agent
test_endpoint "列出所有 Agent" "GET" "/api/v1/agents" "" 200

# 4. 记忆管理
echo ""
echo "=========================================="
echo "记忆管理 API"
echo "=========================================="

# 创建记忆
MEMORY_DATA='{"content":"API测试记忆 - '"$(date '+%Y-%m-%d %H:%M:%S')"'","memory_type":"Episodic","user_id":"'"$TEST_USER_ID"'","session_id":"test-session"}'
test_endpoint "创建记忆" "POST" "/api/v1/memories" "$MEMORY_DATA" 201

# 获取记忆列表
test_endpoint "获取记忆列表" "GET" "/api/v1/memories?page=0&limit=10" "" 200

# 搜索记忆
SEARCH_DATA='{"query":"API测试","limit":10}'
test_endpoint "搜索记忆" "POST" "/api/v1/memories/search" "$SEARCH_DATA" 200

# 5. 聊天功能
echo ""
echo "=========================================="
echo "聊天功能 API"
echo "=========================================="

if [ -n "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ]; then
    CHAT_DATA='{"message":"你好，这是一个API测试消息","user_id":"'"$TEST_USER_ID"'","session_id":"test-session","memorizing":true}'
    test_endpoint "发送聊天消息" "POST" "/api/v1/agents/$AGENT_ID/chat" "$CHAT_DATA" 200
    
    # 获取聊天历史
    test_endpoint "获取聊天历史" "GET" "/api/v1/agents/$AGENT_ID/chat/history?session_id=test-session" "" 200
else
    log_warning "跳过聊天测试（需要 Agent ID）"
    ((WARNINGS++))
fi

# 6. 系统提示词
echo ""
echo "=========================================="
echo "系统提示词 API"
echo "=========================================="

PROMPT_DATA='{"user_id":"'"$TEST_USER_ID"'","context":"API验证测试"}'
test_endpoint "获取系统提示词" "POST" "/api/v1/prompts/system" "$PROMPT_DATA" 200

# 总结
echo ""
echo "=========================================="
echo "API 验证总结"
echo "=========================================="
echo "总测试数: $((PASSED + FAILED))"
log_success "通过: $PASSED"
if [ $FAILED -gt 0 ]; then
    log_error "失败: $FAILED"
fi
if [ $WARNINGS -gt 0 ]; then
    log_warning "警告: $WARNINGS"
fi
echo ""
echo "=========================================="
echo "验证完成！"
echo "=========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
