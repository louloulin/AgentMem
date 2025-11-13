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
export DATABASE_URL=${DATABASE_URL:-sqlite://agentmem.db}

echo "启动 AgentMem Server..."
echo "主机: $SERVER_HOST"
echo "端口: $SERVER_PORT"
echo "数据库: $DATABASE_URL"

# 启动服务
./agent-mem-server
EOF
    
    chmod +x "$DIST_DIR/server/start.sh"
    
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
│   ├── agent-mem-server  # 二进制文件
│   ├── config.example.toml
│   └── start.sh     # 启动脚本
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

- `SERVER_HOST`: 服务器主机地址（默认: 0.0.0.0）
- `SERVER_PORT`: 服务器端口（默认: 8080）
- `DATABASE_URL`: 数据库连接字符串（默认: sqlite://agentmem.db）
- `RUST_LOG`: 日志级别（默认: info）

#### 前端环境变量

- `PORT`: 前端端口（默认: 3000）
- `NEXT_PUBLIC_API_URL`: 后端 API 地址（默认: http://localhost:8080）

### 4. 使用 systemd 管理服务（推荐）

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

