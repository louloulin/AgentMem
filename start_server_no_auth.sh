#!/bin/bash

# AgentMem 服务器启动脚本 - 禁用认证版本
# 支持自动构建 server 和 MCP

set -e

cd "$(dirname "$0")"

# 解析命令行参数
BUILD_SERVER=false
BUILD_MCP=false
SKIP_BUILD=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-server)
            BUILD_SERVER=true
            shift
            ;;
        --build-mcp)
            BUILD_MCP=true
            shift
            ;;
        --build-all)
            BUILD_SERVER=true
            BUILD_MCP=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -h|--help)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --build-server    构建 agent-mem-server"
            echo "  --build-mcp       构建 MCP 示例"
            echo "  --build-all       构建所有组件"
            echo "  --skip-build      跳过构建检查，直接启动"
            echo "  -h, --help        显示此帮助信息"
            echo ""
            echo "示例:"
            echo "  $0                      # 检查并启动（如需要则自动构建）"
            echo "  $0 --build-all          # 强制重新构建所有组件"
            echo "  $0 --build-server       # 仅重新构建 server"
            echo "  $0 --skip-build         # 跳过构建检查"
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            echo "使用 -h 或 --help 查看帮助"
            exit 1
            ;;
    esac
done

echo "🔧 配置 ONNX Runtime 库路径"

# 获取绝对路径
LIB_DIR="$(pwd)/lib"
TARGET_RELEASE_DIR="$(pwd)/target/release"

echo "库目录: $LIB_DIR"
echo "二进制目录: $TARGET_RELEASE_DIR"

# 检查库文件是否存在
if [ ! -f "$LIB_DIR/libonnxruntime.1.22.0.dylib" ]; then
    echo "❌ 错误: 未找到 $LIB_DIR/libonnxruntime.1.22.0.dylib"
    echo "请确保库文件存在"
    exit 1
fi

echo "✅ 找到 ONNX Runtime 1.22.0 库"

# 构建检查和编译
if [ "$SKIP_BUILD" = false ]; then
    echo ""
    echo "📦 检查构建状态..."

    # 检查 server 是否需要构建
    if [ "$BUILD_SERVER" = true ] || [ ! -f "./target/release/agent-mem-server" ]; then
        echo ""
        echo "🔨 构建 agent-mem-server..."
        cargo build --release --bin agent-mem-server --exclude agent-mem-python
        echo "✅ agent-mem-server 构建完成"
    else
        echo "✅ agent-mem-server 已存在"
    fi

    # 检查 MCP 示例是否需要构建
    if [ "$BUILD_MCP" = true ]; then
        echo ""
        echo "🔨 构建 MCP 示例..."
        if cargo build --release --example mcp-stdio-server 2>&1 | grep -q "Finished"; then
            echo "✅ MCP 示例构建完成"
        else
            echo "⚠️  MCP 示例构建失败或不存在"
        fi
    fi
else
    echo "⏭️  跳过构建检查"
fi

# 停止旧进程
echo ""
echo "🛑 停止旧的服务进程..."
pkill -f "agent-mem-server" 2>/dev/null || true
sleep 2

# 设置环境变量
export DYLD_LIBRARY_PATH="$LIB_DIR:$TARGET_RELEASE_DIR:$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.1.22.0.dylib"
export RUST_BACKTRACE=1

# 配置 Embedder (使用 FastEmbed) - 关键修复
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
echo "✅ Embedder配置: $EMBEDDER_PROVIDER / $EMBEDDER_MODEL"

# 配置 LLM Provider (Zhipu AI)
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"

# 🔓 禁用认证（用于测试）
export ENABLE_AUTH="false"
export SERVER_ENABLE_AUTH="false"

echo "🌍 环境变量已设置:"
echo "  DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH"
echo "  ORT_DYLIB_PATH=$ORT_DYLIB_PATH"
echo "  ZHIPU_API_KEY=99a311...*** (已设置)"
echo "  LLM_PROVIDER=$LLM_PROVIDER"
echo "  EMBEDDER_PROVIDER=$EMBEDDER_PROVIDER"
echo "  EMBEDDER_MODEL=$EMBEDDER_MODEL"
echo "  ENABLE_AUTH=$ENABLE_AUTH (禁用认证)"
export http_proxy=http://127.0.0.1:4780 && export https_proxy=http://127.0.0.1:4780
# 启动服务器
echo ""
echo "🚀 启动 AgentMem 服务器 (无认证模式)..."
echo "日志文件: $(pwd)/backend-no-auth.log"
echo ""

nohup ./target/release/agent-mem-server > backend-no-auth.log 2>&1 &
SERVER_PID=$!

echo "✅ 服务器已启动 (PID: $SERVER_PID)"
echo ""

# 等待启动
echo "⏳ 等待服务器启动 (10秒)..."
sleep 10

# 检查进程是否还在运行
if ps -p $SERVER_PID > /dev/null; then
    echo "✅ 服务器进程正在运行"
else
    echo "❌ 服务器进程已退出，请检查日志"
    echo ""
    echo "最后 20 行日志:"
    tail -20 backend-no-auth.log
    exit 1
fi

echo ""
echo "🌐 服务器信息:"
echo "  - 后端 API: http://localhost:8080"
echo "  - 健康检查: http://localhost:8080/health"
echo "  - API 文档: http://localhost:8080/swagger-ui/"
echo "  - 认证状态: 已禁用 (测试模式)"
echo ""

# 健康检查
echo "🏥 执行健康检查..."
sleep 2
HEALTH_RESPONSE=$(curl -s http://localhost:8080/health)
echo "$HEALTH_RESPONSE" | jq -r '.status' 2>/dev/null && echo "✅ 健康检查通过！" || echo "响应: $HEALTH_RESPONSE"

echo ""
echo "✨ 服务器启动完成！"
echo ""
echo "📝 查看实时日志: tail -f backend-no-auth.log"
echo "🛑 停止服务器: pkill -f agent-mem-server"
