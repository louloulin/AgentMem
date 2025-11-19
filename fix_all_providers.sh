#!/bin/bash
# 统一修复所有 provider 的 generate_stream 实现

set -e
cd "$(dirname "$0")"

echo "🔧 统一修复所有 provider 的 generate_stream..."

# 找到所有需要修复的 provider 文件
PROVIDERS=$(find crates/agent-mem-llm/src/providers -name "*.rs" -type f)

for file in $PROVIDERS; do
    if grep -q "generate_stream" "$file"; then
        echo "📝 处理: $file"
        
        # 1. 确保有 Pin 导入（如果没有的话）
        if ! grep -q "use std::pin::Pin" "$file"; then
            # 在第一个 use std:: 后添加 Pin 导入
            if grep -q "^use std::" "$file"; then
                sed -i '' '/^use std::/a\
use std::pin::Pin;
' "$file"
                echo "   ✓ 添加 Pin 导入"
            fi
        fi
        
        # 2. 修复函数签名：Box<dyn Stream + Unpin> -> Pin<Box<dyn Stream + Send>>
        sed -i '' 's/Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>>/Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>/g' "$file"
        sed -i '' 's/Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>>/Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>>/g' "$file"
        
        # 3. 修复返回语句：Ok(Box::new(...)) -> Ok(Box::pin(...))
        sed -i '' 's/Ok(Box::new(stream))/Ok(Box::pin(stream))/g' "$file"
        sed -i '' 's/Ok(Box::new(Box::pin(stream)))/Ok(Box::pin(stream))/g' "$file"
        sed -i '' 's/Ok(Box::new(stream::empty()))/Ok(Box::pin(stream::empty()))/g' "$file"
        
        echo "   ✅ 完成"
    fi
done

echo ""
echo "🎉 所有 provider 修复完成！"
echo ""
echo "🧹 清理重复的 Pin 导入..."

# 清理可能的重复 Pin 导入
for file in $PROVIDERS; do
    if [ $(grep -c "use std::pin::Pin" "$file" 2>/dev/null || echo 0) -gt 1 ]; then
        echo "📝 清理: $file"
        # 保留第一个，删除后续的
        awk '!seen[$0]++ || !/use std::pin::Pin/' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
    fi
done

echo ""
echo "✨ 全部完成！现在可以尝试编译了"
