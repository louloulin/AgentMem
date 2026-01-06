#!/bin/bash

# 商品搜索演示启动脚本

set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                                                           ║"
echo "║     🛒 商品搜索演示 - AgentMem混合检索系统               ║"
echo "║                                                           ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

cd "$(dirname "$0")"

# 检查是否设置了OpenAI API Key
if [ -z "$OPENAI_API_KEY" ]; then
    echo "ℹ️  未设置 OPENAI_API_KEY"
    echo "   将使用基础搜索模式（不使用LLM）"
    echo ""
    echo "   如需启用LLM增强搜索："
    echo "   export OPENAI_API_KEY='your-key-here'"
    echo ""
else
    echo "✅ 检测到 OPENAI_API_KEY"
    echo "   将使用LLM增强查询理解"
    echo ""
fi

echo "🚀 开始编译和运行..."
echo ""

# 运行演示
cargo run --release

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 演示完成！"
echo ""
echo "📚 了解更多："
echo "  • cat README.md"
echo "  • cat ../../MINIMAL_INTEGRATION_GUIDE.md"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

