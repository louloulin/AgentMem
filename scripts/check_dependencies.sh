#!/bin/bash
# 检查项目依赖和工具

MISSING=0

echo "必需工具:"
for cmd in rustc cargo just node npm; do
    if command -v "$cmd" > /dev/null 2>&1; then
        VER=$("$cmd" --version 2>/dev/null | head -1)
        echo "  ✅ $cmd: $VER"
    else
        echo "  ❌ $cmd: 未安装"
        MISSING=$((MISSING + 1))
    fi
done

echo ""
echo "可选工具:"
for cmd in jq curl docker docker-compose; do
    if command -v "$cmd" > /dev/null 2>&1; then
        VER=$("$cmd" --version 2>/dev/null | head -1)
        echo "  ✅ $cmd: $VER"
    else
        echo "  ⚠️  $cmd: 未安装（可选）"
    fi
done

echo ""
if [ $MISSING -gt 0 ]; then
    echo "❌ 缺少 $MISSING 个必需工具，请先安装"
    exit 1
else
    echo "✅ 所有必需工具已安装"
    exit 0
fi


