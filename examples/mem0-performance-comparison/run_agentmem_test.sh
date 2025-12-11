#!/bin/bash
# 运行 AgentMem 性能测试

echo "============================================================"
echo "AgentMem 性能基准测试"
echo "============================================================"

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "错误: 未找到 cargo"
    exit 1
fi

# 编译（如果需要）
if [ ! -f "target/release/agentmem_benchmark" ]; then
    echo "正在编译 AgentMem 测试..."
    cargo build --bin agentmem_benchmark --release
    if [ $? -ne 0 ]; then
        echo "编译失败"
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
