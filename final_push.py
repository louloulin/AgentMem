#!/usr/bin/env python3
import subprocess
import os

os.chdir('/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen')

def run_cmd(cmd):
    """运行命令并返回结果"""
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.returncode, result.stdout, result.stderr

# 检查状态
print("检查 Git 状态...")
code, out, err = run_cmd('git status --short')
print(f"状态输出: {out if out else '(空)'}")

# 添加所有文件
print("\n添加所有更改...")
code, out, err = run_cmd('git add -A')
print(f"返回码: {code}")

# 提交
print("\n提交更改...")
commit_msg = "chore: 清理临时提交脚本文件"
code, out, err = run_cmd(f'git commit -m "{commit_msg}"')
print(f"返回码: {code}")
print(f"输出: {out}")
if err:
    print(f"错误: {err}")

# 查看最后一次提交
print("\n最后一次提交:")
code, out, err = run_cmd('git log -1 --oneline')
print(out)

# 推送
print("\n推送到远程...")
code, out, err = run_cmd('git push origin feature-prod1')
print(f"返回码: {code}")
if out:
    print(f"输出: {out}")
if err:
    print(f"信息: {err}")

print("\n完成!")

