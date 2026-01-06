#!/bin/bash
# 智能清理冗余文档

echo "=== AgentMem 文档智能清理 ==="
echo ""

# 1. 删除小于 500 bytes 的文档（内容过少）
echo "1. 删除极小文档（< 500 bytes）..."
find docs/archive -name "*.md" -type f -size -500c | while read file; do
    echo "  删除: $(basename "$file") ($(wc -c < "$file") bytes)"
    rm -f "$file"
done

# 2. 删除明显的重复最终报告（保留最新的）
echo ""
echo "2. 删除旧版本的最终报告..."
# 找出所有 FINAL/最终 报告，按修改时间排序，保留最新的 5 个
find docs/archive -name "*最终*.md" -o -name "*FINAL*.md" -type f -printf "%T@ %p\n" | sort -rn | tail -n +6 | cut -d' ' -f2- | while read file; do
    echo "  删除旧版: $(basename "$file")"
    rm -f "$file"
done

# 3. 合并相似的实施总结（保留最详细的）
echo ""
echo "3. 删除重复的实施总结..."
# 如果有多个 IMPLEMENTATION_SUMMARY，保留最大的
for pattern in "IMPLEMENTATION_SUMMARY" "EXECUTION_SUMMARY" "COMPLETION_REPORT"; do
    files=$(find docs/archive -name "${pattern}*.md" -type f)
    count=$(echo "$files" | wc -l)
    if [ $count -gt 3 ]; then
        # 保留最大的 3 个
        find docs/archive -name "${pattern}*.md" -type f -exec wc -l {} \; | sort -rn | tail -n +4 | cut -d' ' -f2- | xargs rm -f
        echo "  ${pattern}: 保留最新 3 个，删除其余"
    fi
done

# 4. 删除临时和草稿文件
echo ""
echo "4. 删除临时和草稿文件..."
# 删除明确标记为临时的文件
find docs/archive -name "*tmp*.md" -o -name "*temp*.md" -o -name "*draft*.md" -o -name "*草稿*.md" | xargs rm -f 2>/dev/null
echo "  已删除临时和草稿文件"

# 5. 删除重复的验证报告（保留最新的 10 个）
echo ""
echo "5. 删除旧的验证报告..."
find docs/archive -name "*VERIFICATION*.md" -type f -printf "%T@ %p\n" | sort -rn | tail -n +11 | cut -d' ' -f2- | xargs rm -f 2>/dev/null
echo "  保留最新 10 个验证报告"

# 6. 删除过时的计划文档（保留 ky0.1.md）
echo ""
echo "6. 删除过时的计划文档..."
# 保留主要的改造计划，删除过时的阶段性计划
find docs/plans -name "plan[0-9].*md" -type f | grep -v "ky0.1" | head -10 | xargs rm -f 2>/dev/null
echo "  已删除过时的计划文档"

# 统计
echo ""
echo "=== 清理后统计 ==="
echo "Archive 文档总数: $(find docs/archive -name "*.md" | wc -l | tr -d ' ')"
echo "Reports 文档总数: $(find docs/reports -name "*.md" 2>/dev/null | wc -l | tr -d ' ')"
echo "Plans 文档总数: $(find docs/plans -name "*.md" 2>/dev/null | wc -l | tr -d ' ')"
echo ""

echo "智能清理完成！"
