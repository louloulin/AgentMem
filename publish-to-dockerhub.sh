#!/bin/bash

###############################################################################
# AgentMem Docker Hub 发布脚本
# 
# 功能：
# 1. 自动登录 Docker Hub（如需要）
# 2. 构建并推送镜像到 godlinchong/agentmem
# 3. 支持版本标签管理
#
# 使用方法：
#   ./publish-to-dockerhub.sh [版本标签]
#
# 示例：
#   ./publish-to-dockerhub.sh           # 发布 latest
#   ./publish-to-dockerhub.sh v1.0.0    # 发布 v1.0.0 和 latest
###############################################################################

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Docker Hub 配置
DOCKER_USERNAME="godlinchong"
IMAGE_NAME="agentmem"
VERSION="${1:-latest}"

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

# 检查 Docker
check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker 未安装，请先安装 Docker"
        exit 1
    fi
    
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker 未运行，请启动 Docker Desktop"
        exit 1
    fi
    
    log_success "Docker 运行正常: $(docker --version)"
}

# 检查 Docker Hub 登录状态
check_docker_login() {
    log_info "检查 Docker Hub 登录状态..."
    
    # 检查是否已登录
    if docker info 2>/dev/null | grep -q "Username"; then
        local username=$(docker info 2>/dev/null | grep "Username" | awk '{print $2}')
        log_success "已登录 Docker Hub: $username"
        
        if [ "$username" != "$DOCKER_USERNAME" ]; then
            log_warning "当前登录用户 ($username) 与目标用户 ($DOCKER_USERNAME) 不一致"
            read -p "是否重新登录? (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                docker logout
                docker login
            fi
        fi
    else
        log_warning "未检测到 Docker 登录状态"
        log_info "正在登录 Docker Hub..."
        docker login
    fi
}

# 构建并推送镜像
build_and_push() {
    log_info "========================================="
    log_info "发布 AgentMem 到 Docker Hub"
    log_info "========================================="
    log_info "用户: $DOCKER_USERNAME"
    log_info "镜像: $IMAGE_NAME"
    log_info "版本: $VERSION"
    log_info "========================================="
    
    cd "$PROJECT_ROOT"
    
    # 确定标签
    local tags=()
    if [ "$VERSION" = "latest" ]; then
        tags=("$DOCKER_USERNAME/$IMAGE_NAME:latest")
    else
        # 同时推送版本标签和 latest
        tags=("$DOCKER_USERNAME/$IMAGE_NAME:$VERSION" "$DOCKER_USERNAME/$IMAGE_NAME:latest")
    fi
    
    log_info "将推送以下标签:"
    for tag in "${tags[@]}"; do
        log_info "  - $tag"
    done
    echo ""
    
    # 使用构建脚本推送
    if [ ${#tags[@]} -eq 1 ]; then
        # 单个标签，使用构建脚本
        ./build-docker-linux-amd64.sh \
          --tag "${tags[0]}" \
          --push
    else
        # 多个标签，直接使用 docker buildx
        log_info "构建并推送多个标签..."
        
        local build_cmd=(
            docker buildx build
            --platform linux/amd64
            -f Dockerfile
        )
        
        # 添加所有标签
        for tag in "${tags[@]}"; do
            build_cmd+=(-t "$tag")
        done
        
        build_cmd+=(--push .)
        
        log_info "执行: ${build_cmd[*]}"
        "${build_cmd[@]}"
    fi
    
    log_success "镜像推送成功！"
}

# 验证推送
verify_push() {
    log_info "验证推送结果..."
    
    local primary_tag="$DOCKER_USERNAME/$IMAGE_NAME:$VERSION"
    
    if docker manifest inspect "$primary_tag" > /dev/null 2>&1; then
        log_success "镜像已成功推送到 Docker Hub"
        log_info "镜像地址: https://hub.docker.com/r/$DOCKER_USERNAME/$IMAGE_NAME"
        log_info ""
        log_info "拉取镜像:"
        log_info "  docker pull $primary_tag"
    else
        log_warning "无法验证推送结果，请手动检查"
        log_info "访问: https://hub.docker.com/r/$DOCKER_USERNAME/$IMAGE_NAME"
    fi
}

# 主函数
main() {
    log_info "========================================="
    log_info "AgentMem Docker Hub 发布脚本"
    log_info "========================================="
    
    # 检查依赖
    check_docker
    
    # 检查登录
    check_docker_login
    
    # 构建并推送
    build_and_push
    
    # 验证推送
    verify_push
    
    # 显示总结
    log_info "========================================="
    log_success "发布完成！"
    log_info "========================================="
    log_info "Docker Hub 地址:"
    log_info "  https://hub.docker.com/r/$DOCKER_USERNAME/$IMAGE_NAME"
    log_info ""
    log_info "已推送的标签:"
    if [ "$VERSION" = "latest" ]; then
        log_info "  - $DOCKER_USERNAME/$IMAGE_NAME:latest"
    else
        log_info "  - $DOCKER_USERNAME/$IMAGE_NAME:$VERSION"
        log_info "  - $DOCKER_USERNAME/$IMAGE_NAME:latest"
    fi
    log_info "========================================="
}

# 执行主函数
main "$@"

