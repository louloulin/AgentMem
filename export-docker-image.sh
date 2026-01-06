#!/bin/bash

###############################################################################
# Docker 镜像导出脚本
# 
# 功能：
# 1. 从 Docker Hub 拉取镜像（如需要）
# 2. 将镜像导出为 tar 包
# 3. 支持压缩选项
#
# 使用方法：
#   ./export-docker-image.sh [选项]
#
# 选项：
#   --image IMAGE          镜像名称（默认: godlinchong/agentmem:latest）
#   --output FILE          输出文件路径（默认: agentmem-latest.tar）
#   --compress             压缩 tar 包（生成 .tar.gz）
#   --pull                 强制拉取最新镜像
#   --help                 显示帮助信息
###############################################################################

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
IMAGE_NAME="godlinchong/agentmem:latest"
OUTPUT_FILE="agentmem-latest.tar"
USE_COMPRESS=false
FORCE_PULL=false

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPORT_DIR="${EXPORT_DIR:-$PROJECT_ROOT/dist/docker}"

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
Docker 镜像导出脚本

使用方法：
  ./export-docker-image.sh [选项]

选项：
  --image IMAGE          镜像名称（默认: godlinchong/agentmem:latest）
  --output FILE          输出文件路径（默认: agentmem-latest.tar）
  --compress             压缩 tar 包（生成 .tar.gz）
  --pull                 强制拉取最新镜像
  --help                 显示帮助信息

示例：
  # 导出默认镜像
  ./export-docker-image.sh

  # 导出指定镜像
  ./export-docker-image.sh --image godlinchong/agentmem:v1.0.0

  # 导出并压缩
  ./export-docker-image.sh --compress

  # 导出到指定路径
  ./export-docker-image.sh --output /path/to/agentmem.tar

EOF
}

# 解析命令行参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --image)
                IMAGE_NAME="$2"
                shift 2
                ;;
            --output)
                OUTPUT_FILE="$2"
                shift 2
                ;;
            --compress)
                USE_COMPRESS=true
                shift
                ;;
            --pull)
                FORCE_PULL=true
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

# 检查镜像是否存在
check_image() {
    log_info "检查镜像: $IMAGE_NAME"
    
    if docker image inspect "$IMAGE_NAME" > /dev/null 2>&1; then
        log_success "镜像已存在于本地"
        
        if [ "$FORCE_PULL" = true ]; then
            log_info "强制拉取最新镜像..."
            docker pull "$IMAGE_NAME"
        else
            log_info "使用本地镜像（使用 --pull 强制拉取最新版本）"
        fi
    else
        log_warning "镜像不存在于本地，正在拉取..."
        docker pull "$IMAGE_NAME"
    fi
    
    # 显示镜像信息
    log_info "镜像信息:"
    docker image inspect "$IMAGE_NAME" --format '  - ID: {{.Id}}'
    docker image inspect "$IMAGE_NAME" --format '  - 大小: {{.Size}} bytes'
    docker image inspect "$IMAGE_NAME" --format '  - 创建时间: {{.Created}}'
}

# 导出镜像
export_image() {
    log_info "========================================="
    log_info "导出 Docker 镜像"
    log_info "========================================="
    log_info "镜像: $IMAGE_NAME"
    log_info "输出: $OUTPUT_FILE"
    log_info "压缩: $USE_COMPRESS"
    log_info "========================================="
    
    # 创建输出目录
    mkdir -p "$(dirname "$OUTPUT_FILE")"
    
    # 确定最终输出文件名
    local final_output="$OUTPUT_FILE"
    if [ "$USE_COMPRESS" = true ]; then
        # 如果输出文件没有 .gz 后缀，添加它
        if [[ ! "$final_output" =~ \.tar\.gz$ ]]; then
            final_output="${OUTPUT_FILE%.tar}.tar.gz"
        fi
    fi
    
    log_info "开始导出镜像..."
    log_info "输出文件: $final_output"
    
    if [ "$USE_COMPRESS" = true ]; then
        # 导出并压缩
        log_info "导出并压缩镜像（这可能需要一些时间）..."
        docker save "$IMAGE_NAME" | gzip > "$final_output"
    else
        # 仅导出
        log_info "导出镜像..."
        docker save "$IMAGE_NAME" -o "$final_output"
    fi
    
    # 检查文件是否创建成功
    if [ -f "$final_output" ]; then
        local file_size=$(du -h "$final_output" | cut -f1)
        log_success "镜像导出成功！"
        log_info "文件: $final_output"
        log_info "大小: $file_size"
        
        # 显示文件信息
        log_info ""
        log_info "文件信息:"
        ls -lh "$final_output"
    else
        log_error "镜像导出失败"
        exit 1
    fi
}

# 验证导出文件
verify_export() {
    local final_output="$OUTPUT_FILE"
    if [ "$USE_COMPRESS" = true ]; then
        if [[ ! "$final_output" =~ \.tar\.gz$ ]]; then
            final_output="${OUTPUT_FILE%.tar}.tar.gz"
        fi
    fi
    
    log_info "验证导出文件..."
    
    if [ "$USE_COMPRESS" = true ]; then
        # 验证压缩文件
        if gzip -t "$final_output" 2>/dev/null; then
            log_success "压缩文件验证通过"
        else
            log_warning "压缩文件验证失败，但文件已创建"
        fi
    else
        # 验证 tar 文件
        if tar -tf "$final_output" > /dev/null 2>&1; then
            log_success "tar 文件验证通过"
        else
            log_warning "tar 文件验证失败，但文件已创建"
        fi
    fi
}

# 主函数
main() {
    log_info "========================================="
    log_info "Docker 镜像导出脚本"
    log_info "========================================="
    
    # 解析参数
    parse_args "$@"
    
    # 检查依赖
    check_docker
    
    # 检查镜像
    check_image
    
    # 导出镜像
    export_image
    
    # 验证导出
    verify_export
    
    # 显示总结
    log_info "========================================="
    log_success "导出完成！"
    log_info "========================================="
    log_info "镜像: $IMAGE_NAME"
    
    local final_output="$OUTPUT_FILE"
    if [ "$USE_COMPRESS" = true ]; then
        if [[ ! "$final_output" =~ \.tar\.gz$ ]]; then
            final_output="${OUTPUT_FILE%.tar}.tar.gz"
        fi
    fi
    
    log_info "输出文件: $final_output"
    log_info "文件大小: $(du -h "$final_output" | cut -f1)"
    log_info ""
    log_info "导入镜像:"
    if [ "$USE_COMPRESS" = true ]; then
        log_info "  gunzip -c $final_output | docker load"
    else
        log_info "  docker load -i $final_output"
    fi
    log_info "========================================="
}

# 执行主函数
main "$@"

