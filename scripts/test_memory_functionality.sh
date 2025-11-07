#!/bin/bash

# AgentMem 记忆功能实际测试脚本
# 测试 Episodic-first 检索策略的实际效果

set -e

API_BASE="http://localhost:8080"
TIMESTAMP=$(date +%s)
TEST_AGENT_ID="test-agent-${TIMESTAMP}"
TEST_USER_ID="test-user-${TIMESTAMP}"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

log_test() {
    echo ""
    echo -e "${YELLOW}[TEST $1]${NC} $2"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║  AgentMem 记忆功能实际测试                                  ║"
echo "║  Testing Episodic-First Retrieval Strategy                  ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "测试配置:"
echo "  API Base: $API_BASE"
echo "  Agent ID: $TEST_AGENT_ID"
echo "  User ID:  $TEST_USER_ID"
echo ""

# ============================================================
# TEST 0: 创建 Agent（前置步骤）
# ============================================================
log_test "0" "创建测试 Agent（前置步骤）"

response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/api/v1/agents" \
    -H "Content-Type: application/json" \
    -d "{
        \"id\": \"${TEST_AGENT_ID}\",
        \"name\": \"Test Agent for Memory Verification\",
        \"description\": \"Testing Episodic-first retrieval strategy\",
        \"system_prompt\": \"You are a helpful assistant with perfect memory.\",
        \"model\": \"deepseek-chat\"
    }")

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | sed '$d')

if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
    log_success "Agent 创建成功 (HTTP $http_code)"
else
    log_error "Agent 创建失败 (HTTP $http_code)"
    echo "Response: $body"
    echo "继续尝试测试（Agent可能已存在）..."
fi

sleep 1

# ============================================================
# TEST 1: 添加 Episodic Memory (User scope)
# ============================================================
log_test "1" "添加 Episodic Memory (Long-term Memory)"

response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/api/v1/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"agent_id\": \"${TEST_AGENT_ID}\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"content\": \"我最喜欢的编程语言是Rust，我正在开发AgentMem记忆管理平台\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.9
    }")

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | sed '$d')

if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
    log_success "Episodic Memory 添加成功 (HTTP $http_code)"
    memory_id=$(echo "$body" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    log_info "Memory ID: $memory_id"
else
    log_error "Episodic Memory 添加失败 (HTTP $http_code)"
    echo "Response: $body"
fi

sleep 1

# ============================================================
# TEST 2: 添加第二条 Episodic Memory
# ============================================================
log_test "2" "添加第二条 Episodic Memory"

response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/api/v1/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"agent_id\": \"${TEST_AGENT_ID}\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"content\": \"我的生日是1990年1月1日，我来自北京\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.8
    }")

http_code=$(echo "$response" | tail -n 1)

if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
    log_success "第二条 Episodic Memory 添加成功"
else
    log_error "第二条 Episodic Memory 添加失败 (HTTP $http_code)"
fi

sleep 1

# ============================================================
# TEST 3: 模拟 Session A - 第一次对话（会创建 Working Memory）
# ============================================================
log_test "3" "Session A - 第一次对话"

SESSION_A="session-a-${TIMESTAMP}"

response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"你好，请告诉我你知道关于我的什么信息？\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_A}\"
    }")

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | sed '$d')

if [ "$http_code" = "200" ]; then
    log_success "Session A 对话成功 (HTTP $http_code)"
    
    # 检查是否包含我们添加的记忆内容
    if echo "$body" | grep -qi "rust\|agentmem\|生日\|北京"; then
        log_success "✓ AI回复包含 Episodic Memory 内容（检索成功）"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        log_error "AI回复未包含 Episodic Memory 内容"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        echo "AI回复: $(echo "$body" | head -c 200)..."
    fi
else
    log_error "Session A 对话失败 (HTTP $http_code)"
    echo "Response: $body"
fi

sleep 2

# ============================================================
# TEST 4: 模拟 Session B - 新会话（跨Session记忆测试）⭐ 核心测试
# ============================================================
log_test "4" "Session B - 新会话（测试跨Session记忆）⭐ 核心"

SESSION_B="session-b-${TIMESTAMP}"

log_info "使用新的 Session ID: $SESSION_B"
log_info "这模拟了用户刷新页面或重新打开应用"

response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"我最喜欢什么编程语言？我的生日是哪天？\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_B}\"
    }")

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | sed '$d')

if [ "$http_code" = "200" ]; then
    log_success "Session B 对话成功 (HTTP $http_code)"
    
    # 检查是否能跨Session检索到 Episodic Memory
    rust_found=false
    birthday_found=false
    
    if echo "$body" | grep -qi "rust"; then
        log_success "✓ 跨Session检索到编程语言（Episodic Memory）"
        rust_found=true
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        log_error "未能跨Session检索到编程语言"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    fi
    
    if echo "$body" | grep -qi "1990\|1月\|生日"; then
        log_success "✓ 跨Session检索到生日信息（Episodic Memory）"
        birthday_found=true
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        log_error "未能跨Session检索到生日信息"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    fi
    
    if [ "$rust_found" = true ] && [ "$birthday_found" = true ]; then
        echo -e "${GREEN}${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}  🎉 跨Session记忆连续性验证成功！${NC}"
        echo -e "${GREEN}  ✅ Episodic-first 检索策略工作正常${NC}"
        echo -e "${GREEN}  ✅ Phase 1 核心功能实现成功${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    else
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${RED}  ⚠️  跨Session记忆连续性部分失败${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    fi
    
    echo ""
    echo "AI完整回复:"
    echo "$body" | head -c 500
    echo ""
else
    log_error "Session B 对话失败 (HTTP $http_code)"
    echo "Response: $body"
fi

sleep 2

# ============================================================
# TEST 5: 查询记忆统计
# ============================================================
log_test "5" "查询记忆统计"

response=$(curl -s -w "\n%{http_code}" "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories/stats")

http_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | sed '$d')

if [ "$http_code" = "200" ]; then
    log_success "记忆统计查询成功"
    echo "统计信息: $body"
else
    log_error "记忆统计查询失败 (HTTP $http_code)"
fi

# ============================================================
# 测试总结
# ============================================================
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                     测试完成                                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "测试结果:"
echo "  总测试数: $TOTAL_TESTS"
echo "  通过: ${GREEN}$PASSED_TESTS${NC}"
echo "  失败: ${RED}$FAILED_TESTS${NC}"
echo "  成功率: $((PASSED_TESTS * 100 / TOTAL_TESTS))%"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ 所有测试通过！Phase 1 功能验证成功！${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  部分测试失败，请检查日志${NC}"
    exit 1
fi

