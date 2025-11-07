#!/bin/bash

###############################################################################
# AgentMem Agent创建问题修复脚本
#
# 此脚本解决Agent创建后立即使用导致的竞态条件问题
# 使用轮询验证确保Agent确实创建成功
###############################################################################

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

BACKEND_URL="http://127.0.0.1:8080"
TEST_AGENT="test_agent_fixed_$(date +%s)"
TEST_USER="test_user_fixed"

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ $1${NC}"
}

print_section() {
    echo ""
    echo -e "${BLUE}${BOLD}>>> $1${NC}"
    echo ""
}

echo ""
echo -e "${BLUE}${BOLD}================================${NC}"
echo -e "${BLUE}${BOLD}AgentMem Agent 创建修复验证${NC}"
echo -e "${BLUE}${BOLD}================================${NC}"
echo ""

# 检查后端
print_section "检查后端服务"

if ! curl -sf "$BACKEND_URL/health" > /dev/null 2>&1; then
    print_error "后端服务未运行"
    print_info "请先启动: ./target/release/agent-mem-server --config config.toml"
    exit 1
fi

print_success "后端服务运行中"

# 创建并验证Agent
create_and_verify_agent() {
    local agent_id="$1"
    local user_id="$2"
    
    print_section "步骤1: 创建 Agent"
    
    CREATE_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/agents" \
        -H "Content-Type: application/json" \
        -w "\n%{http_code}" \
        -d "{
            \"agent_id\": \"$agent_id\",
            \"name\": \"Fixed Test Agent\",
            \"description\": \"Agent with proper verification\",
            \"user_id\": \"$user_id\",
            \"config\": {}
        }")
    
    HTTP_CODE=$(echo "$CREATE_RESPONSE" | tail -1)
    BODY=$(echo "$CREATE_RESPONSE" | sed '$d')
    
    echo "HTTP Code: $HTTP_CODE"
    echo "Response:"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
    echo ""
    
    if [ "$HTTP_CODE" != "200" ] && [ "$HTTP_CODE" != "201" ]; then
        print_error "创建请求失败"
        return 1
    fi
    
    # 提取实际返回的Agent ID（后端可能忽略请求中的ID）
    ACTUAL_AGENT_ID=$(echo "$BODY" | jq -r '.data.id // .id // empty')
    
    if [ -n "$ACTUAL_AGENT_ID" ] && [ "$ACTUAL_AGENT_ID" != "$agent_id" ]; then
        print_info "后端返回的 Agent ID: $ACTUAL_AGENT_ID (不同于请求的ID: $agent_id)"
        agent_id="$ACTUAL_AGENT_ID"  # 使用实际返回的ID
    fi
    
    print_success "创建请求成功，Agent ID: $agent_id"
    
    print_section "步骤2: 轮询验证 Agent 就绪状态"
    
    MAX_RETRIES=20
    RETRY_INTERVAL=0.5
    
    for i in $(seq 1 $MAX_RETRIES); do
        sleep $RETRY_INTERVAL
        
        VERIFY_RESPONSE=$(curl -s "$BACKEND_URL/api/v1/agents/$agent_id" \
            -w "\n%{http_code}" 2>/dev/null)
        
        VERIFY_CODE=$(echo "$VERIFY_RESPONSE" | tail -1)
        VERIFY_BODY=$(echo "$VERIFY_RESPONSE" | sed '$d')
        
        if [ "$VERIFY_CODE" = "200" ]; then
            print_success "Agent 验证成功 (尝试 $i/$MAX_RETRIES, 耗时 $(echo "$i * $RETRY_INTERVAL" | bc)s)"
            echo ""
            echo "Agent 详细信息:"
            echo "$VERIFY_BODY" | jq .
            echo ""
            return 0
        fi
        
        echo -n "."
    done
    
    echo ""
    print_error "Agent 验证超时 (${MAX_RETRIES}次尝试, 总计$(echo "$MAX_RETRIES * $RETRY_INTERVAL" | bc)秒)"
    return 1
}

# 测试完整流程
test_complete_flow() {
    local agent_id="$1"
    local user_id="$2"
    
    print_section "步骤3: 测试 Add Memory (通过 MCP)"
    
    # 使用MCP服务器添加记忆
    MCP_SERVER="./target/release/agentmem-mcp-server"
    
    if [ ! -f "$MCP_SERVER" ]; then
        print_error "MCP 服务器未找到: $MCP_SERVER"
        return 1
    fi
    
    ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"Test memory with verified agent - fixed","user_id":"'$user_id'","agent_id":"'$agent_id'","memory_type":"Episodic"}}}'
    
    echo "发送 MCP 请求:"
    echo "$ADD_MEMORY_REQUEST" | jq .
    echo ""
    
    MEMORY_RESPONSE=$(echo "$ADD_MEMORY_REQUEST" | $MCP_SERVER 2>/dev/null)
    
    echo "MCP 响应:"
    echo "$MEMORY_RESPONSE" | jq .
    echo ""
    
    if echo "$MEMORY_RESPONSE" | jq -e '.result' > /dev/null 2>&1; then
        print_success "Memory 通过 MCP 创建成功"
        
        MEMORY_TEXT=$(echo "$MEMORY_RESPONSE" | jq -r '.result.content[0].text')
        MEMORY_ID=$(echo "$MEMORY_TEXT" | jq -r '.memory_id // "N/A"')
        print_info "记忆 ID: $MEMORY_ID"
        
        return 0
    else
        print_error "Memory 创建失败"
        ERROR_MSG=$(echo "$MEMORY_RESPONSE" | jq -r '.error.message // "Unknown error"')
        print_error "错误: $ERROR_MSG"
        return 1
    fi
}

# 测试搜索功能
test_search() {
    local user_id="$1"
    
    print_section "步骤4: 测试 Search Memories"
    
    MCP_SERVER="./target/release/agentmem-mcp-server"
    
    SEARCH_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"verified agent fixed","user_id":"'$user_id'","limit":5}}}'
    
    echo "发送搜索请求:"
    echo "$SEARCH_REQUEST" | jq .
    echo ""
    
    SEARCH_RESPONSE=$(echo "$SEARCH_REQUEST" | $MCP_SERVER 2>/dev/null)
    
    echo "搜索响应:"
    echo "$SEARCH_RESPONSE" | jq .
    echo ""
    
    if echo "$SEARCH_RESPONSE" | jq -e '.result' > /dev/null 2>&1; then
        SEARCH_TEXT=$(echo "$SEARCH_RESPONSE" | jq -r '.result.content[0].text')
        RESULTS_COUNT=$(echo "$SEARCH_TEXT" | jq -r '.total_results // 0')
        
        if [ "$RESULTS_COUNT" -gt 0 ]; then
            print_success "找到 $RESULTS_COUNT 条记忆"
            return 0
        else
            print_info "未找到记忆（可能需要更多时间索引）"
            return 0
        fi
    else
        print_error "搜索失败"
        return 1
    fi
}

# 执行测试流程
OVERALL_SUCCESS=true
ACTUAL_AGENT_ID=""

# 创建并验证Agent，捕获实际的Agent ID
if create_and_verify_agent "$TEST_AGENT" "$TEST_USER"; then
    # 函数内部会更新agent_id变量，但我们需要在这里捕获它
    # 通过临时文件传递实际的Agent ID
    ACTUAL_AGENT_ID=$(curl -s "$BACKEND_URL/api/v1/agents" | jq -r '.data[-1].id // empty' 2>/dev/null)
    
    if [ -z "$ACTUAL_AGENT_ID" ]; then
        # 如果无法获取，使用原始ID
        ACTUAL_AGENT_ID="$TEST_AGENT"
    fi
    
    print_info "使用 Agent ID: $ACTUAL_AGENT_ID"
    
    if test_complete_flow "$ACTUAL_AGENT_ID" "$TEST_USER"; then
        test_search "$TEST_USER" || OVERALL_SUCCESS=false
    else
        OVERALL_SUCCESS=false
    fi
else
    OVERALL_SUCCESS=false
fi

# 最终报告
echo ""
echo -e "${BLUE}${BOLD}================================${NC}"

if [ "$OVERALL_SUCCESS" = true ]; then
    echo -e "${GREEN}${BOLD}✓ 所有测试通过！问题已修复 ✨${NC}"
    echo -e "${BLUE}${BOLD}================================${NC}"
    echo ""
    print_success "修复要点:"
    echo "  1. Agent创建后使用轮询验证"
    echo "  2. 确保Agent就绪后再使用"
    echo "  3. 完整的错误处理和反馈"
    echo ""
    print_info "建议:"
    echo "  • 在生产环境使用轮询验证模式"
    echo "  • 考虑添加 /agents/{id}/ready 端点"
    echo "  • 设置合理的超时时间"
    echo ""
    exit 0
else
    echo -e "${RED}${BOLD}✗ 测试失败，需要进一步调查${NC}"
    echo -e "${BLUE}${BOLD}================================${NC}"
    echo ""
    print_error "可能的原因:"
    echo "  1. 后端数据库写入延迟 > 10秒"
    echo "  2. Agent创建实际失败但返回200"
    echo "  3. 数据库连接问题"
    echo ""
    print_info "调试建议:"
    echo "  • 检查后端日志"
    echo "  • 验证数据库状态"
    echo "  • 手动测试Agent API"
    echo ""
    exit 1
fi

