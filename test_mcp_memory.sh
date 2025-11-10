#!/bin/bash
# MCP记忆功能测试脚本

set -e

export https_proxy=http://127.0.0.1:4780
export http_proxy=http://127.0.0.1:4780
export all_proxy=socks5://127.0.0.1:4780

echo "======================================"
echo "AgentMem MCP 记忆功能测试"
echo "======================================"
echo ""

# 构建MCP服务器
echo "1. 构建MCP服务器..."
cargo build --bin mcp-stdio-server --release

# 测试基本记忆存储
echo ""
echo "2. 测试记忆存储..."
cat <<EOF | cargo run --bin mcp-stdio-server --release 2>/dev/null
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1.0","clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"add_memory","arguments":{"agent_id":"test-agent","content":"这是一个测试记忆","memory_type":"semantic"}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"search_memories","arguments":{"agent_id":"test-agent","query":"测试"}}}
EOF

echo ""
echo "======================================"
echo "测试完成"
echo "======================================"

