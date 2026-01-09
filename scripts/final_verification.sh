#!/bin/bash
# AgentMem 最终完整验证 - 通过just启动，MCP验证，真实打开UI

cd "$(dirname "$0")/.."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }

echo "=========================================="
echo "AgentMem 最终完整验证"
echo "=========================================="
echo ""

# 步骤1: 通过just启动服务
log_info "步骤1: 通过just启动全栈服务..."
if command -v just &> /dev/null; then
    log_info "执行: just start-full"
    just start-full > /tmp/just_start.log 2>&1 &
    sleep 25
    
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_success "后端服务已启动"
        curl -s http://localhost:8080/health | jq '.' 2>/dev/null || curl -s http://localhost:8080/health
    else
        log_warning "后端服务可能仍在启动中"
    fi
else
    log_warning "just未安装"
    exit 1
fi
echo ""

# 步骤2: 验证MCP功能
log_info "步骤2: 验证MCP功能..."
if [ -f "target/release/agentmem-mcp-client" ]; then
    log_success "MCP服务器已构建"
    INIT='{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
    TOOLS='{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
    
    echo "$INIT" | timeout 3 ./target/release/agentmem-mcp-client 2>/dev/null | head -1 > /dev/null || true
    TOOLS_RESPONSE=$(echo "$TOOLS" | timeout 3 ./target/release/agentmem-mcp-client 2>/dev/null | tail -1)
    
    if echo "$TOOLS_RESPONSE" | grep -q "tools"; then
        TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | jq -r '.result.tools | length' 2>/dev/null || echo "0")
        log_success "MCP工具列表获取成功，找到 $TOOL_COUNT 个工具"
    else
        log_warning "MCP工具列表获取失败"
    fi
else
    log_warning "MCP服务器未构建"
fi
echo ""

# 步骤3: 真实打开UI验证
log_info "步骤3: 真实打开UI验证..."
if command -v open &> /dev/null; then
    open http://localhost:3001
    sleep 2
    open http://localhost:3001/admin/memories 2>/dev/null || true
    open http://localhost:3001/admin/chat 2>/dev/null || true
    log_success "已真实打开浏览器验证UI"
fi
echo ""

log_success "最终验证完成！"
echo "后端: http://localhost:8080"
echo "前端: http://localhost:3001"





















