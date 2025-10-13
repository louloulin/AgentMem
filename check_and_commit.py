#!/usr/bin/env python3
"""检查并提交所有未提交的更改"""
import subprocess
import os
import sys

os.chdir('/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen')

print("=== 检查 Git 状态 ===\n")

# 1. 检查状态
result = subprocess.run(['git', 'status', '--short'], capture_output=True, text=True)
status_output = result.stdout.strip()

if not status_output:
    print("✅ 工作区干净，没有未提交的更改\n")
    print("最近的提交:")
    result = subprocess.run(['git', 'log', '-3', '--oneline'], capture_output=True, text=True)
    print(result.stdout)
    sys.exit(0)

print("📝 发现未提交的更改:\n")
print(status_output)
print()

# 2. 添加所有更改
print("添加所有更改...")
result = subprocess.run(['git', 'add', '-A'], capture_output=True, text=True)
if result.returncode == 0:
    print("✅ 添加成功\n")
else:
    print(f"❌ 添加失败: {result.stderr}\n")
    sys.exit(1)

# 3. 提交
commit_msg = """chore: 清理临时提交脚本文件

清理和整理提交过程中创建的临时脚本文件:
- commit_and_push.sh (已清空)
- commit_changes.sh
- commit_now.py
- do_commit.py
- final_commit.sh
- git_commit_final.py
- simple_commit.sh
- check_and_commit.py (本脚本)

这些文件是在解决终端输出问题时创建的辅助脚本。
"""

print("提交更改...")
result = subprocess.run(['git', 'commit', '-m', commit_msg], capture_output=True, text=True)

if result.returncode == 0:
    print("✅ 提交成功\n")
    print("提交信息:")
    result = subprocess.run(['git', 'log', '-1', '--oneline'], capture_output=True, text=True)
    print(result.stdout)
elif 'nothing to commit' in result.stdout or 'nothing to commit' in result.stderr:
    print("⚠️ 没有需要提交的更改\n")
else:
    print(f"❌ 提交失败: {result.stderr}\n")
    sys.exit(1)

# 4. 推送
print("\n推送到远程仓库...")
result = subprocess.run(['git', 'push', 'origin', 'feature-prod1'], capture_output=True, text=True)

if result.returncode == 0:
    print("✅ 推送成功\n")
    print(result.stdout)
else:
    print(f"❌ 推送失败: {result.stderr}\n")
    sys.exit(1)

print("\n=== 完成 ===")

