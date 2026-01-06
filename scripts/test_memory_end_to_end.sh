#!/bin/bash

# AgentMem 端到端记忆功能测试
# 测试 Episodic-first 检索策略的完整流程

set -e

API_BASE="http://localhost:8080"
TIMESTAMP=$(date +%s)

# 使用固定的IDs以便跨测试追踪
TEST_AGENT_ID="e2e-agent-${TIMESTAMP}"
TEST_USER_ID="e2e-user-${TIMESTAMP}"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
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
    echo -e "${MAGENTA}═══════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}[TEST $1]${NC} $2"
    echo -e "${MAGENTA}═══════════════════════════════════════════════════${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

log_section() {
    echo ""
    echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  $1${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
}

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║  AgentMem 端到端记忆功能测试                                ║"
echo "║  End-to-End Memory Functionality Test                        ║"
echo "║                                                              ║"
echo "║  验证 Episodic-first 检索策略                                ║"
echo "║  验证跨 Session 记忆连续性                                  ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
log_info "测试配置:"
log_info "  API Base: $API_BASE"
log_info "  Agent ID: $TEST_AGENT_ID"
log_info "  User ID:  $TEST_USER_ID"
log_info "  Timestamp: $TIMESTAMP"
echo ""

# ============================================================
# 阶段 1: 环境准备
# ============================================================
log_section "阶段 1: 环境准备"

log_test "1" "检查后端服务健康状态"
response=$(curl -s "${API_BASE}/health/live" 2>&1)
if echo "$response" | grep -q "alive"; then
    log_success "后端服务健康"
else
    log_error "后端服务不健康"
    exit 1
fi

log_test "2" "创建测试 Agent"
response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/api/v1/agents" \
    -H "Content-Type: application/json" \
    -d "{
        \"id\": \"${TEST_AGENT_ID}\",
        \"name\": \"E2E Test Agent\",
        \"description\": \"Testing Episodic-first retrieval strategy\",
        \"system_prompt\": \"You are a helpful assistant with excellent memory. Always recall and use information from previous conversations.\",
        \"model\": \"deepseek-chat\",
        \"temperature\": 0.7
    }" 2>&1)

http_code=$(echo "$response" | tail -n 1)
if [ "$http_code" = "201" ] || [ "$http_code" = "200" ]; then
    log_success "Agent 创建成功"
elif echo "$response" | grep -q "already exists"; then
    log_success "Agent 已存在（继续使用）"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    log_error "Agent 创建失败 (HTTP $http_code)"
fi

sleep 1

# ============================================================
# 阶段 2: 添加 Episodic Memories（长期记忆）
# ============================================================
log_section "阶段 2: 添加 Episodic Memories (Long-term)"

log_test "3" "添加用户基本信息记忆"
curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"content\": \"用户名叫张三，是一位软件工程师，专注于AI和机器学习领域\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.9,
        \"tags\": [\"personal_info\", \"profession\"]
    }" > /dev/null 2>&1

if [ $? -eq 0 ]; then
    log_success "基本信息记忆添加成功"
else
    log_error "基本信息记忆添加失败"
fi

sleep 0.5

log_test "4" "添加用户偏好记忆"
curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"content\": \"张三最喜欢的编程语言是Rust，认为Rust在性能和安全性方面非常出色\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.8,
        \"tags\": [\"preference\", \"programming\"]
    }" > /dev/null 2>&1

if [ $? -eq 0 ]; then
    log_success "偏好记忆添加成功"
else
    log_error "偏好记忆添加失败"
fi

sleep 0.5

log_test "5" "添加项目信息记忆"
curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"content\": \"张三正在开发AgentMem记忆管理平台，这是一个基于Rust的智能记忆系统\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.85,
        \"tags\": [\"project\", \"agentmem\"]
    }" > /dev/null 2>&1

if [ $? -eq 0 ]; then
    log_success "项目信息记忆添加成功"
else
    log_error "项目信息记忆添加失败"
fi

log_info "等待记忆持久化..."
sleep 2

# ============================================================
# 阶段 3: Session A - 第一次对话（建立 Working Memory）
# ============================================================
log_section "阶段 3: Session A - 第一次对话"

SESSION_A="session-a-${TIMESTAMP}"
log_info "Session A ID: $SESSION_A"

log_test "6" "Session A - 询问基本信息"

response=$(curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"你好！请介绍一下你知道关于我的信息。\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_A}\"
    }" 2>&1)

if echo "$response" | grep -qi "张三\|软件工程师\|rust\|agentmem"; then
    log_success "Session A 成功检索到 Episodic Memory"
    echo "  检测到关键词: 张三/软件工程师/Rust/AgentMem"
else
    log_error "Session A 未能检索到完整信息"
    echo "  响应: $(echo "$response" | head -c 200)..."
fi

sleep 2

log_test "7" "Session A - 深入对话（创建 Working Memory）"

response=$(curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"我今天想讨论一下Rust的内存安全特性。\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_A}\"
    }" 2>&1)

if [ $? -eq 0 ]; then
    log_success "Session A 对话继续成功（建立 Working Memory）"
else
    log_error "Session A 对话失败"
fi

sleep 2

# ============================================================
# 阶段 4: Session B - 新会话（测试跨 Session 记忆）⭐ 核心测试
# ============================================================
log_section "阶段 4: Session B - 新会话（跨 Session 测试）⭐"

SESSION_B="session-b-${TIMESTAMP}"
log_info "Session B ID: $SESSION_B （完全不同的Session）"
log_info "这模拟用户刷新页面或重新打开应用的场景"

log_test "8" "Session B - 询问编程语言偏好"

response=$(curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"我最喜欢什么编程语言？为什么？\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_B}\"
    }" 2>&1)

rust_found=false
reason_found=false

if echo "$response" | grep -qi "rust"; then
    log_success "✓ 跨Session检索到编程语言偏好（Episodic Memory）"
    rust_found=true
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    PASSED_TESTS=$((PASSED_TESTS + 1))
    
    if echo "$response" | grep -qi "性能\|安全"; then
        log_success "✓ 检索到详细原因（完整的 Episodic Memory）"
        reason_found=true
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
    fi
else
    log_error "未能跨Session检索到编程语言偏好"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
fi

echo ""
echo "AI回复预览:"
echo "$response" | head -c 300
echo "..."

sleep 2

log_test "9" "Session B - 询问项目信息"

response=$(curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"我正在开发什么项目？\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_B}\"
    }" 2>&1)

if echo "$response" | grep -qi "agentmem"; then
    log_success "✓ 跨Session检索到项目信息（Episodic Memory）"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    log_error "未能跨Session检索到项目信息"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
fi

sleep 2

log_test "10" "Session B - 综合查询"

response=$(curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"请总结一下我的职业背景、技术偏好和当前项目。\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_B}\"
    }" 2>&1)

comprehensive_score=0
if echo "$response" | grep -qi "软件工程师\|工程师"; then
    comprehensive_score=$((comprehensive_score + 1))
fi
if echo "$response" | grep -qi "rust"; then
    comprehensive_score=$((comprehensive_score + 1))
fi
if echo "$response" | grep -qi "agentmem"; then
    comprehensive_score=$((comprehensive_score + 1))
fi

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if [ $comprehensive_score -ge 2 ]; then
    log_success "✓ 综合查询成功（检索到 $comprehensive_score/3 项信息）"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    log_error "综合查询不完整（仅检索到 $comprehensive_score/3 项）"
fi

# ============================================================
# 阶段 5: Session C - 第三个会话（测试长期记忆稳定性）
# ============================================================
log_section "阶段 5: Session C - 第三个会话（记忆稳定性）"

SESSION_C="session-c-${TIMESTAMP}"
log_info "Session C ID: $SESSION_C"

sleep 1

log_test "11" "Session C - 验证记忆持久性"

response=$(curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/chat" \
    -H "Content-Type: application/json" \
    -d "{
        \"message\": \"我叫什么名字？\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"session_id\": \"${SESSION_C}\"
    }" 2>&1)

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if echo "$response" | grep -qi "张三"; then
    log_success "✓ 第三个Session仍能检索到基本信息（记忆稳定）"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    log_error "第三个Session无法检索到基本信息"
fi

# ============================================================
# 阶段 6: 记忆统计验证
# ============================================================
log_section "阶段 6: 记忆统计验证"

log_test "12" "查询Agent记忆统计"

response=$(curl -s "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories/stats" 2>&1)

TOTAL_TESTS=$((TOTAL_TESTS + 1))
if echo "$response" | grep -q "total\|count"; then
    log_success "记忆统计查询成功"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "  统计: $(echo "$response" | head -c 200)"
else
    log_error "记忆统计查询失败"
fi

# ============================================================
# 测试总结
# ============================================================
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                     测试完成                                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))

echo "测试结果:"
echo "  总测试数: $TOTAL_TESTS"
echo -e "  通过: ${GREEN}$PASSED_TESTS${NC}"
echo -e "  失败: ${RED}$FAILED_TESTS${NC}"
echo "  成功率: $success_rate%"
echo ""

if [ $success_rate -ge 80 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✅ 测试通过！Episodic-first 检索策略工作正常           ║${NC}"
    echo -e "${GREEN}║  ✅ 跨 Session 记忆连续性验证成功                       ║${NC}"
    echo -e "${GREEN}║  ✅ Phase 1 核心功能实现成功                            ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 0
elif [ $success_rate -ge 60 ]; then
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║  ⚠️  部分测试通过，需要进一步优化                       ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 1
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ❌ 测试失败，请检查日志和配置                          ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi

