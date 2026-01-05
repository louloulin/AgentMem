#!/bin/bash

# ============================================================================
# AgentMem 批量文档删除脚本
# 目的: 删除冗余、过时、低质量的文档，保持项目文档清洁
# ============================================================================

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 统计变量
total_deleted=0
total_size_saved=0

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

log_delete() {
    echo -e "${RED}[DELETE]${NC} $1"
}

# 获取文件大小（字节）
get_file_size() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        stat -f%z "$1" 2>/dev/null || echo 0
    else
        stat -c%s "$1" 2>/dev/null || echo 0
    fi
}

# 格式化文件大小
format_size() {
    local bytes=$1
    if [[ $bytes -lt 1024 ]]; then
        echo "${bytes}B"
    elif [[ $bytes -lt 1048576 ]]; then
        echo "$((bytes / 1024))KB"
    else
        echo "$((bytes / 1048576))MB"
    fi
}

# ============================================================================
# 删除规则
# ============================================================================

# 规则1: 删除极小的Markdown文档（< 300 bytes）
delete_very_small_files() {
    log_info "规则1: 删除极小文档 (< 300 bytes)..."

    local excluded_patterns=(
        "README.md"
        "QUICKSTART.md"
        "CHANGELOG.md"
    )

    while IFS= read -r file; do
        # 检查是否在排除列表中
        local should_exclude=false
        for pattern in "${excluded_patterns[@]}"; do
            if [[ "$(basename "$file")" == "$pattern" ]]; then
                should_exclude=true
                break
            fi
        done

        if [[ "$should_exclude" == "true" ]]; then
            continue
        fi

        local size=$(get_file_size "$file")
        log_delete "$(format_size $size) - $file"
        rm "$file"
        ((total_deleted++))
        ((total_size_saved += size))
    done < <(find . -name "*.md" -type f -size -300c)

    log_success "删除极小文档完成"
}

# 规则2: 删除重复的验证报告（保留最新10个）
delete_duplicate_verification_reports() {
    log_info "规则2: 删除旧验证报告（保留最新10个)..."

    local reports=()
    while IFS= read -r file; do
        reports+=("$file")
    done < <(find . -name "*verification_report*" -o -name "*VERIFICATION*" -o -name "*验证报告*" | sort -r)

    if [[ ${#reports[@]} -gt 10 ]]; then
        local to_delete=$(( ${#reports[@]} - 10 ))
        for ((i=10; i<${#reports[@]}; i++)); do
            local file="${reports[$i]}"
            local size=$(get_file_size "$file")
            log_delete "$(format_size $size) - $file"
            rm "$file"
            ((total_deleted++))
            ((total_size_saved += size))
        done
    fi

    log_success "删除旧验证报告完成"
}

# 规则3: 删除重复的实施总结（保留最新5个）
delete_duplicate_implementation_summaries() {
    log_info "规则3: 删除旧实施总结（保留最新5个)..."

    local reports=()
    while IFS= read -r file; do
        reports+=("$file")
    done < <(find . -name "*IMPLEMENTATION*" -o -name "*实施总结*" | sort -r)

    if [[ ${#reports[@]} -gt 5 ]]; then
        local to_delete=$(( ${#reports[@]} - 5 ))
        for ((i=5; i<${#reports[@]}; i++)); do
            local file="${reports[$i]}"
            local size=$(get_file_size "$file")
            log_delete "$(format_size $size) - $file"
            rm "$file"
            ((total_deleted++))
            ((total_size_saved += size))
        done
    fi

    log_success "删除旧实施总结完成"
}

# 规则4: 删除临时和草稿文件
delete_temporary_files() {
    log_info "规则4: 删除临时和草稿文件..."

    local temp_patterns=(
        "*tmp*.md"
        "*temp*.md"
        "*draft*.md"
        "*草稿*.md"
        "*test*.md"
        "*TEST*.md"
        "*old*.md"
        "*backup*.md"
        "*bak*.md"
    )

    for pattern in "${temp_patterns[@]}"; do
        while IFS= read -r file; do
            local size=$(get_file_size "$file")
            log_delete "$(format_size $size) - $file"
            rm "$file"
            ((total_deleted++))
            ((total_size_saved += size))
        done < <(find . -name "$pattern" -type f 2>/dev/null)
    done

    log_success "删除临时文件完成"
}

# 规则5: 删除过时的分析文档（agentx系列，保留主要版本）
delete_outdated_analysis() {
    log_info "规则5: 删除过时的agentx分析文档..."

    # 这些是已经被整合到最终文档的旧分析
    local outdated_files=(
        "./docs/archive/agentx1.md"
        "./docs/archive/agentx2.md"
        "./docs/archive/ag1.md"
        "./docs/archive/ag25.md"
    )

    for file in "${outdated_files[@]}"; do
        if [[ -f "$file" ]]; then
            local size=$(get_file_size "$file")
            log_delete "$(format_size $size) - $file"
            rm "$file"
            ((total_deleted++))
            ((total_size_saved += size))
        fi
    done

    log_success "删除过时分析文档完成"
}

# 规则6: 删除空的或近空的目录
delete_empty_directories() {
    log_info "规则6: 删除空目录..."

    while IFS= read -r dir; do
        log_delete "空目录: $dir"
        rmdir "$dir" 2>/dev/null || true
    done < <(find . -type d -empty 2>/dev/null)

    log_success "删除空目录完成"
}

# ============================================================================
# 主函数
# ============================================================================

main() {
    echo "============================================================================"
    echo "  AgentMem 批量文档删除脚本"
    echo "============================================================================"
    echo ""

    # 检查是否在项目根目录
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "请在项目根目录运行此脚本"
        exit 1
    fi

    # 执行删除规则
    delete_very_small_files
    echo ""

    delete_duplicate_verification_reports
    echo ""

    delete_duplicate_implementation_summaries
    echo ""

    delete_temporary_files
    echo ""

    delete_outdated_analysis
    echo ""

    delete_empty_directories
    echo ""

    # 打印统计信息
    echo "============================================================================"
    log_success "批量删除完成！"
    echo "============================================================================"
    echo ""
    echo "📊 删除统计:"
    echo "   - 删除文件数: $total_deleted"
    echo "   - 释放空间: $(format_size $total_size_saved)"
    echo ""
    echo "✅ 项目文档更加清洁了！"
    echo ""
}

# 运行主函数
main "$@"
