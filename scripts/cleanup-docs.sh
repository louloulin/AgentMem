#!/bin/bash
# AgentMem 文档清理脚本
# 将根目录的 Markdown 文件移动到 docs/ 相应目录

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "🧹 AgentMem 文档清理"
echo "========================================="
echo ""

# 统计当前根目录的 MD 文件数量
ROOT_MD_COUNT=$(find . -maxdepth 1 -name "*.md" -type f | wc -l | tr -d ' ')
echo "📊 当前根目录 Markdown 文件数量: $ROOT_MD_COUNT"
echo ""

# 创建必要的目录
echo "📁 创建目录结构..."
mkdir -p docs/getting-started
mkdir -p docs/guides
mkdir -p docs/architecture
mkdir -p docs/api
mkdir -p docs/development
mkdir -p docs/deployment
mkdir -p docs/operations
mkdir -p docs/reports/2025-11
mkdir -p docs/archive/legacy
mkdir -p docs/archive/notes
mkdir -p docs/archive/reports

# 定义要保留在根目录的文件
KEEP_IN_ROOT=(
    "README.md"
    "CONTRIBUTING.md"
    "LICENSE"
    "TROUBLESHOOTING.md"
    "CHANGELOG.md"
)

# 移动快速开始文档
echo ""
echo "🚀 移动快速开始文档..."
mv -v QUICK_START_PLUGINS.md docs/getting-started/plugins-quickstart.md 2>/dev/null || true
mv -v QUICK_START_SEARCH.md docs/getting-started/search-quickstart.md 2>/dev/null || true
mv -v QUICK_REFERENCE.md docs/getting-started/quick-reference.md 2>/dev/null || true
mv -v CLAUDE_CODE_QUICKSTART.md docs/getting-started/claude-code-quickstart.md 2>/dev/null || true
mv -v START_CLAUDE_CODE.md docs/getting-started/start-claude-code.md 2>/dev/null || true

# 移动用户指南
echo ""
echo "📖 移动用户指南..."
mv -v AGENTMEM_USER_GUIDE.md docs/guides/user-guide.md 2>/dev/null || true
mv -v DEPLOYMENT.md docs/guides/deployment-guide.md 2>/dev/null || true
mv -v HOW_TO_USE_AGENTMEM_IN_CLAUDE.md docs/guides/claude-integration.md 2>/dev/null || true
mv -v JUSTFILE_GUIDE.md docs/guides/justfile-guide.md 2>/dev/null || true
mv -v UI_TESTING_GUIDE.md docs/guides/ui-testing.md 2>/dev/null || true
mv -v VERIFICATION_GUIDE.md docs/guides/verification.md 2>/dev/null || true

# 移动架构文档
echo ""
echo "🏗️  移动架构文档..."
mv -v AGENTMEM_FINAL_ARCHITECTURE.md docs/architecture/final-architecture.md 2>/dev/null || true
mv -v AGENTMEM_TECHNICAL_DOCUMENTATION.md docs/architecture/technical-documentation.md 2>/dev/null || true
mv -v README_AGENTMEM_ARCHITECTURE_V3.md docs/architecture/architecture-v3.md 2>/dev/null || true
mv -v COMPREHENSIVE_MEMORY_ARCHITECTURE_ANALYSIS.md docs/architecture/memory-architecture-analysis.md 2>/dev/null || true
mv -v ARCHITECTURE_ANALYSIS_COMPLETE.md docs/architecture/analysis-complete.md 2>/dev/null || true
mv -v ARCHITECTURE_ANALYSIS_COMPREHENSIVE.md docs/architecture/analysis-comprehensive.md 2>/dev/null || true

# 移动 API 文档
echo ""
echo "🔌 移动 API 文档..."
mv -v MEMORY_API_COMPARATIVE_ANALYSIS.md docs/api/memory-api-comparison.md 2>/dev/null || true
mv -v CLAUDE_CODE_MCP_COMPLETE_GUIDE.md docs/api/mcp-complete-guide.md 2>/dev/null || true
mv -v CORRECT_MCP_COMMANDS.md docs/api/mcp-commands.md 2>/dev/null || true
mv -v REAL_CLAUDE_COMMANDS.md docs/api/claude-commands.md 2>/dev/null || true

# 移动开发文档
echo ""
echo "💻 移动开发文档..."
mv -v BUILD_IMPROVEMENTS.md docs/development/build-improvements.md 2>/dev/null || true
mv -v ISSUE_ANALYSIS.md docs/development/issue-analysis.md 2>/dev/null || true
mv -v COMPILATION_FIX_REPORT.md docs/development/compilation-fix.md 2>/dev/null || true
mv -v EMBEDDER_FIX_REPORT.md docs/development/embedder-fix.md 2>/dev/null || true

# 移动实施报告到 2025-11
echo ""
echo "📊 移动实施报告..."
for file in *_REPORT.md *_SUMMARY.md *_FIX*.md *_COMPLETE*.md *_VERIFICATION*.md; do
    if [ -f "$file" ]; then
        # 转换为小写并替换下划线
        new_name=$(echo "$file" | tr '[:upper:]' '[:lower:]' | tr '_' '-')
        mv -v "$file" "docs/reports/2025-11/$new_name" 2>/dev/null || true
    fi
done

# 移动临时笔记到归档
echo ""
echo "📝 移动临时笔记..."
mv -v agentmem*.md docs/archive/notes/ 2>/dev/null || true
mv -v claude*.md docs/archive/notes/ 2>/dev/null || true
mv -v bp*.md docs/archive/notes/ 2>/dev/null || true
mv -v cp*.md docs/archive/notes/ 2>/dev/null || true
mv -v x*.md docs/archive/notes/ 2>/dev/null || true
mv -v mcp*.md docs/archive/notes/ 2>/dev/null || true
mv -v plugin*.md docs/archive/notes/ 2>/dev/null || true
mv -v quick*.md docs/archive/notes/ 2>/dev/null || true
mv -v demo*.md docs/archive/notes/ 2>/dev/null || true

# 移动中文文档到归档
echo ""
echo "🇨🇳 移动中文文档..."
mv -v 优先功能实施计划.md docs/archive/legacy/ 2>/dev/null || true
mv -v 实施完成状态.md docs/archive/legacy/ 2>/dev/null || true
mv -v 编译修复报告.md docs/archive/legacy/ 2>/dev/null || true

# 移动验证报告到归档
echo ""
echo "✅ 移动验证报告..."
mv -v verification_report_*.md docs/archive/reports/ 2>/dev/null || true

# 移动所有剩余的大写 MD 文件到归档（除了保留的）
echo ""
echo "🗂️  移动其他文档到归档..."
for file in *.md; do
    if [ -f "$file" ]; then
        # 检查是否在保留列表中
        keep=false
        for keep_file in "${KEEP_IN_ROOT[@]}"; do
            if [ "$file" = "$keep_file" ]; then
                keep=true
                break
            fi
        done
        
        if [ "$keep" = false ]; then
            # 转换为小写并替换下划线
            new_name=$(echo "$file" | tr '[:upper:]' '[:lower:]' | tr '_' '-')
            mv -v "$file" "docs/archive/legacy/$new_name" 2>/dev/null || true
        fi
    fi
done

# 统计清理后的文件数量
echo ""
echo "========================================="
echo "📊 清理统计"
echo "========================================="
ROOT_MD_AFTER=$(find . -maxdepth 1 -name "*.md" -type f | wc -l | tr -d ' ')
MOVED_COUNT=$((ROOT_MD_COUNT - ROOT_MD_AFTER))

echo "清理前根目录 MD 文件: $ROOT_MD_COUNT"
echo "清理后根目录 MD 文件: $ROOT_MD_AFTER"
echo "移动的文件数量: $MOVED_COUNT"
echo ""

# 显示保留的文件
echo "保留在根目录的文件:"
find . -maxdepth 1 -name "*.md" -type f | sort

echo ""
echo "========================================="
echo "✅ 文档清理完成！"
echo "========================================="
echo ""
echo "下一步:"
echo "1. 检查 docs/ 目录结构"
echo "2. 更新文档链接"
echo "3. 创建 CHANGELOG.md"
echo "4. 提交更改"

