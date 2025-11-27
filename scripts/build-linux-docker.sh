#!/bin/bash
# AgentMem Linux amd64 Docker 构建脚本
# 使用 Docker 在 Linux 环境中编译，避免交叉编译的复杂性

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist/server"

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

# 检查 Docker 是否运行
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker 未运行，请启动 Docker Desktop"
        exit 1
    fi
    log_success "Docker 运行正常"
}

# 构建 Docker 镜像
build_docker_image() {
    log_info "构建 Docker 镜像..."
    cd "$PROJECT_ROOT"
    
    docker build \
        -f Dockerfile.linux-build \
        -t agentmem-linux-build:latest \
        --target builder \
        . 2>&1 | tee /tmp/docker-build.log
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "Docker 镜像构建成功"
    else
        log_error "Docker 镜像构建失败，查看日志: /tmp/docker-build.log"
        exit 1
    fi
}

# 提取二进制文件
extract_binary() {
    log_info "提取二进制文件..."
    
    # 创建临时容器
    CONTAINER_ID=$(docker create agentmem-linux-build:latest)
    
    # 创建输出目录
    mkdir -p "$DIST_DIR"
    
    # 从容器中复制二进制文件
    docker cp "$CONTAINER_ID:/workspace/target/x86_64-unknown-linux-gnu/release/agent-mem-server" "$DIST_DIR/agent-mem-server"
    
    # 删除临时容器
    docker rm "$CONTAINER_ID"
    
    # 设置执行权限
    chmod +x "$DIST_DIR/agent-mem-server"
    
    # 验证二进制文件
    if [ -f "$DIST_DIR/agent-mem-server" ]; then
        log_success "二进制文件已提取到: $DIST_DIR/agent-mem-server"
        log_info "文件大小: $(ls -lh "$DIST_DIR/agent-mem-server" | awk '{print $5}')"
        log_info "文件类型: $(file "$DIST_DIR/agent-mem-server" | cut -d: -f2)"
    else
        log_error "二进制文件提取失败"
        exit 1
    fi
}

# 复制 ONNX Runtime 库文件
copy_onnx_libs() {
    log_info "复制 ONNX Runtime 库文件..."
    
    mkdir -p "$DIST_DIR/lib"
    
    # 查找 Linux 版本的 ONNX Runtime 库
    if [ -d "$PROJECT_ROOT/lib/linux-amd64" ]; then
        cp -r "$PROJECT_ROOT/lib/linux-amd64"/* "$DIST_DIR/lib/" 2>/dev/null || true
        log_success "已复制 Linux amd64 库文件"
    elif [ -d "$PROJECT_ROOT/lib/linux" ]; then
        cp -r "$PROJECT_ROOT/lib/linux"/* "$DIST_DIR/lib/" 2>/dev/null || true
        log_success "已复制 Linux 库文件"
    else
        log_warning "未找到 Linux ONNX Runtime 库文件，请手动复制到 $DIST_DIR/lib/"
    fi
}

# 主函数
main() {
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║     AgentMem Linux amd64 Docker 构建脚本                    ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    check_docker
    build_docker_image
    extract_binary
    copy_onnx_libs
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                     构建完成                                  ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    log_success "Linux amd64 二进制文件已生成: $DIST_DIR/agent-mem-server"
    echo ""
    echo "📋 下一步："
    echo "   1. 复制 ONNX Runtime 库文件到 $DIST_DIR/lib/ (如未自动复制)"
    echo "   2. 在 Linux 服务器上运行: $DIST_DIR/agent-mem-server"
    echo ""
}

main "$@"
