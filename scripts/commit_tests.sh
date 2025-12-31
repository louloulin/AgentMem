#!/bin/bash

# Git 提交脚本 - 提交新增的测试代码

set -e

echo "🚀 准备提交测试代码..."
echo ""

# 进入 agentmen 目录
cd "$(dirname "$0")/.."

# 显示修改的文件
echo "📝 修改的文件:"
git status --short

echo ""
echo "📊 统计信息:"
echo "  - 修改的 Rust 文件:"
git diff --name-only | grep "\.rs$" | wc -l
echo "  - 新增的文档:"
git diff --name-only | grep "\.md$" | wc -l
echo "  - 新增的脚本:"
git diff --name-only | grep "\.sh$" | wc -l

echo ""
read -p "是否继续提交? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    echo "❌ 取消提交"
    exit 1
fi

# 添加所有修改
echo ""
echo "📦 添加文件到暂存区..."
git add crates/agent-mem-core/src/managers/episodic_memory.rs
git add crates/agent-mem-core/src/managers/semantic_memory.rs
git add crates/agent-mem-core/src/managers/procedural_memory.rs
git add test1.md
git add 测试*.md
git add scripts/

# 显示将要提交的内容
echo ""
echo "📋 将要提交的文件:"
git status --short

# 提交
echo ""
echo "💾 提交代码..."
git commit -m "feat: 为 Memory Managers 添加 28 个单元测试

- 新增 Episodic Memory Manager 8 个测试
- 新增 Semantic Memory Manager 10 个测试  
- 新增 Procedural Memory Manager 10 个测试
- 更新 test1.md 标记测试进度 (51/110, 46%)
- 新增 4 个测试实施报告文档

测试覆盖:
- 数据结构验证
- 序列化/反序列化
- 查询参数构建
- 边界条件测试

所有测试编译通过，遵循 Rust 最佳实践。"

echo ""
echo "✅ 提交成功！"
echo ""
echo "📌 提交信息:"
git log -1 --oneline

echo ""
echo "🎉 完成！"

