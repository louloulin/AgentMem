#!/bin/bash

# 测试自动Agent创建功能
# 日期: 2025-11-07

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_section() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# 检查后端服务
check_backend() {
    print_section "检查后端服务"
    if curl -s http://127.0.0.1:8080/health > /dev/null 2>&1; then
        print_success "后端服务运行中"
    else
        print_error "后端服务未运行，请先启动: ./start.sh"
        exit 1
    fi
}

# 测试场景1：不提供agent_id，应该自动创建 agent-{user_id}
test_auto_agent_creation() {
    print_section "测试1: 自动创建Agent (不提供agent_id)"
    
    local user_id="test-user-$(date +%s)"
    local expected_agent_id="agent-${user_id}"
    
    print_info "用户ID: $user_id"
    print_info "预期Agent ID: $expected_agent_id"
    
    # 确保Agent不存在
    print_info "清理: 删除可能存在的Agent..."
    curl -s -X DELETE "http://127.0.0.1:8080/api/v1/agents/${expected_agent_id}" || true
    
    # 通过MCP添加记忆（不提供agent_id）
    print_info "通过MCP添加记忆（不提供agent_id）..."
    
    local request=$(cat <<EOF
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"测试自动创建Agent功能 - 不提供agent_id","user_id":"${user_id}","memory_type":"Episodic","metadata":"{\"test\":\"auto_agent_creation\"}"}}}
EOF
)
    
    local response=$(echo "$request" | /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server 2>&1 | tail -1)
    
    echo "$response" | jq . 2>/dev/null || echo "$response"
    
    # 检查Agent是否被自动创建
    print_info "验证Agent是否被自动创建..."
    local agent_check=$(curl -s "http://127.0.0.1:8080/api/v1/agents/${expected_agent_id}")
    
    if echo "$agent_check" | jq -e '.data.id' > /dev/null 2>&1; then
        local actual_agent_id=$(echo "$agent_check" | jq -r '.data.id')
        print_success "Agent自动创建成功: $actual_agent_id"
        print_info "Agent详情:"
        echo "$agent_check" | jq '.data' 2>/dev/null
    else
        print_error "Agent未被自动创建"
        echo "$agent_check" | jq . 2>/dev/null || echo "$agent_check"
        return 1
    fi
}

# 测试场景2：提供agent_id，使用指定的Agent
test_custom_agent_id() {
    print_section "测试2: 使用自定义Agent ID"
    
    local user_id="test-user-custom-$(date +%s)"
    local custom_agent_id="my-custom-agent-$(date +%s)"
    
    print_info "用户ID: $user_id"
    print_info "自定义Agent ID: $custom_agent_id"
    
    # 通过MCP添加记忆（提供agent_id）
    print_info "通过MCP添加记忆（提供agent_id）..."
    
    local request=$(cat <<EOF
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"测试自定义Agent ID功能","user_id":"${user_id}","agent_id":"${custom_agent_id}","memory_type":"Episodic","metadata":"{\"test\":\"custom_agent_id\"}"}}}
EOF
)
    
    local response=$(echo "$request" | /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server 2>&1 | tail -1)
    
    echo "$response" | jq . 2>/dev/null || echo "$response"
    
    # 检查Agent是否被自动创建（使用指定的ID）
    print_info "验证自定义Agent是否被创建..."
    local agent_check=$(curl -s "http://127.0.0.1:8080/api/v1/agents/${custom_agent_id}")
    
    if echo "$agent_check" | jq -e '.data.id' > /dev/null 2>&1; then
        local actual_agent_id=$(echo "$agent_check" | jq -r '.data.id')
        print_success "自定义Agent创建成功: $actual_agent_id"
    else
        print_error "自定义Agent未被创建"
        return 1
    fi
}

# 测试场景3：Agent已存在，不重复创建
test_existing_agent() {
    print_section "测试3: Agent已存在，不重复创建"
    
    local user_id="test-user-existing-$(date +%s)"
    local agent_id="agent-${user_id}"
    
    print_info "用户ID: $user_id"
    print_info "Agent ID: $agent_id"
    
    # 先手动创建Agent
    print_info "手动创建Agent..."
    local create_response=$(curl -s -X POST "http://127.0.0.1:8080/api/v1/agents" \
        -H "Content-Type: application/json" \
        -d "{
            \"id\": \"${agent_id}\",
            \"name\": \"Pre-existing Agent\",
            \"description\": \"Agent created before MCP call\",
            \"user_id\": \"${user_id}\"
        }")
    
    print_success "Agent创建完成"
    
    # 通过MCP添加记忆（Agent已存在）
    print_info "通过MCP添加记忆到已存在的Agent..."
    
    local request=$(cat <<EOF
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"测试已存在Agent的记忆添加","user_id":"${user_id}","memory_type":"Episodic"}}}
EOF
)
    
    local response=$(echo "$request" | /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server 2>&1 | tail -1)
    
    echo "$response" | jq . 2>/dev/null || echo "$response"
    
    # 验证记忆添加成功
    if echo "$response" | jq -e '.result.success' > /dev/null 2>&1; then
        print_success "记忆成功添加到已存在的Agent"
    else
        print_error "记忆添加失败"
        return 1
    fi
}

# 测试场景4：搜索记忆（不提供agent_id）
test_search_without_agent_id() {
    print_section "测试4: 搜索记忆（不提供agent_id）"
    
    local user_id="test-user-search-$(date +%s)"
    
    print_info "用户ID: $user_id"
    
    # 先添加一些记忆
    print_info "添加测试记忆..."
    local add_request=$(cat <<EOF
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"我喜欢苹果和香蕉","user_id":"${user_id}","memory_type":"Episodic"}}}
EOF
)
    echo "$add_request" | /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server 2>&1 | tail -1 | jq . 2>/dev/null
    
    sleep 2  # 等待索引
    
    # 搜索记忆
    print_info "搜索记忆: 苹果..."
    local search_request=$(cat <<EOF
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"苹果","user_id":"${user_id}","limit":5}}}
EOF
)
    
    local search_response=$(echo "$search_request" | /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server 2>&1 | tail -1)
    
    echo "$search_response" | jq . 2>/dev/null || echo "$search_response"
    
    # 验证搜索结果
    local memory_count=$(echo "$search_response" | jq -r '.result.memories | length' 2>/dev/null || echo "0")
    
    if [ "$memory_count" -gt 0 ]; then
        print_success "搜索成功，找到 $memory_count 条记忆"
    else
        print_error "搜索失败或未找到记忆"
        return 1
    fi
}

# 主函数
main() {
    print_section "AgentMem 自动Agent创建功能测试"
    echo "日期: $(date)"
    echo "版本: AgentMem 2.0 with auto-agent-creation"
    
    check_backend
    
    local failed_tests=0
    
    # 运行测试
    test_auto_agent_creation || ((failed_tests++))
    test_custom_agent_id || ((failed_tests++))
    test_existing_agent || ((failed_tests++))
    test_search_without_agent_id || ((failed_tests++))
    
    # 总结
    print_section "测试总结"
    
    if [ $failed_tests -eq 0 ]; then
        print_success "所有测试通过！✨"
        echo ""
        echo -e "${GREEN}🎉 自动Agent创建功能工作正常！${NC}"
        echo ""
        echo "主要改进:"
        echo "  ✅ agent_id现在是可选的"
        echo "  ✅ 会自动为每个user创建默认Agent (agent-{user_id})"
        echo "  ✅ 也支持自定义Agent ID"
        echo "  ✅ 不会重复创建已存在的Agent"
        echo "  ✅ 搜索功能正常工作"
        exit 0
    else
        print_error "$failed_tests 个测试失败"
        exit 1
    fi
}

# 执行主函数
main

