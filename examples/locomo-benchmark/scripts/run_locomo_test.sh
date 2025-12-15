#!/bin/bash
# AgentMem LOCOMO 基准测试运行脚本
#
# 用法:
#   ./scripts/run_locomo_test.sh                    # 离线模式（无LLM）
#   ./scripts/run_locomo_test.sh --with-llm         # 使用LLM（需要环境变量）
#
# 环境变量:
#   OPENAI_API_KEY          - OpenAI API密钥
#   LOCOMO_LLM_PROVIDER     - LLM提供商 (默认: openai)
#   LOCOMO_LLM_MODEL        - LLM模型 (默认: gpt-4o-mini)
#   LOCOMO_LLM_BASE_URL     - 自定义API基址（可选）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$BENCHMARK_DIR"

echo "🚀 AgentMem LOCOMO 基准测试"
echo "================================"
echo ""

# 检查数据是否存在
if [ ! -d "data" ] || [ -z "$(ls -A data/*/ 2>/dev/null)" ]; then
    echo "⚠️  数据目录不存在或为空，尝试转换数据..."
    
    if [ ! -f "LoCoMo/data/locomo10.json" ]; then
        echo "❌ 错误: 未找到 LoCoMo/data/locomo10.json"
        echo "   请先下载 LoCoMo 数据集到 LoCoMo/data/locomo10.json"
        exit 1
    fi
    
    echo "📋 转换 LoCoMo 数据..."
    python3 scripts/convert_locomo.py \
        --input LoCoMo/data/locomo10.json \
        --output data
    
    if [ $? -ne 0 ]; then
        echo "❌ 数据转换失败"
        exit 1
    fi
    
    echo "✅ 数据转换完成"
    echo ""
fi

# 检查数据文件数量
echo "📊 数据统计:"
for category in single_hop multi_hop temporal open_domain adversarial; do
    count=$(find "data/$category" -name "*.json" 2>/dev/null | wc -l | tr -d ' ')
    echo "  - $category: $count 个会话"
done
echo ""

# 构建项目
echo "🔨 构建项目..."
cargo build --release --manifest-path Cargo.toml

if [ $? -ne 0 ]; then
    echo "❌ 构建失败"
    exit 1
fi

echo "✅ 构建完成"
echo ""

# 运行测试
if [ "$1" == "--with-llm" ]; then
    echo "🤖 使用 LLM 模式运行测试..."
    
    if [ -z "$OPENAI_API_KEY" ]; then
        echo "⚠️  警告: OPENAI_API_KEY 未设置，将使用离线模式"
        echo ""
    else
        echo "✅ 检测到 OPENAI_API_KEY"
        echo "   提供商: ${LOCOMO_LLM_PROVIDER:-openai}"
        echo "   模型: ${LOCOMO_LLM_MODEL:-gpt-4o-mini}"
        if [ -n "$LOCOMO_LLM_BASE_URL" ]; then
            echo "   基址: $LOCOMO_LLM_BASE_URL"
        fi
        echo ""
    fi
    
    cargo run --release --manifest-path Cargo.toml \
        -- \
        --dataset-path data \
        ${LOCOMO_LLM_PROVIDER:+--llm-provider "$LOCOMO_LLM_PROVIDER"} \
        ${LOCOMO_LLM_MODEL:+--llm-model "$LOCOMO_LLM_MODEL"} \
        ${OPENAI_API_KEY:+--llm-api-key "$OPENAI_API_KEY"} \
        ${LOCOMO_LLM_BASE_URL:+--llm-base-url "$LOCOMO_LLM_BASE_URL"}
else
    echo "📦 离线模式运行测试（无 LLM）..."
    echo ""
    
    cargo run --release --manifest-path Cargo.toml -- --dataset-path data
fi

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "✅ 测试完成！"
    echo "📄 报告位置: results/reports/"
    
    # 显示最新的报告
    if [ -d "results/reports" ]; then
        LATEST_REPORT=$(ls -t results/reports/*.md 2>/dev/null | head -1)
        if [ -n "$LATEST_REPORT" ]; then
            echo "   最新报告: $LATEST_REPORT"
        fi
    fi
else
    echo ""
    echo "❌ 测试失败 (退出码: $EXIT_CODE)"
    exit $EXIT_CODE
fi
