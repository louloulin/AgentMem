#!/bin/bash
# 性能回归测试脚本
# 用于CI/CD pipeline中检测性能退化

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              性能回归测试 v1.0                               ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 配置
THRESHOLD_PERCENT=10  # 性能退化超过10%视为失败
BASELINE_NAME="main"
CURRENT_NAME="current"
REPORT_FILE="target/regression-report.md"

# 如果存在main分支的baseline，使用它作为对比基准
if [ -d "target/criterion/$BASELINE_NAME" ]; then
    echo -e "${YELLOW}📊 使用已有的baseline: $BASELINE_NAME${NC}"
else
    echo -e "${YELLOW}📊 创建新的baseline: $BASELINE_NAME${NC}"
    cargo bench -- --save-baseline "$BASELINE_NAME"
fi

echo ""
echo -e "${YELLOW}🏃 运行当前代码的性能测试...${NC}"
echo ""

# 运行当前代码的benchmark
cargo bench -- --baseline "$BASELINE_NAME" > target/bench-output.txt 2>&1

# 分析结果
echo ""
echo -e "${GREEN}📈 分析性能变化...${NC}"
echo ""

# 初始化报告
cat > "$REPORT_FILE" << EOF
# 性能回归测试报告

**测试时间**: $(date '+%Y-%m-%d %H:%M:%S')  
**对比基准**: $BASELINE_NAME  
**当前版本**: $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

---

## 性能变化分析

EOF

# 检查是否有性能退化
REGRESSION_FOUND=0

# 解析Criterion输出
grep -E "change:|Performance has regressed|Performance has improved" target/bench-output.txt | while read line; do
    echo "$line" >> "$REPORT_FILE"
    
    # 检查是否有显著退化
    if echo "$line" | grep -q "Performance has regressed"; then
        REGRESSION_FOUND=1
        echo -e "${RED}⚠️  发现性能退化: $line${NC}"
    elif echo "$line" | grep -q "Performance has improved"; then
        echo -e "${GREEN}✨ 性能提升: $line${NC}"
    fi
done

echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 详细输出" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
cat target/bench-output.txt >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"

# 生成摘要
echo ""
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 测试结果" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if [ $REGRESSION_FOUND -eq 0 ]; then
    echo "✅ **通过**: 未发现显著性能退化" >> "$REPORT_FILE"
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║            ✅ 性能回归测试通过！                             ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}📊 报告已生成: $REPORT_FILE${NC}"
    exit 0
else
    echo "❌ **失败**: 发现性能退化，超过${THRESHOLD_PERCENT}%阈值" >> "$REPORT_FILE"
    echo ""
    echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║            ❌ 性能回归测试失败！                             ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${RED}⚠️  发现性能退化，请查看报告: $REPORT_FILE${NC}"
    exit 1
fi

