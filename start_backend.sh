#!/usr/bin/env bash

# AgentMem Backend Startup Script with Embedder Fix
# 功能：启动 AgentMem 后端服务器（修复 Embedder 配置）
# 参考：start_server_with_correct_onnx.sh
# 日期：2025-11-05

set -e

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 启动 AgentMem 后端服务器"
echo "=========================================="
echo ""

# 检查二进制文件
if [ ! -f "./target/release/agent-mem-server" ]; then
    echo "❌ 错误: 未找到编译后的服务器"
    echo "   位置: ./target/release/agent-mem-server"
    echo "   请先运行: just build-release"
    exit 1
fi

# 设置路径
LIB_DIR="$SCRIPT_DIR/lib"
TARGET_RELEASE_DIR="$SCRIPT_DIR/target/release"
LOG_FILE="$SCRIPT_DIR/backend-no-auth.log"

# 配置 ONNX Runtime (关键)
export DYLD_LIBRARY_PATH="$LIB_DIR:$TARGET_RELEASE_DIR:$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.1.22.0.dylib"
export RUST_BACKTRACE=1

# 配置 Embedder (关键修复)
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 配置 LLM Provider
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"

# 服务器配置
export ENABLE_AUTH="false"
export SERVER_ENABLE_AUTH="false"
export AGENT_MEM_ENABLE_AUTH="false"

echo "📋 配置信息:"
echo "  ├─ ONNX Runtime: $ORT_DYLIB_PATH"
echo "  ├─ Library Path: $DYLD_LIBRARY_PATH"
echo "  ├─ Embedder: $EMBEDDER_PROVIDER ($EMBEDDER_MODEL)"
echo "  ├─ LLM: $LLM_PROVIDER ($LLM_MODEL)"
echo "  ├─ Auth: Disabled"
echo "  └─ Log: $LOG_FILE"
echo ""

# 停止现有服务
echo "🛑 停止现有服务..."
pkill -f agent-mem-server 2>/dev/null || true
sleep 2

# 启动服务器
echo "🚀 启动后端服务器..."
nohup ./target/release/agent-mem-server > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

# 保存 PID
echo $SERVER_PID > server.pid
echo "   进程 ID: $SERVER_PID"
echo ""

# 等待服务器就绪
echo "⏳ 等待服务器启动..."
MAX_RETRIES=30
for i in $(seq 1 $MAX_RETRIES); do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ 服务器启动成功！"
        echo ""
        
        # 显示健康检查信息
        echo "📊 健康检查:"
        curl -s http://localhost:8080/health | jq '.' 2>/dev/null || curl -s http://localhost:8080/health
        echo ""
        
        echo "=========================================="
        echo "🌐 服务信息"
        echo "=========================================="
        echo "  🔹 后端 API: http://localhost:8080"
        echo "  🔹 健康检查: http://localhost:8080/health"
        echo "  🔹 API 文档: http://localhost:8080/swagger-ui/"
        echo "  🔹 Metrics: http://localhost:8080/metrics"
        echo "  🔹 Embedder: FastEmbed (BAAI/bge-small-en-v1.5)"
        echo "  🔹 日志文件: $LOG_FILE"
        echo "  🔹 进程 ID: $SERVER_PID"
        echo ""
        echo "💡 使用提示:"
        echo "  • 查看日志: tail -f $LOG_FILE"
        echo "  • 停止服务: just stop 或 kill $SERVER_PID"
        echo "  • 健康检查: curl http://localhost:8080/health | jq"
        echo ""
        
        exit 0
    fi
    
    # 检查进程是否还在运行
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo "❌ 服务器进程已退出"
        echo ""
        echo "📝 最后 20 行日志:"
        tail -20 "$LOG_FILE"
        exit 1
    fi
    
    printf "   等待中... (%d/%d)\r" $i $MAX_RETRIES
    sleep 1
done

echo ""
echo "❌ 服务器启动超时 (${MAX_RETRIES}秒)"
echo ""
echo "📝 日志输出:"
tail -50 "$LOG_FILE"
echo ""
echo "🔍 诊断建议:"
echo "  1. 检查端口 8080 是否被占用: lsof -i :8080"
echo "  2. 检查完整日志: cat $LOG_FILE"
echo "  3. 检查 ONNX Runtime: ls -la $ORT_DYLIB_PATH"
echo "  4. 尝试前台运行: just start-server"
echo ""

exit 1

