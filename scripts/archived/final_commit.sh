#!/bin/bash
set -e

cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

echo "=== 开始提交流程 ==="

# 添加所有更改
git add -A
echo "✅ 已添加所有更改"

# 使用提交消息文件提交
git commit -F COMMIT_MESSAGE.txt
echo "✅ 提交成功"

# 显示最后一次提交
echo ""
echo "最后一次提交:"
git log -1 --oneline

echo ""
echo "=== 提交完成 ==="

