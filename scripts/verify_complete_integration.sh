#!/bin/bash
# AgentMem 完整集成验证脚本
# 验证：服务启动、MCP功能、UI访问、真实实现

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
BACKEND_URL="http://localhost:8080"
FRONTEND_URL="http://localhost:3001"
MCP_SERVER_BIN="./target/release/agentmem-mcp-client"
TEST_USER_ID="test-user-$(date +%s)"
TEST_AGENT_ID="test-agent"
TEST_SESSION_ID="test-session-$(date +%s)"

# 日志函数
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

# 检查命令是否存在
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 未安装"
        return 1
    fi
    return 0
}

# 等待服务就绪
wait_for_service() {
    local url=$1
    local max_attempts=30
    local attempt=0
    
    log_info "等待服务就绪: $url"
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            log_success "服务已就绪: $url"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done
    
    log_error "服务未就绪: $url"
    return 1
}

# 检查后端健康状态
check_backend_health() {
    log_info "检查后端健康状态..."
    
    local response=$(curl -s "$BACKEND_URL/health" 2>/dev/null)
    if echo "$response" | grep -q "healthy\|ok\|status"; then
        log_success "后端服务健康"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 0
    else
        log_error "后端服务不健康"
        return 1
    fi
}

# 测试MCP工具：添加记忆
test_mcp_add_memory() {
    log_info "测试MCP工具: agentmem_add_memory"
    
    local test_content="这是一个测试记忆，用于验证MCP功能 - $(date)"
    local request=$(cat <<EOF
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"$test_content","user_id":"$TEST_USER_ID","agent_id":"$TEST_AGENT_ID","memory_type":"Episodic"}}}
EOF
)
    
    # 初始化MCP协议
    local init_request='{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
    
    # 发送初始化请求
    echo "$init_request" | timeout 5 "$MCP_SERVER_BIN" 2>/dev/null | head -1 > /dev/null || true
    
    # 发送工具调用请求
    local response=$(echo "$request" | timeout 5 "$MCP_SERVER_BIN" 2>/dev/null | tail -1)
    
    if echo "$response" | grep -q "success.*true\|memory_id"; then
        log_success "MCP添加记忆功能正常"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 0
    else
        log_warning "MCP添加记忆功能可能异常（需要后端服务运行）"
        echo "$response"
        return 1
    fi
}

# 测试MCP工具：搜索记忆
test_mcp_search_memories() {
    log_info "测试MCP工具: agentmem_search_memories"
    
    local request=$(cat <<EOF
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"测试","user_id":"$TEST_USER_ID","limit":5}}}
EOF
)
    
    local response=$(echo "$request" | timeout 5 "$MCP_SERVER_BIN" 2>/dev/null | tail -1)
    
    if echo "$response" | grep -q "success.*true\|memories"; then
        log_success "MCP搜索记忆功能正常"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 0
    else
        log_warning "MCP搜索记忆功能可能异常（需要后端服务运行）"
        echo "$response"
        return 1
    fi
}

# 测试MCP工具：列出Agent
test_mcp_list_agents() {
    log_info "测试MCP工具: agentmem_list_agents"
    
    local request=$(cat <<EOF
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_list_agents","arguments":{"user_id":"$TEST_USER_ID"}}}
EOF
)
    
    local response=$(echo "$request" | timeout 5 "$MCP_SERVER_BIN" 2>/dev/null | tail -1)
    
    if echo "$response" | grep -q "success.*true\|agents"; then
        log_success "MCP列出Agent功能正常"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 0
    else
        log_warning "MCP列出Agent功能可能异常（需要后端服务运行）"
        echo "$response"
        return 1
    fi
}

# 测试MCP工具：智能对话
test_mcp_chat() {
    log_info "测试MCP工具: agentmem_chat"
    
    local request=$(cat <<EOF
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"agentmem_chat","arguments":{"message":"你好，请介绍一下AgentMem","user_id":"$TEST_USER_ID","session_id":"$TEST_SESSION_ID","use_memory":true}}}
EOF
)
    
    local response=$(echo "$request" | timeout 10 "$MCP_SERVER_BIN" 2>/dev/null | tail -1)
    
    if echo "$response" | grep -q "success.*true\|response"; then
        log_success "MCP智能对话功能正常"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 0
    else
        log_warning "MCP智能对话功能可能异常（需要后端服务和LLM配置）"
        echo "$response"
        return 1
    fi
}

# 测试MCP工具列表
test_mcp_list_tools() {
    log_info "测试MCP工具列表"
    
    local request='{"jsonrpc":"2.0","id":5,"method":"tools/list","params":{}}'
    
    # 先初始化
    local init_request='{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
    echo "$init_request" | timeout 5 "$MCP_SERVER_BIN" 2>/dev/null | head -1 > /dev/null || true
    
    local response=$(echo "$request" | timeout 5 "$MCP_SERVER_BIN" 2>/dev/null | tail -1)
    
    if echo "$response" | grep -q "tools"; then
        log_success "MCP工具列表功能正常"
        local tool_count=$(echo "$response" | jq -r '.result.tools | length' 2>/dev/null || echo "0")
        log_info "找到 $tool_count 个MCP工具"
        echo "$response" | jq '.result.tools[]?.name' 2>/dev/null || echo "$response"
        return 0
    else
        log_warning "MCP工具列表功能可能异常"
        echo "$response"
        return 1
    fi
}

# 打开浏览器验证UI
open_ui_browser() {
    log_info "打开浏览器验证UI..."
    
    if command -v open &> /dev/null; then
        # macOS
        open "$FRONTEND_URL" 2>/dev/null && log_success "已打开浏览器: $FRONTEND_URL" || log_warning "无法打开浏览器"
    elif command -v xdg-open &> /dev/null; then
        # Linux
        xdg-open "$FRONTEND_URL" 2>/dev/null && log_success "已打开浏览器: $FRONTEND_URL" || log_warning "无法打开浏览器"
    else
        log_warning "无法自动打开浏览器，请手动访问: $FRONTEND_URL"
    fi
}

# 使用Playwright验证UI（如果可用）
test_ui_playwright() {
    log_info "使用Playwright验证UI..."
    
    if [ -f "scripts/verify_ui_playwright.js" ]; then
        if command -v node &> /dev/null; then
            log_info "运行Playwright验证脚本..."
            node scripts/verify_ui_playwright.js 2>&1 | head -50
            return $?
        else
            log_warning "Node.js未安装，跳过Playwright验证"
            return 1
        fi
    else
        log_warning "Playwright验证脚本不存在，跳过"
        return 1
    fi
}

# 验证API端点
test_api_endpoints() {
    log_info "验证API端点..."
    
    local endpoints=(
        "/health"
        "/api/v1/agents"
        "/api/v1/memories?page=0&limit=5"
    )
    
    local passed=0
    local failed=0
    
    for endpoint in "${endpoints[@]}"; do
        local url="$BACKEND_URL$endpoint"
        if curl -s -f "$url" > /dev/null 2>&1; then
            log_success "API端点正常: $endpoint"
            passed=$((passed + 1))
        else
            log_error "API端点异常: $endpoint"
            failed=$((failed + 1))
        fi
    done
    
    log_info "API端点验证: $passed 通过, $failed 失败"
    return $failed
}

# 主验证流程
main() {
    echo "=========================================="
    echo "AgentMem 完整集成验证"
    echo "=========================================="
    echo ""
    
    # 检查必要命令
    log_info "检查必要命令..."
    check_command "curl" || exit 1
    check_command "jq" || log_warning "jq未安装，JSON输出可能不美观"
    check_command "just" || log_warning "just未安装，请手动启动服务"
    
    # 检查MCP服务器二进制
    if [ ! -f "$MCP_SERVER_BIN" ]; then
        log_warning "MCP服务器二进制不存在: $MCP_SERVER_BIN"
        log_info "正在构建MCP服务器..."
        if command -v cargo &> /dev/null; then
            cargo build --package mcp-stdio-server --release --bin agentmem-mcp-client || {
                log_error "MCP服务器构建失败"
                exit 1
            }
        else
            log_error "cargo未安装，无法构建MCP服务器"
            exit 1
        fi
    fi
    
    # 检查服务是否运行
    log_info "检查服务状态..."
    if ! curl -s -f "$BACKEND_URL/health" > /dev/null 2>&1; then
        log_warning "后端服务未运行，请先启动服务:"
        log_info "  just start-server    # 启动后端"
        log_info "  just start-ui        # 启动前端"
        log_info "  just start-full      # 启动全栈"
        echo ""
        read -p "是否现在启动服务? (y/n) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            log_info "启动全栈服务..."
            just start-full &
            sleep 10
            wait_for_service "$BACKEND_URL/health" || {
                log_error "服务启动失败"
                exit 1
            }
        else
            log_warning "跳过服务启动，继续验证MCP功能（可能部分功能无法验证）"
        fi
    else
        log_success "后端服务已运行"
    fi
    
    # 验证后端健康
    if curl -s -f "$BACKEND_URL/health" > /dev/null 2>&1; then
        check_backend_health
        test_api_endpoints
    fi
    
    echo ""
    echo "=========================================="
    echo "MCP功能验证"
    echo "=========================================="
    echo ""
    
    # MCP工具列表验证
    test_mcp_list_tools
    echo ""
    
    # MCP工具功能验证
    test_mcp_add_memory
    echo ""
    
    test_mcp_search_memories
    echo ""
    
    test_mcp_list_agents
    echo ""
    
    test_mcp_chat
    echo ""
    
    # UI验证
    echo "=========================================="
    echo "UI验证"
    echo "=========================================="
    echo ""
    
    if curl -s -f "$FRONTEND_URL" > /dev/null 2>&1; then
        log_success "前端服务已运行"
        
        # 打开浏览器
        open_ui_browser
        sleep 2
        
        # Playwright验证（如果可用）
        test_ui_playwright || log_warning "Playwright验证跳过"
    else
        log_warning "前端服务未运行，跳过UI验证"
        log_info "请运行: just start-ui"
    fi
    
    echo ""
    echo "=========================================="
    echo "验证总结"
    echo "=========================================="
    echo ""
    log_success "完整集成验证完成"
    echo ""
    log_info "服务地址:"
    echo "  后端API: $BACKEND_URL"
    echo "  前端UI:  $FRONTEND_URL"
    echo ""
    log_info "MCP服务器: $MCP_SERVER_BIN"
    echo ""
    log_info "下一步:"
    echo "  1. 在浏览器中验证UI功能"
    echo "  2. 通过MCP客户端测试工具功能"
    echo "  3. 查看日志确认所有功能正常"
}

# 运行主函数
main "$@"




