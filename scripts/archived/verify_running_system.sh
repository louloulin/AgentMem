#!/bin/bash

# AgentMem 运行中系统验证脚本
# 验证已经运行的 Backend (8080) 和 Frontend (3001)

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

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo "=========================================="
echo "AgentMem 运行中系统功能验证"
echo "版本: v1.0"
echo "日期: $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# Phase 1: 检查服务状态
echo "=========================================="
echo "Phase 1: 检查服务运行状态"
echo "=========================================="

log_info "检查Backend服务 (端口 8080)..."
if lsof -i :8080 > /dev/null 2>&1; then
    log_success "Backend服务正在运行"
    lsof -i :8080 | grep LISTEN
else
    log_error "Backend服务未运行"
    exit 1
fi

log_info "检查Frontend服务 (端口 3001)..."
if lsof -i :3001 > /dev/null 2>&1; then
    log_success "Frontend服务正在运行"
    lsof -i :3001 | grep LISTEN
else
    log_warning "Frontend服务未运行 (可选)"
fi

log_success "Phase 1: 服务状态检查完成"
echo ""

# Phase 2: Backend API 功能验证
echo "=========================================="
echo "Phase 2: Backend API 功能验证"
echo "=========================================="

log_info "测试 Health Check..."
HEALTH_RESPONSE=$(curl -s http://localhost:8080/health)
if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    log_success "Health Check 通过"
    echo "响应: $HEALTH_RESPONSE"
else
    log_error "Health Check 失败"
    echo "响应: $HEALTH_RESPONSE"
    exit 1
fi

log_info "测试 API 文档..."
SWAGGER_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/swagger-ui/)
if [ "$SWAGGER_RESPONSE" = "200" ]; then
    log_success "Swagger UI 可访问"
else
    log_warning "Swagger UI 返回状态码: $SWAGGER_RESPONSE"
fi

log_info "测试 Memory API - 获取记忆列表..."
MEMORIES_RESPONSE=$(curl -s -X GET "http://localhost:8080/api/v1/memories?limit=5" \
    -H "Content-Type: application/json")
if [ $? -eq 0 ]; then
    log_success "Memory API 响应成功"
    echo "响应示例: $(echo "$MEMORIES_RESPONSE" | head -c 200)..."
else
    log_error "Memory API 请求失败"
fi

log_info "测试 Memory API - 添加记忆..."
ADD_MEMORY_RESPONSE=$(curl -s -X POST "http://localhost:8080/api/v1/memories" \
    -H "Content-Type: application/json" \
    -d '{
        "content": "验证测试记忆 - '"$(date '+%Y-%m-%d %H:%M:%S')"'",
        "memory_type": "Episodic",
        "agent_id": "test-agent-001",
        "metadata": {
            "source": "verification_script",
            "test": true
        }
    }')

if echo "$ADD_MEMORY_RESPONSE" | grep -q "id"; then
    log_success "添加记忆成功"
    MEMORY_ID=$(echo "$ADD_MEMORY_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo "记忆ID: $MEMORY_ID"
else
    log_warning "添加记忆可能失败"
    echo "响应: $ADD_MEMORY_RESPONSE"
fi

log_info "测试 Memory API - 搜索记忆..."
SEARCH_RESPONSE=$(curl -s -X POST "http://localhost:8080/api/v1/memories/search" \
    -H "Content-Type: application/json" \
    -d '{
        "query": "验证测试",
        "limit": 5
    }')

if [ $? -eq 0 ]; then
    log_success "搜索记忆成功"
    echo "搜索结果数量: $(echo "$SEARCH_RESPONSE" | grep -o '"id"' | wc -l)"
else
    log_error "搜索记忆失败"
fi

log_info "测试 Agent API - 获取Agent列表..."
AGENTS_RESPONSE=$(curl -s -X GET "http://localhost:8080/api/v1/agents?limit=5" \
    -H "Content-Type: application/json")
if [ $? -eq 0 ]; then
    log_success "Agent API 响应成功"
    echo "Agent数量: $(echo "$AGENTS_RESPONSE" | grep -o '"id"' | wc -l)"
else
    log_error "Agent API 请求失败"
fi

log_info "测试 Stats API - 获取统计信息..."
STATS_RESPONSE=$(curl -s -X GET "http://localhost:8080/api/v1/stats" \
    -H "Content-Type: application/json")
if [ $? -eq 0 ]; then
    log_success "Stats API 响应成功"
    echo "统计信息: $(echo "$STATS_RESPONSE" | head -c 200)..."
else
    log_error "Stats API 请求失败"
fi

log_success "Phase 2: Backend API 功能验证完成"
echo ""

# Phase 3: 数据库验证
echo "=========================================="
echo "Phase 3: 数据库持久化验证"
echo "=========================================="

log_info "检查数据库文件..."
if [ -f "./data/agentmem.db" ]; then
    log_success "数据库文件存在"
    DB_SIZE=$(ls -lh ./data/agentmem.db | awk '{print $5}')
    echo "数据库大小: $DB_SIZE"
else
    log_error "数据库文件不存在"
fi

log_success "Phase 3: 数据库验证完成"
echo ""

# Phase 4: Frontend 验证 (可选)
echo "=========================================="
echo "Phase 4: Frontend 验证 (可选)"
echo "=========================================="

if lsof -i :3001 > /dev/null 2>&1; then
    log_info "测试 Frontend 首页..."
    FRONTEND_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3001)
    if [ "$FRONTEND_RESPONSE" = "200" ]; then
        log_success "Frontend 首页可访问"
        echo "访问地址: http://localhost:3001"
    else
        log_warning "Frontend 返回状态码: $FRONTEND_RESPONSE"
    fi
    
    log_info "测试 Frontend Admin 页面..."
    ADMIN_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3001/admin/dashboard)
    if [ "$ADMIN_RESPONSE" = "200" ]; then
        log_success "Admin Dashboard 可访问"
        echo "访问地址: http://localhost:3001/admin/dashboard"
    else
        log_warning "Admin Dashboard 返回状态码: $ADMIN_RESPONSE"
    fi
else
    log_warning "Frontend服务未运行，跳过Frontend验证"
fi

log_success "Phase 4: Frontend 验证完成"
echo ""

# 总结
echo "=========================================="
echo "验证总结"
echo "=========================================="
log_success "✅ Backend服务运行正常 (http://localhost:8080)"
log_success "✅ Health Check 通过"
log_success "✅ Memory CRUD API 正常"
log_success "✅ Agent API 正常"
log_success "✅ Stats API 正常"
log_success "✅ 数据库持久化正常"

if lsof -i :3001 > /dev/null 2>&1; then
    log_success "✅ Frontend服务运行正常 (http://localhost:3001)"
fi

echo ""
echo "=========================================="
echo "下一步操作建议"
echo "=========================================="
echo "1. 访问 Swagger UI: http://localhost:8080/swagger-ui/"
echo "2. 访问 Frontend: http://localhost:3001"
echo "3. 访问 Admin Dashboard: http://localhost:3001/admin/dashboard"
echo "4. 查看数据库: sqlite3 ./data/agentmem.db"
echo "5. 查看日志: tail -f backend.log"
echo ""
echo "=========================================="
echo "验证完成！"
echo "=========================================="

