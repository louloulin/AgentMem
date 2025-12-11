#!/bin/bash
# 运行 Mem0 性能测试

echo "============================================================"
echo "Mem0 性能基准测试"
echo "============================================================"

# 检查 Python 环境
if ! command -v python3 &> /dev/null; then
    echo "错误: 未找到 python3"
    exit 1
fi

# 检查 mem0 是否安装
python3 -c "from mem0 import Memory" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "警告: mem0 未安装"
    echo "请运行: pip install mem0"
    echo ""
    echo "是否现在安装? (y/n)"
    read -r answer
    if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
        pip install mem0
        if [ $? -ne 0 ]; then
            echo "安装失败，请手动安装: pip install mem0"
            exit 1
        fi
    else
        echo "请先安装 mem0: pip install mem0"
        exit 1
    fi
fi

# 运行测试
echo ""
echo "运行 Mem0 性能测试..."
echo ""
python3 mem0_benchmark.py

echo ""
echo "============================================================"
echo "测试完成！"
echo "============================================================"
