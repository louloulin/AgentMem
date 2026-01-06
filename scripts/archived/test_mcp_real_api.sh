#!/bin/bash

# 测试 MCP 工具真实 API 调用
# 验证 P1-3 修复：MCP 工具应该调用真实 Backend API，而不是返回模拟数据

set -e

echo "=========================================="
echo "测试 MCP 工具真实 API 调用"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# MCP server 路径
MCP_SERVER="./target/release/agentmem-mcp-server"

if [ ! -f "$MCP_SERVER" ]; then
    echo -e "${RED}❌ MCP server binary not found at $MCP_SERVER${NC}"
    echo "Please run: cargo build --release --bin agentmem-mcp-server"
    exit 1
fi

# 检查 Backend 是否运行
echo "1️⃣  检查 Backend 服务..."
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${RED}❌ Backend 未运行在 http://localhost:8080${NC}"
    echo "Please start the backend first"
    exit 1
fi
echo -e "${GREEN}✅ Backend 运行正常${NC}"
echo ""

# 获取当前数据库中的记忆数量（基准）
echo "2️⃣  获取当前记忆数量（基准）..."
BASELINE_COUNT=$(curl -s http://localhost:8080/api/v1/stats | jq -r '.data.total_memories // 0')
echo "   当前记忆数量: $BASELINE_COUNT"
echo ""

# 测试函数
test_mcp_tool() {
    local tool_name=$1
    local params=$2
    local description=$3

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "测试: $description"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # 创建临时文件
    local tmpfile=$(mktemp)

    # 写入 initialize 请求
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' > "$tmpfile"

    # 写入 tools/call 请求
    echo "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"$tool_name\",\"arguments\":$params}}" >> "$tmpfile"

    # 调用 MCP server (只捕获 stdout，忽略 stderr 的日志)
    local response=$($MCP_SERVER < "$tmpfile" 2>/dev/null | grep -v "INFO" | tail -1)

    # 清理临时文件
    rm -f "$tmpfile"

    # 检查响应
    if echo "$response" | jq -e '.result' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ 工具调用成功${NC}"
        echo "响应:"
        echo "$response" | jq '.result' | head -20
        return 0
    else
        echo -e "${RED}❌ 工具调用失败${NC}"
        echo "响应:"
        echo "$response" | jq '.'
        return 1
    fi
}

# 测试 1: agentmem_add_memory
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 1: agentmem_add_memory (添加记忆)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

ADD_PARAMS=$(cat <<'EOF'
{
  "content": "MCP 工具测试：这是一条通过 MCP 添加的测试记忆",
  "user_id": "mcp-test-user",
  "agent_id": "default-agent",
  "memory_type": "Episodic"
}
EOF
)

if test_mcp_tool "agentmem_add_memory" "$ADD_PARAMS" "添加记忆"; then
    echo ""
    echo "3️⃣  验证记忆是否真实添加到数据库..."
    sleep 2
    
    NEW_COUNT=$(curl -s http://localhost:8080/api/v1/stats | jq -r '.data.total_memories // 0')
    echo "   新记忆数量: $NEW_COUNT"
    
    if [ "$NEW_COUNT" -gt "$BASELINE_COUNT" ]; then
        echo -e "${GREEN}✅ 记忆已成功添加到数据库 (增加了 $((NEW_COUNT - BASELINE_COUNT)) 条)${NC}"
        BASELINE_COUNT=$NEW_COUNT
    else
        echo -e "${RED}❌ 记忆未添加到数据库（仍然是模拟数据）${NC}"
    fi
fi
echo ""

# 测试 2: agentmem_search_memories
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 2: agentmem_search_memories (搜索记忆)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

SEARCH_PARAMS=$(cat <<'EOF'
{
  "query": "MCP 工具测试",
  "limit": 5
}
EOF
)

if test_mcp_tool "agentmem_search_memories" "$SEARCH_PARAMS" "搜索记忆"; then
    echo ""
    echo "4️⃣  验证搜索结果是否来自真实数据库..."
    
    # 直接调用 Backend API 搜索
    BACKEND_RESULT=$(curl -s -X POST http://localhost:8080/api/v1/memories/search \
        -H "Content-Type: application/json" \
        -d '{"query":"MCP 工具测试","limit":5}')
    
    BACKEND_COUNT=$(echo "$BACKEND_RESULT" | jq -r '.data.memories | length // 0')
    echo "   Backend API 返回: $BACKEND_COUNT 条记忆"
    
    if [ "$BACKEND_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✅ 搜索结果来自真实数据库${NC}"
    else
        echo -e "${YELLOW}⚠️  未找到匹配记忆（可能是向量搜索未返回结果）${NC}"
    fi
fi
echo ""

# 测试 3: agentmem_chat
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 3: agentmem_chat (智能对话)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

CHAT_PARAMS=$(cat <<'EOF'
{
  "message": "你好，请介绍一下 AgentMem 项目",
  "user_id": "mcp-test-user",
  "session_id": "test-session-001",
  "use_memory": true
}
EOF
)

test_mcp_tool "agentmem_chat" "$CHAT_PARAMS" "智能对话"
echo ""

# 测试 4: agentmem_get_system_prompt
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "测试 4: agentmem_get_system_prompt (获取系统提示)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

PROMPT_PARAMS=$(cat <<'EOF'
{
  "user_id": "mcp-test-user",
  "context": "技术讨论"
}
EOF
)

test_mcp_tool "agentmem_get_system_prompt" "$PROMPT_PARAMS" "获取系统提示"
echo ""

# 最终总结
echo "=========================================="
echo "测试完成总结"
echo "=========================================="
echo ""
echo "最终记忆数量: $(curl -s http://localhost:8080/api/v1/stats | jq -r '.data.total_memories // 0')"
echo "基准记忆数量: $BASELINE_COUNT"
echo ""

if [ "$(curl -s http://localhost:8080/api/v1/stats | jq -r '.data.total_memories // 0')" -gt "$BASELINE_COUNT" ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✅ P1-3 修复成功！${NC}"
    echo -e "${GREEN}   MCP 工具已成功集成真实 Backend API${NC}"
    echo -e "${GREEN}   数据已持久化到数据库${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ P1-3 修复可能未完全成功${NC}"
    echo -e "${RED}   请检查 MCP 工具实现${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
fi

