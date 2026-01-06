#!/bin/bash
# AgentMem 标准化性能基准测试套件
# 版本: v1.0
# 日期: 2025-11-03

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置
REPORT_DIR="target/benchmark-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$REPORT_DIR/benchmark_$TIMESTAMP.md"
JSON_FILE="$REPORT_DIR/benchmark_$TIMESTAMP.json"
BASELINE_DIR="target/criterion"

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     AgentMem 标准化性能基准测试套件 v1.0                   ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 创建报告目录
mkdir -p "$REPORT_DIR"

# 初始化报告
cat > "$REPORT_FILE" << EOF
# AgentMem 性能基准测试报告

**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')  
**版本**: $(git describe --tags --always 2>/dev/null || echo "dev")  
**提交**: $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")  
**分支**: $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

---

## 测试环境

- **操作系统**: $(uname -s) $(uname -r)
- **CPU**: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || lscpu | grep "Model name" | cut -d: -f2 | xargs)
- **内存**: $(sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024" GB"}' || free -h | grep Mem | awk '{print $2}')
- **Rust版本**: $(rustc --version)

---

## 测试套件

EOF

# 初始化JSON报告
cat > "$JSON_FILE" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "version": "$(git describe --tags --always 2>/dev/null || echo "dev")",
  "commit": "$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")",
  "environment": {
    "os": "$(uname -s)",
    "cpu": "$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "unknown")",
    "rust_version": "$(rustc --version)"
  },
  "benchmarks": {
EOF

# 定义要运行的benchmark
BENCHMARKS=(
    "agent-mem-core::memory_operations"
    "agent-mem-core::graph_reasoning"
    "agent-mem-server::performance_benchmark"
    "agent-mem-server::database_performance"
    "agent-mem-server::vector_performance"
)

echo -e "${YELLOW}[1/5] 运行核心记忆操作基准测试...${NC}"
echo ""

# 运行benchmarks
TOTAL=${#BENCHMARKS[@]}
CURRENT=0
FAILED=0

for bench in "${BENCHMARKS[@]}"; do
    CURRENT=$((CURRENT + 1))
    PACKAGE=$(echo "$bench" | cut -d: -f1)
    BENCH_NAME=$(echo "$bench" | cut -d: -f3)
    
    echo -e "${YELLOW}[$CURRENT/$TOTAL] 测试: $PACKAGE/$BENCH_NAME${NC}"
    
    # 运行benchmark
    if cargo bench --package "$PACKAGE" --bench "$BENCH_NAME" --no-fail-fast 2>&1 | tee -a "$REPORT_DIR/bench_${BENCH_NAME}_$TIMESTAMP.log"; then
        echo -e "${GREEN}✅ $BENCH_NAME 完成${NC}"
        echo "### ✅ $BENCH_NAME" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "详细结果见: [bench_${BENCH_NAME}_$TIMESTAMP.log](bench_${BENCH_NAME}_$TIMESTAMP.log)" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    else
        echo -e "${RED}❌ $BENCH_NAME 失败${NC}"
        echo "### ❌ $BENCH_NAME (失败)" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        FAILED=$((FAILED + 1))
    fi
    echo ""
done

# 添加性能基准对比
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 性能基准对比" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "| 操作 | 目标 | 当前 | 状态 |" >> "$REPORT_FILE"
echo "|------|------|------|------|" >> "$REPORT_FILE"
echo "| 记忆创建 | < 5ms | TBD | ⏳ |" >> "$REPORT_FILE"
echo "| 记忆检索 | < 3ms | TBD | ⏳ |" >> "$REPORT_FILE"
echo "| 语义搜索 | < 25ms | TBD | ⏳ |" >> "$REPORT_FILE"
echo "| 批量操作(100) | < 100ms | TBD | ⏳ |" >> "$REPORT_FILE"
echo "| 图遍历(100节点) | < 20ms | TBD | ⏳ |" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 生成摘要
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 测试摘要" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "- **总测试数**: $TOTAL" >> "$REPORT_FILE"
echo "- **成功**: $((TOTAL - FAILED))" >> "$REPORT_FILE"
echo "- **失败**: $FAILED" >> "$REPORT_FILE"
echo "- **成功率**: $(( (TOTAL - FAILED) * 100 / TOTAL ))%" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 生成趋势分析
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 性能趋势" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "📊 查看完整的性能趋势图表，请访问: \`$BASELINE_DIR\`" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "使用 Criterion 的基准对比:" >> "$REPORT_FILE"
echo '```bash' >> "$REPORT_FILE"
echo "# 对比两次运行的结果" >> "$REPORT_FILE"
echo "cargo bench --bench memory_operations -- --save-baseline before" >> "$REPORT_FILE"
echo "# (进行代码修改)" >> "$REPORT_FILE"
echo "cargo bench --bench memory_operations -- --baseline before" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 添加建议
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 优化建议" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
if [ $FAILED -gt 0 ]; then
    echo "⚠️ **警告**: 有 $FAILED 个基准测试失败，请检查日志。" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi
echo "1. 定期运行基准测试，监控性能退化" >> "$REPORT_FILE"
echo "2. 在合并PR前运行性能对比" >> "$REPORT_FILE"
echo "3. 设置性能预算和告警阈值" >> "$REPORT_FILE"
echo "4. 使用 \`cargo flamegraph\` 分析热点代码" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 结束JSON
echo "  }" >> "$JSON_FILE"
echo "}" >> "$JSON_FILE"

# 显示报告位置
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  基准测试完成！                              ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}📊 报告已生成:${NC}"
echo -e "  - Markdown: $REPORT_FILE"
echo -e "  - JSON: $JSON_FILE"
echo -e "  - Criterion详细结果: $BASELINE_DIR"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 所有基准测试通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED 个基准测试失败${NC}"
    exit 1
fi

