#!/bin/bash
# 测试MCP核心功能：记忆存储和检索

echo "=== 启动MCP服务器 ==="
./target/release/agentmem-mcp-server &
SERVER_PID=$!
sleep 2

echo ""
echo "=== 测试记忆存储 ==="
curl -s -X POST http://localhost:3000/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "content": "用户喜欢喝咖啡",
    "organization_id": "test-org",
    "user_id": "user-123",
    "agent_id": "agent-456"
  }' | jq

echo ""
echo "=== 测试记忆检索 ==="
curl -s "http://localhost:3000/v1/memories?agent_id=agent-456&limit=10" | jq

echo ""
echo "=== 清理 ==="
kill $SERVER_PID 2>/dev/null

echo "MCP测试完成"
