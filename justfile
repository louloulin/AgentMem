# AgentMem Justfile
# 统一管理项目的构建、测试、部署等任务
# 使用方法: just <command>
# 查看所有命令: just --list

# 默认配置
export RUST_BACKTRACE := "1"
export DYLD_LIBRARY_PATH := justfile_directory() + "/lib:" + justfile_directory() + "/target/release"
export ORT_DYLIB_PATH := justfile_directory() + "/lib/libonnxruntime.1.22.0.dylib"

# LLM 配置
export LLM_PROVIDER := "zhipu"
export LLM_MODEL := "glm-4-plus"
export ZHIPU_API_KEY := "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"

# Embedder 配置
export EMBEDDER_PROVIDER := "fastembed"
export EMBEDDER_MODEL := "BAAI/bge-small-en-v1.5"

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
    rm -f *.log *.pid

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
# 服务启动
# ============================================================================

# 启动 HTTP API 服务器（无认证模式，前台运行）
start-server:
    @echo "🚀 启动 HTTP API 服务器（无认证模式，前台）..."
    @export ENABLE_AUTH="false" && \
    export SERVER_ENABLE_AUTH="false" && \
    ./target/release/agent-mem-server

# 启动 HTTP API 服务器（无认证模式，后台运行）
start-server-no-auth:
    @echo "🚀 启动 HTTP API 服务器（无认证模式，后台）..."
    @bash start_server_no_auth.sh

# 启动 HTTP API 服务器（带 ONNX Runtime 修复，后台运行）
start-server-onnx:
    @echo "🚀 启动 HTTP API 服务器（ONNX Runtime 修复版，后台）..."
    @bash start_server_with_correct_onnx.sh

# 启动 HTTP API 服务器（后台运行，通用）
start-server-bg:
    @echo "🚀 启动 HTTP API 服务器（后台）..."
    @bash start_server_no_auth.sh

# 启动 MCP Stdio 服务器
start-mcp:
    @echo "🚀 启动 MCP Stdio 服务器..."
    @./target/release/agentmem-mcp-server

# 启动前端 UI
start-ui:
    @echo "🚀 启动前端 UI..."
    cd agentmem-ui && npm run dev

# 启动全栈（后端 + 前端）
start-full:
    @echo "🚀 启动全栈服务..."
    @bash start_full_stack.sh

# 停止所有服务
stop:
    @echo "🛑 停止所有服务..."
    @pkill -f "agent-mem-server" || true
    @pkill -f "agentmem-mcp-server" || true
    @pkill -f "next dev" || true
    @echo "✅ 所有服务已停止"

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
    @bash test_mcp_functionality.sh

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
    @echo "后端服务:"
    @curl -s http://localhost:8080/health | jq '.' || echo "❌ 后端未运行"
    @echo ""
    @echo "前端服务:"
    @curl -s http://localhost:3001 > /dev/null && echo "✅ 前端运行中" || echo "❌ 前端未运行"

# 查看实时日志
logs service="backend":
    @echo "📝 查看 {{service}} 日志..."
    @if [ "{{service}}" = "backend" ]; then \
        tail -f backend-no-auth.log 2>/dev/null || tail -f backend-test.log 2>/dev/null || echo "❌ 日志文件不存在"; \
    elif [ "{{service}}" = "frontend" ]; then \
        tail -f frontend.log 2>/dev/null || echo "❌ 日志文件不存在"; \
    elif [ "{{service}}" = "ui" ]; then \
        tail -f agentmem-ui/ui.log 2>/dev/null || echo "❌ 日志文件不存在"; \
    else \
        echo "❌ 未知服务: {{service}}"; \
    fi

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
    @just watch &
    @just start-ui

# 清理并重新构建
rebuild: clean build-release
    @echo "✅ 重新构建完成"

# ============================================================================
# 信息查看
# ============================================================================

# 显示项目信息
info:
    @echo "📊 AgentMem 项目信息"
    @echo "===================="
    @echo "版本: $(cargo pkgid | cut -d# -f2)"
    @echo "Rust 版本: $(rustc --version)"
    @echo "Cargo 版本: $(cargo --version)"
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
    @echo "RUST_BACKTRACE: $RUST_BACKTRACE"
    @echo "LLM_PROVIDER: $LLM_PROVIDER"
    @echo "LLM_MODEL: $LLM_MODEL"
    @echo "EMBEDDER_PROVIDER: $EMBEDDER_PROVIDER"
    @echo "EMBEDDER_MODEL: $EMBEDDER_MODEL"
    @echo "DYLD_LIBRARY_PATH: $DYLD_LIBRARY_PATH"

