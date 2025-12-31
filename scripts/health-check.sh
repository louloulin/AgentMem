#!/bin/bash
# AgentMem 健康检查脚本

set -e

CONTAINER_NAME="agentmem"
API_URL="http://localhost:8080"
DATA_DIR="$HOME/agentmem"

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${GREEN}✅ $1${NC}"; }
log_warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

echo "🔍 AgentMem 健康检查"
echo "==================="
echo ""

ERRORS=0

# 1. 检查进程
echo "1️⃣  检查进程状态..."
if pgrep -f "agent-mem-server" > /dev/null || docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
    log_info "进程运行中"
else
    log_error "进程未运行"
    ERRORS=$((ERRORS + 1))
fi

# 2. 检查端口
echo ""
echo "2️⃣  检查端口监听..."
if lsof -i :8080 > /dev/null 2>&1 || docker port "$CONTAINER_NAME" 2>/dev/null | grep -q "8080"; then
    log_info "端口 8080 监听中"
else
    log_error "端口 8080 未监听"
    ERRORS=$((ERRORS + 1))
fi

# 3. 检查 API
echo ""
echo "3️⃣  检查 API 响应..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/health" 2>/dev/null || echo "000")

if [[ "$HTTP_CODE" == "200" ]]; then
    log_info "API 响应正常 (HTTP $HTTP_CODE)"
else
    log_error "API 响应异常 (HTTP $HTTP_CODE)"
    ERRORS=$((ERRORS + 1))
fi

# 4. 检查数据库
echo ""
echo "4️⃣  检查数据库文件..."
if [[ -f "$DATA_DIR/data/agentmem.db" ]]; then
    SIZE=$(du -h "$DATA_DIR/data/agentmem.db" | cut -f1)
    log_info "数据库文件存在 (大小: $SIZE)"
else
    log_warn "数据库文件不存在 (将在首次使用时创建)"
fi

# 5. 检查向量存储
if [[ -d "$DATA_DIR/data/vectors.lance" ]]; then
    log_info "向量存储目录存在"
else
    log_warn "向量存储目录不存在 (将在首次使用时创建)"
fi

# 6. 显示服务信息
echo ""
echo "5️⃣  服务详细信息:"
if command -v jq &> /dev/null; then
    curl -s "$API_URL/health" | jq '.' 2>/dev/null || echo "   无法获取详细信息"
else
    curl -s "$API_URL/health" || echo "   无法获取详细信息"
fi

# 7. 显示资源使用
echo ""
echo "6️⃣  资源使用情况:"
if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
    docker stats "$CONTAINER_NAME" --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"
fi

# 总结
echo ""
echo "==================="
if [[ $ERRORS -eq 0 ]]; then
    log_info "所有检查通过！✨"
    exit 0
else
    log_error "发现 $ERRORS 个问题"
    echo ""
    echo "💡 故障排查:"
    echo "   1. 查看日志: docker logs -f $CONTAINER_NAME"
    echo "   2. 重启服务: docker restart $CONTAINER_NAME"
    echo "   3. 查看文档: https://docs.agentmem.ai/troubleshooting"
    exit 1
fi
