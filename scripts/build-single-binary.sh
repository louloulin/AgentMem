#!/bin/bash
# AgentMem 单二进制打包脚本
#
# 用法:
#   ./scripts/build-single-binary.sh [target] [profile]
#
# 参数:
#   target  - 目标平台 (linux-x64, linux-arm64, macos-x64, macos-arm64, windows-x64)
#   profile - 构建配置 (dev, release, minimal, production)
#
# 示例:
#   ./scripts/build-single-binary.sh linux-x64 production
#   ./scripts/build-single-binary.sh macos-arm64 release

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印信息
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 默认参数
TARGET="${1:-linux-x64}"
PROFILE="${2:-release}"

info "AgentMem 单二进制打包"
info "目标平台: $TARGET"
info "构建配置: $PROFILE"

# 映射目标平台到 Rust target
case "$TARGET" in
    linux-x64)
        RUST_TARGET="x86_64-unknown-linux-gnu"
        ;;
    linux-arm64)
        RUST_TARGET="aarch64-unknown-linux-gnu"
        ;;
    macos-x64)
        RUST_TARGET="x86_64-apple-darwin"
        ;;
    macos-arm64)
        RUST_TARGET="aarch64-apple-darwin"
        ;;
    windows-x64)
        RUST_TARGET="x86_64-pc-windows-msvc"
        ;;
    *)
        error "未知的目标平台: $TARGET"
        error "支持的平台: linux-x64, linux-arm64, macos-x64, macos-arm64, windows-x64"
        exit 1
        ;;
esac

# 设置优化选项
case "$PROFILE" in
    dev|development)
        OPT_LEVEL="0"
        LTO="false"
        STRIP="false"
        ;;
    release)
        OPT_LEVEL="2"
        LTO="true"
        STRIP="true"
        ;;
    minimal)
        OPT_LEVEL="z"
        LTO="true"
        STRIP="true"
        ;;
    production|prod)
        OPT_LEVEL="3"
        LTO="true"
        STRIP="true"
        ;;
    *)
        error "未知的构建配置: $PROFILE"
        error "支持的配置: dev, release, minimal, production"
        exit 1
        ;;
esac

# 检查 Rust 工具链
info "检查 Rust 工具链..."
if ! command -v cargo &> /dev/null; then
    error "未找到 cargo，请安装 Rust"
    exit 1
fi

# 检查目标平台是否已安装
info "检查目标平台: $RUST_TARGET"
if ! rustup target list --installed | grep -q "$RUST_TARGET"; then
    warn "目标平台未安装，正在安装..."
    rustup target add "$RUST_TARGET"
fi

# 设置环境变量
export CARGO_PROFILE_RELEASE_OPT_LEVEL="$OPT_LEVEL"
export CARGO_PROFILE_RELEASE_LTO="$LTO"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="1"
export CARGO_PROFILE_RELEASE_PANIC="abort"

# 构建
info "开始构建..."
cargo build --release \
    --target "$RUST_TARGET" \
    --features "embedded-db,embedded-vector"

# 查找生成的二进制文件
BINARY_NAME="agentmem"
if [ "$TARGET" = "windows-x64" ]; then
    BINARY_NAME="agentmem.exe"
fi

BINARY_PATH="target/$RUST_TARGET/release/$BINARY_NAME"

if [ ! -f "$BINARY_PATH" ]; then
    error "未找到构建的二进制文件: $BINARY_PATH"
    exit 1
fi

info "二进制文件构建完成: $BINARY_PATH"

# 获取原始大小
ORIGINAL_SIZE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null)
info "原始大小: $(numfmt --to=iec-i --suffix=B $ORIGINAL_SIZE 2>/dev/null || echo $ORIGINAL_SIZE bytes)"

# Strip 符号
if [ "$STRIP" = "true" ]; then
    info "移除符号..."
    if command -v strip &> /dev/null; then
        strip "$BINARY_PATH"
        STRIPPED_SIZE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null)
        info "Strip 后大小: $(numfmt --to=iec-i --suffix=B $STRIPPED_SIZE 2>/dev/null || echo $STRIPPED_SIZE bytes)"
    else
        warn "未找到 strip 命令，跳过"
    fi
fi

# 创建输出目录
OUTPUT_DIR="dist/$TARGET"
mkdir -p "$OUTPUT_DIR"

# 复制二进制文件
info "复制到输出目录: $OUTPUT_DIR"
cp "$BINARY_PATH" "$OUTPUT_DIR/"

# 复制配置文件
info "复制配置文件..."
mkdir -p "$OUTPUT_DIR/config"
cp crates/agent-mem-deployment/templates/*.toml "$OUTPUT_DIR/config/" 2>/dev/null || true

# 生成元数据
info "生成元数据..."
cat > "$OUTPUT_DIR/metadata.json" <<EOF
{
  "version": "$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')",
  "target": "$TARGET",
  "rust_target": "$RUST_TARGET",
  "profile": "$PROFILE",
  "opt_level": "$OPT_LEVEL",
  "lto": $LTO,
  "stripped": $STRIP,
  "binary_size": $STRIPPED_SIZE,
  "build_time": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "features": ["embedded-db", "embedded-vector"]
}
EOF

# 生成 README
info "生成 README..."
cat > "$OUTPUT_DIR/README.md" <<EOF
# AgentMem 单二进制包

## 版本信息

- 版本: $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
- 目标平台: $TARGET
- 构建配置: $PROFILE
- 构建时间: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## 文件说明

- \`$BINARY_NAME\`: AgentMem 可执行文件
- \`config/\`: 配置文件模板
- \`metadata.json\`: 构建元数据

## 快速开始

1. 运行 AgentMem:
   \`\`\`bash
   ./$BINARY_NAME
   \`\`\`

2. 使用自定义配置:
   \`\`\`bash
   ./$BINARY_NAME --config config/config.prod.toml
   \`\`\`

3. 查看帮助:
   \`\`\`bash
   ./$BINARY_NAME --help
   \`\`\`

## 配置文件

可用的配置模板:
- \`config.dev.toml\`: 开发环境配置
- \`config.prod.toml\`: 生产环境配置
- \`config.test.toml\`: 测试环境配置
- \`config.minimal.toml\`: 最小化配置
- \`config.full.toml\`: 完整功能配置

## 系统要求

- 操作系统: $(echo $TARGET | cut -d'-' -f1)
- 架构: $(echo $TARGET | cut -d'-' -f2)
- 磁盘空间: 至少 100 MB

## 更多信息

访问: https://github.com/yourusername/agentmem
EOF

# 完成
FINAL_SIZE=$(stat -f%z "$OUTPUT_DIR/$BINARY_NAME" 2>/dev/null || stat -c%s "$OUTPUT_DIR/$BINARY_NAME" 2>/dev/null)
info "打包完成!"
info "输出目录: $OUTPUT_DIR"
info "最终大小: $(numfmt --to=iec-i --suffix=B $FINAL_SIZE 2>/dev/null || echo $FINAL_SIZE bytes)"

# 计算压缩率
if [ "$ORIGINAL_SIZE" -gt 0 ]; then
    REDUCTION=$(echo "scale=2; ($ORIGINAL_SIZE - $FINAL_SIZE) * 100 / $ORIGINAL_SIZE" | bc)
    info "大小减少: ${REDUCTION}%"
fi

info "✓ 构建成功!"

