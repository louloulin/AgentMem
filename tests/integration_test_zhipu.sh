#!/bin/bash

# AgentMem 集成测试脚本 - Zhipu AI Provider
# 全面测试 AgentMem 的核心功能

set -e

BASE_URL="http://localhost:8080"
API_BASE="${BASE_URL}/api/v1"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 打印函数
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_test() {
    echo -e "${YELLOW}[TEST $((TOTAL_TESTS + 1))]${NC} $1"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

print_success() {
    echo -e "${GREEN}✓ PASSED${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

print_failure() {
    echo -e "${RED}✗ FAILED${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# 等待服务器启动
wait_for_server() {
    print_header "等待服务器启动"
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "${BASE_URL}/health" > /dev/null 2>&1; then
            print_success "服务器已启动"
            return 0
        fi
        attempt=$((attempt + 1))
        echo -n "."
        sleep 1
    done
    
    print_failure "服务器启动超时"
    exit 1
}

# 测试 1: 健康检查
test_health_check() {
    print_header "测试 1: 健康检查"
    print_test "GET /health"
    
    response=$(curl -s "${BASE_URL}/health")
    status=$(echo "$response" | jq -r '.status')
    
    if [ "$status" = "healthy" ]; then
        print_success "健康检查通过"
        print_info "响应: $response"
    else
        print_failure "健康检查失败"
        echo "响应: $response"
    fi
}

# 测试 2: Metrics
test_metrics() {
    print_header "测试 2: Metrics 端点"
    print_test "GET /metrics"
    
    response=$(curl -s "${BASE_URL}/metrics")
    
    if echo "$response" | jq -e '.metrics' > /dev/null 2>&1; then
        print_success "Metrics 端点正常"
        print_info "响应: $response"
    else
        print_failure "Metrics 端点失败"
        echo "响应: $response"
    fi
}

# 测试 3: 创建 Organization
test_create_organization() {
    print_header "测试 3: 创建 Organization"
    print_test "POST /api/v1/organizations"
    
    response=$(curl -s -X POST "${API_BASE}/organizations" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "测试组织",
            "description": "AgentMem 集成测试组织"
        }')
    
    org_id=$(echo "$response" | jq -r '.id // .organization_id // empty')
    
    if [ -n "$org_id" ]; then
        print_success "Organization 创建成功"
        print_info "Organization ID: $org_id"
        echo "$org_id" > /tmp/agentmem_test_org_id
    else
        print_failure "Organization 创建失败"
        echo "响应: $response"
    fi
}

# 测试 4: 创建 User
test_create_user() {
    print_header "测试 4: 创建 User"
    print_test "POST /api/v1/users"
    
    org_id=$(cat /tmp/agentmem_test_org_id 2>/dev/null || echo "test-org-001")
    
    response=$(curl -s -X POST "${API_BASE}/users" \
        -H "Content-Type: application/json" \
        -d "{
            \"organization_id\": \"$org_id\",
            \"name\": \"测试用户\",
            \"email\": \"test@agentmem.ai\"
        }")
    
    user_id=$(echo "$response" | jq -r '.id // .user_id // empty')
    
    if [ -n "$user_id" ]; then
        print_success "User 创建成功"
        print_info "User ID: $user_id"
        echo "$user_id" > /tmp/agentmem_test_user_id
    else
        print_failure "User 创建失败"
        echo "响应: $response"
    fi
}

# 测试 5: 创建 Agent
test_create_agent() {
    print_header "测试 5: 创建 Agent"
    print_test "POST /api/v1/agents"
    
    org_id=$(cat /tmp/agentmem_test_org_id 2>/dev/null || echo "test-org-001")
    
    response=$(curl -s -X POST "${API_BASE}/agents" \
        -H "Content-Type: application/json" \
        -d "{
            \"organization_id\": \"$org_id\",
            \"name\": \"测试 Agent\",
            \"description\": \"AgentMem 集成测试 Agent\",
            \"agent_type\": \"conversational\"
        }")
    
    agent_id=$(echo "$response" | jq -r '.id // .agent_id // empty')
    
    if [ -n "$agent_id" ]; then
        print_success "Agent 创建成功"
        print_info "Agent ID: $agent_id"
        echo "$agent_id" > /tmp/agentmem_test_agent_id
    else
        print_failure "Agent 创建失败"
        echo "响应: $response"
    fi
}

# 测试 6: 添加 Episodic Memory
test_add_episodic_memory() {
    print_header "测试 6: 添加 Episodic Memory"
    print_test "POST /api/v1/memories"
    
    org_id=$(cat /tmp/agentmem_test_org_id 2>/dev/null || echo "test-org-001")
    user_id=$(cat /tmp/agentmem_test_user_id 2>/dev/null || echo "test-user-001")
    agent_id=$(cat /tmp/agentmem_test_agent_id 2>/dev/null || echo "test-agent-001")
    
    response=$(curl -s -X POST "${API_BASE}/memories" \
        -H "Content-Type: application/json" \
        -d "{
            \"organization_id\": \"$org_id\",
            \"user_id\": \"$user_id\",
            \"agent_id\": \"$agent_id\",
            \"content\": \"用户询问了关于 AgentMem 的功能\",
            \"memory_type\": \"episodic\",
            \"importance\": 0.8
        }")
    
    memory_id=$(echo "$response" | jq -r '.id // .memory_id // empty')
    
    if [ -n "$memory_id" ]; then
        print_success "Episodic Memory 添加成功"
        print_info "Memory ID: $memory_id"
        echo "$memory_id" > /tmp/agentmem_test_memory_id
    else
        print_failure "Episodic Memory 添加失败"
        echo "响应: $response"
    fi
}

# 测试 7: 检索 Memory
test_retrieve_memory() {
    print_header "测试 7: 检索 Memory"
    print_test "GET /api/v1/memories"
    
    org_id=$(cat /tmp/agentmem_test_org_id 2>/dev/null || echo "test-org-001")
    user_id=$(cat /tmp/agentmem_test_user_id 2>/dev/null || echo "test-user-001")
    
    response=$(curl -s "${API_BASE}/memories?organization_id=$org_id&user_id=$user_id")
    
    count=$(echo "$response" | jq -r '.memories | length // 0')
    
    if [ "$count" -gt 0 ]; then
        print_success "Memory 检索成功"
        print_info "检索到 $count 条记忆"
    else
        print_failure "Memory 检索失败"
        echo "响应: $response"
    fi
}

# 测试 8: Chat 功能 (使用 Zhipu AI)
test_chat_with_zhipu() {
    print_header "测试 8: Chat 功能 (Zhipu AI)"
    print_test "POST /api/v1/chat/completions"
    
    org_id=$(cat /tmp/agentmem_test_org_id 2>/dev/null || echo "test-org-001")
    user_id=$(cat /tmp/agentmem_test_user_id 2>/dev/null || echo "test-user-001")
    agent_id=$(cat /tmp/agentmem_test_agent_id 2>/dev/null || echo "test-agent-001")
    
    response=$(curl -s -X POST "${API_BASE}/chat/completions" \
        -H "Content-Type: application/json" \
        -d "{
            \"organization_id\": \"$org_id\",
            \"user_id\": \"$user_id\",
            \"agent_id\": \"$agent_id\",
            \"messages\": [
                {
                    \"role\": \"user\",
                    \"content\": \"你好，请简单介绍一下你自己\"
                }
            ]
        }")
    
    content=$(echo "$response" | jq -r '.choices[0].message.content // .content // empty')
    
    if [ -n "$content" ]; then
        print_success "Chat 功能正常 (Zhipu AI)"
        print_info "AI 响应: ${content:0:100}..."
    else
        print_failure "Chat 功能失败"
        echo "响应: $response"
    fi
}

# 测试总结
print_summary() {
    print_header "测试总结"
    echo -e "总测试数: ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "通过: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "失败: ${RED}$FAILED_TESTS${NC}"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}✓ 所有测试通过！${NC}\n"
        exit 0
    else
        echo -e "\n${RED}✗ 有测试失败${NC}\n"
        exit 1
    fi
}

# 主测试流程
main() {
    print_header "AgentMem 集成测试 - Zhipu AI Provider"
    
    wait_for_server
    test_health_check
    test_metrics
    test_create_organization
    test_create_user
    test_create_agent
    test_add_episodic_memory
    test_retrieve_memory
    test_chat_with_zhipu
    
    print_summary
}

# 运行测试
main

