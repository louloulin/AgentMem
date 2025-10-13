#!/usr/bin/env python3
import os
import sys

os.chdir('/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen')

print("=== 开始提交流程 ===\n")

# 1. 添加所有更改
print("1. 添加所有更改...")
ret = os.system('git add -A')
if ret == 0:
    print("✅ 添加成功\n")
else:
    print(f"❌ 添加失败 (返回码: {ret})\n")
    sys.exit(1)

# 2. 提交
print("2. 提交更改...")
ret = os.system('git commit -F COMMIT_MESSAGE.txt')
if ret == 0:
    print("✅ 提交成功\n")
elif ret == 256:  # Nothing to commit
    print("⚠️ 没有需要提交的更改\n")
else:
    print(f"❌ 提交失败 (返回码: {ret})\n")

# 3. 显示最后一次提交
print("3. 显示最后一次提交...")
os.system('git log -1 --oneline')

print("\n=== 提交流程完成 ===")

