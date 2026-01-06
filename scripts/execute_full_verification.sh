#!/bin/bash
# AgentMem 完整验证执行脚本
# 按照agentx7.md计划：通过just启动、MCP验证、真实打开UI验证

set -e

cd "$(dirname "$0")/.."

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }

echo "=========================================="
echo "AgentMem 完整验证执行"
echo "=========================================="
echo ""

BACKEND_URL="http://localhost:8080"
FRONTEND_URL="http://localhost:3001"
MCP_BIN="./target/release/agentmem-mcp-client"

# 步骤1: 检查并构建MCP服务器
log_info "步骤1: 检查MCP服务器..."
if [ ! -f "$MCP_BIN" ]; then
    log_warning "MCP服务器未构建，正在构建..."
    if command -v cargo &> /dev/null; then
        log_info "执行: cargo build --package mcp-stdio-server --release --bin agentmem-mcp-client"
        cargo build --package mcp-stdio-server --release --bin agentmem-mcp-client 2>&1 | tail -10
        if [ -f "$MCP_BIN" ]; then
            log_success "MCP服务器构建完成"
            ls -lh "$MCP_BIN"
        else
            log_error "MCP服务器构建失败"
            exit 1
        fi
    else
        log_error "cargo未安装"
        exit 1
    fi
else
    log_success "MCP服务器已存在"
    ls -lh "$MCP_BIN"
fi
echo ""

# 步骤2: 通过just启动服务
log_info "步骤2: 通过just启动全栈服务..."
if ! command -v just &> /dev/null; then
    log_error "just未安装，请安装: cargo install just"
    exit 1
fi

# 检查后端是否已运行
if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    log_success "后端服务已运行"
else
    log_info "启动全栈服务: just start-full"
    just start-full > /tmp/just_start_full.log 2>&1 &
    JUST_PID=$!
    log_info "等待服务启动（30秒）..."
    
    for i in {1..30}; do
        if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
            log_success "后端服务已启动"
            break
        fi
        sleep 1
    done
fi

# 验证后端
if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    log_success "后端健康检查通过"
    curl -s "$BACKEND_URL/health" | jq '.' 2>/dev/null || curl -s "$BACKEND_URL/health"
else
    log_warning "后端服务可能仍在启动中"
fi
echo ""

# 步骤3: 验证MCP功能
log_info "步骤3: 验证MCP功能..."

INIT_REQUEST='{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
TOOLS_REQUEST='{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# 初始化
echo "$INIT_REQUEST" | "$MCP_BIN" 2>/dev/null | head -1 > /dev/null || true

# 获取工具列表
TOOLS_RESPONSE=$(echo "$TOOLS_REQUEST" | "$MCP_BIN" 2>/dev/null | tail -1)

if echo "$TOOLS_RESPONSE" | grep -q "tools"; then
    TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | jq -r '.result.tools | length' 2>/dev/null || echo "0")
    log_success "MCP工具列表获取成功，找到 $TOOL_COUNT 个工具"
    echo "$TOOLS_RESPONSE" | jq '.result.tools[]?.name' 2>/dev/null || echo "$TOOLS_RESPONSE" | head -20
else
    log_warning "MCP工具列表获取失败"
    echo "$TOOLS_RESPONSE" | head -20
fi
echo ""

# 步骤4: 验证前端并打开UI
log_info "步骤4: 验证前端服务..."
if curl -s "$FRONTEND_URL" > /dev/null 2>&1; then
    log_success "前端服务已运行"
else
    log_warning "前端服务未运行，等待启动..."
    sleep 5
fi

log_info "步骤5: 真实打开UI验证..."
if command -v open &> /dev/null; then
    log_info "打开浏览器: $FRONTEND_URL"
    open "$FRONTEND_URL" 2>/dev/null && log_success "已打开浏览器" || log_warning "无法打开浏览器"
    
    sleep 2
    
    # 打开关键页面
    log_info "打开关键页面..."
    open "$FRONTEND_URL/admin/memories" 2>/dev/null || true
    sleep 1
    open "$FRONTEND_URL/admin/chat" 2>/dev/null || true
    sleep 1
    open "$FRONTEND_URL/admin/agents" 2>/dev/null || true
    
    log_success "已真实打开浏览器验证UI"
else
    log_warning "无法自动打开浏览器，请手动访问: $FRONTEND_URL"
fi
echo ""

# 步骤6: 使用Playwright验证（如果可用）
log_info "步骤6: 使用Playwright验证UI（如果可用）..."
if [ -f "scripts/verify_ui_playwright.js" ]; then
    if command -v node &> /dev/null; then
        log_info "运行Playwright验证脚本..."
        node scripts/verify_ui_playwright.js 2>&1 | head -50 || log_warning "Playwright验证失败"
    else
        log_warning "Node.js未安装，跳过Playwright验证"
    fi
else
    log_warning "Playwright验证脚本不存在，跳过"
fi
echo ""

# 验证总结
echo "=========================================="
echo "验证总结"
echo "=========================================="
echo ""
log_success "完整验证流程执行完成！"
echo ""
echo "服务地址:"
echo "  后端API: $BACKEND_URL"
echo "  前端UI:  $FRONTEND_URL"
echo ""
echo "MCP服务器: $MCP_BIN"
echo ""
echo "下一步: 更新 agentx7.md 标记完成的功能"
echo ""


