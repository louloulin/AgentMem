#!/bin/bash
# Mem0 vs AgentMem 性能对比测试脚本

echo "============================================================"
echo "Mem0 vs AgentMem 性能对比测试"
echo "============================================================"

# 配置代理（如果存在代理配置脚本）
if [ -f "./setup_proxy.sh" ]; then
    echo "配置代理..."
    source ./setup_proxy.sh
fi

# 检查 Python 环境
if ! command -v python3 &> /dev/null; then
    echo "错误: 未找到 python3"
    exit 1
fi

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "错误: 未找到 cargo"
    exit 1
fi

# 运行 Mem0 测试
echo ""
echo "============================================================"
echo "运行 Mem0 测试..."
echo "============================================================"
if [ -f "run_mem0_test.sh" ]; then
    ./run_mem0_test.sh
else
    python3 mem0_benchmark.py
fi

# 运行 AgentMem 测试
echo ""
echo "============================================================"
echo "运行 AgentMem 测试..."
echo "============================================================"
if [ -f "run_agentmem_test.sh" ]; then
    ./run_agentmem_test.sh
else
    cargo run --bin agentmem_benchmark --release
fi

echo ""
echo "============================================================"
echo "测试完成！"
echo "============================================================"
echo ""
echo "请对比两个测试的结果，分析性能差异。"
echo ""
echo "注意:"
echo "  - Mem0 目标性能: 10,000 ops/s (infer=False)"
echo "  - AgentMem 当前性能: ~470 ops/s (批量添加)"
echo "  - 性能差距主要来自 FastEmbed 的内部优化差异"
echo ""
echo "单独运行测试:"
echo "  - Mem0: ./run_mem0_test.sh"
echo "  - AgentMem: ./run_agentmem_test.sh"
echo ""
