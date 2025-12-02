#!/bin/bash

###############################################################################
# Docker 构建修复脚本
# 
# 功能：
# 1. 清理 Docker 构建缓存
# 2. 使用修复后的 Dockerfile.multiarch 重新构建
# 3. 验证构建结果
#
# 使用方法：
#   ./fix-docker-build.sh [选项]
#
# 选项：
#   --platform PLATFORM    指定平台（默认: linux/amd64）
#   --tag TAG              指定镜像标签（默认: agentmem:latest）
#   --no-cache             不使用缓存构建（推荐首次构建）
#   --help                 显示帮助信息
###############################################################################

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
PLATFORM="linux/amd64"
IMAGE_TAG="agentmem:latest"
USE_CACHE=false
DOCKERFILE="Dockerfile.multiarch"

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
Docker 构建修复脚本

使用方法：
  ./fix-docker-build.sh [选项]

选项：
  --platform PLATFORM    指定目标平台（默认: linux/amd64）
  --tag TAG              指定镜像标签（默认: agentmem:latest）
  --no-cache             不使用缓存构建（推荐首次构建）
  --help                 显示帮助信息

示例：
  # 清理缓存并重新构建（推荐）
  ./fix-docker-build.sh --no-cache

  # 构建特定平台
  ./fix-docker-build.sh --platform linux/arm64 --no-cache

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
            --no-cache)
                USE_CACHE=false
                shift
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

# 清理 Docker 缓存
clean_cache() {
    log_info "清理 Docker 构建缓存..."
    
    if docker buildx prune -af > /dev/null 2>&1; then
        log_success "构建缓存已清理"
    else
        log_warning "清理缓存时出现警告（可能没有缓存）"
    fi
}

# 检查 Dockerfile
check_dockerfile() {
    if [ ! -f "$DOCKERFILE" ]; then
        log_error "Dockerfile 不存在: $DOCKERFILE"
        exit 1
    fi
    
    # 检查是否包含修复
    if grep -q "target-feature=-avx512f" "$DOCKERFILE"; then
        log_success "Dockerfile 包含 AVX-512 修复"
    else
        log_warning "Dockerfile 可能不包含 AVX-512 修复，请检查"
    fi
}

# 构建镜像
build_image() {
    log_info "========================================="
    log_info "开始构建 Docker 镜像"
    log_info "========================================="
    log_info "平台: $PLATFORM"
    log_info "镜像标签: $IMAGE_TAG"
    log_info "Dockerfile: $DOCKERFILE"
    log_info "使用缓存: $USE_CACHE"
    log_info "========================================="
    
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
        log_info "构建模式: 不使用缓存（确保使用最新修复）"
    else
        log_info "构建模式: 使用缓存"
    fi
    
    # 添加输出选项
    build_cmd+=(--load)
    
    # 添加构建上下文
    build_cmd+=(.)
    
    log_info "执行: ${build_cmd[*]}"
    log_info ""
    
    # 执行构建
    if "${build_cmd[@]}"; then
        log_success "Docker 镜像构建成功: $IMAGE_TAG"
        return 0
    else
        log_error "Docker 镜像构建失败"
        return 1
    fi
}

# 验证镜像
verify_image() {
    log_info "验证镜像..."
    
    if docker image inspect "$IMAGE_TAG" > /dev/null 2>&1; then
        log_success "镜像已加载到本地"
        
        # 显示镜像信息
        log_info "镜像信息:"
        docker image inspect "$IMAGE_TAG" --format '  - ID: {{.Id}}' | head -c 20
        echo "..."
        docker image inspect "$IMAGE_TAG" --format '  - 大小: {{.Size}} bytes'
        docker image inspect "$IMAGE_TAG" --format '  - 创建时间: {{.Created}}'
        
        return 0
    else
        log_warning "镜像未找到，可能构建失败"
        return 1
    fi
}

# 主函数
main() {
    log_info "========================================="
    log_info "Docker 构建修复脚本"
    log_info "========================================="
    
    # 解析参数
    parse_args "$@"
    
    # 检查 Dockerfile
    check_dockerfile
    
    # 清理缓存
    clean_cache
    
    # 构建镜像
    if build_image; then
        # 验证镜像
        verify_image
        
        # 显示总结
        log_info "========================================="
        log_success "构建完成！"
        log_info "========================================="
        log_info "镜像标签: $IMAGE_TAG"
        log_info "平台: $PLATFORM"
        log_info ""
        log_info "下一步："
        log_info "1. 运行容器: docker run -p 8080:8080 $IMAGE_TAG"
        log_info "2. 查看日志: docker logs <container_id>"
        log_info "3. 进入容器: docker exec -it <container_id> /bin/bash"
        log_info "========================================="
    else
        log_error "构建失败，请检查错误信息"
        log_info ""
        log_info "故障排查："
        log_info "1. 确认 Docker Desktop 内存至少 8GB（推荐 16GB）"
        log_info "2. 检查 Dockerfile.multiarch 是否包含最新修复"
        log_info "3. 查看详细错误信息：docker buildx build --progress=plain ..."
        exit 1
    fi
}

# 执行主函数
main "$@"

