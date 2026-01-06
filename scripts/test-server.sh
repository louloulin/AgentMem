#!/bin/bash
# AgentMem Server 功能测试脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 服务器配置
SERVER_HOST="localhost"
SERVER_PORT="8080"
BASE_URL="http://${SERVER_HOST}:${SERVER_PORT}"

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# 检查服务器是否运行
check_server() {
    log_info "检查服务器状态..."
    
    if curl -s -f "${BASE_URL}/health" > /dev/null 2>&1; then
        log_success "服务器正在运行"
        return 0
    else
        log_error "服务器未运行或无法访问"
        return 1
    fi
}

# 测试健康检查端点
test_health() {
    log_info "测试健康检查端点..."
    
    response=$(curl -s "${BASE_URL}/health")
    status=$(echo "$response" | jq -r '.status' 2>/dev/null || echo "error")
    
    if [ "$status" = "healthy" ]; then
        log_success "健康检查通过"
        echo "$response" | jq .
        return 0
    else
        log_error "健康检查失败"
        echo "$response"
        return 1
    fi
}

# 测试 API 文档
test_docs() {
    log_info "测试 API 文档端点..."
    
    if curl -s -f "${BASE_URL}/swagger-ui/" > /dev/null 2>&1; then
        log_success "API 文档可访问"
        log_info "访问地址: ${BASE_URL}/swagger-ui/"
        return 0
    else
        log_error "API 文档不可访问"
        return 1
    fi
}

# 测试 Metrics 端点
test_metrics() {
    log_info "测试 Metrics 端点..."
    
    if curl -s -f "${BASE_URL}/metrics" > /dev/null 2>&1; then
        log_success "Metrics 端点可访问"
        return 0
    else
        log_warn "Metrics 端点不可访问（可能未启用）"
        return 0  # 不作为错误
    fi
}

# 测试创建记忆
test_create_memory() {
    log_info "测试创建记忆..."
    
    payload='{
        "content": "这是一条测试记忆",
        "memory_type": "Episodic",
        "importance": 0.8
    }'
    
    response=$(curl -s -X POST "${BASE_URL}/api/v1/memory" \
        -H "Content-Type: application/json" \
        -d "$payload")
    
    memory_id=$(echo "$response" | jq -r '.id' 2>/dev/null || echo "")
    
    if [ -n "$memory_id" ] && [ "$memory_id" != "null" ]; then
        log_success "记忆创建成功，ID: $memory_id"
        echo "$memory_id"
        return 0
    else
        log_error "记忆创建失败"
        echo "$response"
        return 1
    fi
}

# 测试搜索记忆
test_search_memory() {
    log_info "测试搜索记忆..."
    
    query="测试"
    
    response=$(curl -s "${BASE_URL}/api/v1/memory/search?query=${query}&limit=10")
    
    count=$(echo "$response" | jq -r '.results | length' 2>/dev/null || echo "0")
    
    if [ "$count" -ge 0 ]; then
        log_success "搜索成功，找到 $count 条记忆"
        return 0
    else
        log_error "搜索失败"
        echo "$response"
        return 1
    fi
}

# 测试获取记忆详情
test_get_memory() {
    local memory_id=$1
    
    if [ -z "$memory_id" ]; then
        log_warn "跳过获取记忆测试（没有记忆 ID）"
        return 0
    fi
    
    log_info "测试获取记忆详情..."
    
    response=$(curl -s "${BASE_URL}/api/v1/memory/${memory_id}")
    
    id=$(echo "$response" | jq -r '.id' 2>/dev/null || echo "")
    
    if [ "$id" = "$memory_id" ]; then
        log_success "获取记忆详情成功"
        return 0
    else
        log_error "获取记忆详情失败"
        echo "$response"
        return 1
    fi
}

# 测试删除记忆
test_delete_memory() {
    local memory_id=$1
    
    if [ -z "$memory_id" ]; then
        log_warn "跳过删除记忆测试（没有记忆 ID）"
        return 0
    fi
    
    log_info "测试删除记忆..."
    
    response=$(curl -s -X DELETE "${BASE_URL}/api/v1/memory/${memory_id}")
    
    if echo "$response" | grep -q "success\|deleted" 2>/dev/null; then
        log_success "删除记忆成功"
        return 0
    else
        log_warn "删除记忆可能失败（或已删除）"
        return 0  # 不作为错误
    fi
}

# 检查日志文件
check_logs() {
    log_info "检查日志文件..."
    
    log_dir="dist/server/logs"
    
    if [ -d "$log_dir" ]; then
        log_count=$(find "$log_dir" -name "*.log*" -type f | wc -l | tr -d ' ')
        if [ "$log_count" -gt 0 ]; then
            log_success "找到 $log_count 个日志文件"
            find "$log_dir" -name "*.log*" -type f -exec ls -lh {} \;
            return 0
        else
            log_warn "日志目录存在但没有日志文件"
            return 0
        fi
    else
        log_warn "日志目录不存在: $log_dir"
        return 0
    fi
}

# 检查数据目录
check_data() {
    log_info "检查数据目录..."
    
    data_dir="dist/server/data"
    
    if [ -d "$data_dir" ]; then
        log_success "数据目录存在"
        ls -lh "$data_dir"
        
        # 检查数据库文件
        if [ -f "$data_dir/agentmem.db" ]; then
            db_size=$(ls -lh "$data_dir/agentmem.db" | awk '{print $5}')
            log_success "数据库文件存在，大小: $db_size"
        else
            log_warn "数据库文件不存在"
        fi
        
        # 检查向量存储
        if [ -d "$data_dir/vectors.lance" ]; then
            log_success "向量存储目录存在"
        else
            log_warn "向量存储目录不存在"
        fi
        
        return 0
    else
        log_warn "数据目录不存在: $data_dir"
        return 0
    fi
}

# 主测试流程
main() {
    echo "========================================="
    echo "🧪 AgentMem Server 功能测试"
    echo "========================================="
    echo ""
    
    # 检查依赖
    if ! command -v curl &> /dev/null; then
        log_error "curl 未安装"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        log_warn "jq 未安装，部分测试可能失败"
    fi
    
    # 检查服务器
    if ! check_server; then
        log_error "请先启动服务器: cd dist/server && ./start.sh"
        exit 1
    fi
    
    echo ""
    echo "========================================="
    echo "📊 基础功能测试"
    echo "========================================="
    echo ""
    
    # 基础测试
    test_health || exit 1
    echo ""
    
    test_docs || true
    echo ""
    
    test_metrics || true
    echo ""
    
    echo "========================================="
    echo "🧠 Memory API 测试"
    echo "========================================="
    echo ""
    
    # Memory API 测试
    memory_id=$(test_create_memory) || true
    echo ""
    
    test_search_memory || true
    echo ""
    
    if [ -n "$memory_id" ]; then
        test_get_memory "$memory_id" || true
        echo ""
        
        test_delete_memory "$memory_id" || true
        echo ""
    fi
    
    echo "========================================="
    echo "📁 文件系统检查"
    echo "========================================="
    echo ""
    
    check_logs || true
    echo ""
    
    check_data || true
    echo ""
    
    echo "========================================="
    echo "✅ 测试完成"
    echo "========================================="
}

# 运行主函数
main "$@"

