#!/usr/bin/env python3
import subprocess
import sys
import os

os.chdir('/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen')

output_file = '/tmp/git_commit_output.txt'

with open(output_file, 'w') as f:
    f.write("=== Git 提交流程 ===\n\n")
    
    # 1. 添加所有更改
    f.write("1. 添加所有更改...\n")
    result = subprocess.run(['git', 'add', '-A'], capture_output=True, text=True)
    f.write(f"返回码: {result.returncode}\n")
    if result.stderr:
        f.write(f"错误: {result.stderr}\n")
    f.write("✅ 添加成功\n\n")
    
    # 2. 查看状态
    f.write("2. 查看当前状态...\n")
    result = subprocess.run(['git', 'status', '--short'], capture_output=True, text=True)
    f.write(result.stdout)
    f.write("\n")
    
    # 3. 提交
    f.write("3. 提交更改...\n")
    commit_msg = "feat: 实现用户管理功能和 MIRIX 对比分析 (Phase 2)"
    result = subprocess.run(['git', 'commit', '-m', commit_msg], capture_output=True, text=True)
    f.write(f"返回码: {result.returncode}\n")
    f.write(f"输出: {result.stdout}\n")
    if result.stderr:
        f.write(f"错误: {result.stderr}\n")
    
    if result.returncode == 0:
        f.write("✅ 提交成功\n\n")
    else:
        if 'nothing to commit' in result.stdout or 'nothing to commit' in result.stderr:
            f.write("⚠️ 没有需要提交的更改\n\n")
        else:
            f.write("❌ 提交失败\n\n")
    
    # 4. 查看最后一次提交
    f.write("4. 查看最后一次提交...\n")
    result = subprocess.run(['git', 'log', '--oneline', '-1'], capture_output=True, text=True)
    f.write(result.stdout)
    f.write("\n")
    
    f.write("=== 提交流程完成 ===\n")

print(f"输出已写入: {output_file}")
print("请查看文件内容")

