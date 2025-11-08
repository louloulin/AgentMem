#!/bin/bash
# AgentMem 真实功能验证脚本
# 使用真实的 Zhipu AI 和 FastEmbed

set -e

cd "$(dirname "$0")"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║      AgentMem 真实功能验证（使用 Zhipu AI）                     ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# 设置环境变量
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
export http_proxy="http://127.0.0.1:4780"
export https_proxy="http://127.0.0.1:4780"
export RUST_LOG=info

echo "✅ 环境变量已配置"
echo "   LLM: $LLM_PROVIDER / $LLM_MODEL"
echo "   Embedder: $EMBEDDER_PROVIDER / $EMBEDDER_MODEL"
echo ""

# 创建测试目录
TEST_DIR="$(pwd)/test_output"
mkdir -p "$TEST_DIR"
export AGENTMEM_DB_PATH="$TEST_DIR/agentmem_test.db"

echo "📁 测试数据库: $AGENTMEM_DB_PATH"
echo ""

# 测试列表
tests=(
    "default_behavior_test"
    "p1_session_flexibility_test"
    "orchestrator_intelligence_test"
)

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 运行单元测试"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

total_passed=0
total_failed=0

for test in "${tests[@]}"; do
    echo "运行测试: $test"
    if cargo test --package agent-mem --test "$test" -- --test-threads=1 2>&1 | tee "$TEST_DIR/${test}.log" | grep -q "test result: ok"; then
        passed=$(grep "test result: ok" "$TEST_DIR/${test}.log" | grep -oP '\d+(?= passed)' | head -1)
        echo "✅ $test: $passed 个测试通过"
        total_passed=$((total_passed + passed))
    else
        echo "❌ $test: 测试失败"
        total_failed=$((total_failed + 1))
    fi
    echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 运行真实验证示例"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "运行 P0 真实验证..."
cd examples/p0-real-verification
if cargo run 2>&1 | tee "$TEST_DIR/p0_real_verification.log" | grep -q "🎉 P0 真实验证完成"; then
    echo "✅ P0 真实验证通过"
else
    echo "❌ P0 真实验证失败"
    total_failed=$((total_failed + 1))
fi
cd ../..
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 运行 MCP 功能验证"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if python3 verify_mcp_features.py 2>&1 | tee "$TEST_DIR/mcp_verification.log" | grep -q "所有功能验证通过"; then
    echo "✅ MCP 功能验证通过"
else
    echo "❌ MCP 功能验证失败"
    total_failed=$((total_failed + 1))
fi
echo ""

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                   验证总结                                      ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "✅ 单元测试通过: $total_passed 个"
echo "❌ 测试失败: $total_failed 个"
echo ""
echo "📄 测试日志保存在: $TEST_DIR"
echo ""

if [ $total_failed -eq 0 ]; then
    echo "🎉 所有验证通过！AgentMem 功能完整且正常！"
    exit 0
else
    echo "⚠️  有 $total_failed 个验证失败，请检查日志"
    exit 1
fi
EOF

chmod +x test_agentmem_real.sh
cat test_agentmem_real.sh

