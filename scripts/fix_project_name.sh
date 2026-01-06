#!/bin/bash
# 批量替换项目名称 AgentDB -> AgentMem

set -e

echo "开始批量替换项目名称..."

# 计数器
count=0

# 查找并替换所有包含 AgentDB 的文件
find . -type f \( -name "*.md" -o -name "*.toml" -o -name "*.yml" -o -name "*.yaml" \) -print0 | while IFS= read -r -d '' file; do
    if grep -q -i "agentdb" "$file" 2>/dev/null; then
        echo "处理: $file"
        # 使用 sed 进行替换（不区分大小写）
        sed -i.bak 's/AgentDB/AgentMem/gI; s/AGENTDB/AGENTMEM/gI' "$file"
        rm -f "${file}.bak"
        ((count++)) || true
    fi
done

echo ""
echo "替换完成！"
echo "处理文件数: $count"

# 验证
echo ""
echo "验证结果："
remaining=$(find . -type f \( -name "*.md" -o -name "*.toml" -o -name "*.yml" -o -name "*.yaml" \) -exec grep -l -i "agentdb" {} \; 2>/dev/null | wc -l)
echo "剩余包含 'AgentDB' 的文件数: $remaining"

if [ "$remaining" -eq 0 ]; then
    echo "✅ 所有引用已成功替换！"
else
    echo "⚠️  还有 $remaining 个文件包含 'AgentDB' 引用"
    find . -type f \( -name "*.md" -o -name "*.toml" -o -name "*.yml" -o -name "*.yaml" \) -exec grep -l -i "agentdb" {} \; 2>/dev/null | head -10
fi
