#!/bin/bash

# AgentMem 完整验证脚本
# 运行所有验证：MCP、后端 API、前端 UI

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 报告目录
REPORT_DIR="./verification-reports/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$REPORT_DIR"

echo "=========================================="
echo "AgentMem 完整功能验证"
echo "版本: v1.0"
echo "日期: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 检查服务状态
log_info "检查服务状态..."
BACKEND_OK=false
FRONTEND_OK=false

if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    log_success "后端服务运行正常"
    BACKEND_OK=true
else
    log_error "后端服务未运行"
    echo "请先启动后端: just start-server-no-auth"
fi

if curl -s http://localhost:3001 > /dev/null 2>&1; then
    log_success "前端服务运行正常"
    FRONTEND_OK=true
else
    log_error "前端服务未运行"
    echo "请先启动前端: just start-ui 或 just start-full"
fi

if [ "$BACKEND_OK" = false ] && [ "$FRONTEND_OK" = false ]; then
    log_error "前后端服务都未运行，无法进行验证"
    exit 1
fi

# 1. MCP 验证
echo ""
echo "=========================================="
echo "1. MCP 功能验证"
echo "=========================================="
if [ "$BACKEND_OK" = true ]; then
    log_info "运行 MCP 验证..."
    if bash scripts/verify_mcp_functionality.sh > "$REPORT_DIR/mcp-verification.log" 2>&1; then
        log_success "MCP 验证完成"
    else
        log_error "MCP 验证失败，查看日志: $REPORT_DIR/mcp-verification.log"
    fi
else
    log_error "跳过 MCP 验证（后端未运行）"
fi

# 2. 后端 API 验证
echo ""
echo "=========================================="
echo "2. 后端 API 验证"
echo "=========================================="
if [ "$BACKEND_OK" = true ]; then
    log_info "运行后端 API 验证..."
    if bash scripts/verify_server_api.sh > "$REPORT_DIR/api-verification.log" 2>&1; then
        log_success "后端 API 验证完成"
    else
        log_error "后端 API 验证失败，查看日志: $REPORT_DIR/api-verification.log"
    fi
else
    log_error "跳过后端 API 验证（后端未运行）"
fi

# 3. OpenAPI 验证
echo ""
echo "=========================================="
echo "3. OpenAPI 规范验证"
echo "=========================================="
if [ "$BACKEND_OK" = true ]; then
    log_info "运行 OpenAPI 验证..."
    if bash scripts/verify_openapi.sh > "$REPORT_DIR/openapi-verification.log" 2>&1; then
        log_success "OpenAPI 验证完成"
    else
        log_error "OpenAPI 验证失败，查看日志: $REPORT_DIR/openapi-verification.log"
    fi
else
    log_error "跳过 OpenAPI 验证（后端未运行）"
fi

# 4. 前端 UI 验证（Playwright）
echo ""
echo "=========================================="
echo "4. 前端 UI 验证（Playwright）"
echo "=========================================="
if [ "$FRONTEND_OK" = true ] && [ "$BACKEND_OK" = true ]; then
    log_info "检查 Playwright 依赖..."
    
    # 检查是否安装了 Playwright
    if command -v npx &> /dev/null; then
        cd agentmem-ui
        
        # 检查 package.json 中是否有 playwright
        if ! grep -q "playwright" package.json 2>/dev/null; then
            log_info "安装 Playwright..."
            npm install --save-dev @playwright/test playwright || true
        fi
        
        # 安装 Playwright 浏览器（如果需要）
        npx playwright install chromium 2>/dev/null || true
        
        cd ..
        
        log_info "运行 Playwright UI 验证..."
        if node scripts/verify_ui_playwright.js > "$REPORT_DIR/ui-verification.log" 2>&1; then
            log_success "前端 UI 验证完成"
        else
            log_error "前端 UI 验证失败，查看日志: $REPORT_DIR/ui-verification.log"
        fi
    else
        log_error "未找到 npx，无法运行 Playwright 测试"
        log_info "请安装 Node.js 和 npm"
    fi
else
    log_error "跳过前端 UI 验证（前后端服务未运行）"
fi

# 生成总结报告
echo ""
echo "=========================================="
echo "验证总结报告"
echo "=========================================="
echo "报告目录: $REPORT_DIR"
echo ""
echo "生成的报告文件:"
ls -lh "$REPORT_DIR" 2>/dev/null || echo "无报告文件"
echo ""
echo "查看详细报告:"
echo "  MCP 验证: cat $REPORT_DIR/mcp-verification.log"
echo "  API 验证: cat $REPORT_DIR/api-verification.log"
echo "  OpenAPI 验证: cat $REPORT_DIR/openapi-verification.log"
echo "  UI 验证: cat $REPORT_DIR/ui-verification.log"
echo ""
echo "=========================================="
echo "完整验证完成！"
echo "=========================================="
