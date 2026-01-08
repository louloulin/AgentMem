#!/bin/bash
# Batch fix test API migrations from Legacy to Memory V4

set -e

echo "=========================================="
echo "AgentMem 2.6 - 批量修复测试 API 迁移"
echo "=========================================="
echo ""

# 找出所有需要修复的 Rust 文件
find crates/agent-mem-core -name "*.rs" -type f | while read file; do
    # 备份文件
    cp "$file" "$file.bak"

    # 修复 1: MemoryBuilder → Memory::new
    sed -i '' 's/MemoryBuilder::new()/Memory::new/g' "$file"

    # 修复 2: .content(Content::Text( → Memory::new 的第四个参数
    # 这个需要更复杂的处理，暂时跳过

    # 修复 3: 移除 .build()
    sed -i '' '/\.build()$/d' "$file"

    # 修复 4: 移除 MemoryBuilder 导入
    sed -i '' '/use.*MemoryBuilder,/d' "$file"
    sed -i '' '/use.*MemoryBuilder/d' "$file"

    # 修复 5: Metadata 导入移除 (V4 不需要)
    sed -i '' '/use agent_mem_traits.*Metadata,/d' "$file"

    # 修复 6: Content 导入移除 (V4 不需要)
    sed -i '' '/use agent_mem_traits.*Content,/d' "$file"
    sed -i '' '/use agent_mem_traits.*Content/d' "$file"

    # 如果文件有变化，输出
    if ! diff -q "$file" "$file.bak" > /dev/null 2>&1; then
        echo "✓ 已修复: $file"
        rm "$file.bak"
    else
        rm "$file.bak"
    fi
done

echo ""
echo "批量修复完成！"
echo "请运行 cargo test 验证修复效果"
