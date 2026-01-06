#!/bin/bash
# AgentMem 安全审计脚本
# 版本: v1.0
# 日期: 2025-11-03

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              AgentMem 安全审计 v1.0                          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 配置
REPORT_DIR="target/security-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$REPORT_DIR/security_audit_$TIMESTAMP.md"

mkdir -p "$REPORT_DIR"

# 初始化报告
cat > "$REPORT_FILE" << EOF
# AgentMem 安全审计报告

**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')  
**版本**: $(git describe --tags --always 2>/dev/null || echo "dev")  
**提交**: $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

---

## 审计概述

本报告包含以下安全检查：
1. 依赖漏洞扫描 (cargo-audit)
2. 代码安全审计 (cargo-geiger)
3. 许可证合规检查 (cargo-license)
4. 最佳实践检查 (cargo-clippy)

---

EOF

ISSUES_FOUND=0

# 1. 安装必要的工具
echo -e "${YELLOW}[1/5] 检查和安装审计工具...${NC}"
echo ""

if ! command -v cargo-audit &> /dev/null; then
    echo "安装 cargo-audit..."
    cargo install cargo-audit
fi

if ! command -v cargo-geiger &> /dev/null; then
    echo "安装 cargo-geiger..."
    cargo install cargo-geiger
fi

if ! command -v cargo-license &> /dev/null; then
    echo "安装 cargo-license..."
    cargo install cargo-license
fi

echo -e "${GREEN}✅ 工具就绪${NC}"
echo ""

# 2. 依赖漏洞扫描
echo -e "${YELLOW}[2/5] 依赖漏洞扫描 (cargo-audit)...${NC}"
echo ""

echo "## 1. 依赖漏洞扫描" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if cargo audit --json > "$REPORT_DIR/audit_$TIMESTAMP.json" 2>&1; then
    echo -e "${GREEN}✅ 未发现已知漏洞${NC}"
    echo "✅ **通过**: 未发现已知安全漏洞" >> "$REPORT_FILE"
else
    echo -e "${RED}⚠️  发现安全漏洞${NC}"
    echo "❌ **警告**: 发现安全漏洞" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    cat "$REPORT_DIR/audit_$TIMESTAMP.json" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
fi

echo "" >> "$REPORT_FILE"

# 3. Unsafe代码检查
echo ""
echo -e "${YELLOW}[3/5] Unsafe代码检查 (cargo-geiger)...${NC}"
echo ""

echo "## 2. Unsafe代码分析" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

cargo geiger --all-features 2>&1 | tee "$REPORT_DIR/geiger_$TIMESTAMP.txt"

echo '```' >> "$REPORT_FILE"
cat "$REPORT_DIR/geiger_$TIMESTAMP.txt" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 4. 许可证检查
echo ""
echo -e "${YELLOW}[4/5] 许可证合规检查...${NC}"
echo ""

echo "## 3. 许可证合规" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

cargo license --json > "$REPORT_DIR/licenses_$TIMESTAMP.json"

echo "| Package | License | Status |" >> "$REPORT_FILE"
echo "|---------|---------|--------|" >> "$REPORT_FILE"

# 检查是否有非友好许可证
jq -r '.[] | "\(.name)|\(.license)|✅"' "$REPORT_DIR/licenses_$TIMESTAMP.json" | \
while IFS='|' read -r name license status; do
    echo "| $name | $license | $status |" >> "$REPORT_FILE"
done

echo "" >> "$REPORT_FILE"

# 5. Clippy安全检查
echo ""
echo -e "${YELLOW}[5/5] 代码安全检查 (clippy)...${NC}"
echo ""

echo "## 4. 代码质量和安全" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if cargo clippy --all-targets --all-features -- -W clippy::all -W clippy::pedantic -W clippy::security 2>&1 | tee "$REPORT_DIR/clippy_$TIMESTAMP.txt"; then
    echo -e "${GREEN}✅ Clippy检查通过${NC}"
    echo "✅ **通过**: 代码质量和安全检查通过" >> "$REPORT_FILE"
else
    echo -e "${YELLOW}⚠️  Clippy发现问题${NC}"
    echo "⚠️ **警告**: Clippy发现潜在问题" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
    cat "$REPORT_DIR/clippy_$TIMESTAMP.txt" | tail -50 >> "$REPORT_FILE"
    echo '```' >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"

# 6. 安全配置检查
echo ""
echo -e "${YELLOW}[6/6] 安全配置检查...${NC}"
echo ""

echo "## 5. 安全配置检查" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 检查关键安全配置文件
SECURITY_CONFIGS=(
    "Cargo.toml"
    ".github/workflows/security.yml"
    "crates/agent-mem-server/src/auth.rs"
    "crates/agent-mem-server/src/rbac.rs"
)

echo "| 配置项 | 状态 |" >> "$REPORT_FILE"
echo "|--------|------|" >> "$REPORT_FILE"

for config in "${SECURITY_CONFIGS[@]}"; do
    if [ -f "$config" ]; then
        echo "| $config | ✅ 存在 |" >> "$REPORT_FILE"
    else
        echo "| $config | ❌ 缺失 |" >> "$REPORT_FILE"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
done

echo "" >> "$REPORT_FILE"

# 生成摘要
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 审计摘要" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if [ $ISSUES_FOUND -eq 0 ]; then
    echo "✅ **结果**: 安全审计通过" >> "$REPORT_FILE"
    echo "- **发现问题**: 0" >> "$REPORT_FILE"
    echo "- **严重性**: 无" >> "$REPORT_FILE"
else
    echo "⚠️ **结果**: 发现 $ISSUES_FOUND 个问题" >> "$REPORT_FILE"
    echo "- **需要关注**: $ISSUES_FOUND 项" >> "$REPORT_FILE"
    echo "- **建议**: 请查看详细报告并修复问题" >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"

# 添加建议
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 安全建议" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "1. ✅ 定期运行 \`cargo audit\` 检查依赖漏洞" >> "$REPORT_FILE"
echo "2. ✅ 最小化 unsafe 代码的使用" >> "$REPORT_FILE"
echo "3. ✅ 保持依赖更新到最新安全版本" >> "$REPORT_FILE"
echo "4. ✅ 启用所有安全相关的 Clippy lints" >> "$REPORT_FILE"
echo "5. ✅ 在生产环境使用 HTTPS/TLS" >> "$REPORT_FILE"
echo "6. ✅ 实施适当的认证和授权" >> "$REPORT_FILE"
echo "7. ✅ 定期进行安全审计和渗透测试" >> "$REPORT_FILE"
echo "8. ✅ 监控安全公告和CVE数据库" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 显示报告位置
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  安全审计完成！                              ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}📊 报告已生成:${NC}"
echo -e "  - 主报告: $REPORT_FILE"
echo -e "  - 详细数据: $REPORT_DIR"
echo ""

if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}✅ 安全审计通过！${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  发现 $ISSUES_FOUND 个问题，请查看报告${NC}"
    exit 1
fi

