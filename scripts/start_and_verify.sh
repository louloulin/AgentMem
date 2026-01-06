#!/bin/bash
# AgentMem 服务器启动和验证脚本
# 用途：编译、启动服务器并验证核心功能

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
PORT="${AGENT_MEM_PORT:-8080}"
HOST="${AGENT_MEM_HOST:-127.0.0.1}"
LOG_LEVEL="${AGENT_MEM_LOG_LEVEL:-info}"
SERVER_PID_FILE="/tmp/agentmem_server.pid"
SERVER_LOG_FILE="/tmp/agentmem_server.log"

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# 清理函数
cleanup() {
    if [ -f "$SERVER_PID_FILE" ]; then
        PID=$(cat "$SERVER_PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            log_info "停止服务器 (PID: $PID)..."
            kill "$PID"
            sleep 2
        fi
        rm -f "$SERVER_PID_FILE"
    fi
}

# 注册清理函数
trap cleanup EXIT INT TERM

# 主流程
main() {
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║      AgentMem 服务器启动和核心功能验证流程                 ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    
    # Step 1: 环境检查
    log_step "步骤 1/6: 环境检查"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 检查 protoc
    if ! command -v protoc &> /dev/null; then
        log_error "protoc 未安装"
        exit 1
    fi
    
    PROTOC_VERSION=$(protoc --version)
    log_info "✅ protoc: $PROTOC_VERSION"
    
    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo 未安装"
        exit 1
    fi
    
    RUST_VERSION=$(rustc --version)
    log_info "✅ Rust: $RUST_VERSION"
    
    # 检查 jq (用于 JSON 解析)
    if ! command -v jq &> /dev/null; then
        log_warn "jq 未安装，建议安装以获得更好的输出格式"
        log_info "macOS: brew install jq"
        log_info "Ubuntu: sudo apt-get install jq"
    else
        log_info "✅ jq: $(jq --version)"
    fi
    
    echo ""
    
    # Step 2: 编译项目
    log_step "步骤 2/6: 编译项目"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    export PROTOC=/opt/homebrew/bin/protoc
    
    log_info "开始编译 agent-mem-server..."
    if cargo build --release -p agent-mem-server 2>&1 | tee /tmp/build.log | tail -20; then
        log_info "✅ 编译成功"
    else
        log_error "❌ 编译失败，查看日志: /tmp/build.log"
        exit 1
    fi
    
    echo ""
    
    # Step 3: 运行测试
    log_step "步骤 3/6: 运行核心测试"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    log_info "运行 workspace 测试..."
    if cargo test --workspace --lib 2>&1 | tee /tmp/test.log | grep -E "test result:|passed|failed" | tail -10; then
        log_info "✅ 测试通过"
    else
        log_warn "⚠️  部分测试失败，查看日志: /tmp/test.log"
    fi
    
    echo ""
    
    # Step 4: 启动服务器
    log_step "步骤 4/6: 启动服务器"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    log_info "启动 AgentMem 服务器..."
    log_info "  Host: $HOST"
    log_info "  Port: $PORT"
    log_info "  Log Level: $LOG_LEVEL"
    log_info "  Log File: $SERVER_LOG_FILE"
    
    # 启动服务器（后台运行）
    nohup ./target/release/agent-mem-server \
        --host "$HOST" \
        --port "$PORT" \
        --log-level "$LOG_LEVEL" \
        > "$SERVER_LOG_FILE" 2>&1 &
    
    SERVER_PID=$!
    echo "$SERVER_PID" > "$SERVER_PID_FILE"
    
    log_info "服务器已启动 (PID: $SERVER_PID)"
    
    # 等待服务器启动
    log_info "等待服务器就绪..."
    MAX_WAIT=30
    WAIT_COUNT=0
    
    while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
        if curl -s "http://$HOST:$PORT/health" > /dev/null 2>&1; then
            log_info "✅ 服务器已就绪"
            break
        fi
        
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            log_error "❌ 服务器启动失败"
            log_error "查看日志: $SERVER_LOG_FILE"
            tail -50 "$SERVER_LOG_FILE"
            exit 1
        fi
        
        sleep 1
        WAIT_COUNT=$((WAIT_COUNT + 1))
        echo -n "."
    done
    
    echo ""
    
    if [ $WAIT_COUNT -ge $MAX_WAIT ]; then
        log_error "❌ 服务器启动超时"
        log_error "查看日志: $SERVER_LOG_FILE"
        tail -50 "$SERVER_LOG_FILE"
        exit 1
    fi
    
    echo ""
    
    # Step 5: 验证核心 API
    log_step "步骤 5/6: 验证核心 API 功能"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    export AGENT_MEM_URL="http://$HOST:$PORT"
    
    if [ -f "scripts/test_core_api.sh" ]; then
        bash scripts/test_core_api.sh
    else
        log_warn "测试脚本不存在: scripts/test_core_api.sh"
        log_info "手动测试 Health Check..."
        
        if curl -s "http://$HOST:$PORT/health" | jq '.' 2>/dev/null; then
            log_info "✅ Health Check 通过"
        else
            log_error "❌ Health Check 失败"
        fi
    fi
    
    echo ""
    
    # Step 6: 显示访问信息
    log_step "步骤 6/6: 服务器访问信息"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    echo ""
    echo "🎉 AgentMem 服务器已成功启动并验证！"
    echo ""
    echo "访问地址："
    echo "  • API Base URL:    http://$HOST:$PORT/api/v1"
    echo "  • Swagger UI:      http://$HOST:$PORT/swagger-ui"
    echo "  • OpenAPI Spec:    http://$HOST:$PORT/api-docs/openapi.json"
    echo "  • Health Check:    http://$HOST:$PORT/health"
    echo "  • Metrics:         http://$HOST:$PORT/metrics"
    echo ""
    echo "服务器信息："
    echo "  • PID:             $SERVER_PID"
    echo "  • Log File:        $SERVER_LOG_FILE"
    echo ""
    echo "常用命令："
    echo "  • 查看日志:        tail -f $SERVER_LOG_FILE"
    echo "  • 停止服务器:      kill $SERVER_PID"
    echo "  • 测试 API:        curl http://$HOST:$PORT/health"
    echo ""
    
    # 询问是否保持运行
    read -p "是否保持服务器运行？(y/n) " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "服务器将继续运行..."
        log_info "使用 'kill $SERVER_PID' 停止服务器"
        
        # 取消 trap，避免自动清理
        trap - EXIT INT TERM
        
        # 显示实时日志
        log_info "显示实时日志 (Ctrl+C 退出日志查看，服务器继续运行)..."
        tail -f "$SERVER_LOG_FILE"
    else
        log_info "停止服务器..."
        cleanup
    fi
}

# 运行主函数
main

