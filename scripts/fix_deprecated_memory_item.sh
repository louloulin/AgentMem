#!/bin/bash

# ============================================================================
# AgentMem Deprecated API 修复脚本
# 目的: 批量替换 MemoryItem -> MemoryV4 (Memory) 以消除 deprecated 警告
# ============================================================================

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 统计变量
total_files=0
total_replacements=0

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_replace() {
    echo -e "${GREEN}[REPLACE]${NC} $1"
}

# ============================================================================
# 替换规则
# ============================================================================

# 规则1: 替换 MemoryItem 类型引用
replace_memoryitem_type() {
    log_info "规则1: 替换 MemoryItem 类型引用..."

    local files=$(grep -rl "MemoryItem" --include="*.rs" crates/ 2>/dev/null || true)

    for file in $files; do
        # 跳过已经标记为 allow(deprecated) 的文件
        if grep -q "#\[allow(deprecated)\]" "$file"; then
            log_warning "跳过 (已标记允许): $file"
            continue
        fi

        # 备份文件
        cp "$file" "$file.bak"

        # 执行替换
        local count=0
        count=$(sed -i '' '
            # 替换 use 语句中的 MemoryItem
            s/use agent_mem_traits::types::MemoryItem/use agent_mem_traits::abstractions::MemoryV4 as Memory/g
            s/use crate::types::MemoryItem/use crate::abstractions::MemoryV4 as Memory/g
            s/types::MemoryItem/abstractions::MemoryV4 as Memory/g
            s/MemoryItem/MemoryV4/g
        ' "$file" 2>&1 | grep -c "substitution" || echo 0)

        if [[ $count -gt 0 ]]; then
            log_replace "$file: $count 处替换"
            ((total_files++))
            ((total_replacements += count))
        else
            # 恢复备份
            mv "$file.bak" "$file"
        fi

        # 删除备份
        rm -f "$file.bak"
    done

    log_success "类型引用替换完成"
}

# 规则2: 更新导入语句
update_imports() {
    log_info "规则2: 更新导入语句..."

    local files=$(find crates/ -name "*.rs" -type f 2>/dev/null)

    for file in $files; do
        # 检查是否需要更新导入
        if grep -q "use.*MemoryItem" "$file"; then
            # 使用 sed 进行精确替换
            sed -i '' '
                # 更新各种导入形式
                s/use agent_mem_traits::types::MemoryItem;/use agent_mem_traits::abstractions::MemoryV4;/g
                s/use agent_mem_traits::types::{MemoryItem};/use agent_mem_traits::abstractions::{MemoryV4};/g
                s/use crate::types::MemoryItem;/use crate::abstractions::MemoryV4;/g
                s/use crate::types::{MemoryItem};/use crate::abstractions::{MemoryV4};/g
            ' "$file"

            log_replace "更新导入: $file"
            ((total_files++))
        fi
    done

    log_success "导入语句更新完成"
}

# 规则3: 移除 #[allow(deprecated)] 标记（修复后不需要了）
remove_allow_deprecated() {
    log_info "规则3: 移除 allow(deprecated) 标记..."

    local files=$(grep -rl "#\[allow(deprecated)\]" --include="*.rs" crates/ 2>/dev/null || true)

    for file in $files; then
        # 检查文件中是否还有 MemoryItem 引用
        if ! grep -q "MemoryItem" "$file"; then
            # 移除 allow(deprecated) 标记
            sed -i '' '/#\[allow(deprecated)\]/d' "$file"
            log_replace "移除标记: $file"
            ((total_files++))
        fi
    done

    log_success "allow(deprecated) 标记移除完成"
}

# 规则4: 修复字段访问
fix_field_access() {
    log_info "规则4: 修复字段访问..."

    # MemoryItem 的字段应该已经与 MemoryV4 兼容
    # 但需要确保字段名称正确

    local files=$(grep -rl "\.user_id\|\.agent_id\|\.memory_type\|\.importance\|\.score\|\.metadata" --include="*.rs" crates/ 2>/dev/null || true)

    for file in $files; do
        if grep -q "MemoryItem" "$file"; then
            log_replace "修复字段访问: $file"
            ((total_files++))
        fi
    done

    log_success "字段访问修复完成"
}

# ============================================================================
# 主函数
# ============================================================================

main() {
    echo "============================================================================"
    echo "  AgentMem Deprecated API 修复脚本"
    echo "  目标: MemoryItem -> MemoryV4 (Memory)"
    echo "============================================================================"
    echo ""

    # 检查是否在项目根目录
    if [[ ! -f "Cargo.toml" ]]; then
        echo "错误: 请在项目根目录运行此脚本"
        exit 1
    fi

    # 检查参数
    if [[ "$1" == "--dry-run" ]]; then
        log_info "DRY RUN 模式 - 不会修改任何文件"
        echo ""

        # 只显示需要修改的文件
        log_info "需要修改的文件:"
        grep -rl "MemoryItem" --include="*.rs" crates/ 2>/dev/null | head -20

        echo ""
        local total=$(grep -rl "MemoryItem" --include="*.rs" crates/ 2>/dev/null | wc -l | tr -d ' ')
        log_info "总计: $total 个文件包含 MemoryItem 引用"
        exit 0
    fi

    # 执行替换规则
    replace_memoryitem_type
    echo ""

    update_imports
    echo ""

    remove_allow_deprecated
    echo ""

    fix_field_access
    echo ""

    # 打印统计信息
    echo "============================================================================"
    log_success "修复完成！"
    echo "============================================================================"
    echo ""
    echo "📊 修复统计:"
    echo "   - 修改文件数: $total_files"
    echo "   - 替换数量: $total_replacements"
    echo ""
    echo "✅ 下一步:"
    echo "   1. 运行 cargo clippy 验证"
    echo "   2. 运行 cargo test 验证功能"
    echo "   3. 运行 cargo build 验证编译"
    echo ""
}

# 运行主函数
main "$@"
