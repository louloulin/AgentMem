#!/bin/bash
# 清理测试数据脚本

echo "🧹 清理 AgentMem 嵌入式模式测试数据..."

# 清理测试数据目录
if [ -d "test-data" ]; then
    echo "  删除 test-data/ ..."
    rm -rf test-data/
    echo "  ✅ test-data/ 已删除"
else
    echo "  ⚠️  test-data/ 不存在"
fi

# 清理默认向量存储
if [ -d "data" ]; then
    echo "  删除 data/ ..."
    rm -rf data/
    echo "  ✅ data/ 已删除"
else
    echo "  ⚠️  data/ 不存在"
fi

echo ""
echo "✅ 清理完成！"

