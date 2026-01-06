#!/bin/bash
# AgentMem 文档整理脚本

set -e

echo "开始整理 AgentMem 项目文档..."

# 进入项目根目录
cd "$(dirname "$0")/.."

# 创建文档目录结构
echo "创建文档目录结构..."
mkdir -p docs/{user,developer,community,guides,reports,plans,integration,performance,docker,bugfix,archive}

# 核心文档（保留在根目录）
echo "核心文档将保留在根目录"
CORE_DOCS=(
    "README.md"
    "README_ARCHITECTURE.md"
    "README_API_对比测试.md"
    "README_第二阶段_CN.md"
    "CONTRIBUTING.md"
    "CHANGELOG.md"
    "SECURITY.md"
    "CODE_OF_CONDUCT.md"
    "QUICKSTART.md"
    "TROUBLESHOOTING.md"
    "LICENSE"
    "ky0.1.md"
)

# 移动用户指南
echo "移动用户指南..."
for file in *guide*.md *GUIDE*.md; do
    if [[ -f "$file" ]]; then
        # 保留核心的 TROUBLESHOOTING.md
        if [[ "$file" != "TROUBLESHOOTING.md" ]]; then
            mv "$file" docs/guides/ 2>/dev/null || true
        fi
    fi
done

# 移动所有报告（按关键字匹配）
echo "移动报告类文档..."
for keyword in ANALYSIS REPORT SUMMARY VERIFICATION COMPLETE FINAL ACHIEVEMENT COMPLETION EXECUTION PROGRESS; do
    for file in *${keyword}*.md; do
        if [[ -f "$file" ]]; then
            # 检查是否是核心文档
            is_core=false
            for core_doc in "${CORE_DOCS[@]}"; do
                if [[ "$file" == "$core_doc" ]]; then
                    is_core=true
                    break
                fi
            done

            if [[ "$is_core" == "false" ]]; then
                mv "$file" docs/reports/ 2>/dev/null || true
            fi
        fi
    done
done

# 移动计划和阶段文档
echo "移动计划类文档..."
mv plan*.md docs/plans/ 2>/dev/null || true
mv PHASE*.md docs/plans/ 2>/dev/null || true
mv phase*.md docs/plans/ 2>/dev/null || true
mv AGENTMEM_ACTION_PLAN*.md docs/plans/ 2>/dev/null || true
mv COMPREHENSIVE_REFORM*.md docs/plans/ 2>/dev/null || true

# 移动集成文档
echo "移动集成文档..."
mv LUMOSAI*.md docs/integration/ 2>/dev/null || true
mv MAAS*.md docs/integration/ 2>/dev/null || true
mv 华为MAAS*.md docs/integration/ 2>/dev/null || true
mv HUAWEI*.md docs/integration/ 2>/dev/null || true

# 移动性能文档
echo "移动性能分析文档..."
mv PERFORMANCE*.md docs/performance/ 2>/dev/null || true
mv OPTIMIZATION*.md docs/performance/ 2>/dev/null || true
mv perf*.md docs/performance/ 2>/dev/null || true
mv *性能*.md docs/performance/ 2>/dev/null || true
mv V2性能*.md docs/performance/ 2>/dev/null || true
mv V3优化*.md docs/performance/ 2>/dev/null || true

# 移动 Docker 文档
echo "移动 Docker 文档..."
mv DOCKER*.md docs/docker/ 2>/dev/null || true

# 移动 Bug 修复文档
echo "移动 Bug 修复文档..."
mv BUG*.md docs/bugfix/ 2>/dev/null || true
mv FIX*.md docs/bugfix/ 2>/dev/null || true
mv ROOT_CAUSE*.md docs/bugfix/ 2>/dev/null || true

# 移动草稿和临时文档
echo "移动草稿文档到 archive..."
mv ag*.md docs/archive/ 2>/dev/null || true
mv agent*.md docs/archive/ 2>/dev/null || true
mv agentx*.md docs/archive/ 2>/dev/null || true
mv agentmem*.md docs/archive/ 2>/dev/null || true
mv chat*.md docs/archive/ 2>/dev/null || true
mv ppt*.md docs/archive/ 2>/dev/null || true
mv pp*.md docs/archive/ 2>/dev/null || true
mv store*.md docs/archive/ 2>/dev/null || true
mv value*.md docs/archive/ 2>/dev/null || true
mv context*.md docs/archive/ 2>/dev/null || true
mv agetmem*.md docs/archive/ 2>/dev/null || true
mv mem0*.md docs/archive/ 2>/dev/null || true
mv LOCOMO*.md docs/archive/ 2>/dev/null || true

# 移动剩余的杂项文档（排除核心文档）
echo "移动剩余杂项文档..."
for file in *.md; do
    if [[ -f "$file" ]]; then
        # 检查是否是核心文档
        is_core=false
        for core_doc in "${CORE_DOCS[@]}"; do
            if [[ "$file" == "$core_doc" ]]; then
                is_core=true
                break
            fi
        done

        # 如果不是核心文档，移动到 archive
        if [[ "$is_core" == "false" ]]; then
            mv "$file" docs/archive/ 2>/dev/null || true
        fi
    fi
done

echo ""
echo "文档整理完成！"
echo ""
echo "根目录保留的核心文档："
ls -1 *.md 2>/dev/null | while read file; do
    echo "  - $file"
done
echo ""
echo "文档分类统计："
echo "  用户指南: $(ls -1 docs/guides/*.md 2>/dev/null | wc -l) 个"
echo "  报告: $(ls -1 docs/reports/*.md 2>/dev/null | wc -l) 个"
echo "  计划: $(ls -1 docs/plans/*.md 2>/dev/null | wc -l) 个"
echo "  集成: $(ls -1 docs/integration/*.md 2>/dev/null | wc -l) 个"
echo "  性能: $(ls -1 docs/performance/*.md 2>/dev/null | wc -l) 个"
echo "  Docker: $(ls -1 docs/docker/*.md 2>/dev/null | wc -l) 个"
echo "  Bug修复: $(ls -1 docs/bugfix/*.md 2>/dev/null | wc -l) 个"
echo "  归档: $(ls -1 docs/archive/*.md 2>/dev/null | wc -l) 个"
