#!/bin/bash

# AgentMem UI 简化验证脚本（不使用 Playwright）
# 通过 curl 和基本检查验证 UI 功能

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
FRONTEND_URL="http://localhost:3001"
BACKEND_URL="http://localhost:8080"

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 测试结果
PASSED=0
FAILED=0
WARNINGS=0

# 测试函数
test_page() {
    local name=$1
    local url=$2
    
    log_info "测试: $name"
    
    response=$(curl -s -w "\n%{http_code}" "$FRONTEND_URL$url" 2>&1)
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        if [ -n "$body" ] && [ ${#body} -gt 100 ]; then
            log_success "$name - HTTP $http_code (内容长度: ${#body})"
            ((PASSED++))
            return 0
        else
            log_warning "$name - HTTP $http_code (内容可能为空)"
            ((WARNINGS++))
            return 0
        fi
    else
        log_error "$name - HTTP $http_code"
        ((FAILED++))
        return 1
    fi
}

echo "=========================================="
echo "AgentMem UI 简化验证"
echo "版本: v1.0"
echo "日期: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 检查前端服务
log_info "检查前端服务..."
if curl -s "$FRONTEND_URL" > /dev/null 2>&1; then
    log_success "前端服务运行正常"
else
    log_error "前端服务未运行，请先启动: just start-ui 或 just start-full"
    exit 1
fi

# 检查后端服务
log_info "检查后端服务..."
if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    log_success "后端服务运行正常"
else
    log_warning "后端服务未运行，某些功能可能不可用"
    ((WARNINGS++))
fi

# 测试各个页面
echo ""
echo "=========================================="
echo "页面加载测试"
echo "=========================================="

test_page "首页" "/"
test_page "Dashboard" "/dashboard"
test_page "Chat" "/chat"
test_page "记忆管理" "/admin/memories"
test_page "Agent 管理" "/admin/agents"

# 测试 API 端点（如果后端运行）
if curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    echo ""
    echo "=========================================="
    echo "后端 API 端点测试"
    echo "=========================================="
    
    # 健康检查
    if curl -s "$BACKEND_URL/health" | grep -q "healthy"; then
        log_success "后端健康检查正常"
        ((PASSED++))
    else
        log_error "后端健康检查失败"
        ((FAILED++))
    fi
    
    # Dashboard 统计
    if curl -s "$BACKEND_URL/api/v1/stats/dashboard" | grep -q "total_memories\|total_agents"; then
        log_success "Dashboard API 正常"
        ((PASSED++))
    else
        log_warning "Dashboard API 可能异常"
        ((WARNINGS++))
    fi
    
    # Agent 列表
    if curl -s "$BACKEND_URL/api/v1/agents" | grep -q "agents\|data"; then
        log_success "Agent API 正常"
        ((PASSED++))
    else
        log_warning "Agent API 可能异常"
        ((WARNINGS++))
    fi
fi

# 总结
echo ""
echo "=========================================="
echo "UI 验证总结"
echo "=========================================="
echo "总测试数: $((PASSED + FAILED + WARNINGS))"
log_success "通过: $PASSED"
if [ $FAILED -gt 0 ]; then
    log_error "失败: $FAILED"
fi
if [ $WARNINGS -gt 0 ]; then
    log_warning "警告: $WARNINGS"
fi
echo ""
echo "=========================================="
echo "验证完成！"
echo "=========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
