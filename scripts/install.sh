#!/bin/bash
# AgentMem 一键安装脚本
# 支持 Linux 和 macOS

set -e

VERSION="0.2.0"
INSTALL_DIR="/opt/agentmem"
DATA_DIR="$HOME/agentmem"

echo "🚀 AgentMem 一键安装脚本 v${VERSION}"
echo "======================================"
echo ""

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 日志函数
log_info() { echo -e "${GREEN}✅ $1${NC}"; }
log_warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# 检测操作系统
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        ARCH=$(uname -m)
        if [[ "$ARCH" == "x86_64" ]]; then
            BINARY_ARCH="amd64"
        elif [[ "$ARCH" == "aarch64" ]]; then
            BINARY_ARCH="arm64"
        else
            log_error "不支持的架构: $ARCH"
            exit 1
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        ARCH=$(uname -m)
        if [[ "$ARCH" == "x86_64" ]]; then
            BINARY_ARCH="amd64"
        elif [[ "$ARCH" == "arm64" ]]; then
            BINARY_ARCH="arm64"
        else
            log_error "不支持的架构: $ARCH"
            exit 1
        fi
    else
        log_error "不支持的操作系统: $OSTYPE"
        exit 1
    fi

    log_info "检测到系统: $OS $ARCH"
}

# 检查依赖
check_dependencies() {
    echo "🔍 检查依赖..."

    # 检查 curl
    if ! command -v curl &> /dev/null; then
        log_error "需要安装 curl"
        if [[ "$OS" == "linux" ]]; then
            echo "   安装命令: sudo apt-get install curl"
        else
            echo "   安装命令: brew install curl"
        fi
        exit 1
    fi

    # 检查 Docker（可选）
    if command -v docker &> /dev/null; then
        log_info "Docker 已安装"
        HAS_DOCKER=true
    else
        log_warn "Docker 未安装（可选）"
        HAS_DOCKER=false
    fi
}

# 下载二进制
download_binary() {
    echo ""
    echo "📥 下载 AgentMem ${VERSION}..."

    # 如果使用 Docker，跳过下载
    if [[ "$HAS_DOCKER" == true && "$USE_DOCKER" == true ]]; then
        log_info "使用 Docker 模式，跳过二进制下载"
        return
    fi

    # 构建下载 URL
    if [[ "$OS" == "linux" ]]; then
        BINARY_NAME="agentmem-linux-${BINARY_ARCH}"
    else
        BINARY_NAME="agentmem-macos-${BINARY_ARCH}"
    fi

    DOWNLOAD_URL="https://github.com/agentmem/agentmem/releases/download/v${VERSION}/${BINARY_NAME}"

    # 下载到临时目录
    TMP_FILE="/tmp/agentmem"

    log_info "从 $DOWNLOAD_URL 下载..."

    if curl -L "$DOWNLOAD_URL" -o "$TMP_FILE"; then
        chmod +x "$TMP_FILE"
        log_info "下载完成"
    else
        log_error "下载失败"
        echo ""
        echo "💡 提示：您可以手动下载："
        echo "   1. 访问: https://github.com/agentmem/agentmem/releases"
        echo "   2. 下载: ${BINARY_NAME}"
        echo "   3. 放到: ${TMP_FILE}"
        exit 1
    fi
}

# 安装二进制
install_binary() {
    echo ""
    echo "📦 安装 AgentMem..."

    # 创建安装目录
    sudo mkdir -p "$INSTALL_DIR"

    # 移动二进制
    sudo mv "$TMP_FILE" "$INSTALL_DIR/agentmem"

    # 创建符号链接
    sudo ln -sf "$INSTALL_DIR/agentmem" /usr/local/bin/agentmem

    log_info "安装完成: $INSTALL_DIR/agentmem"
}

# 初始化数据库
init_database() {
    echo ""
    echo "🗄️  初始化数据库..."

    # 创建数据目录
    mkdir -p "$DATA_DIR/data"

    # 初始化数据库
    if [[ "$HAS_DOCKER" == true && "$USE_DOCKER" == true ]]; then
        # Docker 模式
        log_info "使用 Docker 初始化数据库..."
        docker run --rm -v "$DATA_DIR/data:/data" \
            agentmem/agentmem:v${VERSION} \
            init --db-path /data/agentmem.db
    else
        # 本地模式
        if agentmem init --db-path "$DATA_DIR/data/agentmem.db" 2>/dev/null; then
            log_info "数据库初始化完成"
        else
            log_warn "数据库初始化失败，将在首次运行时自动创建"
        fi
    fi
}

# 配置服务
configure_service() {
    echo ""
    echo "⚙️  配置系统服务..."

    # 询问是否配置服务
    read -p "是否配置为系统服务？(y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_warn "跳过服务配置"
        return
    fi

    if [[ "$OS" == "linux" ]]; then
        # systemd 服务
        log_info "配置 systemd 服务..."

        sudo tee /etc/systemd/system/agentmem.service > /dev/null <<EOF
[Unit]
Description=AgentMem AI Memory Service
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$DATA_DIR
ExecStart=$INSTALL_DIR/agentmem server \\
    --db-path $DATA_DIR/data/agentmem.db \\
    --vector-path $DATA_DIR/data/vectors.lance \\
    --port 8080
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

        sudo systemctl daemon-reload
        sudo systemctl enable agentmem
        log_info "systemd 服务配置完成"

    elif [[ "$OS" == "macos" ]]; then
        # launchd 服务
        log_info "配置 launchd 服务..."

        PLIST_FILE="$HOME/Library/LaunchAgents/com.agentmem.service"

        mkdir -p "$(dirname "$PLIST_FILE")"

        tee "$PLIST_FILE" > /dev/null <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.agentmem</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/agentmem</string>
        <string>server</string>
        <string>--db-path</string>
        <string>$DATA_DIR/data/agentmem.db</string>
        <string>--vector-path</string>
        <string>$DATA_DIR/data/vectors.lance</string>
        <string>--port</string>
        <string>8080</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>WorkingDirectory</key>
    <string>$DATA_DIR</string>
    <key>StandardOutPath</key>
    <string>$DATA_DIR/logs/agentmem.log</string>
    <key>StandardErrorPath</key>
    <string>$DATA_DIR/logs/agentmem.error.log</string>
</dict>
</plist>
EOF

        launchctl load "$PLIST_FILE"
        log_info "launchd 服务配置完成"
    fi
}

# 启动服务
start_service() {
    echo ""
    echo "🚀 启动 AgentMem 服务..."

    # 如果配置了系统服务，使用服务启动
    if [[ "$OS" == "linux" ]] && systemctl is-enabled --quiet agentmem 2>/dev/null; then
        sudo systemctl start agentmem
        log_info "使用 systemd 启动服务"

    elif [[ "$OS" == "macos" ]] && launchctl list | grep -q "com.agentmem"; then
        launchctl start com.agentmem
        log_info "使用 launchd 启动服务"

    else
        # 直接启动
        log_warn "未配置系统服务，使用前台启动"
        echo ""
        echo "💡 提示：您可以手动启动服务："
        echo "   $INSTALL_DIR/agentmem server"
        echo ""
        echo "或者使用 Docker："
        echo "   docker run -d -p 8080:8080 -v $DATA_DIR/data:/data agentmem/agentmem:v${VERSION}"
        return
    fi

    # 等待服务启动
    echo "⏳ 等待服务启动..."
    sleep 5

    # 健康检查
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_info "AgentMem 服务启动成功！"
    else
        log_error "AgentMem 服务启动失败"
        echo ""
        echo "查看日志:"
        if [[ "$OS" == "linux" ]]; then
            echo "   sudo journalctl -u agentmem -f"
        else
            echo "   cat $DATA_DIR/logs/agentmem.error.log"
        fi
        exit 1
    fi
}

# 显示完成信息
show_completion() {
    echo ""
    echo "🎉 安装完成！"
    echo ""
    echo "📍 服务信息:"
    echo "   API 地址:   http://localhost:8080"
    echo "   健康检查:   http://localhost:8080/health"
    echo "   API 文档:   http://localhost:8080/swagger-ui/"
    echo "   数据目录:   $DATA_DIR"
    echo ""
    echo "📖 文档:"
    echo "   快速开始:   https://docs.agentmem.ai/quickstart"
    echo "   API 文档:   https://docs.agentmem.ai/api"
    echo "   GitHub:     https://github.com/agentmem/agentmem"
    echo ""
    echo "🔧 常用命令:"
    echo "   查看状态:   agentmem status"
    echo "   停止服务:   sudo systemctl stop agentmem  (Linux)"
    echo "               launchctl unload ~/Library/LaunchAgents/com.agentmem.plist  (macOS)"
    echo "   查看日志:   sudo journalctl -u agentmem -f  (Linux)"
    echo "               tail -f $DATA_DIR/logs/agentmem.log  (macOS)"
    echo ""
    echo "✨ 快速测试:"
    echo "   curl http://localhost:8080/health | jq"
}

# 主函数
main() {
    # 解析参数
    USE_DOCKER=false
    for arg in "$@"; do
        case $arg in
            --docker)
                USE_DOCKER=true
                ;;
            --help)
                echo "用法: $0 [--docker]"
                echo ""
                echo "选项:"
                echo "  --docker    使用 Docker 模式"
                echo "  --help      显示此帮助信息"
                exit 0
                ;;
        esac
    done

    # 执行安装步骤
    detect_os
    check_dependencies
    download_binary
    install_binary
    init_database
    configure_service
    start_service
    show_completion
}

main "$@"
