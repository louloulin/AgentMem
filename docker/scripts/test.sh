#!/bin/bash

# AgentMem 测试脚本
# 用于运行各种测试和验证

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# 启动测试环境
start_test_env() {
    log_info "启动测试环境..."
    
    # 停止可能存在的测试容器
    docker-compose -f docker-compose.test.yml down -v 2>/dev/null || true
    
    # 启动测试环境
    docker-compose -f docker-compose.test.yml up -d chroma-test qdrant-test postgres-test redis-test
    
    log_info "等待服务启动..."
    sleep 30
    
    # 检查服务状态
    if ! curl -f -s http://localhost:18000/api/v1/heartbeat > /dev/null; then
        log_error "Chroma 测试服务未就绪"
        return 1
    fi
    
    if ! curl -f -s http://localhost:16333/health > /dev/null; then
        log_error "Qdrant 测试服务未就绪"
        return 1
    fi
    
    log_success "测试环境启动完成"
}

# 停止测试环境
stop_test_env() {
    log_info "停止测试环境..."
    docker-compose -f docker-compose.test.yml down -v
    log_success "测试环境已停止"
}

# 运行单元测试
run_unit_tests() {
    log_info "运行单元测试..."
    
    cd ..
    cargo test --lib --bins
    
    if [ $? -eq 0 ]; then
        log_success "单元测试通过"
    else
        log_error "单元测试失败"
        return 1
    fi
}

# 运行集成测试
run_integration_tests() {
    log_info "运行集成测试..."
    
    start_test_env
    
    cd ..
    
    # 设置测试环境变量
    export CHROMA_URL="http://localhost:18000"
    export QDRANT_URL="http://localhost:16333"
    export POSTGRES_URL="postgresql://test_user:test_password@localhost:15432/agentmem_test"
    export REDIS_URL="redis://:test_password@localhost:16379"
    
    # 运行集成测试
    cargo test --features integration-tests integration_tests
    
    local result=$?
    
    stop_test_env
    
    if [ $result -eq 0 ]; then
        log_success "集成测试通过"
    else
        log_error "集成测试失败"
        return 1
    fi
}

# 运行性能测试
run_performance_tests() {
    log_info "运行性能测试..."
    
    start_test_env
    
    cd ..
    
    # 设置测试环境变量
    export CHROMA_URL="http://localhost:18000"
    export QDRANT_URL="http://localhost:16333"
    export POSTGRES_URL="postgresql://test_user:test_password@localhost:15432/agentmem_test"
    export REDIS_URL="redis://:test_password@localhost:16379"
    
    # 运行性能测试
    cargo test --release --features integration-tests performance_tests
    
    local result=$?
    
    stop_test_env
    
    if [ $result -eq 0 ]; then
        log_success "性能测试通过"
    else
        log_error "性能测试失败"
        return 1
    fi
}

# 运行兼容性测试
run_compatibility_tests() {
    log_info "运行 Mem0 兼容性测试..."
    
    start_test_env
    
    cd ..
    
    # 设置测试环境变量
    export CHROMA_URL="http://localhost:18000"
    export QDRANT_URL="http://localhost:16333"
    export POSTGRES_URL="postgresql://test_user:test_password@localhost:15432/agentmem_test"
    export REDIS_URL="redis://:test_password@localhost:16379"
    
    # 运行兼容性测试
    cargo test --package agent-mem-compat compatibility_tests
    
    local result=$?
    
    stop_test_env
    
    if [ $result -eq 0 ]; then
        log_success "兼容性测试通过"
    else
        log_error "兼容性测试失败"
        return 1
    fi
}

# 运行压力测试
run_stress_tests() {
    log_info "运行压力测试..."
    
    start_test_env
    
    cd ..
    
    # 设置测试环境变量
    export CHROMA_URL="http://localhost:18000"
    export QDRANT_URL="http://localhost:16333"
    export POSTGRES_URL="postgresql://test_user:test_password@localhost:15432/agentmem_test"
    export REDIS_URL="redis://:test_password@localhost:16379"
    
    # 运行压力测试
    cargo run --release --example stress-test -- --concurrent 50 --duration 30s
    
    local result=$?
    
    stop_test_env
    
    if [ $result -eq 0 ]; then
        log_success "压力测试通过"
    else
        log_error "压力测试失败"
        return 1
    fi
}

# 运行基准测试
run_benchmark_tests() {
    log_info "运行基准测试..."
    
    start_test_env
    
    cd ..
    
    # 设置测试环境变量
    export CHROMA_URL="http://localhost:18000"
    export QDRANT_URL="http://localhost:16333"
    export POSTGRES_URL="postgresql://test_user:test_password@localhost:15432/agentmem_test"
    export REDIS_URL="redis://:test_password@localhost:16379"
    
    # 运行基准测试
    cargo run --release --example performance-benchmark
    
    local result=$?
    
    stop_test_env
    
    if [ $result -eq 0 ]; then
        log_success "基准测试完成"
    else
        log_error "基准测试失败"
        return 1
    fi
}

# 运行所有测试
run_all_tests() {
    log_info "运行所有测试..."
    
    local failed_tests=()
    
    # 运行单元测试
    if ! run_unit_tests; then
        failed_tests+=("unit")
    fi
    
    # 运行集成测试
    if ! run_integration_tests; then
        failed_tests+=("integration")
    fi
    
    # 运行兼容性测试
    if ! run_compatibility_tests; then
        failed_tests+=("compatibility")
    fi
    
    # 运行性能测试
    if ! run_performance_tests; then
        failed_tests+=("performance")
    fi
    
    # 检查结果
    if [ ${#failed_tests[@]} -eq 0 ]; then
        log_success "所有测试通过！"
        return 0
    else
        log_error "以下测试失败: ${failed_tests[*]}"
        return 1
    fi
}

# 检查 Mock 清理进度
check_mock_cleanup() {
    log_info "检查 Mock 清理进度..."
    
    cd ..
    
    local mock_count=$(find . -name "*.rs" -exec grep -l "mock\|Mock\|模拟" {} \; | wc -l)
    
    echo "当前 Mock 文件数量: $mock_count"
    
    if [ "$mock_count" -eq 0 ]; then
        log_success "🎉 所有 Mock 已清理完成！"
    else
        log_warning "还有 $mock_count 个文件包含 Mock 实现"
        echo ""
        echo "剩余 Mock 文件:"
        find . -name "*.rs" -exec grep -l "mock\|Mock\|模拟" {} \; | head -10
    fi
}

# 代码质量检查
check_code_quality() {
    log_info "运行代码质量检查..."
    
    cd ..
    
    # Clippy 检查
    log_info "运行 Clippy..."
    cargo clippy -- -D warnings
    
    # 格式检查
    log_info "检查代码格式..."
    cargo fmt -- --check
    
    # 安全审计
    log_info "运行安全审计..."
    cargo audit
    
    log_success "代码质量检查完成"
}

# 显示帮助信息
show_help() {
    echo "AgentMem 测试脚本"
    echo ""
    echo "用法: $0 [命令]"
    echo ""
    echo "命令:"
    echo "  unit              运行单元测试"
    echo "  integration       运行集成测试"
    echo "  performance       运行性能测试"
    echo "  compatibility     运行兼容性测试"
    echo "  stress            运行压力测试"
    echo "  benchmark         运行基准测试"
    echo "  all               运行所有测试"
    echo "  mock-check        检查 Mock 清理进度"
    echo "  quality           代码质量检查"
    echo "  start-env         启动测试环境"
    echo "  stop-env          停止测试环境"
    echo "  help              显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 unit           # 运行单元测试"
    echo "  $0 integration    # 运行集成测试"
    echo "  $0 all            # 运行所有测试"
    echo ""
}

# 主函数
main() {
    local command=${1:-"help"}
    
    case $command in
        "unit")
            run_unit_tests
            ;;
        "integration")
            run_integration_tests
            ;;
        "performance")
            run_performance_tests
            ;;
        "compatibility")
            run_compatibility_tests
            ;;
        "stress")
            run_stress_tests
            ;;
        "benchmark")
            run_benchmark_tests
            ;;
        "all")
            run_all_tests
            ;;
        "mock-check")
            check_mock_cleanup
            ;;
        "quality")
            check_code_quality
            ;;
        "start-env")
            start_test_env
            ;;
        "stop-env")
            stop_test_env
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# 切换到脚本目录
cd "$(dirname "$0")/.."

# 执行主函数
main "$@"
