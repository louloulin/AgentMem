#!/bin/bash
# AgentMem 快速真实验证

cd "$(dirname "$0")/.."

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }

echo "=========================================="
echo "AgentMem 快速真实验证"
echo "=========================================="
echo ""

# 检查服务
log_info "检查服务状态..."
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    log_success "后端服务已运行"
else
    log_info "启动服务..."
    just start-full &
    sleep 20
fi

# 验证后端
log_info "验证后端..."
HEALTH=$(curl -s http://localhost:8080/health 2>/dev/null)
if [ -n "$HEALTH" ]; then
    log_success "后端健康检查通过"
    echo "$HEALTH" | jq '.' 2>/dev/null || echo "$HEALTH"
fi
echo ""

# 打开UI
log_info "打开UI验证..."
if command -v open &> /dev/null; then
    open http://localhost:3001
    sleep 1
    open http://localhost:3001/admin/memories 2>/dev/null || true
    open http://localhost:3001/admin/chat 2>/dev/null || true
    log_success "已打开浏览器"
fi
echo ""

log_success "验证完成！"
echo "后端: http://localhost:8080"
echo "前端: http://localhost:3001"




