#!/bin/bash

# AgentMem 文档智能整理脚本
# 按照agentmem51.md Task 1计划整理所有文档

set -e
cd "$(dirname "$0")"

echo "=========================================="
echo "📚 AgentMem 文档智能整理"
echo "=========================================="
echo ""

# 创建目标目录结构
mkdir -p docs/reports/{implementation,verification,analysis,progress}
mkdir -p docs/reports/archived/{2025-11-01,2025-11-02,2025-11-03}
mkdir -p docs/guides/{quickstart,troubleshooting,development}
mkdir -p docs/architecture
mkdir -p docs/api
mkdir -p docs/operations

# 统计
TOTAL=0
MOVED=0
SKIPPED=0

echo "1️⃣  分析文档分类..."
echo ""

# 定义文档分类规则
declare -A DOC_CATEGORIES=(
    # 核心文档 (保留在根目录)
    ["README.md"]="root"
    ["CONTRIBUTING.md"]="root"
    ["agentmem51.md"]="root"
    ["agentmem50.md"]="root"
    ["QUICK_REFERENCE.md"]="root"
    
    # 实施报告 -> docs/reports/implementation/
    ["*IMPLEMENTATION*.md"]="docs/reports/implementation"
    ["*COMPLETE*.md"]="docs/reports/implementation"
    ["*FIX*.md"]="docs/reports/implementation"
    
    # 验证报告 -> docs/reports/verification/
    ["*VERIFICATION*.md"]="docs/reports/verification"
    ["*VALIDATION*.md"]="docs/reports/verification"
    ["*TEST*.md"]="docs/reports/verification"
    
    # 分析报告 -> docs/reports/analysis/
    ["*ANALYSIS*.md"]="docs/reports/analysis"
    ["*SUMMARY*.md"]="docs/reports/analysis"
    
    # 进度报告 -> docs/reports/progress/
    ["*PROGRESS*.md"]="docs/reports/progress"
    ["*STATUS*.md"]="docs/reports/progress"
    ["PHASE*.md"]="docs/reports/progress"
    ["P0*.md"]="docs/reports/progress"
    
    # 架构文档 -> docs/architecture/
    ["*ARCHITECTURE*.md"]="docs/architecture"
    ["*DESIGN*.md"]="docs/architecture"
    ["*ROADMAP*.md"]="docs/architecture"
    
    # 快速指南 -> docs/guides/quickstart/
    ["*QUICK*.md"]="docs/guides/quickstart"
    ["*START*.md"]="docs/guides/quickstart"
    
    # 历史文档 -> docs/reports/archived/
    ["agentmem3*.md"]="docs/reports/archived/2025-11-01"
    ["agentmem4*.md"]="docs/reports/archived/2025-11-02"
    ["*2025_11_01*.md"]="docs/reports/archived/2025-11-01"
    ["*2025_11_02*.md"]="docs/reports/archived/2025-11-02"
    ["*20251027*.md"]="docs/reports/archived/2025-11-01"
    ["*20251101*.md"]="docs/reports/archived/2025-11-01"
)

# 移动文件函数
move_file() {
    local file=$1
    local dest_dir=$2
    local basename=$(basename "$file")
    
    if [ "$dest_dir" = "root" ]; then
        echo "  ⏭️  保留: $basename (核心文档)"
        ((SKIPPED++))
        return
    fi
    
    if [ -f "$dest_dir/$basename" ]; then
        echo "  ⚠️  跳过: $basename (目标已存在)"
        ((SKIPPED++))
        return
    fi
    
    mv "$file" "$dest_dir/"
    echo "  ✅ 移动: $basename -> $dest_dir/"
    ((MOVED++))
}

# 处理所有md文件
echo "2️⃣  开始整理文档..."
echo ""

for md_file in *.md; do
    [ -f "$md_file" ] || continue
    ((TOTAL++))
    
    # 检查是否是核心文档
    if [[ "${DOC_CATEGORIES[$md_file]}" == "root" ]]; then
        SKIPPED=$((SKIPPED + 1))
        continue
    fi
    
    # 根据文件名模式分类
    MOVED_FLAG=0
    
    # 实施报告
    if [[ $md_file == *"IMPLEMENTATION"* ]] || \
       [[ $md_file == *"COMPLETE"* ]] || \
       [[ $md_file == *"FIX"* ]]; then
        move_file "$md_file" "docs/reports/implementation"
        MOVED_FLAG=1
    
    # 验证报告
    elif [[ $md_file == *"VERIFICATION"* ]] || \
         [[ $md_file == *"VALIDATION"* ]] || \
         [[ $md_file == *"TEST"* ]]; then
        move_file "$md_file" "docs/reports/verification"
        MOVED_FLAG=1
    
    # 分析报告
    elif [[ $md_file == *"ANALYSIS"* ]] || \
         [[ $md_file == *"SUMMARY"* ]]; then
        move_file "$md_file" "docs/reports/analysis"
        MOVED_FLAG=1
    
    # 进度报告
    elif [[ $md_file == *"PROGRESS"* ]] || \
         [[ $md_file == *"STATUS"* ]] || \
         [[ $md_file == PHASE*.md ]] || \
         [[ $md_file == P0*.md ]] || \
         [[ $md_file == *"TASK"*.md ]]; then
        move_file "$md_file" "docs/reports/progress"
        MOVED_FLAG=1
    
    # 架构文档
    elif [[ $md_file == *"ARCHITECTURE"* ]] || \
         [[ $md_file == *"DESIGN"* ]] || \
         [[ $md_file == *"ROADMAP"* ]]; then
        move_file "$md_file" "docs/architecture"
        MOVED_FLAG=1
    
    # 快速指南
    elif [[ $md_file == *"QUICK"* ]] || \
         [[ $md_file == *"START"* ]]; then
        move_file "$md_file" "docs/guides/quickstart"
        MOVED_FLAG=1
    
    # 历史文档
    elif [[ $md_file == agentmem3*.md ]] || \
         [[ $md_file == agentmem4*.md ]]; then
        move_file "$md_file" "docs/reports/archived/2025-11-01"
        MOVED_FLAG=1
    
    elif [[ $md_file == *"2025_11_01"* ]] || \
         [[ $md_file == *"20251027"* ]] || \
         [[ $md_file == *"20251101"* ]]; then
        move_file "$md_file" "docs/reports/archived/2025-11-01"
        MOVED_FLAG=1
    
    elif [[ $md_file == *"2025_11_02"* ]]; then
        move_file "$md_file" "docs/reports/archived/2025-11-02"
        MOVED_FLAG=1
    
    elif [[ $md_file == *"2025_11_03"* ]]; then
        move_file "$md_file" "docs/reports/archived/2025-11-03"
        MOVED_FLAG=1
    
    # 其他文档 -> 分析报告
    else
        move_file "$md_file" "docs/reports/analysis"
        MOVED_FLAG=1
    fi
done

echo ""
echo "3️⃣  生成文档索引..."
echo ""

# 生成索引
cat > docs/ORGANIZED_INDEX.md << 'INDEXEOF'
# AgentMem 文档组织索引

**生成时间**: $(date +%Y-%m-%d)  
**文档总数**: 统计中...

---

## 📁 文档结构

### 根目录 (核心文档)
- `README.md` - 项目主文档
- `CONTRIBUTING.md` - 贡献指南
- `agentmem51.md` - 生产就绪度评估 (最新)
- `agentmem50.md` - 技术完整度分析
- `QUICK_REFERENCE.md` - 快速参考指南

### 📊 reports/ (报告)
#### implementation/ (实施报告)
INDEXEOF

# 统计各目录文件
find docs/reports/implementation -name "*.md" -type f | wc -l | xargs printf "- %d 个实施报告\n" >> docs/ORGANIZED_INDEX.md

cat >> docs/ORGANIZED_INDEX.md << 'INDEXEOF'

#### verification/ (验证报告)
INDEXEOF

find docs/reports/verification -name "*.md" -type f | wc -l | xargs printf "- %d 个验证报告\n" >> docs/ORGANIZED_INDEX.md

cat >> docs/ORGANIZED_INDEX.md << 'INDEXEOF'

#### analysis/ (分析报告)
INDEXEOF

find docs/reports/analysis -name "*.md" -type f | wc -l | xargs printf "- %d 个分析报告\n" >> docs/ORGANIZED_INDEX.md

cat >> docs/ORGANIZED_INDEX.md << 'INDEXEOF'

#### progress/ (进度报告)
INDEXEOF

find docs/reports/progress -name "*.md" -type f | wc -l | xargs printf "- %d 个进度报告\n" >> docs/ORGANIZED_INDEX.md

cat >> docs/ORGANIZED_INDEX.md << 'INDEXEOF'

#### archived/ (历史文档)
INDEXEOF

find docs/reports/archived -name "*.md" -type f | wc -l | xargs printf "- %d 个历史文档\n" >> docs/ORGANIZED_INDEX.md

cat >> docs/ORGANIZED_INDEX.md << 'INDEXEOF'

### 🏗️ architecture/ (架构文档)
INDEXEOF

find docs/architecture -name "*.md" -type f | wc -l | xargs printf "- %d 个架构文档\n" >> docs/ORGANIZED_INDEX.md

cat >> docs/ORGANIZED_INDEX.md << 'INDEXEOF'

### 📖 guides/ (指南)
INDEXEOF

find docs/guides -name "*.md" -type f | wc -l | xargs printf "- %d 个指南文档\n" >> docs/ORGANIZED_INDEX.md

cat >> docs/ORGANIZED_INDEX.md << 'INDEXEOF'

---

## 🔍 查找文档

### 按类型
- 实施报告: `docs/reports/implementation/`
- 验证报告: `docs/reports/verification/`
- 分析报告: `docs/reports/analysis/`
- 进度报告: `docs/reports/progress/`
- 历史文档: `docs/reports/archived/`
- 架构设计: `docs/architecture/`
- 使用指南: `docs/guides/`

### 按日期
- 2025-11-01: `docs/reports/archived/2025-11-01/`
- 2025-11-02: `docs/reports/archived/2025-11-02/`
- 2025-11-03: `docs/reports/archived/2025-11-03/`

---

**注**: 此索引由 `organize_all_docs.sh` 自动生成
INDEXEOF

echo "✅ 索引已生成: docs/ORGANIZED_INDEX.md"
echo ""

echo "=========================================="
echo "📊 整理统计"
echo "=========================================="
echo ""
echo "总文件数: $TOTAL"
echo "已移动: $MOVED"
echo "已跳过: $SKIPPED"
echo ""
echo "📁 新文档结构:"
echo ""
tree -L 3 -d docs/ 2>/dev/null || find docs -type d | sed 's|[^/]*/|  |g'
echo ""
echo "=========================================="
echo "✅ 文档整理完成！"
echo "=========================================="
echo ""
echo "查看索引: cat docs/ORGANIZED_INDEX.md"
echo "查看根目录: ls -la *.md"
echo ""
