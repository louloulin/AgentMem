#!/bin/bash

# AgentMem 服务器启动脚本 - 测试模式（禁用认证）

set -e

cd "$(dirname "$0")"

echo "🔧 配置 ONNX Runtime 库路径"

# 获取绝对路径
LIB_DIR="$(pwd)/lib"
TARGET_RELEASE_DIR="$(pwd)/target/release"

# 检查库文件
if [ ! -f "$LIB_DIR/libonnxruntime.1.22.0.dylib" ]; then
    echo "❌ 错误: 未找到 ONNX Runtime 库"
    exit 1
fi

echo "✅ 找到 ONNX Runtime 1.22.0 库"

# 停止旧进程
echo "🛑 停止旧的服务进程..."
pkill -f "agent-mem-server" 2>/dev/null || true
sleep 3

# 设置环境变量
export DYLD_LIBRARY_PATH="$LIB_DIR:$TARGET_RELEASE_DIR:$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.1.22.0.dylib"
export RUST_BACKTRACE=1

# 配置 Embedder
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 配置 LLM Provider
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"

# 🔓 禁用认证（测试模式）- 使用正确的环境变量
export AGENT_MEM_ENABLE_AUTH="false"

echo "🌍 环境变量已设置:"
echo "  AGENT_MEM_ENABLE_AUTH=$AGENT_MEM_ENABLE_AUTH ⚠️ (认证已禁用)"
echo "  EMBEDDER_PROVIDER=$EMBEDDER_PROVIDER"
echo "  LLM_PROVIDER=$LLM_PROVIDER"

# 启动服务器
echo ""
echo "🚀 启动 AgentMem 服务器 (测试模式 - 无认证)..."
echo "日志文件: $(pwd)/backend-test.log"

nohup ./target/release/agent-mem-server > backend-test.log 2>&1 &
SERVER_PID=$!

echo "✅ 服务器已启动 (PID: $SERVER_PID)"

# 等待启动
echo "⏳ 等待服务器启动 (15秒)..."
sleep 15

# 检查进程
if ps -p $SERVER_PID > /dev/null; then
    echo "✅ 服务器进程正在运行"
else
    echo "❌ 服务器进程已退出"
    tail -30 backend-test.log
    exit 1
fi

echo ""
echo "🌐 服务器信息:"
echo "  - 后端 API: http://localhost:8080"
echo "  - 健康检查: http://localhost:8080/health"
echo "  - API 文档: http://localhost:8080/swagger-ui/"
echo "  - 认证状态: ⚠️  已禁用 (仅用于测试)"

# 健康检查
echo ""
echo "🏥 执行健康检查..."
sleep 2
HEALTH=$(curl -s http://localhost:8080/health)
echo "$HEALTH" | jq '.' 2>/dev/null || echo "响应: $HEALTH"

echo ""
echo "✨ 服务器启动完成！"
echo ""
echo "📝 查看日志: tail -f backend-test.log"
echo "🛑 停止: pkill -f agent-mem-server"
