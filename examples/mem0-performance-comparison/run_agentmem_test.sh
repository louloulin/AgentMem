#!/bin/bash
# 运行 AgentMem 性能测试

echo "============================================================"
echo "AgentMem 性能基准测试"
echo "============================================================"

cd "$(dirname "$0")"

# 检查是否已编译
if [ ! -f "./target/release/agentmem_benchmark" ]; then
    echo "正在编译 AgentMem 测试..."
    cargo build --bin agentmem_benchmark --release
    if [ $? -ne 0 ]; then
        echo "❌ 编译失败"
        exit 1
    fi
fi

# 运行测试
echo ""
echo "运行 AgentMem 性能测试..."
echo ""
./target/release/agentmem_benchmark

echo ""
echo "============================================================"
echo "测试完成！"
echo "============================================================"
