#!/bin/bash
# 简单测试日志功能

cd dist/server

echo "========================================="
echo "测试 1: 检查二进制文件"
echo "========================================="
ls -lh agent-mem-server
file agent-mem-server
echo ""

echo "========================================="
echo "测试 2: 运行服务器（前台，5秒后 Ctrl+C）"
echo "========================================="
timeout 5 ./agent-mem-server || echo "超时退出"
echo ""

echo "========================================="
echo "测试 3: 检查日志目录"
echo "========================================="
ls -la logs/ 2>/dev/null || echo "logs 目录不存在"
echo ""

if [ -d "logs" ]; then
    echo "========================================="
    echo "测试 4: 查看日志文件"
    echo "========================================="
    find logs -name "*.log" -type f -exec echo "文件: {}" \; -exec head -20 {} \; -exec echo "" \;
fi

