#!/bin/bash

# AgentMem 日志和脚本清理脚本

set -e
cd "$(dirname "$0")"

echo "=========================================="
echo "🧹 AgentMem 日志和脚本清理"
echo "=========================================="
echo ""

# 创建归档目录
mkdir -p logs/archived
mkdir -p scripts/archived

LOGS_MOVED=0
SCRIPTS_KEPT=0
SCRIPTS_ARCHIVED=0

echo "1️⃣  归档日志文件..."
echo ""

# 移动所有日志文件到归档目录
for log in *.log; do
    [ -f "$log" ] || continue
    mv "$log" logs/archived/
    echo "  📦 归档: $log"
    LOGS_MOVED=$((LOGS_MOVED + 1))
done

echo ""
echo "2️⃣  整理脚本文件..."
echo ""

# 保留的核心启动脚本
KEEP_SCRIPTS=(
    "start_server_with_correct_onnx.sh"
    "start_full_stack.sh"
    "start_server_no_auth.sh"
    "organize_docs_simple.sh"
    "quick-start.sh"
)

# 检查并标记保留的脚本
for script in *.sh; do
    [ -f "$script" ] || continue
    
    # 检查是否在保留列表中
    KEEP=0
    for keep_script in "${KEEP_SCRIPTS[@]}"; do
        if [ "$script" = "$keep_script" ]; then
            KEEP=1
            echo "  ✅ 保留: $script"
            SCRIPTS_KEPT=$((SCRIPTS_KEPT + 1))
            break
        fi
    done
    
    # 归档不需要的脚本
    if [ $KEEP -eq 0 ]; then
        mv "$script" scripts/archived/
        echo "  📦 归档: $script"
        SCRIPTS_ARCHIVED=$((SCRIPTS_ARCHIVED + 1))
    fi
done

echo ""
echo "=========================================="
echo "📊 清理统计"
echo "=========================================="
echo ""
echo "日志文件归档: $LOGS_MOVED 个"
echo "脚本保留: $SCRIPTS_KEPT 个"
echo "脚本归档: $SCRIPTS_ARCHIVED 个"
echo ""
echo "归档位置:"
echo "  - logs/archived/"
echo "  - scripts/archived/"
echo ""
echo "=========================================="
echo "✅ 清理完成！"
echo "=========================================="
echo ""
echo "保留的核心脚本:"
for script in "${KEEP_SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        echo "  - $script"
    fi
done
echo ""
