#!/bin/bash

# 手动测试 MCP server

echo "=== 测试 MCP Server 手动调用 ==="

# 1. Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | RUST_LOG=debug ./target/release/agentmem-mcp-server 2>&1 | head -50

