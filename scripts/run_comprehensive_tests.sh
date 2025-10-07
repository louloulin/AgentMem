#!/bin/bash

# AgentMem 全面测试执行脚本
# 参考 MIRIX 测试系统设计
# 版本: 1.0
# 日期: 2025-10-07

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 测试统计
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
START_TIME=$(date +%s)

# 打印带颜色的消息
print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC} $1"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
}

print_section() {
    echo -e "\n${CYAN}▶ $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${PURPLE}ℹ${NC} $1"
}

# 运行测试并记录结果
run_test_suite() {
    local test_name=$1
    local test_command=$2
    
    echo -e "\n${CYAN}Running: ${test_name}${NC}"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        print_success "$test_name passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        print_error "$test_name failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# 检查依赖
check_dependencies() {
    print_section "Checking Dependencies"
    
    # 检查 Rust
    if command -v cargo &> /dev/null; then
        print_success "Rust/Cargo installed: $(cargo --version)"
    else
        print_error "Rust/Cargo not found. Please install from https://rustup.rs/"
        exit 1
    fi
    
    # 检查 PostgreSQL
    if command -v psql &> /dev/null; then
        print_success "PostgreSQL installed: $(psql --version)"
    else
        print_warning "PostgreSQL not found. Some tests may be skipped."
    fi
    
    # 检查 Redis
    if command -v redis-cli &> /dev/null; then
        print_success "Redis installed: $(redis-cli --version)"
    else
        print_warning "Redis not found. Some tests may be skipped."
    fi
    
    # 检查测试工具
    if cargo install --list | grep -q "cargo-tarpaulin"; then
        print_success "cargo-tarpaulin installed"
    else
        print_info "Installing cargo-tarpaulin for coverage..."
        cargo install cargo-tarpaulin
    fi
}

# 设置测试环境
setup_test_environment() {
    print_section "Setting Up Test Environment"
    
    # 设置环境变量
    export RUST_BACKTRACE=1
    export RUST_LOG=debug
    export DATABASE_URL=${DATABASE_URL:-"postgres://postgres:postgres@localhost/agentmem_test"}
    export REDIS_URL=${REDIS_URL:-"redis://localhost:6379/15"}
    
    print_info "DATABASE_URL: $DATABASE_URL"
    print_info "REDIS_URL: $REDIS_URL"
    
    # 创建测试数据库（如果不存在）
    if command -v psql &> /dev/null; then
        print_info "Creating test database..."
        psql -U postgres -c "DROP DATABASE IF EXISTS agentmem_test;" 2>/dev/null || true
        psql -U postgres -c "CREATE DATABASE agentmem_test;" 2>/dev/null || true
        print_success "Test database ready"
    fi
}

# Phase 1: 单元测试
run_unit_tests() {
    print_header "Phase 1: Unit Tests"
    
    print_section "Running All Unit Tests"
    run_test_suite "Unit Tests - All Crates" \
        "cargo test --lib --all-features --verbose"
    
    print_section "Running Core Module Tests"
    run_test_suite "Core Memory Tests" \
        "cargo test --package agent-mem-core --lib core_memory --verbose"
    
    run_test_suite "Agent State Tests" \
        "cargo test --package agent-mem-core --lib agent_state --verbose"
    
    run_test_suite "Hierarchy Tests" \
        "cargo test --package agent-mem-core --lib hierarchy --verbose"
    
    print_section "Running Storage Tests"
    run_test_suite "Storage Backend Tests" \
        "cargo test --package agent-mem-storage --lib --verbose"
    
    print_section "Running LLM Provider Tests"
    run_test_suite "LLM Provider Tests" \
        "cargo test --package agent-mem-llm --lib --verbose"
    
    print_section "Running Embeddings Tests"
    run_test_suite "Embeddings Tests" \
        "cargo test --package agent-mem-embeddings --lib --verbose"
    
    print_section "Running Tools Tests"
    run_test_suite "Tools Tests" \
        "cargo test --package agent-mem-tools --lib --verbose"
    
    print_section "Running Utils Tests"
    run_test_suite "Utils Tests" \
        "cargo test --package agent-mem-utils --lib --verbose"
}

# Phase 2: 集成测试
run_integration_tests() {
    print_header "Phase 2: Integration Tests"
    
    print_section "Running Server Integration Tests"
    run_test_suite "Server Integration Tests" \
        "cargo test --package agent-mem-server --test integration_tests --verbose"
    
    run_test_suite "Auth Integration Tests" \
        "cargo test --package agent-mem-server --test auth_integration_test --verbose"
    
    run_test_suite "Chat API Tests" \
        "cargo test --package agent-mem-server --test chat_api_test --verbose"
    
    run_test_suite "Streaming Tests" \
        "cargo test --package agent-mem-server --test streaming_test --verbose"
    
    print_section "Running Core Integration Tests"
    run_test_suite "Orchestrator Integration Tests" \
        "cargo test --package agent-mem-core --test orchestrator_integration_test --verbose"
    
    run_test_suite "Tool Calling Tests" \
        "cargo test --package agent-mem-core --test tool_calling_test --verbose"
}

# Phase 3: E2E 测试
run_e2e_tests() {
    print_header "Phase 3: End-to-End Tests"
    
    print_section "Running E2E Workflow Tests"
    run_test_suite "E2E Workflow Tests" \
        "cargo test --package agent-mem-server --test e2e_workflow_test --verbose"
    
    print_section "Running Real Implementation Tests"
    run_test_suite "Real Implementation Tests" \
        "cargo test --test integration_real_implementations --verbose"
}

# Phase 4: 性能基准测试
run_benchmark_tests() {
    print_header "Phase 4: Performance Benchmarks"
    
    print_section "Running Storage Benchmarks"
    if [ "$SKIP_BENCHMARKS" != "true" ]; then
        run_test_suite "Storage Benchmarks" \
            "cargo bench --package agent-mem-storage --verbose"
        
        print_section "Running Search Benchmarks"
        run_test_suite "Search Benchmarks" \
            "cargo bench --package agent-mem-core --verbose"
    else
        print_warning "Benchmarks skipped (SKIP_BENCHMARKS=true)"
    fi
}

# 生成测试覆盖率报告
generate_coverage_report() {
    print_header "Generating Coverage Report"
    
    print_section "Running Coverage Analysis"
    
    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin \
            --out Html \
            --out Xml \
            --output-dir coverage \
            --all-features \
            --workspace \
            --exclude-files "examples/*" "tests/*" "benches/*" \
            --verbose
        
        print_success "Coverage report generated in coverage/"
        
        # 显示覆盖率摘要
        if [ -f "coverage/cobertura.xml" ]; then
            print_info "Coverage report: coverage/index.html"
        fi
    else
        print_warning "cargo-tarpaulin not available, skipping coverage"
    fi
}

# 打印测试摘要
print_test_summary() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    local minutes=$((duration / 60))
    local seconds=$((duration % 60))
    
    echo ""
    print_header "Test Execution Summary"
    
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC} 📊 Overall Results                                              ${CYAN}║${NC}"
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC}   Total Test Suites: ${TOTAL_TESTS}                                        ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   ${GREEN}✓${NC} Passed: ${PASSED_TESTS}                                              ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   ${RED}✗${NC} Failed: ${FAILED_TESTS}                                              ${CYAN}║${NC}"
    
    if [ $TOTAL_TESTS -gt 0 ]; then
        local success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo -e "${CYAN}║${NC}   📈 Success Rate: ${success_rate}%                                    ${CYAN}║${NC}"
    fi
    
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC} ⏱️  Performance                                                 ${CYAN}║${NC}"
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC}   Total Duration: ${minutes}m ${seconds}s                                   ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}✅ All tests passed!${NC}\n"
        return 0
    else
        echo -e "\n${RED}❌ Some tests failed. Please check the output above.${NC}\n"
        return 1
    fi
}

# 主函数
main() {
    clear
    
    print_header "AgentMem Comprehensive Test Suite v1.0"
    echo -e "${PURPLE}参考 MIRIX 测试系统设计${NC}\n"
    
    # 解析命令行参数
    SKIP_BENCHMARKS=false
    SKIP_COVERAGE=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-benchmarks)
                SKIP_BENCHMARKS=true
                shift
                ;;
            --skip-coverage)
                SKIP_COVERAGE=true
                shift
                ;;
            --unit-only)
                UNIT_ONLY=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --skip-benchmarks    Skip performance benchmarks"
                echo "  --skip-coverage      Skip coverage report generation"
                echo "  --unit-only          Run only unit tests"
                echo "  --help               Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # 执行测试流程
    check_dependencies
    setup_test_environment
    
    # Phase 1: 单元测试
    run_unit_tests
    
    if [ "$UNIT_ONLY" != "true" ]; then
        # Phase 2: 集成测试
        run_integration_tests
        
        # Phase 3: E2E 测试
        run_e2e_tests
        
        # Phase 4: 性能测试
        run_benchmark_tests
        
        # 生成覆盖率报告
        if [ "$SKIP_COVERAGE" != "true" ]; then
            generate_coverage_report
        fi
    fi
    
    # 打印摘要
    print_test_summary
}

# 运行主函数
main "$@"

