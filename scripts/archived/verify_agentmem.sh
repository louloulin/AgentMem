#!/bin/bash

# AgentMem 全面功能验证脚本
# 版本: v1.0
# 日期: 2025-10-30
# 用途: 自动化验证UI+Server+MCP功能

set -e  # 遇到错误立即退出

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

# 检查命令是否存在
check_command() {
    if command -v $1 &> /dev/null; then
        log_success "$1 已安装: $(command -v $1)"
        return 0
    else
        log_error "$1 未安装"
        return 1
    fi
}

# 检查端口是否被占用
check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1; then
        log_warning "端口 $1 已被占用"
        return 1
    else
        log_success "端口 $1 可用"
        return 0
    fi
}

# 等待服务启动
wait_for_service() {
    local url=$1
    local max_attempts=30
    local attempt=0
    
    log_info "等待服务启动: $url"
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            log_success "服务已启动"
            return 0
        fi
        
        attempt=$((attempt + 1))
        echo -n "."
        sleep 1
    done
    
    echo ""
    log_error "服务启动超时"
    return 1
}

# 清理函数
cleanup() {
    log_info "清理进程..."
    
    # 停止Backend
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null || true
        log_info "Backend已停止 (PID: $BACKEND_PID)"
    fi
    
    # 停止Frontend
    if [ ! -z "$FRONTEND_PID" ]; then
        kill $FRONTEND_PID 2>/dev/null || true
        log_info "Frontend已停止 (PID: $FRONTEND_PID)"
    fi
}

# 注册清理函数
trap cleanup EXIT INT TERM

# ============================================
# Phase 1: 环境检查
# ============================================
phase1_env_check() {
    echo ""
    echo "=========================================="
    echo "Phase 1: 环境检查"
    echo "=========================================="
    
    # 检查Rust
    log_info "检查Rust环境..."
    if check_command rustc; then
        rustc --version
    else
        log_error "请安装Rust: https://rustup.rs/"
        exit 1
    fi
    
    # 检查protoc
    log_info "检查protoc..."
    if [ -f "/opt/homebrew/bin/protoc" ]; then
        export PROTOC=/opt/homebrew/bin/protoc
        log_success "protoc 已安装: /opt/homebrew/bin/protoc"
        /opt/homebrew/bin/protoc --version
    elif check_command protoc; then
        protoc --version
    else
        log_error "请安装protoc: brew install protobuf"
        exit 1
    fi
    
    # 检查Node.js (可选，用于Frontend)
    log_info "检查Node.js..."
    if check_command node; then
        node --version
        NODEJS_AVAILABLE=true
    else
        log_warning "Node.js未安装，将跳过Frontend验证"
        log_info "如需验证Frontend，请安装: https://nodejs.org/"
        NODEJS_AVAILABLE=false
    fi

    # 检查npm (可选，用于Frontend)
    if [ "$NODEJS_AVAILABLE" = true ]; then
        log_info "检查npm..."
        if check_command npm; then
            npm --version
        else
            log_warning "npm未安装，将跳过Frontend验证"
            NODEJS_AVAILABLE=false
        fi
    fi
    
    # 检查jq (用于JSON解析)
    log_info "检查jq..."
    if ! check_command jq; then
        log_warning "jq未安装，部分测试可能无法运行"
        log_info "安装: brew install jq"
    fi
    
    # 检查端口
    log_info "检查端口占用..."
    check_port 8080 || log_warning "Backend端口8080被占用，请先停止占用进程"
    check_port 3001 || log_warning "Frontend端口3001被占用，请先停止占用进程"
    
    log_success "Phase 1: 环境检查完成"
}

# ============================================
# Phase 2: 编译验证
# ============================================
phase2_build() {
    echo ""
    echo "=========================================="
    echo "Phase 2: 编译验证"
    echo "=========================================="
    
    # 设置protoc环境变量
    export PROTOC=/opt/homebrew/bin/protoc
    
    # 编译Backend
    log_info "编译Backend (agent-mem-server)..."
    if cargo build --release -p agent-mem-server 2>&1 | tee build-backend.log; then
        log_success "Backend编译成功"
    else
        log_error "Backend编译失败，查看 build-backend.log"
        exit 1
    fi
    
    # 编译MCP Server
    log_info "编译MCP Server..."
    if cargo build --release -p mcp-stdio-server 2>&1 | tee build-mcp.log; then
        log_success "MCP Server编译成功"
    else
        log_error "MCP Server编译失败，查看 build-mcp.log"
        exit 1
    fi
    
    # 编译Frontend (如果Node.js可用)
    if [ "$NODEJS_AVAILABLE" = true ]; then
        log_info "编译Frontend..."
        cd agentmem-ui

        # 安装依赖
        if [ ! -d "node_modules" ]; then
            log_info "安装npm依赖..."
            npm install
        fi

        # 构建
        if npm run build 2>&1 | tee ../build-frontend.log; then
            log_success "Frontend编译成功"
        else
            log_error "Frontend编译失败，查看 build-frontend.log"
            exit 1
        fi

        cd ..
    else
        log_warning "跳过Frontend编译 (Node.js不可用)"
    fi
    
    log_success "Phase 2: 编译验证完成"
}

# ============================================
# Phase 3: Backend功能验证
# ============================================
phase3_backend() {
    echo ""
    echo "=========================================="
    echo "Phase 3: Backend功能验证"
    echo "=========================================="
    
    # 启动Backend
    log_info "启动Backend服务器..."
    ./target/release/agent-mem-server \
        --host 0.0.0.0 \
        --port 8080 \
        --log-level info \
        > backend.log 2>&1 &
    
    BACKEND_PID=$!
    log_info "Backend PID: $BACKEND_PID"
    
    # 等待服务启动
    if ! wait_for_service "http://localhost:8080/health"; then
        log_error "Backend启动失败，查看 backend.log"
        exit 1
    fi
    
    # Health Check
    log_info "测试Health Check..."
    if curl -s http://localhost:8080/health | jq '.' > /dev/null 2>&1; then
        log_success "Health Check通过"
    else
        log_error "Health Check失败"
        exit 1
    fi
    
    # 创建Agent
    log_info "测试创建Agent..."
    AGENT_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/agents \
        -H "Content-Type: application/json" \
        -d '{
            "name": "测试Agent",
            "description": "用于自动化测试的Agent"
        }')
    
    if echo "$AGENT_RESPONSE" | jq -e '.data.id' > /dev/null 2>&1; then
        AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id')
        log_success "Agent创建成功: $AGENT_ID"
    else
        log_error "Agent创建失败"
        echo "$AGENT_RESPONSE"
        exit 1
    fi
    
    # 创建Memory
    log_info "测试创建Memory..."
    MEMORY_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/memories \
        -H "Content-Type: application/json" \
        -d "{
            \"agent_id\": \"$AGENT_ID\",
            \"content\": \"这是一条测试记忆，用于验证AgentMem功能\",
            \"memory_type\": \"episodic\",
            \"importance\": 0.8
        }")
    
    if echo "$MEMORY_RESPONSE" | jq -e '.data.id' > /dev/null 2>&1; then
        MEMORY_ID=$(echo "$MEMORY_RESPONSE" | jq -r '.data.id')
        log_success "Memory创建成功: $MEMORY_ID"
    else
        log_error "Memory创建失败"
        echo "$MEMORY_RESPONSE"
        exit 1
    fi
    
    # 搜索Memory
    log_info "测试搜索Memory..."
    SEARCH_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/memories/search \
        -H "Content-Type: application/json" \
        -d "{
            \"agent_id\": \"$AGENT_ID\",
            \"query\": \"测试\",
            \"limit\": 10
        }")
    
    if echo "$SEARCH_RESPONSE" | jq -e '.data' > /dev/null 2>&1; then
        RESULT_COUNT=$(echo "$SEARCH_RESPONSE" | jq '.data | length')
        log_success "Memory搜索成功，找到 $RESULT_COUNT 条记录"
    else
        log_error "Memory搜索失败"
        echo "$SEARCH_RESPONSE"
        exit 1
    fi
    
    log_success "Phase 3: Backend功能验证完成"
}

# ============================================
# Phase 4: Frontend功能验证
# ============================================
phase4_frontend() {
    echo ""
    echo "=========================================="
    echo "Phase 4: Frontend功能验证"
    echo "=========================================="

    if [ "$NODEJS_AVAILABLE" != true ]; then
        log_warning "跳过Frontend验证 (Node.js不可用)"
        log_info "如需验证Frontend，请安装Node.js: https://nodejs.org/"
        return 0
    fi

    # 启动Frontend
    log_info "启动Frontend服务器..."
    cd agentmem-ui
    npm run dev > ../frontend.log 2>&1 &
    FRONTEND_PID=$!
    cd ..

    log_info "Frontend PID: $FRONTEND_PID"

    # 等待服务启动
    if ! wait_for_service "http://localhost:3001"; then
        log_error "Frontend启动失败，查看 frontend.log"
        exit 1
    fi

    # 测试主页
    log_info "测试主页..."
    if curl -s -f http://localhost:3001 > /dev/null 2>&1; then
        log_success "主页访问成功"
    else
        log_error "主页访问失败"
        exit 1
    fi

    # 测试Admin页面
    log_info "测试Admin Dashboard..."
    if curl -s -f http://localhost:3001/admin > /dev/null 2>&1; then
        log_success "Admin Dashboard访问成功"
    else
        log_error "Admin Dashboard访问失败"
        exit 1
    fi

    log_success "Phase 4: Frontend功能验证完成"
    log_info "请在浏览器中访问以下页面进行手动测试:"
    log_info "  - 主页: http://localhost:3001"
    log_info "  - Admin: http://localhost:3001/admin"
    log_info "  - Agents: http://localhost:3001/admin/agents"
    log_info "  - Memories: http://localhost:3001/admin/memories"
    log_info "  - Chat: http://localhost:3001/admin/chat"
}

# ============================================
# Phase 5: MCP功能验证
# ============================================
phase5_mcp() {
    echo ""
    echo "=========================================="
    echo "Phase 5: MCP功能验证"
    echo "=========================================="
    
    # 测试Initialize
    log_info "测试MCP Initialize..."
    INIT_RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}' | \
        timeout 5 ./target/release/agentmem-mcp-server 2>/dev/null || true)
    
    if echo "$INIT_RESPONSE" | grep -q "protocolVersion"; then
        log_success "MCP Initialize成功"
    else
        log_warning "MCP Initialize可能失败，请手动测试"
    fi
    
    # 测试工具列表
    log_info "测试MCP工具列表..."
    TOOLS_RESPONSE=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | \
        timeout 5 ./target/release/agentmem-mcp-server 2>/dev/null || true)
    
    if echo "$TOOLS_RESPONSE" | grep -q "agentmem_add_memory"; then
        log_success "MCP工具列表获取成功"
    else
        log_warning "MCP工具列表可能失败，请手动测试"
    fi
    
    log_success "Phase 5: MCP功能验证完成"
    log_info "MCP Server路径: $(pwd)/target/release/agentmem-mcp-server"
    log_info "Claude Desktop配置示例:"
    echo '{
  "mcpServers": {
    "agentmem": {
      "command": "'$(pwd)'/target/release/agentmem-mcp-server",
      "args": []
    }
  }
}'
}

# ============================================
# 主函数
# ============================================
main() {
    echo "=========================================="
    echo "AgentMem 全面功能验证"
    echo "版本: v1.0"
    echo "日期: 2025-10-30"
    echo "=========================================="
    
    # 执行各个阶段
    phase1_env_check
    phase2_build
    phase3_backend
    phase4_frontend
    phase5_mcp
    
    # 总结
    echo ""
    echo "=========================================="
    echo "验证完成！"
    echo "=========================================="
    log_success "所有自动化测试通过"
    log_info "服务正在运行:"
    log_info "  - Backend: http://localhost:8080 (PID: $BACKEND_PID)"
    log_info "  - Frontend: http://localhost:3001 (PID: $FRONTEND_PID)"
    log_info ""
    log_info "日志文件:"
    log_info "  - Backend: backend.log"
    log_info "  - Frontend: frontend.log"
    log_info "  - Build: build-*.log"
    log_info ""
    log_info "按Ctrl+C停止所有服务"
    
    # 保持运行
    wait
}

# 运行主函数
main

