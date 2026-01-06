#!/bin/bash
# 清理冗余文档脚本

set -e

echo "开始清理冗余文档..."

# 统计清理前
echo "=== 清理前统计 ==="
echo "Archive 文档数: $(find docs/archive -name "*.md" | wc -l)"
echo ""

# 1. 删除明显的重复文档（保留最新版本）
echo "1. 删除重复的最终版本报告..."
# 保留最新的，删除旧的
find docs/archive -name "*最终*.md" -type f | sort | head -n -20 | xargs rm -f 2>/dev/null || true
echo "已删除临时最终版本报告"

# 2. 删除临时测试文件
echo "2. 删除临时测试文件..."
find docs/archive -name "*test*.md" -type f | grep -i "temp\|tmp\|temporary" | xargs rm -f 2>/dev/null || true
echo "已删除临时测试文件"

# 3. 删除空的或接近空的文档
echo "3. 删除空文档..."
find docs/archive -name "*.md" -type f -size -100c | xargs rm -f 2>/dev/null || true
echo "已删除空文档（< 100 bytes）"

# 4. 删除明显的草稿文件（已通过 organize_docs 移动）
echo "4. 草稿文件已在 archive 中"

# 5. 合并重复的主题报告
echo "5. 分析重复主题..."
# 这里可以添加更复杂的逻辑来识别和合并重复内容

# 统计清理后
echo ""
echo "=== 清理后统计 ==="
echo "Archive 文档数: $(find docs/archive -name "*.md" | wc -l)"
echo ""

# 按类型统计
echo "=== 文档类型统计 ==="
echo "SUMMARY 文档: $(find docs/archive -name "*SUMMARY*.md" | wc -l)"
echo "REPORT 文档: $(find docs/archive -name "*REPORT*.md" | wc -l)"
echo "FINAL 文档: $(find docs/archive -name "*FINAL*.md" | wc -l)"
echo "ANALYSIS 文档: $(find docs/archive -name "*ANALYSIS*.md" | wc -l)"
echo ""

echo "冗余文档清理完成！"
