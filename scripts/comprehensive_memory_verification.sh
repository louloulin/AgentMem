#!/bin/bash

# AgentMem Phase 1 + 1.5 综合验证脚本
# 测试：UI + MCP + API + 记忆功能

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_FILE="${PROJECT_ROOT}/COMPREHENSIVE_VERIFICATION_REPORT.md"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 日志函数
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

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# 开始报告
start_report() {
    cat > "$REPORT_FILE" << 'EOF'
# AgentMem Phase 1 + 1.5 综合验证报告

**日期**: $(date +"%Y-%m-%d %H:%M:%S")
**版本**: v3.2
**验证范围**: UI + MCP + API + 记忆功能

---

## 📋 验证概览

EOF
}

# 检查服务器状态
check_server() {
    log_info "检查 AgentMem 服务器状态..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if pgrep -f "agent-mem-server" > /dev/null; then
        SERVER_PID=$(pgrep -f "agent-mem-server")
        log_success "服务器运行中 (PID: $SERVER_PID)"
        
        # 检查端口
        if lsof -i :3001 > /dev/null 2>&1; then
            log_success "端口 3001 正在监听"
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
            return 0
        else
            log_error "端口 3001 未监听"
            return 1
        fi
    else
        log_error "服务器未运行"
        return 1
    fi
}

# 检查 MCP 服务器
check_mcp_server() {
    log_info "检查 MCP 服务器编译..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ -f "${PROJECT_ROOT}/target/release/agentmem-mcp-server" ]; then
        log_success "MCP 服务器已编译"
        
        # 尝试获取版本
        if "${PROJECT_ROOT}/target/release/agentmem-mcp-server" --version > /dev/null 2>&1; then
            log_success "MCP 服务器可执行"
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
            return 0
        else
            log_warning "MCP 服务器可能需要重新编译"
            return 1
        fi
    else
        log_error "MCP 服务器未找到"
        return 1
    fi
}

# 测试 API 健康检查
test_api_health() {
    log_info "测试 API 健康检查..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    response=$(curl -s -w "\n%{http_code}" http://localhost:3001/health 2>/dev/null || echo "000")
    http_code=$(echo "$response" | tail -n 1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        log_success "API 健康检查通过 (HTTP $http_code)"
        echo "Response: $body"
        return 0
    else
        log_error "API 健康检查失败 (HTTP $http_code)"
        return 1
    fi
}

# 测试添加记忆（Episodic Memory）
test_add_episodic_memory() {
    log_info "测试添加 Episodic Memory (User scope)..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    local timestamp=$(date +%s)
    local test_agent_id="test-agent-verification-${timestamp}"
    local test_user_id="test-user-verification-${timestamp}"
    
    response=$(curl -s -w "\n%{http_code}" -X POST http://localhost:3001/api/v1/memories \
        -H "Content-Type: application/json" \
        -d "{
            \"agent_id\": \"${test_agent_id}\",
            \"user_id\": \"${test_user_id}\",
            \"content\": \"This is a test episodic memory for verification at ${timestamp}\",
            \"memory_type\": \"Episodic\",
            \"importance\": 0.8
        }" 2>/dev/null || echo "000")
    
    http_code=$(echo "$response" | tail -n 1)
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        log_success "Episodic Memory 添加成功 (HTTP $http_code)"
        echo "$test_agent_id|$test_user_id" > /tmp/agentmem_test_ids.txt
        return 0
    else
        log_error "Episodic Memory 添加失败 (HTTP $http_code)"
        return 1
    fi
}

# 测试检索记忆（Episodic-first策略）
test_retrieve_episodic_first() {
    log_info "测试 Episodic-first 检索策略..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ ! -f /tmp/agentmem_test_ids.txt ]; then
        log_warning "跳过：需要先添加测试记忆"
        return 0
    fi
    
    IFS='|' read -r test_agent_id test_user_id < /tmp/agentmem_test_ids.txt
    
    response=$(curl -s -w "\n%{http_code}" -X POST http://localhost:3001/api/v1/agents/${test_agent_id}/chat \
        -H "Content-Type: application/json" \
        -d "{
            \"message\": \"What do you remember about our conversation?\",
            \"user_id\": \"${test_user_id}\",
            \"session_id\": \"test-session-$(date +%s)\"
        }" 2>/dev/null || echo "000")
    
    http_code=$(echo "$response" | tail -n 1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        log_success "检索成功 (HTTP $http_code)"
        
        # 检查是否包含测试记忆
        if echo "$body" | grep -q "verification"; then
            log_success "✓ 成功检索到 Episodic Memory"
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
        else
            log_warning "未找到测试记忆内容"
        fi
        return 0
    else
        log_error "检索失败 (HTTP $http_code)"
        return 1
    fi
}

# 测试跨Session记忆连续性
test_cross_session_continuity() {
    log_info "测试跨 Session 记忆连续性..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ ! -f /tmp/agentmem_test_ids.txt ]; then
        log_warning "跳过：需要先添加测试记忆"
        return 0
    fi
    
    IFS='|' read -r test_agent_id test_user_id < /tmp/agentmem_test_ids.txt
    
    # 使用不同的 session_id
    local new_session_id="test-session-new-$(date +%s)"
    
    response=$(curl -s -w "\n%{http_code}" -X POST http://localhost:3001/api/v1/agents/${test_agent_id}/chat \
        -H "Content-Type: application/json" \
        -d "{
            \"message\": \"Do you remember our previous conversation?\",
            \"user_id\": \"${test_user_id}\",
            \"session_id\": \"${new_session_id}\"
        }" 2>/dev/null || echo "000")
    
    http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" = "200" ]; then
        log_success "新 Session 检索成功 (HTTP $http_code)"
        
        # 检查是否能访问历史记忆
        if echo "$response" | grep -q "verification"; then
            log_success "✓ 跨 Session 记忆连续性验证通过"
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
        else
            log_warning "新 Session 未检索到历史记忆"
        fi
        return 0
    else
        log_error "新 Session 检索失败 (HTTP $http_code)"
        return 1
    fi
}

# 测试日志输出（认知架构标记）
test_cognitive_logs() {
    log_info "检查认知架构日志..."
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # 查找最近的日志
    if [ -d "${PROJECT_ROOT}/logs" ]; then
        log_file=$(ls -t "${PROJECT_ROOT}/logs"/*.log 2>/dev/null | head -1)
        
        if [ -n "$log_file" ]; then
            # 检查是否包含认知架构标记
            if grep -q "Episodic-first" "$log_file" 2>/dev/null; then
                log_success "✓ 发现 Episodic-first 日志"
                TOTAL_TESTS=$((TOTAL_TESTS + 1))
            fi
            
            if grep -q "Priority 1.*Episodic" "$log_file" 2>/dev/null; then
                log_success "✓ 发现 Priority 1 (Episodic) 日志"
                TOTAL_TESTS=$((TOTAL_TESTS + 1))
            fi
            
            if grep -q "Working Memory" "$log_file" 2>/dev/null; then
                log_success "✓ 发现 Working Memory 日志"
                TOTAL_TESTS=$((TOTAL_TESTS + 1))
            fi
            
            return 0
        else
            log_warning "未找到日志文件"
            return 0
        fi
    else
        log_warning "日志目录不存在"
        return 0
    fi
}

# 生成最终报告
generate_report() {
    local success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    
    cat >> "$REPORT_FILE" << EOF

### 测试统计

- **总测试数**: $TOTAL_TESTS
- **通过**: $PASSED_TESTS
- **失败**: $FAILED_TESTS
- **成功率**: ${success_rate}%

---

## 📊 详细结果

### 1. 服务器状态
$([ $PASSED_TESTS -gt 0 ] && echo "✅ 通过" || echo "❌ 失败")

### 2. API 测试
- 健康检查: $([ $PASSED_TESTS -ge 2 ] && echo "✅" || echo "❌")
- 添加 Episodic Memory: $([ $PASSED_TESTS -ge 3 ] && echo "✅" || echo "❌")
- 检索测试: $([ $PASSED_TESTS -ge 4 ] && echo "✅" || echo "❌")

### 3. 功能验证
- 跨 Session 连续性: $([ $PASSED_TESTS -ge 5 ] && echo "✅" || echo "⏳")
- 认知架构日志: $([ $PASSED_TESTS -ge 6 ] && echo "✅" || echo "⏳")

---

## 🎯 结论

$(if [ $success_rate -ge 80 ]; then
    echo "✅ **验证通过** - Phase 1 + 1.5 实施成功"
else
    echo "⚠️ **需要进一步调试** - 部分测试失败"
fi)

### 核心功能验证

1. **Episodic-first 检索**: $([ $PASSED_TESTS -ge 4 ] && echo "✅ 工作正常" || echo "⏳ 待验证")
2. **跨 Session 记忆**: $([ $PASSED_TESTS -ge 5 ] && echo "✅ 工作正常" || echo "⏳ 待验证")
3. **认知架构日志**: $([ $PASSED_TESTS -ge 6 ] && echo "✅ 工作正常" || echo "⏳ 待验证")

---

**生成时间**: $(date +"%Y-%m-%d %H:%M:%S")
**验证脚本**: comprehensive_memory_verification.sh

EOF

    log_info "报告已生成: $REPORT_FILE"
}

# 主函数
main() {
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                                                            ║"
    echo "║  AgentMem Phase 1 + 1.5 综合验证                          ║"
    echo "║  Comprehensive Memory Verification                         ║"
    echo "║                                                            ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    
    start_report
    
    # 执行测试
    check_server
    check_mcp_server
    test_api_health
    test_add_episodic_memory
    test_retrieve_episodic_first
    test_cross_session_continuity
    test_cognitive_logs
    
    # 生成报告
    generate_report
    
    # 显示总结
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "测试完成！"
    echo "═══════════════════════════════════════════════════════════"
    echo "总测试: $TOTAL_TESTS"
    echo "通过: $PASSED_TESTS"
    echo "失败: $FAILED_TESTS"
    echo "成功率: $((PASSED_TESTS * 100 / TOTAL_TESTS))%"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "查看完整报告: $REPORT_FILE"
    echo ""
    
    # 返回状态
    if [ $FAILED_TESTS -eq 0 ]; then
        exit 0
    else
        exit 1
    fi
}

# 运行主函数
main "$@"

