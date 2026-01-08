#!/bin/bash
# 自动修复 async 测试函数的返回类型
#
# 问题: async 测试函数使用 ? 操作符但没有返回 Result
# 解决: 添加 -> Result<(), Box<dyn std::error::Error>> 返回类型

set -e

echo "=========================================="
echo "AgentMem 2.6 - 修复 async 测试函数"
echo "=========================================="
echo ""

# 找到所有包含 async 测试函数的 Rust 文件
find crates/agent-mem-core -name "*.rs" -type f | while read file; do
    # 检查文件是否包含 async 测试函数且使用了 ? 操作符
    if grep -q "#\[tokio::test\]" "$file" && grep -q "\.await?" "$file"; then
        echo "处理文件: $file"

        # 备份文件
        cp "$file" "$file.bak"

        # 使用 sed 修复每个 async 测试函数
        # 模式: async fn test_name() {
        # 替换为: async fn test_name() -> Result<(), Box<dyn std::error::Error>> {

        # 注意: 这个脚本需要更复杂的逻辑来正确处理
        # 我们使用 Perl 来进行更复杂的文本处理
        perl -i -pe '
            # 在 #[tokio::test] 后面的 async fn 行添加返回类型
            if (/#\[tokio::test\]/ ... /^    \}/) {
                if (/async fn (\w+)\(\) \{/ && !/->/) {
                    s/async fn (\w+)\(\) \{/async fn $1() -> Result<(), Box<dyn std::error::Error>> {/;
                }
            }
        ' "$file" 2>/dev/null || true

        # 如果文件有变化，输出
        if ! diff -q "$file" "$file.bak" > /dev/null 2>&1; then
            echo "  ✓ 已修复: $file"
            rm "$file.bak"
        else
            rm "$file.bak"
        fi
    fi
done

echo ""
echo "修复完成！"
echo "请运行 cargo test 验证"
