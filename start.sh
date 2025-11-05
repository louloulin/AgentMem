#!/usr/bin/env bash

# AgentMem 一键启动脚本
# 功能：快速启动完整的 AgentMem 服务（后端 + 前端）
# 日期：2025-11-05

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# 显示帮助信息
show_help() {
    cat << EOF
AgentMem 一键启动脚本

用法:
  $0 [选项]

选项:
  -h, --help          显示此帮助信息
  -b, --backend-only  仅启动后端
  -f, --frontend-only 仅启动前端
  -s, --stop          停止所有服务
  -r, --restart       重启所有服务
  -l, --logs          查看日志

示例:
  $0                  # 启动完整服务
  $0 -b               # 仅启动后端
  $0 -s               # 停止服务
  $0 -r               # 重启服务

EOF
}

# 停止服务
stop_services() {
    print_header "停止 AgentMem 服务"
    
    print_info "停止后端服务..."
    pkill -f agent-mem-server 2>/dev/null || print_warning "后端未运行"
    
    print_info "停止前端服务..."
    pkill -f "next dev" 2>/dev/null || print_warning "前端未运行"
    
    sleep 2
    print_success "所有服务已停止"
}

# 检查端口
check_port() {
    local port=$1
    if lsof -i :$port > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# 启动后端
start_backend() {
    print_header "启动后端服务"
    
    # 检查二进制文件
    if [ ! -f "./target/release/agent-mem-server" ]; then
        print_error "未找到编译后的服务器"
        print_info "正在编译..."
        cargo build --release --bin agent-mem-server --features agent-mem/plugins
    fi
    
    # 停止现有服务
    print_info "停止现有后端服务..."
    pkill -f agent-mem-server 2>/dev/null || true
    sleep 2
    
    # 配置环境变量
    export DYLD_LIBRARY_PATH="$SCRIPT_DIR/lib:$SCRIPT_DIR/target/release:$DYLD_LIBRARY_PATH"
    export ORT_DYLIB_PATH="$SCRIPT_DIR/lib/libonnxruntime.1.22.0.dylib"
    export EMBEDDER_PROVIDER="fastembed"
    export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
    export LLM_PROVIDER="zhipu"
    export LLM_MODEL="glm-4-plus"
    export ENABLE_AUTH="false"
    export SERVER_ENABLE_AUTH="false"
    export AGENT_MEM_ENABLE_AUTH="false"
    
    print_info "配置信息:"
    echo "  • Embedder: $EMBEDDER_PROVIDER ($EMBEDDER_MODEL)"
    echo "  • LLM: $LLM_PROVIDER ($LLM_MODEL)"
    echo "  • Auth: Disabled"
    echo ""
    
    # 启动服务器
    print_info "启动后端服务器..."
    nohup ./target/release/agent-mem-server > backend-no-auth.log 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > backend.pid
    
    # 等待启动
    print_info "等待后端就绪..."
    for i in {1..30}; do
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            print_success "后端启动成功！(PID: $BACKEND_PID)"
            return 0
        fi
        
        # 检查进程是否还在运行
        if ! kill -0 $BACKEND_PID 2>/dev/null; then
            print_error "后端进程已退出"
            echo ""
            print_info "最后20行日志:"
            tail -20 backend-no-auth.log
            return 1
        fi
        
        sleep 1
    done
    
    print_error "后端启动超时"
    return 1
}

# 启动前端
start_frontend() {
    print_header "启动前端服务"
    
    # 检查目录
    if [ ! -d "agentmem-ui" ]; then
        print_error "未找到前端目录: agentmem-ui"
        return 1
    fi
    
    # 停止现有服务
    print_info "停止现有前端服务..."
    pkill -f "next dev" 2>/dev/null || true
    sleep 2
    
    # 启动前端
    print_info "启动前端服务..."
    cd agentmem-ui
    nohup npm run dev > ../frontend.log 2>&1 &
    FRONTEND_PID=$!
    cd ..
    echo $FRONTEND_PID > frontend.pid
    
    # 等待启动
    print_info "等待前端就绪..."
    for i in {1..30}; do
        if curl -s http://localhost:3001 > /dev/null 2>&1; then
            print_success "前端启动成功！(PID: $FRONTEND_PID)"
            return 0
        fi
        sleep 1
    done
    
    print_error "前端启动超时"
    return 1
}

# 显示服务状态
show_status() {
    print_header "服务状态"
    
    # 后端状态
    if check_port 8080; then
        print_success "后端: 运行中 (http://localhost:8080)"
        if [ -f backend.pid ]; then
            echo "  PID: $(cat backend.pid)"
        fi
    else
        print_warning "后端: 未运行"
    fi
    
    # 前端状态
    if check_port 3001; then
        print_success "前端: 运行中 (http://localhost:3001)"
        if [ -f frontend.pid ]; then
            echo "  PID: $(cat frontend.pid)"
        fi
    else
        print_warning "前端: 未运行"
    fi
    
    echo ""
}

# 显示访问信息
show_access_info() {
    print_header "访问信息"
    
    cat << EOF
${GREEN}🌐 Web 界面${NC}
  • 主页:         http://localhost:3001
  • 记忆管理:     http://localhost:3001/admin/memories
  • 知识图谱:     http://localhost:3001/admin/graph
  • 插件管理:     http://localhost:3001/admin/plugins

${CYAN}🔧 API 端点${NC}
  • API文档:      http://localhost:8080/swagger-ui/
  • 健康检查:     http://localhost:8080/health
  • Metrics:      http://localhost:8080/metrics

${YELLOW}📝 日志文件${NC}
  • 后端日志:     tail -f backend-no-auth.log
  • 前端日志:     tail -f frontend.log

${PURPLE}🛠️  管理命令${NC}
  • 停止服务:     $0 -s
  • 重启服务:     $0 -r
  • 查看日志:     $0 -l
  • 查看状态:     just status

EOF
}

# 查看日志
show_logs() {
    print_header "查看日志"
    
    echo "选择要查看的日志:"
    echo "  1) 后端日志 (backend-no-auth.log)"
    echo "  2) 前端日志 (frontend.log)"
    echo "  3) 两者都看"
    echo ""
    read -p "请选择 [1-3]: " choice
    
    case $choice in
        1)
            print_info "后端日志 (按 Ctrl+C 退出):"
            tail -f backend-no-auth.log
            ;;
        2)
            print_info "前端日志 (按 Ctrl+C 退出):"
            tail -f frontend.log
            ;;
        3)
            print_info "查看两个日志 (按 Ctrl+C 退出):"
            tail -f backend-no-auth.log -f frontend.log
            ;;
        *)
            print_error "无效选择"
            ;;
    esac
}

# 主函数
main() {
    # 解析参数
    case "${1:-}" in
        -h|--help)
            show_help
            exit 0
            ;;
        -b|--backend-only)
            start_backend
            show_status
            ;;
        -f|--frontend-only)
            start_frontend
            show_status
            ;;
        -s|--stop)
            stop_services
            ;;
        -r|--restart)
            stop_services
            sleep 2
            start_backend
            start_frontend
            show_status
            show_access_info
            ;;
        -l|--logs)
            show_logs
            ;;
        "")
            # 默认：启动完整服务
            print_header "🚀 启动 AgentMem 完整服务"
            echo ""
            
            # 启动后端
            if ! start_backend; then
                print_error "后端启动失败"
                exit 1
            fi
            
            echo ""
            
            # 启动前端
            if ! start_frontend; then
                print_error "前端启动失败"
                exit 1
            fi
            
            echo ""
            
            # 显示状态和访问信息
            show_status
            show_access_info
            
            print_success "AgentMem 服务已全部启动！🎉"
            ;;
        *)
            print_error "未知选项: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"

