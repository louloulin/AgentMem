#!/bin/bash

# 简化版文档整理脚本

cd "$(dirname "$0")"

echo "=========================================="
echo "📚 AgentMem 文档整理"
echo "=========================================="
echo ""

# 创建目录
mkdir -p docs/reports/implementation
mkdir -p docs/reports/verification
mkdir -p docs/reports/analysis
mkdir -p docs/reports/progress
mkdir -p docs/reports/archived
mkdir -p docs/architecture
mkdir -p docs/guides

MOVED=0
KEPT=0

echo "开始整理..."
echo ""

# 保留核心文档
CORE_DOCS="README.md CONTRIBUTING.md agentmem51.md agentmem50.md QUICK_REFERENCE.md"

for md in *.md; do
    [ -f "$md" ] || continue
    
    # 检查是否是核心文档
    KEEP=0
    for core in $CORE_DOCS; do
        if [ "$md" = "$core" ]; then
            KEEP=1
            echo "✅ 保留: $md"
            KEPT=$((KEPT + 1))
            break
        fi
    done
    
    [ $KEEP -eq 1 ] && continue
    
    # 分类移动
    case "$md" in
        *IMPLEMENTATION*|*COMPLETE*|*FIX*|*REPORT*)
            mv "$md" docs/reports/implementation/ 2>/dev/null && echo "  📄 $md -> implementation/" && MOVED=$((MOVED + 1))
            ;;
        *VERIFICATION*|*VALIDATION*|*TEST*)
            mv "$md" docs/reports/verification/ 2>/dev/null && echo "  🧪 $md -> verification/" && MOVED=$((MOVED + 1))
            ;;
        *ANALYSIS*|*SUMMARY*)
            mv "$md" docs/reports/analysis/ 2>/dev/null && echo "  📊 $md -> analysis/" && MOVED=$((MOVED + 1))
            ;;
        *PROGRESS*|*STATUS*|PHASE*|P0*|*TASK*)
            mv "$md" docs/reports/progress/ 2>/dev/null && echo "  📈 $md -> progress/" && MOVED=$((MOVED + 1))
            ;;
        *ARCHITECTURE*|*DESIGN*|*ROADMAP*)
            mv "$md" docs/architecture/ 2>/dev/null && echo "  🏗️  $md -> architecture/" && MOVED=$((MOVED + 1))
            ;;
        *QUICK*|*START*|*GUIDE*)
            mv "$md" docs/guides/ 2>/dev/null && echo "  📖 $md -> guides/" && MOVED=$((MOVED + 1))
            ;;
        agentmem3*|agentmem4*|*2025_11_0*|*202510*)
            mv "$md" docs/reports/archived/ 2>/dev/null && echo "  📦 $md -> archived/" && MOVED=$((MOVED + 1))
            ;;
        *)
            mv "$md" docs/reports/analysis/ 2>/dev/null && echo "  📄 $md -> analysis/" && MOVED=$((MOVED + 1))
            ;;
    esac
done

echo ""
echo "=========================================="
echo "统计:"
echo "  移动: $MOVED 个文件"
echo "  保留: $KEPT 个核心文档"
echo "=========================================="
echo ""

# 列出根目录剩余的md文件
echo "根目录核心文档:"
ls -1 *.md 2>/dev/null | sed 's/^/  - /'

echo ""
echo "✅ 完成！"
