#!/bin/bash
# 验证 LOCOMO 测试环境设置

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$BENCHMARK_DIR"

echo "🔍 验证 AgentMem LOCOMO 测试环境"
echo "================================"
echo ""

# 检查 Rust 工具链
echo "1. 检查 Rust 工具链..."
if command -v cargo &> /dev/null; then
    echo "   ✅ cargo: $(cargo --version)"
    echo "   ✅ rustc: $(rustc --version)"
else
    echo "   ❌ 未找到 Rust 工具链，请先安装 Rust"
    exit 1
fi
echo ""

# 检查数据文件
echo "2. 检查数据文件..."
if [ -f "LoCoMo/data/locomo10.json" ]; then
    echo "   ✅ 找到原始数据: LoCoMo/data/locomo10.json"
    SIZE=$(du -h "LoCoMo/data/locomo10.json" | cut -f1)
    echo "      大小: $SIZE"
else
    echo "   ⚠️  未找到原始数据: LoCoMo/data/locomo10.json"
    echo "      请从 https://snap-research.github.io/locomo/ 下载"
fi

if [ -d "data" ] && [ -n "$(ls -A data/*/ 2>/dev/null)" ]; then
    echo "   ✅ 找到转换后的数据目录: data/"
    for category in single_hop multi_hop temporal open_domain adversarial; do
        if [ -d "data/$category" ]; then
            count=$(find "data/$category" -name "*.json" 2>/dev/null | wc -l | tr -d ' ')
            echo "      - $category: $count 个会话"
        fi
    done
else
    echo "   ⚠️  数据目录不存在或为空"
    echo "      运行: python3 scripts/convert_locomo.py --input LoCoMo/data/locomo10.json --output data"
fi
echo ""

# 检查 Python（用于数据转换）
echo "3. 检查 Python 环境..."
if command -v python3 &> /dev/null; then
    echo "   ✅ python3: $(python3 --version)"
else
    echo "   ⚠️  未找到 python3，无法运行数据转换脚本"
fi
echo ""

# 检查项目构建
echo "4. 检查项目构建..."
if cargo check --manifest-path Cargo.toml &> /dev/null; then
    echo "   ✅ 项目可以编译"
else
    echo "   ❌ 项目编译失败，请检查错误信息"
    exit 1
fi
echo ""

# 检查环境变量（可选）
echo "5. 检查 LLM 配置（可选）..."
if [ -n "$OPENAI_API_KEY" ]; then
    echo "   ✅ OPENAI_API_KEY 已设置"
else
    echo "   ℹ️  OPENAI_API_KEY 未设置（将使用离线模式）"
fi

if [ -n "$LOCOMO_LLM_PROVIDER" ]; then
    echo "   ✅ LOCOMO_LLM_PROVIDER: $LOCOMO_LLM_PROVIDER"
else
    echo "   ℹ️  LOCOMO_LLM_PROVIDER 未设置（默认: openai）"
fi

if [ -n "$LOCOMO_LLM_MODEL" ]; then
    echo "   ✅ LOCOMO_LLM_MODEL: $LOCOMO_LLM_MODEL"
else
    echo "   ℹ️  LOCOMO_LLM_MODEL 未设置（默认: gpt-4o-mini）"
fi
echo ""

echo "✅ 环境检查完成！"
echo ""
echo "下一步："
echo "  - 离线测试: ./scripts/run_locomo_test.sh"
echo "  - LLM测试: OPENAI_API_KEY=xxx ./scripts/run_locomo_test.sh --with-llm"
