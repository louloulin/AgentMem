#!/bin/bash

###############################################################################
# AgentMem 打包发布脚本
# 
# 功能：
# 1. 构建 Rust 后端服务器 (agent-mem-server)
# 2. 构建 Next.js 前端 (agentmem-ui)
# 3. 支持独立部署模式
# 4. 生成发布包
#
# 使用方法：
#   ./build-release.sh [选项]
#
# 选项：
#   --ui-only       仅构建前端
#   --server-only   仅构建后端
#   --all           构建前端和后端（默认）
#   --release       发布模式（优化构建）
#   --dev           开发模式（快速构建）
#   --clean         清理构建缓存
#   --help          显示帮助信息
###############################################################################

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
BUILD_UI=true
BUILD_SERVER=true
BUILD_MODE="release"
CLEAN_BUILD=false

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$PROJECT_ROOT/agentmem-ui"
SERVER_DIR="$PROJECT_ROOT/crates/agent-mem-server"
DIST_DIR="$PROJECT_ROOT/dist"

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

# 显示帮助信息
show_help() {
    cat << EOF
AgentMem 打包发布脚本

使用方法：
  ./build-release.sh [选项]

选项：
  --ui-only       仅构建前端
  --server-only   仅构建后端
  --all           构建前端和后端（默认）
  --release       发布模式（优化构建，默认）
  --dev           开发模式（快速构建）
  --clean         清理构建缓存
  --help          显示帮助信息

示例：
  # 构建所有组件（发布模式）
  ./build-release.sh

  # 仅构建前端
  ./build-release.sh --ui-only

  # 仅构建后端（开发模式）
  ./build-release.sh --server-only --dev

  # 清理并重新构建
  ./build-release.sh --clean --all

EOF
}

# 解析命令行参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --ui-only)
                BUILD_UI=true
                BUILD_SERVER=false
                shift
                ;;
            --server-only)
                BUILD_UI=false
                BUILD_SERVER=true
                shift
                ;;
            --all)
                BUILD_UI=true
                BUILD_SERVER=true
                shift
                ;;
            --release)
                BUILD_MODE="release"
                shift
                ;;
            --dev)
                BUILD_MODE="dev"
                shift
                ;;
            --clean)
                CLEAN_BUILD=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    # 检查 Node.js
    if $BUILD_UI; then
        if ! command -v node &> /dev/null; then
            log_error "未找到 Node.js，请先安装 Node.js"
            exit 1
        fi
        log_success "Node.js 版本: $(node --version)"
        
        if ! command -v npm &> /dev/null; then
            log_error "未找到 npm，请先安装 npm"
            exit 1
        fi
        log_success "npm 版本: $(npm --version)"
    fi
    
    # 检查 Rust
    if $BUILD_SERVER; then
        if ! command -v cargo &> /dev/null; then
            log_error "未找到 Cargo，请先安装 Rust"
            exit 1
        fi
        log_success "Cargo 版本: $(cargo --version)"
    fi
}

# 清理构建缓存
clean_build() {
    log_info "清理构建缓存..."
    
    if $BUILD_UI; then
        log_info "清理前端缓存..."
        cd "$UI_DIR"
        rm -rf .next out node_modules/.cache
        log_success "前端缓存已清理"
    fi
    
    if $BUILD_SERVER; then
        log_info "清理后端缓存..."
        cd "$PROJECT_ROOT"
        cargo clean -p agent-mem-server
        log_success "后端缓存已清理"
    fi
    
    # 清理发布目录
    if [ -d "$DIST_DIR" ]; then
        log_info "清理发布目录..."
        rm -rf "$DIST_DIR"
        log_success "发布目录已清理"
    fi
}

# 构建前端
build_ui() {
    log_info "========================================="
    log_info "开始构建前端 (agentmem-ui)"
    log_info "========================================="
    
    cd "$UI_DIR"
    
    # 安装依赖
    if [ ! -d "node_modules" ]; then
        log_info "安装前端依赖..."
        npm install
        log_success "前端依赖安装完成"
    fi
    
    # 构建前端
    log_info "构建 Next.js 应用..."
    # 始终使用 production 模式构建，避免 Next.js 警告
    NODE_ENV=production npm run build
    
    log_success "前端构建完成"
    
    # 创建发布目录
    mkdir -p "$DIST_DIR/ui"
    
    # 复制构建产物
    log_info "复制前端构建产物..."
    cp -r .next "$DIST_DIR/ui/"
    cp -r public "$DIST_DIR/ui/"
    cp package.json "$DIST_DIR/ui/"
    cp next.config.ts "$DIST_DIR/ui/"
    
    # 创建启动脚本
    cat > "$DIST_DIR/ui/start.sh" << 'EOF'
#!/bin/bash
# AgentMem UI 启动脚本

# 设置环境变量
export NODE_ENV=production
export PORT=${PORT:-3000}
export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:8080}

echo "启动 AgentMem UI..."
echo "端口: $PORT"
echo "API URL: $NEXT_PUBLIC_API_URL"

# 安装生产依赖
if [ ! -d "node_modules" ]; then
    echo "安装依赖..."
    npm install --production
fi

# 启动服务
npm start
EOF
    
    chmod +x "$DIST_DIR/ui/start.sh"
    
    log_success "前端发布包已生成: $DIST_DIR/ui"
}

# 构建后端
build_server() {
    log_info "========================================="
    log_info "开始构建后端 (agent-mem-server)"
    log_info "========================================="
    
    cd "$PROJECT_ROOT"
    
    # 构建后端
    log_info "构建 Rust 服务器..."
    if [ "$BUILD_MODE" = "release" ]; then
        cargo build --package agent-mem-server --release
        BINARY_PATH="target/release/agent-mem-server"
    else
        cargo build --package agent-mem-server
        BINARY_PATH="target/debug/agent-mem-server"
    fi
    
    log_success "后端构建完成"
    
    # 创建发布目录
    mkdir -p "$DIST_DIR/server"
    
    # 复制二进制文件
    log_info "复制后端二进制文件..."
    cp "$BINARY_PATH" "$DIST_DIR/server/"

    # 复制 ONNX Runtime 库文件
    log_info "复制 ONNX Runtime 库文件..."
    mkdir -p "$DIST_DIR/server/lib"

    # 检查并复制 lib 目录下的所有库文件
    if [ -d "lib" ]; then
        cp -r lib/* "$DIST_DIR/server/lib/" 2>/dev/null || true
        log_success "已复制 lib 目录下的库文件"
    else
        log_warning "未找到 lib 目录，跳过库文件复制"
    fi

    # 创建配置文件示例
    cat > "$DIST_DIR/server/config.example.toml" << 'EOF'
# AgentMem Server 配置文件

[server]
host = "0.0.0.0"
port = 8080

[database]
url = "sqlite://agentmem.db"

[cors]
allowed_origins = ["http://localhost:3000"]

[mcp]
enabled = true
EOF
    
    # 创建启动脚本
    cat > "$DIST_DIR/server/start.sh" << 'EOF'
#!/bin/bash
# AgentMem Server 启动脚本

# 设置环境变量
export RUST_LOG=${RUST_LOG:-info}
export SERVER_HOST=${SERVER_HOST:-0.0.0.0}
export SERVER_PORT=${SERVER_PORT:-8080}
export DATABASE_URL=${DATABASE_URL:-file:./data/agentmem.db}

# 获取绝对路径
LIB_DIR="$(pwd)/lib"

# 设置库路径（macOS 使用 DYLD_LIBRARY_PATH，Linux 使用 LD_LIBRARY_PATH）
if [[ "$OSTYPE" == "darwin"* ]]; then
    export DYLD_LIBRARY_PATH="$LIB_DIR:$DYLD_LIBRARY_PATH"
    export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.1.22.0.dylib"
else
    export LD_LIBRARY_PATH="$LIB_DIR:$LD_LIBRARY_PATH"
    export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.so.1.22.0"
fi

export RUST_BACKTRACE=1

# 配置 Embedder (使用 FastEmbed) - 推荐配置
export EMBEDDER_PROVIDER=${EMBEDDER_PROVIDER:-"fastembed"}
export EMBEDDER_MODEL=${EMBEDDER_MODEL:-"BAAI/bge-small-en-v1.5"}

# 配置 LLM Provider (可选)
# 支持的 Provider: openai, zhipu, ollama 等
# export LLM_PROVIDER="zhipu"
# export LLM_MODEL="glm-4.6"
# export ZHIPU_API_KEY="your_api_key_here"
#
# 或使用 OpenAI:
# export LLM_PROVIDER="openai"
# export LLM_MODEL="gpt-4"
# export OPENAI_API_KEY="your_api_key_here"

# 认证配置（默认启用）
export ENABLE_AUTH=${ENABLE_AUTH:-"true"}
export SERVER_ENABLE_AUTH=${SERVER_ENABLE_AUTH:-"true"}

# 代理配置（如需要）
# export http_proxy=http://127.0.0.1:4780
# export https_proxy=http://127.0.0.1:4780

echo "========================================="
echo "🚀 启动 AgentMem Server"
echo "========================================="
echo "主机: $SERVER_HOST"
echo "端口: $SERVER_PORT"
echo "数据库: $DATABASE_URL"
echo "Embedder: $EMBEDDER_PROVIDER / $EMBEDDER_MODEL"
echo "认证: $ENABLE_AUTH"

if [ -n "$LLM_PROVIDER" ]; then
    echo "LLM Provider: $LLM_PROVIDER / $LLM_MODEL"
else
    echo "⚠️  LLM Provider 未配置，Intelligence 组件将不可用"
fi

if [ -d "$LIB_DIR" ]; then
    echo "库目录: $LIB_DIR"
else
    echo "⚠️  警告: 未找到 lib 目录，ONNX Runtime 可能无法加载"
fi

echo "========================================="
echo ""
echo "⏳ 正在启动服务器..."
echo "   首次运行时，FastEmbed 会下载模型文件（约 100MB）"
echo "   这可能需要几分钟时间，请耐心等待..."
echo ""

# 启动服务
./agent-mem-server
EOF
    
    chmod +x "$DIST_DIR/server/start.sh"

    # 创建带完整配置的启动脚本示例
    cat > "$DIST_DIR/server/start-with-zhipu.sh" << 'EOF'
#!/bin/bash
# AgentMem Server 启动脚本 (智谱 AI 配置示例)

# 设置环境变量
export RUST_LOG=${RUST_LOG:-info}
export SERVER_HOST=${SERVER_HOST:-0.0.0.0}
export SERVER_PORT=${SERVER_PORT:-8080}
export DATABASE_URL=${DATABASE_URL:-file:./data/agentmem.db}

# 获取绝对路径
LIB_DIR="$(pwd)/lib"

# 设置库路径
if [[ "$OSTYPE" == "darwin"* ]]; then
    export DYLD_LIBRARY_PATH="$LIB_DIR:$DYLD_LIBRARY_PATH"
    export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.1.22.0.dylib"
else
    export LD_LIBRARY_PATH="$LIB_DIR:$LD_LIBRARY_PATH"
    export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.so.1.22.0"
fi

export RUST_BACKTRACE=1

# 配置 Embedder (使用 FastEmbed)
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 配置 LLM Provider (智谱 AI)
export ZHIPU_API_KEY="your_zhipu_api_key_here"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"

# 🔓 禁用认证（用于测试）
export ENABLE_AUTH="false"
export SERVER_ENABLE_AUTH="false"

# 代理配置（如需要）
# export http_proxy=http://127.0.0.1:4780
# export https_proxy=http://127.0.0.1:4780

echo "========================================="
echo "🚀 启动 AgentMem Server (智谱 AI)"
echo "========================================="
echo "主机: $SERVER_HOST"
echo "端口: $SERVER_PORT"
echo "数据库: $DATABASE_URL"
echo "Embedder: $EMBEDDER_PROVIDER / $EMBEDDER_MODEL"
echo "LLM Provider: $LLM_PROVIDER / $LLM_MODEL"
echo "认证: $ENABLE_AUTH (禁用)"
echo "库目录: $LIB_DIR"
echo "========================================="
echo ""
echo "⏳ 正在启动服务器..."
echo "   首次运行时，FastEmbed 会下载模型文件（约 100MB）"
echo "   这可能需要几分钟时间，请耐心等待..."
echo ""

# 启动服务
./agent-mem-server
EOF

    chmod +x "$DIST_DIR/server/start-with-zhipu.sh"

    log_success "后端发布包已生成: $DIST_DIR/server"
}

# 生成部署文档
generate_deployment_docs() {
    log_info "生成部署文档..."
    
    cat > "$DIST_DIR/README.md" << 'EOF'
# AgentMem 部署指南

## 目录结构

```
dist/
├── ui/              # 前端应用
│   ├── .next/       # Next.js 构建产物
│   ├── public/      # 静态资源
│   ├── package.json
│   └── start.sh     # 启动脚本
├── server/          # 后端服务
│   ├── agent-mem-server       # 二进制文件
│   ├── lib/                   # ONNX Runtime 库文件
│   │   └── libonnxruntime.*   # ONNX Runtime 动态库
│   ├── config.example.toml    # 配置文件示例
│   ├── start.sh               # 基础启动脚本
│   └── start-with-zhipu.sh    # 智谱 AI 配置示例
└── README.md        # 本文件
```

## 部署步骤

### 1. 部署后端服务

```bash
cd server

# 复制配置文件
cp config.example.toml config.toml

# 编辑配置文件（可选）
vim config.toml

# 启动服务
./start.sh
```

后端服务默认运行在 `http://0.0.0.0:8080`

### 2. 部署前端应用

```bash
cd ui

# 设置 API 地址
export NEXT_PUBLIC_API_URL=http://your-server-ip:8080

# 启动服务
./start.sh
```

前端应用默认运行在 `http://localhost:3000`

### 3. 环境变量配置

#### 后端环境变量

**基础配置:**
- `SERVER_HOST`: 服务器主机地址（默认: 0.0.0.0）
- `SERVER_PORT`: 服务器端口（默认: 8080）
- `DATABASE_URL`: 数据库连接字符串（默认: sqlite://agentmem.db）
- `RUST_LOG`: 日志级别（默认: info）

**Embedder 配置（必需）:**
- `EMBEDDER_PROVIDER`: Embedder 提供商（推荐: fastembed）
- `EMBEDDER_MODEL`: Embedder 模型（推荐: BAAI/bge-small-en-v1.5）

**LLM 配置（可选）:**

使用智谱 AI:
```bash
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"
export ZHIPU_API_KEY="your_api_key_here"
```

使用 OpenAI:
```bash
export LLM_PROVIDER="openai"
export LLM_MODEL="gpt-4"
export OPENAI_API_KEY="your_api_key_here"
```

**认证配置:**
- `ENABLE_AUTH`: 是否启用认证（默认: true）
- `SERVER_ENABLE_AUTH`: 服务器认证开关（默认: true）

**库路径配置（自动设置）:**
- macOS: `DYLD_LIBRARY_PATH` 和 `ORT_DYLIB_PATH`
- Linux: `LD_LIBRARY_PATH` 和 `ORT_DYLIB_PATH`

#### 前端环境变量

- `PORT`: 前端端口（默认: 3000）
- `NEXT_PUBLIC_API_URL`: 后端 API 地址（默认: http://localhost:8080）

### 4. 快速启动示例

#### 使用基础配置启动（仅 Embedder）

```bash
cd server
./start.sh
```

#### 使用智谱 AI 配置启动

```bash
cd server
# 编辑 start-with-zhipu.sh，设置你的 API Key
vim start-with-zhipu.sh
# 启动
./start-with-zhipu.sh
```

#### 自定义配置启动

```bash
cd server
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"
export ZHIPU_API_KEY="your_api_key_here"
export ENABLE_AUTH="false"  # 禁用认证（测试用）
./start.sh
```

### 5. 库文件说明

后端服务依赖 ONNX Runtime 库文件，构建脚本会自动从项目根目录的 `lib/` 目录复制到 `dist/server/lib/`。

**macOS:**
- `libonnxruntime.1.22.0.dylib`

**Linux:**
- `libonnxruntime.so.1.22.0`

如果启动时提示找不到库文件，请确保：
1. `lib/` 目录存在且包含正确的库文件
2. 启动脚本正确设置了 `DYLD_LIBRARY_PATH` (macOS) 或 `LD_LIBRARY_PATH` (Linux)

### 6. 使用 systemd 管理服务（推荐）

#### 后端服务

创建 `/etc/systemd/system/agentmem-server.service`:

```ini
[Unit]
Description=AgentMem Server
After=network.target

[Service]
Type=simple
User=agentmem
WorkingDirectory=/opt/agentmem/server
Environment="RUST_LOG=info"
Environment="SERVER_HOST=0.0.0.0"
Environment="SERVER_PORT=8080"
Environment="DATABASE_URL=sqlite://agentmem.db"
Environment="LD_LIBRARY_PATH=/opt/agentmem/server/lib"
Environment="ORT_DYLIB_PATH=/opt/agentmem/server/lib/libonnxruntime.so.1.22.0"
Environment="EMBEDDER_PROVIDER=fastembed"
Environment="EMBEDDER_MODEL=BAAI/bge-small-en-v1.5"
Environment="ENABLE_AUTH=true"
# 可选: LLM 配置
# Environment="LLM_PROVIDER=zhipu"
# Environment="LLM_MODEL=glm-4.6"
# Environment="ZHIPU_API_KEY=your_api_key_here"
ExecStart=/opt/agentmem/server/agent-mem-server
Restart=always

[Install]
WantedBy=multi-user.target
```

#### 前端服务

创建 `/etc/systemd/system/agentmem-ui.service`:

```ini
[Unit]
Description=AgentMem UI
After=network.target

[Service]
Type=simple
User=agentmem
WorkingDirectory=/opt/agentmem/ui
Environment="NODE_ENV=production"
Environment="PORT=3000"
Environment="NEXT_PUBLIC_API_URL=http://localhost:8080"
ExecStart=/opt/agentmem/ui/start.sh
Restart=always

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable agentmem-server
sudo systemctl enable agentmem-ui
sudo systemctl start agentmem-server
sudo systemctl start agentmem-ui
```

### 5. 使用 Nginx 反向代理（可选）

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # 前端
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # 后端 API
    location /api {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## 故障排查

### 后端无法启动

1. 检查端口是否被占用：`lsof -i :8080`
2. 检查数据库连接：确保 DATABASE_URL 正确
3. 查看日志：`RUST_LOG=debug ./agent-mem-server`

### 前端无法连接后端

1. 检查 NEXT_PUBLIC_API_URL 是否正确
2. 检查 CORS 配置
3. 检查网络连接

## 性能优化

1. 使用 CDN 加速静态资源
2. 启用 Nginx gzip 压缩
3. 配置数据库连接池
4. 使用 Redis 缓存

EOF
    
    log_success "部署文档已生成: $DIST_DIR/README.md"
}

# 主函数
main() {
    log_info "========================================="
    log_info "AgentMem 打包发布脚本"
    log_info "========================================="
    
    # 解析参数
    parse_args "$@"
    
    # 检查依赖
    check_dependencies
    
    # 清理构建（如果需要）
    if $CLEAN_BUILD; then
        clean_build
    fi
    
    # 构建前端
    if $BUILD_UI; then
        build_ui
    fi
    
    # 构建后端
    if $BUILD_SERVER; then
        build_server
    fi
    
    # 生成部署文档
    generate_deployment_docs
    
    # 显示总结
    log_info "========================================="
    log_success "构建完成！"
    log_info "========================================="
    log_info "发布包位置: $DIST_DIR"
    
    if $BUILD_UI; then
        log_info "前端: $DIST_DIR/ui"
    fi
    
    if $BUILD_SERVER; then
        log_info "后端: $DIST_DIR/server"
    fi
    
    log_info ""
    log_info "下一步："
    log_info "1. 查看部署文档: cat $DIST_DIR/README.md"
    log_info "2. 部署后端: cd $DIST_DIR/server && ./start.sh"
    log_info "3. 部署前端: cd $DIST_DIR/ui && ./start.sh"
    log_info "========================================="
}

# 执行主函数
main "$@"

