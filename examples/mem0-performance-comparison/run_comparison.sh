#!/bin/bash
# Mem0 vs AgentMem 性能对比测试脚本

echo "============================================================"
echo "Mem0 vs AgentMem 性能对比测试"
echo "============================================================"

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
python3 mem0_benchmark.py

# 运行 AgentMem 测试
echo ""
echo "============================================================"
echo "运行 AgentMem 测试..."
echo "============================================================"
cargo run --bin agentmem_benchmark --release

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
