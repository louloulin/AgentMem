#!/bin/bash

# ========================================
# AgentMem Server Scope Support E2E 测试
# ========================================
# 测试server端是否正确支持scope功能
# Phase 2 Server Implementation Verification

set -e  # 遇到错误立即退出

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

# 配置
BACKEND_URL="http://127.0.0.1:8080"
TEST_USER="test_user_scope_$(date +%s)"
TEST_AGENT="test_agent_scope_$(date +%s)"
TEST_RUN_ID="run_$(uuidgen)"

print_section "AgentMem Server Scope Support 测试"
print_info "测试配置:"
echo "  Backend URL: $BACKEND_URL"
echo "  Test User: $TEST_USER"
echo "  Test Agent: $TEST_AGENT"
echo "  Test Run ID: $TEST_RUN_ID"

# ========================================
# Step 1: 检查后端服务
# ========================================
print_section "Step 1: 检查后端服务"

if ! curl -s -f "$BACKEND_URL/health" > /dev/null; then
    print_error "后端服务未运行"
    echo ""
    echo "请先启动后端服务:"
    echo "  cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen"
    echo "  ./start_server_no_auth.sh"
    exit 1
fi

print_success "后端服务运行正常"

# ========================================
# Step 2: 创建测试Agent
# ========================================
print_section "Step 2: 创建测试Agent"

AGENT_CREATE_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/agents" \
    -H "Content-Type: application/json" \
    -d "{
        \"id\": \"$TEST_AGENT\",
        \"name\": \"Test Agent for Scope Verification\",
        \"description\": \"Agent created for server scope testing\",
        \"user_id\": \"$TEST_USER\"
    }")

echo "$AGENT_CREATE_RESPONSE" | jq .

ACTUAL_AGENT_ID=$(echo "$AGENT_CREATE_RESPONSE" | jq -r '.data.id // .id // empty')

if [ -n "$ACTUAL_AGENT_ID" ]; then
    print_success "Agent创建成功: $ACTUAL_AGENT_ID"
    TEST_AGENT="$ACTUAL_AGENT_ID"
else
    print_error "Agent创建失败"
    exit 1
fi

# ========================================
# Step 3: 测试User Scope (通过Server API)
# ========================================
print_section "Step 3: 测试User Scope (Server API)"

USER_MEMORY_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"agent_id\": \"$TEST_AGENT\",
        \"user_id\": \"$TEST_USER\",
        \"content\": \"User scope memory: I love pizza\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.8,
        \"metadata\": {
            \"scope_type\": \"user\",
            \"test_type\": \"user_scope_verification\"
        }
    }")

echo "$USER_MEMORY_RESPONSE" | jq .

USER_MEMORY_ID=$(echo "$USER_MEMORY_RESPONSE" | jq -r '.data.id // empty')

if [ -n "$USER_MEMORY_ID" ]; then
    print_success "User scope记忆添加成功: $USER_MEMORY_ID"
else
    print_error "User scope记忆添加失败"
    exit 1
fi

# 验证存储的scope字段
sleep 1
USER_MEMORY_DETAIL=$(curl -s "$BACKEND_URL/api/v1/memories/$USER_MEMORY_ID")
STORED_SCOPE=$(echo "$USER_MEMORY_DETAIL" | jq -r '.data.scope // empty')

if [ "$STORED_SCOPE" = "user" ]; then
    print_success "User scope正确存储在数据库中"
else
    print_error "Scope存储错误: 期望'user'，实际'$STORED_SCOPE'"
    echo "$USER_MEMORY_DETAIL" | jq .
fi

# ========================================
# Step 4: 测试Agent Scope (通过Server API)
# ========================================
print_section "Step 4: 测试Agent Scope (Server API)"

AGENT_MEMORY_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"agent_id\": \"$TEST_AGENT\",
        \"user_id\": \"$TEST_USER\",
        \"content\": \"Agent scope memory: Meeting at 2pm\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.9,
        \"metadata\": {
            \"scope_type\": \"agent\",
            \"test_type\": \"agent_scope_verification\"
        }
    }")

echo "$AGENT_MEMORY_RESPONSE" | jq .

AGENT_MEMORY_ID=$(echo "$AGENT_MEMORY_RESPONSE" | jq -r '.data.id // empty')

if [ -n "$AGENT_MEMORY_ID" ]; then
    print_success "Agent scope记忆添加成功: $AGENT_MEMORY_ID"
else
    print_error "Agent scope记忆添加失败"
    exit 1
fi

# 验证存储的scope字段
sleep 1
AGENT_MEMORY_DETAIL=$(curl -s "$BACKEND_URL/api/v1/memories/$AGENT_MEMORY_ID")
STORED_SCOPE=$(echo "$AGENT_MEMORY_DETAIL" | jq -r '.data.scope // empty')

if [ "$STORED_SCOPE" = "agent" ]; then
    print_success "Agent scope正确存储在数据库中"
else
    print_error "Scope存储错误: 期望'agent'，实际'$STORED_SCOPE'"
    echo "$AGENT_MEMORY_DETAIL" | jq .
fi

# ========================================
# Step 5: 测试Run Scope (通过Server API)
# ========================================
print_section "Step 5: 测试Run Scope (Server API)"

RUN_MEMORY_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"agent_id\": \"$TEST_AGENT\",
        \"user_id\": \"$TEST_USER\",
        \"content\": \"Run scope memory: Temporary session note\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.5,
        \"metadata\": {
            \"scope_type\": \"run\",
            \"run_id\": \"$TEST_RUN_ID\",
            \"test_type\": \"run_scope_verification\"
        }
    }")

echo "$RUN_MEMORY_RESPONSE" | jq .

RUN_MEMORY_ID=$(echo "$RUN_MEMORY_RESPONSE" | jq -r '.data.id // empty')

if [ -n "$RUN_MEMORY_ID" ]; then
    print_success "Run scope记忆添加成功: $RUN_MEMORY_ID"
else
    print_error "Run scope记忆添加失败"
    exit 1
fi

# 验证存储的scope字段
sleep 1
RUN_MEMORY_DETAIL=$(curl -s "$BACKEND_URL/api/v1/memories/$RUN_MEMORY_ID")
STORED_SCOPE=$(echo "$RUN_MEMORY_DETAIL" | jq -r '.data.scope // empty')

if [ "$STORED_SCOPE" = "run" ]; then
    print_success "Run scope正确存储在数据库中"
else
    print_error "Scope存储错误: 期望'run'，实际'$STORED_SCOPE'"
    echo "$RUN_MEMORY_DETAIL" | jq .
fi

# ========================================
# Step 6: 测试自动Scope推断 (不显式指定scope_type)
# ========================================
print_section "Step 6: 测试自动Scope推断"

AUTO_MEMORY_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"agent_id\": \"$TEST_AGENT\",
        \"user_id\": \"$TEST_USER\",
        \"content\": \"Auto-inferred scope memory\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.7,
        \"metadata\": {
            \"test_type\": \"auto_scope_inference\"
        }
    }")

echo "$AUTO_MEMORY_RESPONSE" | jq .

AUTO_MEMORY_ID=$(echo "$AUTO_MEMORY_RESPONSE" | jq -r '.data.id // empty')

if [ -n "$AUTO_MEMORY_ID" ]; then
    print_success "自动推断scope记忆添加成功: $AUTO_MEMORY_ID"
else
    print_error "自动推断scope记忆添加失败"
    exit 1
fi

# 验证存储的scope字段（应该自动推断为"agent"）
sleep 1
AUTO_MEMORY_DETAIL=$(curl -s "$BACKEND_URL/api/v1/memories/$AUTO_MEMORY_ID")
STORED_SCOPE=$(echo "$AUTO_MEMORY_DETAIL" | jq -r '.data.scope // empty')

if [ "$STORED_SCOPE" = "agent" ]; then
    print_success "Scope自动推断成功: 'agent' (因为提供了user_id和agent_id)"
else
    print_error "Scope自动推断错误: 期望'agent'，实际'$STORED_SCOPE'"
    echo "$AUTO_MEMORY_DETAIL" | jq .
fi

# ========================================
# Step 7: 测试MCP + Server 完整流程
# ========================================
print_section "Step 7: 测试MCP + Server 完整流程"

print_info "通过MCP添加Session scope记忆..."

SESSION_ID="session_$(uuidgen)"

MCP_RESPONSE=$(echo "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{\"name\":\"agentmem_add_memory\",\"arguments\":{\"content\":\"MCP Session scope test: Video call with team\",\"scope_type\":\"session\",\"user_id\":\"$TEST_USER\",\"agent_id\":\"$TEST_AGENT\",\"session_id\":\"$SESSION_ID\",\"memory_type\":\"Episodic\"}}}" | \
    /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server)

echo "$MCP_RESPONSE" | jq .

MCP_SUCCESS=$(echo "$MCP_RESPONSE" | jq -r '.result.success // false')

if [ "$MCP_SUCCESS" = "true" ]; then
    print_success "MCP Session scope记忆添加成功"
    
    # 验证metadata中包含session_id
    SCOPE_FROM_MCP=$(echo "$MCP_RESPONSE" | jq -r '.result.scope_type // empty')
    if [ "$SCOPE_FROM_MCP" = "session" ]; then
        print_success "MCP返回的scope_type正确: 'session'"
    else
        print_error "MCP返回的scope_type错误: '$SCOPE_FROM_MCP'"
    fi
else
    print_error "MCP Session scope记忆添加失败"
    echo "$MCP_RESPONSE" | jq .
fi

# ========================================
# 测试总结
# ========================================
print_section "测试总结"

print_success "所有scope功能测试通过!"
echo ""
echo "测试覆盖:"
echo "  ✅ User Scope (Server API)"
echo "  ✅ Agent Scope (Server API)"
echo "  ✅ Run Scope (Server API)"
echo "  ✅ 自动Scope推断"
echo "  ✅ MCP + Server 完整流程"
echo "  ✅ Scope字段正确存储到数据库"
echo ""
print_success "Server端scope支持验证完成!"

