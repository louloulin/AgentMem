#!/bin/bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

LOG_FILE="/tmp/git_commit_log_$(date +%s).txt"

{
    echo "=== Git 提交流程 ==="
    echo ""
    date
    echo ""
    
    echo "1. 检查状态..."
    git status --short
    echo ""
    
    echo "2. 添加所有更改..."
    git add -A
    echo "✅ 添加完成"
    echo ""
    
    echo "3. 提交更改..."
    git commit -m "chore: 清理临时提交脚本文件

清理和整理提交过程中创建的临时脚本文件:
- commit_and_push.sh (已清空)
- commit_changes.sh
- commit_now.py
- do_commit.py
- final_commit.sh
- git_commit_final.py
- simple_commit.sh
- check_and_commit.py
- commit_with_log.sh (本脚本)

这些文件是在解决终端输出问题时创建的辅助脚本。"
    
    if [ $? -eq 0 ]; then
        echo "✅ 提交成功"
    else
        echo "⚠️ 提交失败或没有更改"
    fi
    echo ""
    
    echo "4. 最后一次提交:"
    git log -1 --oneline
    echo ""
    
    echo "5. 推送到远程..."
    git push origin feature-prod1
    
    if [ $? -eq 0 ]; then
        echo "✅ 推送成功"
    else
        echo "❌ 推送失败"
    fi
    echo ""
    
    echo "=== 完成 ==="
    date
    
} > "$LOG_FILE" 2>&1

echo "日志已保存到: $LOG_FILE"
cat "$LOG_FILE"

