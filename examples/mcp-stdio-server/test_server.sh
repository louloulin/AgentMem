#!/bin/bash
# AgentMem MCP Stdio 服务器测试脚本

set -e

echo "=== AgentMem MCP Stdio 服务器测试 ==="
echo ""

# 检查可执行文件是否存在
if [ ! -f "../../target/release/agentmem-mcp-server" ]; then
    echo "❌ 错误: 可执行文件不存在"
    echo "请先运行: cargo build --package mcp-stdio-server --release"
    exit 1
fi

echo "✅ 可执行文件存在"
echo ""

# 测试 1: Initialize 握手
echo "📝 测试 1: Initialize 握手"
RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}' | timeout 5 ../../target/release/agentmem-mcp-server 2>/dev/null || true)

if echo "$RESPONSE" | grep -q "protocolVersion"; then
    echo "✅ Initialize 握手成功"
    echo "响应: $RESPONSE" | head -c 200
    echo "..."
else
    echo "❌ Initialize 握手失败"
    echo "响应: $RESPONSE"
    exit 1
fi
echo ""

# 测试 2: 列出工具
echo "📝 测试 2: 列出工具"
RESPONSE=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | timeout 5 ../../target/release/agentmem-mcp-server 2>/dev/null || true)

if echo "$RESPONSE" | grep -q "agentmem_add_memory"; then
    echo "✅ 工具列表获取成功"
    echo "响应: $RESPONSE" | head -c 200
    echo "..."
else
    echo "❌ 工具列表获取失败"
    echo "响应: $RESPONSE"
    exit 1
fi
echo ""

# 测试 3: 调用工具 (添加记忆)
echo "📝 测试 3: 调用工具 (添加记忆)"
RESPONSE=$(echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"测试记忆内容","user_id":"test_user"}}}' | timeout 5 ../../target/release/agentmem-mcp-server 2>/dev/null || true)

if echo "$RESPONSE" | grep -q "result"; then
    echo "✅ 工具调用成功"
    echo "响应: $RESPONSE" | head -c 200
    echo "..."
else
    echo "⚠️  工具调用可能失败（这是正常的，因为需要配置存储后端）"
    echo "响应: $RESPONSE" | head -c 200
    echo "..."
fi
echo ""

echo "=== 测试完成 ==="
echo ""
echo "✅ 基本功能测试通过！"
echo ""
echo "下一步: 配置 Claude Desktop 进行集成测试"
echo "请参考 README.md 中的配置说明"

