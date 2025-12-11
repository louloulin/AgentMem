#!/bin/bash
# 运行 Mem0 性能测试

echo "============================================================"
echo "Mem0 性能基准测试"
echo "============================================================"

cd "$(dirname "$0")"

# 检查是否安装了 mem0
python3 -c "import mem0" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ mem0 未安装"
    echo ""
    echo "请先安装 mem0:"
    echo "  pip install mem0"
    echo ""
    echo "或使用虚拟环境:"
    echo "  python3 -m venv venv"
    echo "  source venv/bin/activate  # Linux/Mac"
    echo "  pip install mem0"
    exit 1
fi

echo "✅ mem0 已安装"
echo ""

# 运行测试
echo "运行 Mem0 性能测试..."
echo ""
python3 mem0_simple_benchmark.py

echo ""
echo "============================================================"
echo "测试完成！"
echo "============================================================"
