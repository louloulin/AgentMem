#!/bin/bash
# AgentMem 完整验证流程脚本
# 1. 构建MCP服务器
# 2. 启动前后端服务
# 3. 验证MCP功能
# 4. 打开UI验证
# 5. 更新文档

set -e

cd "$(dirname "$0")/.."

echo "=========================================="
echo "AgentMem 完整验证流程"
echo "=========================================="
echo ""

# 步骤1: 构建MCP服务器
echo "步骤1: 构建MCP服务器..."
if [ ! -f "target/release/agentmem-mcp-client" ]; then
    echo "正在构建MCP服务器..."
    cargo build --package mcp-stdio-server --release --bin agentmem-mcp-client
    echo "✅ MCP服务器构建完成"
else
    echo "✅ MCP服务器已存在"
fi
echo ""

# 步骤2: 启动前后端服务
echo "步骤2: 启动前后端服务..."
echo "使用 just start-full 启动服务..."
just start-full || {
    echo "⚠️  服务启动可能失败，继续验证..."
}
echo "等待服务就绪..."
sleep 15
echo ""

# 步骤3: 验证服务
echo "步骤3: 验证服务..."
bash scripts/verify_complete_integration.sh
echo ""

# 步骤4: 打开UI
echo "步骤4: 打开UI验证..."
if command -v open &> /dev/null; then
    open http://localhost:3001
    echo "✅ 已打开浏览器"
else
    echo "⚠️  请手动访问: http://localhost:3001"
fi
echo ""

# 步骤5: 提示更新文档
echo "=========================================="
echo "验证完成"
echo "=========================================="
echo ""
echo "下一步: 更新 agentx7.md 文档"
echo "  运行: 更新文档标记完成的功能"
echo ""




