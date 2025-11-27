#!/bin/bash
# AgentMem Linux amd64 使用 cross 工具交叉编译脚本
# cross 是一个基于 Docker 的 Rust 交叉编译工具，简化了交叉编译过程

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 代理设置（支持从环境变量或参数传入）
# 使用方法：
#   1. 通过环境变量: export HTTP_PROXY='http://127.0.0.1:7890' && bash build-linux-cross.sh
#   2. 通过参数: bash build-linux-cross.sh --proxy http://127.0.0.1:7890
#   3. 使用系统代理: bash build-linux-cross.sh --proxy-system

setup_proxy() {
    local proxy_arg="$1"
    
    if [ "$proxy_arg" = "--proxy-system" ]; then
        # 尝试从系统环境变量读取代理
        if [ -n "$http_proxy" ]; then
            export HTTP_PROXY="$http_proxy"
        fi
        if [ -n "$https_proxy" ]; then
            export HTTPS_PROXY="$https_proxy"
        fi
        if [ -n "$all_proxy" ]; then
            export ALL_PROXY="$all_proxy"
        fi
    elif [ -n "$proxy_arg" ] && [ "$proxy_arg" != "--proxy" ]; then
        # 设置指定的代理地址
        export HTTP_PROXY="$proxy_arg"
        export HTTPS_PROXY="$proxy_arg"
        if [[ "$proxy_arg" == socks5://* ]]; then
            export ALL_PROXY="$proxy_arg"
        fi
    fi
    
    # 如果已设置代理，传递给 Docker 和 cross
    if [ -n "$HTTP_PROXY" ] || [ -n "$HTTPS_PROXY" ]; then
        export DOCKER_BUILDKIT=1
        # cross 工具会自动使用 HTTP_PROXY 和 HTTPS_PROXY
        log_info "代理设置: HTTP_PROXY=${HTTP_PROXY:-未设置}, HTTPS_PROXY=${HTTPS_PROXY:-未设置}"
    fi
}

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# 检查 cross 工具
check_cross() {
    if ! command -v cross &> /dev/null; then
        log_error "cross 工具未安装"
        log_info "安装 cross: cargo install cross --git https://github.com/cross-rs/cross"
        exit 1
    fi
    log_success "cross 工具已安装: $(cross --version 2>&1 | head -1)"
}

# 检查 Docker
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker 未运行，请启动 Docker Desktop"
        exit 1
    fi
    log_success "Docker 运行正常"
}

# 安装 Rust target
install_target() {
    log_info "安装 Rust target: x86_64-unknown-linux-gnu..."
    rustup target add x86_64-unknown-linux-gnu
    log_success "Rust target 已安装"
}

# 使用 cross 构建
build_with_cross() {
    log_info "使用 cross 工具构建 Linux amd64 版本..."
    cd "$PROJECT_ROOT"
    
    local build_mode="${1:-release}"
    
    # 设置代理环境变量（传递给 Docker 容器）
    local cross_env=()
    if [ -n "$HTTP_PROXY" ]; then
        cross_env+=(--env HTTP_PROXY="$HTTP_PROXY")
    fi
    if [ -n "$HTTPS_PROXY" ]; then
        cross_env+=(--env HTTPS_PROXY="$HTTPS_PROXY")
    fi
    if [ -n "$ALL_PROXY" ]; then
        cross_env+=(--env ALL_PROXY="$ALL_PROXY")
    fi
    
    local build_cmd=(cross build --package agent-mem-server --features lumosai)
    
    if [ "$build_mode" = "release" ]; then
        build_cmd+=(--release)
        log_info "构建模式: Release (优化)"
    else
        log_info "构建模式: Debug (快速)"
    fi
    
    build_cmd+=(--target x86_64-unknown-linux-gnu)
    
    log_info "执行: ${build_cmd[*]}"
    "${build_cmd[@]}"
    
    if [ "$build_mode" = "release" ]; then
        BINARY_PATH="target/x86_64-unknown-linux-gnu/release/agent-mem-server"
    else
        BINARY_PATH="target/x86_64-unknown-linux-gnu/debug/agent-mem-server"
    fi
    
    if [ -f "$PROJECT_ROOT/$BINARY_PATH" ]; then
        log_success "构建完成: $BINARY_PATH"
        log_info "文件大小: $(ls -lh "$PROJECT_ROOT/$BINARY_PATH" | awk '{print $5}')"
    else
        log_error "构建失败，二进制文件不存在"
        exit 1
    fi
}

# 主函数
main() {
    # 解析代理参数
    local build_mode="release"
    local proxy_set=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --proxy)
                if [ -z "${2:-}" ]; then
                    log_error "选项 --proxy 需要一个代理地址参数"
                    exit 1
                fi
                setup_proxy "$2"
                proxy_set=true
                shift 2
                ;;
            --proxy-system)
                setup_proxy "--proxy-system"
                proxy_set=true
                shift
                ;;
            release|debug)
                build_mode="$1"
                shift
                ;;
            *)
                log_warning "未知参数: $1，忽略"
                shift
                ;;
        esac
    done
    
    # 如果没有通过参数设置代理，检查环境变量
    if [ "$proxy_set" = false ] && ([ -n "$HTTP_PROXY" ] || [ -n "$HTTPS_PROXY" ]); then
        log_info "检测到环境变量中的代理设置"
    fi
    
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║     AgentMem Linux amd64 cross 工具构建脚本                ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    check_cross
    check_docker
    install_target
    build_with_cross "$build_mode"
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                     构建完成                                  ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    log_success "Linux amd64 二进制文件: $PROJECT_ROOT/$BINARY_PATH"
    echo ""
    echo "📋 说明："
    echo "   cross 工具会自动使用 Docker 进行交叉编译"
    echo "   无需手动配置 OpenSSL 或其他系统依赖"
    echo ""
}

main "$@"
