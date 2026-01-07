#!/bin/bash

# AgentMem 全栈启动脚本 - 使用 justfile 命令
# 此脚本已更新为使用统一的 justfile 启动逻辑

set -e

cd "$(dirname "$0")"

echo "=========================================="
echo "🚀 AgentMem 全栈启动"
echo "=========================================="
echo ""

# 检查是否安装了 just
if ! command -v just &> /dev/null; then
    echo "❌ 错误: 未找到 just 命令"
    echo "请安装 just: cargo install just"
    exit 1
fi

# 使用 justfile 启动全栈服务
echo "使用 justfile 启动服务..."
just start-full

echo ""
echo "=========================================="
echo "✅ 全栈启动完成！"
echo "=========================================="
echo ""
echo "🌐 访问地址:"
echo "   前端: http://localhost:3001"
echo "   后端: http://localhost:8080"
echo ""
echo "📝 日志文件:"
echo "   后端: backend.log"
echo "   前端: frontend.log"
echo ""
echo "🛑 停止服务:"
echo "   just stop"
echo "   或"
echo "   后端: pkill -f agent-mem-server"
echo "   前端: pkill -f 'next dev'"
echo ""
