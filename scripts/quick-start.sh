#!/bin/bash
# AgentMem 快速启动脚本（开发环境）

set -e

VERSION="0.2.0"
IMAGE_NAME="agentmem/agentmem"
CONTAINER_NAME="agentmem"
DATA_DIR="./data"

echo "🚀 AgentMem 快速启动"
echo "===================="
echo ""

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${GREEN}✅ $1${NC}"; }
log_warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# 检查 Docker
if ! command -v docker &> /dev/null; then
    log_error "需要先安装 Docker"
    echo "   访问: https://docs.docker.com/get-docker/"
    exit 1
fi

log_info "Docker 已安装"

# 检查是否已有容器在运行
if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
    log_warn "AgentMem 容器已在运行"
    echo ""
    echo "📍 API 地址: http://localhost:8080"
    echo "📖 API 文档: http://localhost:8080/swagger-ui/"
    echo ""
    echo "停止容器: docker stop $CONTAINER_NAME"
    exit 0
fi

# 创建数据目录
mkdir -p "$DATA_DIR"
log_info "数据目录: $DATA_DIR"

# 拉取镜像
echo ""
echo "📥 拉取 AgentMem 镜像..."
if docker pull "$IMAGE_NAME:v$VERSION"; then
    log_info "镜像拉取完成"
else
    log_error "镜像拉取失败"
    exit 1
fi

# 启动容器
echo ""
echo "🚀 启动 AgentMem 容器..."

docker run -d \
    --name "$CONTAINER_NAME" \
    -p 8080:8080 \
    -v "$(pwd)/$DATA_DIR:/data" \
    -e AGENTMEM_DB_PATH=/data/agentmem.db \
    -e AGENTMEM_VECTOR_PATH=/data/vectors.lance \
    -e AGENTMEM_ENABLE_AUTH=false \
    -e RUST_LOG=info \
    --restart unless-stopped \
    "$IMAGE_NAME:v$VERSION" > /dev/null

# 等待启动
echo "⏳ 等待服务启动..."
sleep 5

# 健康检查
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    log_info "AgentMem 启动成功！"
else
    log_error "启动失败，查看日志:"
    echo "   docker logs $CONTAINER_NAME"
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
    exit 1
fi

# 显示完成信息
echo ""
echo "🎉 启动完成！"
echo ""
echo "📍 服务信息:"
echo "   API 地址:   http://localhost:8080"
echo "   健康检查:   http://localhost:8080/health"
echo "   API 文档:   http://localhost:8080/swagger-ui/"
echo "   数据目录:   $DATA_DIR"
echo ""
echo "🔧 常用命令:"
echo "   查看日志:   docker logs -f $CONTAINER_NAME"
echo "   停止服务:   docker stop $CONTAINER_NAME"
echo "   重启服务:   docker restart $CONTAINER_NAME"
echo "   删除容器:   docker rm -f $CONTAINER_NAME"
echo ""
echo "✨ 快速测试:"
echo "   curl http://localhost:8080/health | jq"
echo ""
echo "📖 下一步:"
echo "   1. 查看 API 文档: http://localhost:8080/swagger-ui/"
echo "   2. 运行示例: cd examples_new && python quick_start.py"
echo "   3. 阅读文档: https://docs.agentmem.ai"
