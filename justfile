# AgentMem Justfile
# 统一管理项目的构建、测试、部署等任务
# 使用方法: just <command>
# 查看所有命令: just --list

# ============================================================================
# 全局配置
# ============================================================================

# 默认配置
export RUST_BACKTRACE := "1"
export PROJECT_ROOT := justfile_directory()
export DYLD_LIBRARY_PATH := PROJECT_ROOT + "/lib:" + PROJECT_ROOT + "/target/release"
export ORT_DYLIB_PATH := PROJECT_ROOT + "/lib/libonnxruntime.1.22.0.dylib"

# LLM 配置
# 注意: API Key 应该通过环境变量设置，不要硬编码
# 使用方式: export ZHIPU_API_KEY="your-key" && just <command>
# 或者创建 .env 文件: echo "ZHIPU_API_KEY=your-key" > .env
# 默认值仅用于开发环境，生产环境必须通过环境变量设置
export LLM_PROVIDER := "zhipu"
export LLM_MODEL := "glm-4.6"
# ZHIPU_API_KEY 必须通过环境变量设置，不再硬编码
# 如果未设置，相关功能将无法使用
# 设置方式: export ZHIPU_API_KEY="your-key"

# Embedder 配置
export EMBEDDER_PROVIDER := "fastembed"
export EMBEDDER_MODEL := "BAAI/bge-small-en-v1.5"

# 服务器配置
export SERVER_PORT := "8080"
export UI_PORT := "3001"
export BACKEND_BINARY := PROJECT_ROOT + "/target/release/agent-mem-server"
export BACKEND_BINARY_DEBUG := PROJECT_ROOT + "/target/debug/agent-mem-server"
export BACKEND_LOG := PROJECT_ROOT + "/backend.log"
export FRONTEND_LOG := PROJECT_ROOT + "/frontend.log"
export BACKEND_PID := PROJECT_ROOT + "/backend.pid"
export FRONTEND_PID := PROJECT_ROOT + "/frontend.pid"

# 认证配置（开发环境默认关闭）
export ENABLE_AUTH := "false"
export SERVER_ENABLE_AUTH := "false"
export AGENT_MEM_ENABLE_AUTH := "false"

# 默认任务：显示帮助
default:
    @just --list

# ============================================================================
# 构建相关
# ============================================================================

# 构建所有项目（debug 模式）
build:
    @echo "🔨 构建 AgentMem (debug 模式)..."
    cargo build

# 构建所有项目（release 模式）
build-release:
    @echo "🔨 构建 AgentMem (release 模式)..."
    cargo build --release

# 构建 HTTP API 服务器
build-server:
    @echo "🔨 构建 HTTP API 服务器..."
    cargo build --package agent-mem-server --release

# 构建 MCP Stdio 服务器
build-mcp:
    @echo "🔨 构建 MCP Stdio 服务器..."
    cargo build --package mcp-stdio-server --release

# 构建前端 UI
build-ui:
    @echo "🔨 构建前端 UI..."
    cd agentmem-ui && npm install && npm run build

# 清理构建产物
clean:
    @echo "🧹 清理构建产物..."
    cargo clean
    rm -rf agentmem-ui/node_modules agentmem-ui/.next
    @just clean-logs
    @just clean-pids

# ============================================================================
# 测试相关
# ============================================================================

# 运行所有测试
test:
    @echo "🧪 运行所有测试..."
    cargo test --workspace

# 运行特定包的测试
test-package package:
    @echo "🧪 运行 {{package}} 测试..."
    cargo test --package {{package}}

# 运行集成测试
test-integration:
    @echo "🧪 运行集成测试..."
    cargo test --test '*' --workspace

# 运行 Working Memory 测试
test-working-memory:
    @echo "🧪 运行 Working Memory 测试..."
    cargo test --package agent-mem-core working_memory

# 运行 MCP 功能测试
test-mcp:
    @echo "🧪 测试 MCP 服务器..."
    cd examples/mcp-stdio-server && bash test_server.sh

# 运行性能基准测试
bench:
    @echo "📊 运行性能基准测试..."
    cargo bench --workspace

# ============================================================================
# 代码质量
# ============================================================================

# 运行 clippy 检查
clippy:
    @echo "🔍 运行 Clippy 检查..."
    cargo clippy --workspace --all-targets -- -D warnings

# 格式化代码
fmt:
    @echo "✨ 格式化代码..."
    cargo fmt --all

# 检查代码格式
fmt-check:
    @echo "🔍 检查代码格式..."
    cargo fmt --all -- --check

# 生成文档
doc:
    @echo "📚 生成文档..."
    cargo doc --workspace --no-deps --open

# 运行安全审计
audit:
    @echo "🔒 运行安全审计..."
    cargo audit

# ============================================================================
# 服务启动辅助函数（内部使用）
# ============================================================================

# 等待服务就绪（智能健康检查）
_wait-healthy url max_attempts="30":
    @env SHELLOPTS= /bin/bash -lc 'set +u; i=1; while [ $i -le {{max_attempts}} ]; do \
        if curl -s {{url}} > /dev/null 2>&1; then \
            echo "✅ 服务已就绪 (尝试 $i/{{max_attempts}})"; \
            exit 0; \
        fi; \
        echo "⏳ 等待服务启动... ($i/{{max_attempts}})"; \
        i=$((i + 1)); \
        sleep 1; \
    done; \
    echo "❌ 服务启动超时"; \
    exit 1'

# 停止现有服务进程
_stop-backend:
    @bash -c 'if pkill -f "agent-mem-server" 2>/dev/null; then \
        echo "🛑 停止现有后端服务..."; \
        sleep 2; \
    else \
        echo "ℹ️  后端服务未运行"; \
    fi'

_stop-frontend:
    @bash -c 'if pkill -f "next dev" 2>/dev/null; then \
        echo "🛑 停止现有前端服务..."; \
        sleep 2; \
    else \
        echo "ℹ️  前端服务未运行"; \
    fi'

# ============================================================================
# 服务启动
# ============================================================================

# 启动 HTTP API 服务器（前台运行，无认证模式）
start-server:
    @echo "🚀 启动 HTTP API 服务器（前台运行）..."
    @bash -c 'if [ ! -f "./target/release/agent-mem-server" ]; then echo "❌ 二进制文件不存在: ./target/release/agent-mem-server"; exit 1; fi'
    @bash -c 'if lsof -i :8080 > /dev/null 2>&1; then echo "⚠️  端口 8080 已被占用"; exit 1; fi'
    @just _stop-backend
    @bash -c 'export ENABLE_AUTH="false" && \
    export SERVER_ENABLE_AUTH="false" && \
    export AGENT_MEM_ENABLE_AUTH="false" && \
    export EMBEDDER_PROVIDER="fastembed" && \
    export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5" && \
    export DYLD_LIBRARY_PATH="$(pwd)/lib:$(pwd)/target/release:$$DYLD_LIBRARY_PATH" && \
    export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib" && \
    ./target/release/agent-mem-server'

# 启动 HTTP API 服务器（后台运行，无认证模式）
start-server-bg:
    @echo "🚀 启动 HTTP API 服务器（后台运行）..."
    @bash -c 'if [ ! -f "./target/release/agent-mem-server" ]; then echo "❌ 二进制文件不存在: ./target/release/agent-mem-server"; exit 1; fi'
    @bash -c 'if lsof -i :8080 > /dev/null 2>&1; then echo "⚠️  端口 8080 已被占用"; exit 1; fi'
    @just _stop-backend
    @bash -c 'export ENABLE_AUTH="false" && \
    export SERVER_ENABLE_AUTH="false" && \
    export AGENT_MEM_ENABLE_AUTH="false" && \
    export EMBEDDER_PROVIDER="fastembed" && \
    export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5" && \
    export DYLD_LIBRARY_PATH="$(pwd)/lib:$(pwd)/target/release:$$DYLD_LIBRARY_PATH" && \
    export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib" && \
    nohup ./target/release/agent-mem-server > backend.log 2>&1 & \
    PID=$! && echo $PID > backend.pid && \
    echo "📝 后端 PID: $PID" && \
    echo "📝 日志文件: backend.log"'
    @just _wait-healthy "http://localhost:8080/health"
    @echo "✅ 后端服务已启动"
    @echo "   • API: http://localhost:8080"
    @echo "   • 健康检查: http://localhost:8080/health"
    @echo "   • API文档: http://localhost:8080/swagger-ui/"

# 启动 HTTP API 服务器（带插件支持，后台运行）
start-server-plugins:
    @echo "🚀 启动 HTTP API 服务器（插件支持，后台运行）..."
    @echo "1️⃣  编译带插件的服务器..."
    @cargo build --release --bin agent-mem-server --features agent-mem/plugins
    @bash -c 'if lsof -i :8080 > /dev/null 2>&1; then echo "⚠️  端口 8080 已被占用"; exit 1; fi'
    @just _stop-backend
    @bash -c 'export ENABLE_AUTH="false" && \
    export SERVER_ENABLE_AUTH="false" && \
    export AGENT_MEM_ENABLE_AUTH="false" && \
    export EMBEDDER_PROVIDER="fastembed" && \
    export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5" && \
    export DYLD_LIBRARY_PATH="$(pwd)/lib:$(pwd)/target/release:$$DYLD_LIBRARY_PATH" && \
    export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib" && \
    nohup ./target/release/agent-mem-server > backend.log 2>&1 & \
    PID=$$! && echo $$PID > backend.pid && \
    echo "📝 后端 PID: $$PID"'
    @just _wait-healthy "http://localhost:8080/health"
    @echo "✅ 后端服务已启动（插件支持）"
    @echo "   • 插件API: http://localhost:8080/api/v1/plugins"

# 启动 HTTP API 服务器（带 LumosAI 功能，后台运行）
start-server-lumosai:
    @echo "🚀 启动 HTTP API 服务器（LumosAI 功能，后台运行）..."
    @echo "1️⃣  编译带 lumosai feature 的服务器..."
    @cargo build --bin agent-mem-server --features lumosai
    @bash -c 'if lsof -i :8080 > /dev/null 2>&1; then echo "⚠️  端口 8080 已被占用"; exit 1; fi'
    @just _stop-backend
    @bash -c 'export ENABLE_AUTH="false" && \
    export SERVER_ENABLE_AUTH="false" && \
    export AGENT_MEM_ENABLE_AUTH="false" && \
    export EMBEDDER_PROVIDER="fastembed" && \
    export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5" && \
    export DYLD_LIBRARY_PATH="$(pwd)/lib:$(pwd)/target/debug:$$DYLD_LIBRARY_PATH" && \
    export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib" && \
    nohup ./target/debug/agent-mem-server > backend.log 2>&1 & \
    echo $$! > backend.pid && \
    echo "📝 后端 PID: $$(cat backend.pid)"'
    @just _wait-healthy "http://localhost:8080/health"
    @echo "✅ 后端服务已启动（LumosAI 已启用）"

# 启动 MCP Stdio 服务器
start-mcp:
    @echo "🚀 启动 MCP Stdio 服务器..."
    @bash -c 'if [ ! -f "./target/release/agentmem-mcp-server" ]; then echo "❌ 二进制文件不存在: ./target/release/agentmem-mcp-server"; exit 1; fi'
    @./target/release/agentmem-mcp-server

# 启动前端 UI（前台运行）
start-ui:
    @echo "🚀 启动前端 UI（前台运行）..."
    @bash -c 'if lsof -i :3001 > /dev/null 2>&1; then echo "⚠️  端口 3001 已被占用"; exit 1; fi'
    @just _stop-frontend
    @cd agentmem-ui && npm run dev

# 启动前端 UI（后台运行）
start-ui-bg:
    @echo "🚀 启动前端 UI（后台运行）..."
    @bash -c 'if lsof -i :3001 > /dev/null 2>&1; then echo "⚠️  端口 3001 已被占用"; exit 1; fi'
    @just _stop-frontend
    @bash -c 'cd agentmem-ui && \
    if [ ! -d "node_modules" ]; then \
        echo "📦 安装前端依赖..."; \
        npm install; \
    fi && \
    nohup npm run dev > ../frontend.log 2>&1 & \
    PID=$! && echo $PID > ../frontend.pid && \
    echo "📝 前端 PID: $PID" && \
    echo "📝 日志文件: frontend.log"'
    @just _wait-healthy "http://localhost:3001"
    @echo "✅ 前端服务已启动"
    @echo "   • Web UI: http://localhost:3001"

# 启动全栈（后端 + 前端，后台运行）
start-full:
    @echo "🚀 启动全栈服务（后端 + 前端）..."
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "1️⃣  启动后端服务..."
    @just start-server-bg
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "2️⃣  启动前端服务..."
    @just start-ui-bg
    @echo ""
    @echo "╔════════════════════════════════════════════════════════╗"
    @echo "║  ✅ AgentMem 全栈服务已启动                           ║"
    @echo "╠════════════════════════════════════════════════════════╣"
    @echo "║  🔹 后端API: http://localhost:8080                    ║"
    @echo "║  🔹 前端UI:  http://localhost:3001                    ║"
    @echo "║  🔹 健康检查: http://localhost:8080/health            ║"
    @echo "║  🔹 API文档: http://localhost:8080/swagger-ui/        ║"
    @echo "║  🔹 Embedder: FastEmbed (BAAI/bge-small-en-v1.5)      ║"
    @echo "╚════════════════════════════════════════════════════════╝"
    @echo ""
    @echo "📝 日志文件:"
    @echo "   • 后端: tail -f backend.log"
    @echo "   • 前端: tail -f frontend.log"
    @echo ""
    @echo "🛑 停止服务: just stop"
    @echo ""
    @echo "💡 提示: 也可以使用 start_full_stack.sh 脚本启动"

# 启动全栈（带插件支持）
start-full-plugins:
    @echo "🚀 启动全栈服务（插件支持）..."
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "1️⃣  启动后端服务（插件支持）..."
    @just start-server-plugins
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "2️⃣  启动前端服务..."
    @just start-ui-bg
    @echo ""
    @echo "╔════════════════════════════════════════════════════════╗"
    @echo "║  ✅ AgentMem 全栈服务已启动（插件支持）               ║"
    @echo "╠════════════════════════════════════════════════════════╣"
    @echo "║  🔹 后端API: http://localhost:8080                    ║"
    @echo "║  🔹 前端UI:  http://localhost:3001                    ║"
    @echo "║  🔹 健康检查: http://localhost:8080/health            ║"
    @echo "║  🔹 插件API: http://localhost:8080/api/v1/plugins     ║"
    @echo "║  🔹 API文档: http://localhost:8080/swagger-ui/        ║"
    @echo "║  🔹 Embedder: FastEmbed (BAAI/bge-small-en-v1.5)      ║"
    @echo "╚════════════════════════════════════════════════════════╝"

# 停止所有服务
stop:
    @echo "🛑 停止所有服务..."
    @just _stop-backend
    @just _stop-frontend
    @bash -c 'pkill -f "agentmem-mcp-server" 2>/dev/null && echo "🛑 停止 MCP 服务器" || true'
    @bash -c 'rm -f backend.pid frontend.pid 2>/dev/null && echo "🧹 清理 PID 文件" || true'
    @echo "✅ 所有服务已停止"

# 重启所有服务
restart:
    @echo "🔄 重启所有服务..."
    @just stop
    @sleep 2
    @just start-full

# ============================================================================
# 数据库管理
# ============================================================================

# 初始化数据库
db-init:
    @echo "🗄️  初始化数据库..."
    @bash scripts/init_db.sh

# 运行数据库迁移
db-migrate:
    @echo "🗄️  运行数据库迁移..."
    @sqlx migrate run

# 备份数据库
db-backup:
    @echo "💾 备份数据库..."
    @bash scripts/backup.sh

# 恢复数据库
db-restore:
    @echo "♻️  恢复数据库..."
    @bash scripts/restore.sh

# ============================================================================
# MCP 相关
# ============================================================================

# 验证 MCP 工具功能
mcp-verify:
    @echo "🔍 验证 MCP 工具功能..."
    @bash -c 'if [ ! -f "./target/release/agentmem-mcp-server" ]; then \
        echo "❌ MCP 服务器未编译，正在编译..."; \
        just build-mcp; \
    fi'
    @echo "运行 MCP 测试..."
    @just mcp-test-chat

# 测试 MCP Chat 功能并验证 Working Memory
mcp-test-chat:
    @echo "💬 测试 MCP Chat 功能..."
    @echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/agentmem-mcp-server 2>/dev/null | head -1
    @echo ""
    @echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_chat","arguments":{"message":"你好，请介绍一下AgentMem","user_id":"test-user","session_id":"test-session-001","use_memory":true}}}' | ./target/release/agentmem-mcp-server 2>/dev/null | tail -1

# 配置 Claude Desktop
mcp-setup-claude:
    @echo "⚙️  配置 Claude Desktop..."
    @echo "配置文件位置: ~/Library/Application Support/Claude/claude_desktop_config.json"
    @cat examples/mcp-stdio-server/claude_desktop_config.json

# ============================================================================
# 开发工具
# ============================================================================

# 监听文件变化并自动重新编译
watch:
    @echo "👀 监听文件变化..."
    cargo watch -x build

# 监听并运行测试
watch-test:
    @echo "👀 监听并运行测试..."
    cargo watch -x test

# 运行示例程序
run-example example:
    @echo "🎯 运行示例: {{example}}"
    cargo run --example {{example}}

# 检查项目健康状态
health:
    @echo "🏥 检查项目健康状态..."
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "后端服务 (http://localhost:8080):"
    @bash -c 'if curl -s http://localhost:8080/health > /dev/null 2>&1; then \
        echo "✅ 后端运行中"; \
        curl -s http://localhost:8080/health | jq . 2>/dev/null || curl -s http://localhost:8080/health; \
        if [ -f backend.pid ]; then \
            echo "   PID: $$(cat backend.pid)"; \
        fi; \
    else \
        echo "❌ 后端未运行"; \
    fi'
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "前端服务 (http://localhost:3001):"
    @bash -c 'if curl -s http://localhost:3001 > /dev/null 2>&1; then \
        echo "✅ 前端运行中"; \
        if [ -f frontend.pid ]; then \
            echo "   PID: $$(cat frontend.pid)"; \
        fi; \
    else \
        echo "❌ 前端未运行"; \
    fi'
    @echo ""

# 查看实时日志
logs service="backend":
    @echo "📝 查看 {{service}} 日志..."
    @bash -c 'if [ "{{service}}" = "backend" ]; then \
        if [ -f backend.log ]; then \
            tail -f backend.log; \
        else \
            echo "❌ 日志文件不存在: backend.log"; \
            echo "💡 提示: 使用 just start-server-bg 启动后台服务"; \
        fi; \
    elif [ "{{service}}" = "frontend" ]; then \
        if [ -f frontend.log ]; then \
            tail -f frontend.log; \
        else \
            echo "❌ 日志文件不存在: frontend.log"; \
            echo "💡 提示: 使用 just start-ui-bg 启动后台前端服务"; \
        fi; \
    elif [ "{{service}}" = "all" ]; then \
        echo "📝 查看所有日志 (按 Ctrl+C 退出)..."; \
        tail -f backend.log frontend.log 2>/dev/null || echo "❌ 日志文件不存在"; \
    else \
        echo "❌ 未知服务: {{service}}"; \
        echo "💡 可用选项: backend, frontend, all"; \
    fi'

# 查看服务状态
status:
    @echo "📊 服务状态"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @bash -c 'echo "后端服务:"; \
    if [ -f backend.pid ]; then \
        BACKEND_PID=$(cat backend.pid 2>/dev/null); \
        if [ -n "$BACKEND_PID" ] && kill -0 "$BACKEND_PID" 2>/dev/null; then \
            echo "  ✅ 运行中 (PID: $BACKEND_PID)"; \
            echo "  📝 日志: backend.log"; \
        else \
            echo "  ⚠️  PID 文件存在但进程未运行"; \
        fi; \
    elif lsof -i :8080 > /dev/null 2>&1; then \
        echo "  ⚠️  端口被占用但 PID 文件不存在"; \
    else \
        echo "  ❌ 未运行"; \
    fi; \
    echo ""; \
    echo "前端服务:"; \
    if [ -f frontend.pid ]; then \
        FRONTEND_PID=$(cat frontend.pid 2>/dev/null); \
        if [ -n "$FRONTEND_PID" ] && kill -0 "$FRONTEND_PID" 2>/dev/null; then \
            echo "  ✅ 运行中 (PID: $FRONTEND_PID)"; \
            echo "  📝 日志: frontend.log"; \
        else \
            echo "  ⚠️  PID 文件存在但进程未运行"; \
        fi; \
    elif lsof -i :3001 > /dev/null 2>&1; then \
        echo "  ⚠️  端口被占用但 PID 文件不存在"; \
    else \
        echo "  ❌ 未运行"; \
    fi'

# ============================================================================
# 部署相关
# ============================================================================

# 构建 Docker 镜像
docker-build:
    @echo "🐳 构建 Docker 镜像..."
    docker build -t agentmem:latest .

# 启动 Docker Compose
docker-up:
    @echo "🐳 启动 Docker Compose..."
    docker-compose up -d

# 停止 Docker Compose
docker-down:
    @echo "🐳 停止 Docker Compose..."
    docker-compose down

# 构建生产版本
build-prod:
    @echo "🏭 构建生产版本..."
    cargo build --release --workspace
    cd agentmem-ui && npm run build

# 部署到生产环境
deploy-prod:
    @echo "🚀 部署到生产环境..."
    @echo "⚠️  请确保已配置生产环境变量"
    @just build-prod
    @echo "✅ 构建完成，请手动部署到服务器"

# ============================================================================
# 快捷命令
# ============================================================================

# 快速开始：构建并启动所有服务
quick-start: build-release
    @echo "⚡ 快速启动..."
    @just start-full

# 完整验证：构建、测试、启动
verify: build-release test
    @echo "✅ 验证完成"
    @just health

# 开发模式：构建并启动（带热重载）
dev:
    @echo "🔧 开发模式..."
    @echo "⚠️  注意: 此命令将在前台运行，按 Ctrl+C 停止"
    @just watch &
    @just start-ui

# 清理并重新构建
rebuild: clean build-release
    @echo "✅ 重新构建完成"

# 一键启动：检查构建 -> 启动服务 -> 显示状态
go:
    @echo "🚀 AgentMem 一键启动"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @bash -c 'if [ ! -f "./target/release/agent-mem-server" ]; then \
        echo "📦 未找到编译后的服务器，开始构建..."; \
        just build-server; \
    else \
        echo "✅ 找到编译后的服务器"; \
    fi'
    @echo ""
    @just start-full
    @echo ""
    @just status

# 检查依赖和工具
check-deps:
    @echo "🔍 检查依赖和工具..."
    @echo ""
    @echo "必需工具:"
    @bash scripts/check_dependencies.sh 2>/dev/null || bash -c '\
    MISSING=0; \
    for cmd in rustc cargo just node npm; do \
        if command -v $$cmd > /dev/null 2>&1; then \
            VER=$$($$cmd --version 2>/dev/null | head -1); \
            echo "  ✅ $$cmd: $$VER"; \
        else \
            echo "  ❌ $$cmd: 未安装"; \
            MISSING=$$((MISSING + 1)); \
        fi; \
    done; \
    echo ""; \
    echo "可选工具:"; \
    for cmd in jq curl docker docker-compose; do \
        if command -v $$cmd > /dev/null 2>&1; then \
            VER=$$($$cmd --version 2>/dev/null | head -1); \
            echo "  ✅ $$cmd: $$VER"; \
        else \
            echo "  ⚠️  $$cmd: 未安装（可选）"; \
        fi; \
    done; \
    echo ""; \
    if [ $$MISSING -gt 0 ]; then \
        echo "❌ 缺少 $$MISSING 个必需工具，请先安装"; \
        exit 1; \
    else \
        echo "✅ 所有必需工具已安装"; \
    fi'

# 清理日志文件
clean-logs:
    @echo "🧹 清理日志文件..."
    @bash -c 'rm -f *.log backend.log frontend.log 2>/dev/null && echo "✅ 日志文件已清理" || echo "ℹ️  没有日志文件需要清理"'

# 清理 PID 文件
clean-pids:
    @echo "🧹 清理 PID 文件..."
    @bash -c 'rm -f *.pid backend.pid frontend.pid 2>/dev/null && echo "✅ PID 文件已清理" || echo "ℹ️  没有 PID 文件需要清理"'

# 完整清理：停止服务 + 清理文件
clean-all: stop
    @just clean-logs
    @just clean-pids
    @echo "✅ 完整清理完成"

# 查看最近的日志（最后N行）
tail-logs service="backend" lines="50":
    @echo "📝 查看 {{service}} 最近 {{lines}} 行日志..."
    @bash -c 'if [ "{{service}}" = "backend" ]; then \
        if [ -f backend.log ]; then \
            tail -n {{lines}} backend.log; \
        else \
            echo "❌ 日志文件不存在: backend.log"; \
        fi; \
    elif [ "{{service}}" = "frontend" ]; then \
        if [ -f frontend.log ]; then \
            tail -n {{lines}} frontend.log; \
        else \
            echo "❌ 日志文件不存在: frontend.log"; \
        fi; \
    else \
        echo "❌ 未知服务: {{service}}"; \
        echo "💡 可用选项: backend, frontend"; \
    fi'

# ============================================================================
# 信息查看
# ============================================================================

# 显示项目信息
info:
    @echo "📊 AgentMem 项目信息"
    @echo "===================="
    @bash -c 'VERSION=$(cargo pkgid 2>/dev/null | cut -d# -f2 || echo "workspace"); echo "版本: $VERSION"'
    @bash -c 'RUST_VER=$(rustc --version 2>/dev/null || echo "未安装"); echo "Rust 版本: $RUST_VER"'
    @bash -c 'CARGO_VER=$(cargo --version 2>/dev/null || echo "未安装"); echo "Cargo 版本: $CARGO_VER"'
    @echo ""
    @echo "服务地址:"
    @echo "  - 后端 API: http://localhost:8080"
    @echo "  - 前端 UI: http://localhost:3001"
    @echo "  - API 文档: http://localhost:8080/swagger-ui/"
    @echo ""
    @echo "数据库:"
    @echo "  - 主数据库: ./agentmem.db"
    @echo "  - 历史数据: ./data/history.db"

# 显示环境变量
env:
    @echo "🌍 环境变量"
    @echo "==========="
    @bash -c 'echo "RUST_BACKTRACE: ${RUST_BACKTRACE:-未设置}"; \
    echo "LLM_PROVIDER: ${LLM_PROVIDER:-未设置}"; \
    echo "LLM_MODEL: ${LLM_MODEL:-未设置}"; \
    echo "EMBEDDER_PROVIDER: ${EMBEDDER_PROVIDER:-未设置}"; \
    echo "EMBEDDER_MODEL: ${EMBEDDER_MODEL:-未设置}"; \
    echo "DYLD_LIBRARY_PATH: ${DYLD_LIBRARY_PATH:-未设置}"; \
    echo "ORT_DYLIB_PATH: ${ORT_DYLIB_PATH:-未设置}"'

# ============================================================================
# 演示相关（按照 x.md 演示计划）
# ============================================================================

# 演示环境变量
export DEMO_API_URL := "http://localhost:8080"
export DEMO_UI_URL := "http://localhost:3001"
export DEMO_USER_ID := "default"
export DEMO_ORG_ID := "default-org"

# 演示准备：清理并重置演示环境
demo-prepare:
    @echo "🧹 准备演示环境..."
    @echo "=========================================="
    @echo "1. 停止现有服务..."
    @just stop
    @echo ""
    @echo "2. 清理数据库（可选，谨慎使用）..."
    @echo "⚠️  跳过数据库清理（保留现有数据）"
    @echo ""
    @echo "3. 初始化数据库..."
    @just db-init
    @echo ""
    @echo "✅ 演示环境准备完成"

# 创建演示数据（30条记忆，按照 x.md 计划）
demo-create-data:
    @echo "📝 创建演示数据..."
    @echo "=========================================="
    @bash scripts/create_demo_memories_from_plan.sh
    @echo ""
    @echo "✅ 演示数据创建完成"

# 验证演示数据
demo-verify-data:
    @echo "🔍 验证演示数据..."
    @echo "=========================================="
    @echo "检查记忆数量..."
    @curl -s "$(DEMO_API_URL)/api/v1/memories?page=0&limit=1" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" | \
        jq -r '.data.memories | length as $count | "找到 \($count) 条记忆"' || echo "查询失败"
    @echo ""
    @echo "检查 Agent..."
    @curl -s "$(DEMO_API_URL)/api/v1/agents" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" | \
        jq -r '.data | length as $count | "找到 \($count) 个 Agent"' || echo "查询失败"

# 验证 UI 功能（按照 x.md 测试用例）
demo-verify-ui:
    @echo "🧪 验证 UI 功能..."
    @echo "=========================================="
    @bash scripts/verify_ui_functionality.sh

# 启动演示服务（后端 + 前端）
demo-start:
    @echo "🚀 启动演示服务..."
    @echo "=========================================="
    @just start-full
    @echo ""
    @echo "✅ 服务启动完成"
    @echo ""
    @echo "🌐 访问地址:"
    @echo "  前端 UI: $(DEMO_UI_URL)"
    @echo "  后端 API: $(DEMO_API_URL)"
    @echo "  API 文档: $(DEMO_API_URL)/swagger-ui/"

# 打开浏览器验证页面
demo-open-browser:
    @echo "🌐 打开浏览器验证页面..."
    @echo "=========================================="
    @echo "正在打开以下页面..."
    @open $(DEMO_UI_URL)/admin/memories || echo "请手动访问: $(DEMO_UI_URL)/admin/memories"
    @sleep 1
    @open $(DEMO_UI_URL)/admin/agents || echo "请手动访问: $(DEMO_UI_URL)/admin/agents"
    @sleep 1
    @open $(DEMO_UI_URL)/admin/graph || echo "请手动访问: $(DEMO_UI_URL)/admin/graph"
    @echo ""
    @echo "✅ 浏览器页面已打开"
    @echo ""
    @echo "📋 验证清单（按照 x.md 计划）:"
    @echo "  1. 搜索功能测试（7个测试用例）"
    @echo "     - 王总、张总、AI产品、融资、会议、技术相关的工作、陈副总"
    @echo "  2. 记忆类型过滤（Semantic、Episodic）"
    @echo "  3. 分页功能验证"
    @echo "  4. 记忆详情查看"

# 演示搜索测试（7个测试用例）
demo-test-search:
    @echo "🔍 演示搜索测试（7个测试用例）..."
    @echo "=========================================="
    @echo ""
    @echo "测试用例1: 基础信息检索 - '王总'"
    @curl -s -X POST "$(DEMO_API_URL)/api/v1/memories/search" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" \
        -d '{"query":"王总","limit":10}' | \
        jq -r '.data | length as $count | "✅ 找到 \($count) 条结果"' || echo "❌ 测试失败"
    @echo ""
    @echo "测试用例2: 关系网络查询 - '张总'"
    @curl -s -X POST "$(DEMO_API_URL)/api/v1/memories/search" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" \
        -d '{"query":"张总","limit":10}' | \
        jq -r '.data | length as $count | "✅ 找到 \($count) 条结果"' || echo "❌ 测试失败"
    @echo ""
    @echo "测试用例3: 项目状态查询 - 'AI产品'"
    @curl -s -X POST "$(DEMO_API_URL)/api/v1/memories/search" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" \
        -d '{"query":"AI产品","limit":10}' | \
        jq -r '.data | length as $count | "✅ 找到 \($count) 条结果"' || echo "❌ 测试失败"
    @echo ""
    @echo "测试用例4: 历史对话查询 - '融资'"
    @curl -s -X POST "$(DEMO_API_URL)/api/v1/memories/search" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" \
        -d '{"query":"融资","limit":10}' | \
        jq -r '.data | length as $count | "✅ 找到 \($count) 条结果"' || echo "❌ 测试失败"
    @echo ""
    @echo "测试用例5: 个性化建议 - '会议'"
    @curl -s -X POST "$(DEMO_API_URL)/api/v1/memories/search" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" \
        -d '{"query":"会议","limit":10}' | \
        jq -r '.data | length as $count | "✅ 找到 \($count) 条结果"' || echo "❌ 测试失败"
    @echo ""
    @echo "测试用例6: 语义搜索 - '技术相关的工作'"
    @curl -s -X POST "$(DEMO_API_URL)/api/v1/memories/search" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" \
        -d '{"query":"技术相关的工作","limit":10}' | \
        jq -r '.data | length as $count | "✅ 找到 \($count) 条结果"' || echo "❌ 测试失败"
    @echo ""
    @echo "测试用例7: 团队成员查询 - '陈副总'"
    @curl -s -X POST "$(DEMO_API_URL)/api/v1/memories/search" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" \
        -H "Content-Type: application/json" \
        -d '{"query":"陈副总","limit":10}' | \
        jq -r '.data | length as $count | "✅ 找到 \($count) 条结果"' || echo "❌ 测试失败"
    @echo ""
    @echo "✅ 搜索测试完成"

# 完整演示流程：准备 + 创建数据 + 启动服务 + 验证
demo-full:
    @echo "🎬 完整演示流程"
    @echo "=========================================="
    @echo ""
    @echo "步骤1: 构建项目..."
    @just build-release
    @echo ""
    @echo "步骤2: 启动服务..."
    @just demo-start
    @echo ""
    @echo "步骤3: 创建演示数据..."
    @just demo-create-data
    @echo ""
    @echo "步骤4: 验证数据..."
    @just demo-verify-data
    @echo ""
    @echo "步骤5: 验证 UI 功能..."
    @just demo-verify-ui
    @echo ""
    @echo "步骤6: 打开浏览器..."
    @just demo-open-browser
    @echo ""
    @echo "=========================================="
    @echo "✅ 演示准备完成！"
    @echo "=========================================="
    @echo ""
    @echo "📋 下一步："
    @echo "  1. 在浏览器中验证搜索功能（7个测试用例）"
    @echo "  2. 验证记忆类型过滤"
    @echo "  3. 验证分页功能"
    @echo "  4. 按照 x.md 计划进行演示"
    @echo ""
    @echo "📝 详细指南: docs/BROWSER_VERIFICATION_GUIDE.md"

# 快速演示：假设服务已运行，只创建数据和打开浏览器
demo-quick:
    @echo "⚡ 快速演示准备..."
    @echo "=========================================="
    @echo "步骤1: 创建演示数据..."
    @just demo-create-data
    @echo ""
    @echo "步骤2: 验证数据..."
    @just demo-verify-data
    @echo ""
    @echo "步骤3: 打开浏览器..."
    @just demo-open-browser
    @echo ""
    @echo "✅ 快速演示准备完成"

# 演示重置：清理数据并重新创建
demo-reset:
    @echo "🔄 重置演示数据..."
    @echo "=========================================="
    @echo "⚠️  警告：这将删除所有现有记忆"
    @echo "按 Ctrl+C 取消，或等待 5 秒继续..."
    @sleep 5
    @echo ""
    @echo "正在重置..."
    @bash scripts/create_demo_memories_from_plan.sh
    @echo ""
    @echo "✅ 演示数据已重置"

# 演示状态检查
demo-status:
    @echo "📊 演示环境状态"
    @echo "=========================================="
    @echo ""
    @echo "后端服务:"
    @curl -s $(DEMO_API_URL)/health | jq '.' 2>/dev/null || echo "❌ 后端未运行"
    @echo ""
    @echo "前端服务:"
    @curl -s $(DEMO_UI_URL) > /dev/null 2>&1 && echo "✅ 前端运行中" || echo "❌ 前端未运行"
    @echo ""
    @echo "演示数据:"
    @curl -s "$(DEMO_API_URL)/api/v1/memories?page=0&limit=1" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" | \
        jq -r '.data.memories | length as $count | "记忆数量: \($count)"' 2>/dev/null || echo "❌ 无法查询"
    @echo ""
    @echo "Agent:"
    @curl -s "$(DEMO_API_URL)/api/v1/agents" \
        -H "X-User-ID: $(DEMO_USER_ID)" \
        -H "X-Organization-ID: $(DEMO_ORG_ID)" | \
        jq -r '.data | length as $count | "Agent数量: \($count)"' 2>/dev/null || echo "❌ 无法查询"

# 演示帮助
demo-help:
    @echo "📖 演示命令帮助"
    @echo "=========================================="
    @echo ""
    @echo "准备阶段:"
    @echo "  just demo-prepare       - 准备演示环境（清理并初始化）"
    @echo "  just demo-create-data   - 创建演示数据（30条记忆）"
    @echo "  just demo-verify-data   - 验证演示数据"
    @echo ""
    @echo "启动服务:"
    @echo "  just demo-start         - 启动演示服务（后端+前端）"
    @echo "  just demo-quick         - 快速演示（假设服务已运行）"
    @echo ""
    @echo "验证测试:"
    @echo "  just demo-verify-ui     - 验证 UI 功能（完整测试）"
    @echo "  just demo-test-search   - 测试搜索功能（7个测试用例）"
    @echo ""
    @echo "浏览器:"
    @echo "  just demo-open-browser  - 打开浏览器验证页面"
    @echo ""
    @echo "完整流程:"
    @echo "  just demo-full          - 完整演示流程（构建+启动+数据+验证）"
    @echo ""
    @echo "其他:"
    @echo "  just demo-status        - 检查演示环境状态"
    @echo "  just demo-reset         - 重置演示数据"
    @echo ""
    @echo "📝 详细文档:"
    @echo "  - 演示计划: x.md"
    @echo "  - 浏览器验证指南: docs/BROWSER_VERIFICATION_GUIDE.md"
    @echo "  - UI验证报告: docs/UI_FUNCTIONALITY_VERIFICATION_REPORT.md"

