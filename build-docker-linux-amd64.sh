#!/bin/bash

###############################################################################
# AgentMem Linux amd64 Docker 构建脚本
# 
# 功能：
# 1. 构建 Linux amd64 Docker 镜像
# 2. 使用 Docker buildx 支持多平台构建（可选）
# 3. 支持本地构建和推送镜像
#
# 使用方法：
#   ./build-docker-linux-amd64.sh [选项]
#
# 选项：
#   --platform linux/amd64    指定平台（默认）
#   --tag TAG                  指定镜像标签（默认: agentmem:latest）
#   --push                     构建后推送到仓库
#   --load                     构建后加载到本地（默认）
#   --no-cache                 不使用缓存构建
#   --help                     显示帮助信息
###############################################################################

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
PLATFORM="linux/amd64"
IMAGE_TAG="agentmem:latest"
USE_PUSH=false
USE_LOAD=true
USE_CACHE=true
DOCKERFILE="Dockerfile"

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

# 显示帮助信息
show_help() {
    cat << EOF
AgentMem Linux amd64 Docker 构建脚本

使用方法：
  ./build-docker-linux-amd64.sh [选项]

选项：
  --platform PLATFORM    指定目标平台（默认: linux/amd64）
  --tag TAG              指定镜像标签（默认: agentmem:latest）
  --push                 构建后推送到仓库
  --load                 构建后加载到本地（默认）
  --no-cache             不使用缓存构建
  --dockerfile FILE      指定 Dockerfile（默认: Dockerfile）
  --help                 显示帮助信息

示例：
  # 构建 Linux amd64 镜像（本地）
  ./build-docker-linux-amd64.sh

  # 构建并推送到仓库
  ./build-docker-linux-amd64.sh --tag myregistry/agentmem:v1.0.0 --push

  # 不使用缓存构建
  ./build-docker-linux-amd64.sh --no-cache

EOF
}

# 解析命令行参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --platform)
                PLATFORM="$2"
                shift 2
                ;;
            --tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            --push)
                USE_PUSH=true
                USE_LOAD=false
                shift
                ;;
            --load)
                USE_LOAD=true
                USE_PUSH=false
                shift
                ;;
            --no-cache)
                USE_CACHE=false
                shift
                ;;
            --dockerfile)
                DOCKERFILE="$2"
                shift 2
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
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

# 检查 Dockerfile
check_dockerfile() {
    if [ ! -f "$PROJECT_ROOT/$DOCKERFILE" ]; then
        log_error "Dockerfile 不存在: $DOCKERFILE"
        exit 1
    fi
    
    log_success "使用 Dockerfile: $DOCKERFILE"
}

# 构建 Docker 镜像
build_image() {
    log_info "========================================="
    log_info "开始构建 Linux amd64 Docker 镜像"
    log_info "========================================="
    log_info "平台: $PLATFORM"
    log_info "镜像标签: $IMAGE_TAG"
    log_info "Dockerfile: $DOCKERFILE"
    log_info "========================================="
    
    cd "$PROJECT_ROOT"
    
    # 构建命令
    local build_cmd=(
        docker buildx build
        --platform "$PLATFORM"
        -f "$DOCKERFILE"
        -t "$IMAGE_TAG"
    )
    
    # 添加缓存选项
    if [ "$USE_CACHE" = false ]; then
        build_cmd+=(--no-cache)
        log_info "构建模式: 不使用缓存"
    else
        log_info "构建模式: 使用缓存"
    fi
    
    # 添加输出选项
    if [ "$USE_PUSH" = true ]; then
        build_cmd+=(--push)
        log_info "输出: 推送到仓库"
    elif [ "$USE_LOAD" = true ]; then
        build_cmd+=(--load)
        log_info "输出: 加载到本地"
    fi
    
    # 添加构建上下文
    build_cmd+=(.)
    
    log_info "执行: ${build_cmd[*]}"
    log_info ""
    
    # 执行构建
    if "${build_cmd[@]}"; then
        log_success "Docker 镜像构建成功: $IMAGE_TAG"
    else
        log_error "Docker 镜像构建失败"
        exit 1
    fi
}

# 验证镜像
verify_image() {
    if [ "$USE_LOAD" = true ]; then
        log_info "验证镜像..."
        
        if docker image inspect "$IMAGE_TAG" > /dev/null 2>&1; then
            log_success "镜像已加载到本地"
            
            # 显示镜像信息
            log_info "镜像信息:"
            docker image inspect "$IMAGE_TAG" --format '  - ID: {{.Id}}'
            docker image inspect "$IMAGE_TAG" --format '  - 大小: {{.Size}} bytes'
            docker image inspect "$IMAGE_TAG" --format '  - 创建时间: {{.Created}}'
        else
            log_warning "镜像未找到，可能构建失败"
        fi
    fi
}

# 主函数
main() {
    log_info "========================================="
    log_info "AgentMem Linux amd64 Docker 构建脚本"
    log_info "========================================="
    
    # 解析参数
    parse_args "$@"
    
    # 检查依赖
    check_docker
    check_dockerfile
    
    # 构建镜像
    build_image
    
    # 验证镜像
    verify_image
    
    # 显示总结
    log_info "========================================="
    log_success "构建完成！"
    log_info "========================================="
    log_info "镜像标签: $IMAGE_TAG"
    log_info "平台: $PLATFORM"
    
    if [ "$USE_LOAD" = true ]; then
        log_info ""
        log_info "下一步："
        log_info "1. 运行容器: docker run -p 8080:8080 $IMAGE_TAG"
        log_info "2. 查看日志: docker logs <container_id>"
        log_info "3. 进入容器: docker exec -it <container_id> /bin/bash"
    fi
    
    log_info "========================================="
}

# 执行主函数
main "$@"

