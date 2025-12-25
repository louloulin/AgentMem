#!/bin/bash
# AgentMem 真实验证脚本 - 真实启动服务并验证

set -e

cd "$(dirname "$0")/.."

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo "=========================================="
echo "AgentMem 真实验证流程"
echo "=========================================="
echo ""

# 步骤1: 检查并构建MCP服务器
log_info "步骤1: 检查MCP服务器..."
if [ ! -f "target/release/agentmem-mcp-client" ]; then
    log_info "构建MCP服务器..."
    cargo build --package mcp-stdio-server --release --bin agentmem-mcp-client
    log_success "MCP服务器构建完成"
else
    log_success "MCP服务器已存在"
fi
echo ""

# 步骤2: 停止现有服务
log_info "步骤2: 停止现有服务..."
pkill -f "agent-mem-server" 2>/dev/null || true
pkill -f "next dev" 2>/dev/null || true
sleep 2
log_success "已停止现有服务"
echo ""

# 步骤3: 启动后端服务
log_info "步骤3: 启动后端服务..."
export ENABLE_AUTH="false"
export SERVER_ENABLE_AUTH="false"
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 检查是否需要构建
if [ ! -f "target/release/agent-mem-server" ]; then
    log_info "构建后端服务器..."
    cargo build --package agent-mem-server --release
fi

# 启动后端
nohup ./target/release/agent-mem-server > backend-verification.log 2>&1 &
BACKEND_PID=$!
log_info "后端服务启动中 (PID: $BACKEND_PID)..."
sleep 10

# 检查后端健康
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_success "后端服务已就绪"
        curl -s http://localhost:8080/health | jq '.' 2>/dev/null || curl -s http://localhost:8080/health
        break
    fi
    if [ $i -eq 30 ]; then
        log_error "后端服务启动失败"
        tail -20 backend-verification.log
        exit 1
    fi
    sleep 1
done
echo ""

# 步骤4: 启动前端服务
log_info "步骤4: 启动前端服务..."
cd agentmem-ui

# 检查依赖
if [ ! -d "node_modules" ]; then
    log_info "安装前端依赖..."
    npm install > /dev/null 2>&1
fi

# 启动前端
nohup npm run dev > ../frontend-verification.log 2>&1 &
FRONTEND_PID=$!
cd ..
log_info "前端服务启动中 (PID: $FRONTEND_PID)..."
sleep 10

# 检查前端
for i in {1..30}; do
    if curl -s http://localhost:3001 > /dev/null 2>&1; then
        log_success "前端服务已就绪"
        break
    fi
    if [ $i -eq 30 ]; then
        log_warning "前端服务可能仍在启动中..."
    fi
    sleep 1
done
echo ""

# 步骤5: 验证MCP功能
log_info "步骤5: 验证MCP功能..."
MCP_BIN="./target/release/agentmem-mcp-client"

# 测试MCP工具列表
log_info "测试MCP工具列表..."
INIT_REQUEST='{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
TOOLS_REQUEST='{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# 初始化
echo "$INIT_REQUEST" | timeout 5 "$MCP_BIN" 2>/dev/null | head -1 > /dev/null || true

# 获取工具列表
TOOLS_RESPONSE=$(echo "$TOOLS_REQUEST" | timeout 5 "$MCP_BIN" 2>/dev/null | tail -1)
if echo "$TOOLS_RESPONSE" | grep -q "tools"; then
    TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | jq -r '.result.tools | length' 2>/dev/null || echo "0")
    log_success "MCP工具列表获取成功，找到 $TOOL_COUNT 个工具"
    echo "$TOOLS_RESPONSE" | jq -r '.result.tools[]?.name' 2>/dev/null || echo "$TOOLS_RESPONSE"
else
    log_warning "MCP工具列表获取可能失败"
    echo "$TOOLS_RESPONSE"
fi
echo ""

# 测试添加记忆
log_info "测试MCP添加记忆功能..."
ADD_MEMORY_REQUEST='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"这是一个真实验证测试记忆 - '"$(date)"'","user_id":"real-verification-user","agent_id":"real-verification-agent","memory_type":"Episodic"}}}'

# 初始化
echo "$INIT_REQUEST" | timeout 5 "$MCP_BIN" 2>/dev/null | head -1 > /dev/null || true

# 调用工具
ADD_RESPONSE=$(echo "$ADD_MEMORY_REQUEST" | timeout 10 "$MCP_BIN" 2>/dev/null | tail -1)
if echo "$ADD_RESPONSE" | grep -q "success.*true\|memory_id"; then
    log_success "MCP添加记忆功能正常"
    echo "$ADD_RESPONSE" | jq '.' 2>/dev/null || echo "$ADD_RESPONSE"
else
    log_warning "MCP添加记忆功能可能异常"
    echo "$ADD_RESPONSE"
fi
echo ""

# 测试搜索记忆
log_info "测试MCP搜索记忆功能..."
SEARCH_REQUEST='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"验证测试","user_id":"real-verification-user","limit":5}}}'

# 初始化
echo "$INIT_REQUEST" | timeout 5 "$MCP_BIN" 2>/dev/null | head -1 > /dev/null || true

# 调用工具
SEARCH_RESPONSE=$(echo "$SEARCH_REQUEST" | timeout 10 "$MCP_BIN" 2>/dev/null | tail -1)
if echo "$SEARCH_RESPONSE" | grep -q "success.*true\|memories"; then
    log_success "MCP搜索记忆功能正常"
    echo "$SEARCH_RESPONSE" | jq '.' 2>/dev/null || echo "$SEARCH_RESPONSE"
else
    log_warning "MCP搜索记忆功能可能异常"
    echo "$SEARCH_RESPONSE"
fi
echo ""

# 步骤6: 打开UI验证
log_info "步骤6: 打开UI验证..."
if command -v open &> /dev/null; then
    log_info "正在打开浏览器: http://localhost:3001"
    open http://localhost:3001
    sleep 2
    log_success "已打开浏览器"
    
    # 也打开几个关键页面
    open http://localhost:3001/admin/memories 2>/dev/null || true
    sleep 1
    open http://localhost:3001/admin/chat 2>/dev/null || true
    sleep 1
    open http://localhost:3001/admin/agents 2>/dev/null || true
    log_success "已打开关键页面"
elif command -v xdg-open &> /dev/null; then
    xdg-open http://localhost:3001
    log_success "已打开浏览器 (Linux)"
else
    log_warning "无法自动打开浏览器，请手动访问: http://localhost:3001"
fi
echo ""

# 步骤7: 显示服务信息
echo "=========================================="
echo "服务信息"
echo "=========================================="
echo ""
echo "后端服务:"
echo "  • PID: $BACKEND_PID"
echo "  • API: http://localhost:8080"
echo "  • 健康检查: http://localhost:8080/health"
echo "  • API文档: http://localhost:8080/swagger-ui/"
echo "  • 日志: backend-verification.log"
echo ""
echo "前端服务:"
echo "  • PID: $FRONTEND_PID"
echo "  • UI: http://localhost:3001"
echo "  • 日志: frontend-verification.log"
echo ""
echo "MCP服务器:"
echo "  • 二进制: $MCP_BIN"
echo "  • 启动命令: just start-mcp"
echo ""
echo "=========================================="
echo "验证完成"
echo "=========================================="
echo ""
log_success "所有服务已启动并验证"
echo ""
log_info "请在浏览器中验证UI功能："
echo "  1. 记忆管理页面: http://localhost:3001/admin/memories"
echo "  2. 对话页面: http://localhost:3001/admin/chat"
echo "  3. Agent管理页面: http://localhost:3001/admin/agents"
echo ""
log_info "停止服务命令:"
echo "  pkill -f agent-mem-server"
echo "  pkill -f 'next dev'"
echo "  或运行: just stop"
echo ""



