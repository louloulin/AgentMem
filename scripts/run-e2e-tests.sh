#!/bin/bash

# E2E 测试运行脚本
# 
# 用法:
#   ./scripts/run-e2e-tests.sh [options]
#
# 选项:
#   --start-server    启动测试服务器
#   --stop-server     停止测试服务器
#   --clean           清理测试数据
#   --verbose         详细输出

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_PID_FILE="/tmp/agentmem-test-server.pid"
SERVER_LOG_FILE="/tmp/agentmem-test-server.log"
SERVER_PORT=3000
SERVER_HOST="127.0.0.1"

# 函数：打印信息
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# 函数：打印成功
success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# 函数：打印警告
warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# 函数：打印错误
error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 函数：检查服务器是否运行
check_server() {
    if curl -s "http://${SERVER_HOST}:${SERVER_PORT}/health" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# 函数：启动测试服务器
start_server() {
    info "Starting test server..."
    
    # 检查服务器是否已经运行
    if check_server; then
        warning "Server is already running"
        return 0
    fi
    
    # 设置测试环境变量
    export AGENTMEM_HOST="${SERVER_HOST}"
    export AGENTMEM_PORT="${SERVER_PORT}"
    export DATABASE_URL="${DATABASE_URL:-sqlite://agentmem_test.db}"
    export RUST_LOG="${RUST_LOG:-info}"
    
    # 启动服务器
    cd "${BASE_DIR}"
    cargo run --bin agent-mem-server > "${SERVER_LOG_FILE}" 2>&1 &
    SERVER_PID=$!
    
    # 保存 PID
    echo "${SERVER_PID}" > "${SERVER_PID_FILE}"
    
    # 等待服务器启动
    info "Waiting for server to start..."
    for i in {1..30}; do
        if check_server; then
            success "Server started successfully (PID: ${SERVER_PID})"
            return 0
        fi
        sleep 1
    done
    
    error "Server failed to start"
    cat "${SERVER_LOG_FILE}"
    return 1
}

# 函数：停止测试服务器
stop_server() {
    info "Stopping test server..."
    
    if [ -f "${SERVER_PID_FILE}" ]; then
        SERVER_PID=$(cat "${SERVER_PID_FILE}")
        if kill -0 "${SERVER_PID}" 2>/dev/null; then
            kill "${SERVER_PID}"
            rm -f "${SERVER_PID_FILE}"
            success "Server stopped (PID: ${SERVER_PID})"
        else
            warning "Server process not found (PID: ${SERVER_PID})"
            rm -f "${SERVER_PID_FILE}"
        fi
    else
        warning "Server PID file not found"
    fi
}

# 函数：清理测试数据
clean_test_data() {
    info "Cleaning test data..."
    
    # 删除测试数据库
    if [ -f "agentmem_test.db" ]; then
        rm -f agentmem_test.db
        success "Test database deleted"
    fi
    
    # 删除日志文件
    if [ -f "${SERVER_LOG_FILE}" ]; then
        rm -f "${SERVER_LOG_FILE}"
        success "Server log deleted"
    fi
}

# 函数：运行 E2E 测试
run_tests() {
    local verbose=$1
    
    info "Running E2E tests..."
    
    cd "${BASE_DIR}"
    
    if [ "${verbose}" = "true" ]; then
        cargo test --test e2e_api_test -- --ignored --test-threads=1 --nocapture
    else
        cargo test --test e2e_api_test -- --ignored --test-threads=1
    fi
    
    if [ $? -eq 0 ]; then
        success "All E2E tests passed! 🎉"
        return 0
    else
        error "Some E2E tests failed"
        return 1
    fi
}

# 函数：显示帮助
show_help() {
    cat << EOF
E2E 测试运行脚本

用法:
  $0 [options]

选项:
  --start-server    启动测试服务器
  --stop-server     停止测试服务器
  --clean           清理测试数据
  --verbose         详细输出
  --help            显示此帮助信息

示例:
  # 运行完整的 E2E 测试流程
  $0

  # 只启动服务器
  $0 --start-server

  # 运行测试（详细输出）
  $0 --verbose

  # 停止服务器并清理
  $0 --stop-server --clean
EOF
}

# 主函数
main() {
    local start_server_flag=false
    local stop_server_flag=false
    local clean_flag=false
    local verbose_flag=false
    local run_tests_flag=true
    
    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --start-server)
                start_server_flag=true
                run_tests_flag=false
                shift
                ;;
            --stop-server)
                stop_server_flag=true
                run_tests_flag=false
                shift
                ;;
            --clean)
                clean_flag=true
                shift
                ;;
            --verbose)
                verbose_flag=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 执行操作
    if [ "${start_server_flag}" = "true" ]; then
        start_server
        exit $?
    fi
    
    if [ "${stop_server_flag}" = "true" ]; then
        stop_server
        if [ "${clean_flag}" = "true" ]; then
            clean_test_data
        fi
        exit 0
    fi
    
    if [ "${clean_flag}" = "true" ] && [ "${run_tests_flag}" = "false" ]; then
        clean_test_data
        exit 0
    fi
    
    # 默认：运行完整的测试流程
    if [ "${run_tests_flag}" = "true" ]; then
        info "Starting E2E test suite..."
        
        # 1. 清理旧数据
        if [ "${clean_flag}" = "true" ]; then
            clean_test_data
        fi
        
        # 2. 启动服务器
        start_server || exit 1
        
        # 3. 运行测试
        run_tests "${verbose_flag}"
        TEST_RESULT=$?
        
        # 4. 停止服务器
        stop_server
        
        # 5. 清理（如果指定）
        if [ "${clean_flag}" = "true" ]; then
            clean_test_data
        fi
        
        exit ${TEST_RESULT}
    fi
}

# 运行主函数
main "$@"

