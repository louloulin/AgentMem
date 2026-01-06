#!/bin/bash
# 使用 Docker 构建 Linux 版本，支持代理配置

set -e

PROXY_PORT="${PROXY_PORT:-4780}"
PROXY_HOST="${PROXY_HOST:-127.0.0.1}"

# 检测操作系统
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS 使用 host.docker.internal
    DOCKER_PROXY="http://host.docker.internal:${PROXY_PORT}"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux 使用网关地址
    DOCKER_PROXY="http://172.17.0.1:${PROXY_PORT}"
else
    DOCKER_PROXY="http://${PROXY_HOST}:${PROXY_PORT}"
fi

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     Docker 构建 Linux 版本（代理: $DOCKER_PROXY）          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 检查 Docker
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker 未运行"
    exit 1
fi

# 构建镜像
echo "🔨 开始构建 Docker 镜像..."
export HTTP_PROXY="$DOCKER_PROXY"
export HTTPS_PROXY="$DOCKER_PROXY"
export DOCKER_BUILDKIT=1

docker build \
    --build-arg HTTP_PROXY="$DOCKER_PROXY" \
    --build-arg HTTPS_PROXY="$DOCKER_PROXY" \
    -f Dockerfile.linux-build \
    -t agentmem-linux-build:latest \
    --target builder \
    . 2>&1 | tee /tmp/docker-build-proxy.log

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo ""
    echo "✅ Docker 镜像构建成功"
    echo ""
    echo "📦 提取二进制文件..."
    
    # 创建临时容器
    CONTAINER_ID=$(docker create agentmem-linux-build:latest)
    
    # 创建输出目录
    mkdir -p dist/server
    
    # 从容器中复制二进制文件
    docker cp "$CONTAINER_ID:/workspace/target/x86_64-unknown-linux-gnu/release/agent-mem-server" dist/server/agent-mem-server
    
    # 删除临时容器
    docker rm "$CONTAINER_ID"
    
    # 设置执行权限
    chmod +x dist/server/agent-mem-server
    
    echo "✅ 二进制文件已提取到: dist/server/agent-mem-server"
    ls -lh dist/server/agent-mem-server
else
    echo ""
    echo "❌ Docker 镜像构建失败"
    echo "   查看日志: /tmp/docker-build-proxy.log"
    exit 1
fi
