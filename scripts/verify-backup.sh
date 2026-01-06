#!/bin/bash
# AgentMem 备份验证脚本
# 版本: 1.0
# 作者: AgentMem Team
# 描述: 验证备份文件的完整性和可恢复性

set -euo pipefail

# ============================================================================
# 配置
# ============================================================================

# 日志配置
LOG_DIR="${LOG_DIR:-/var/log/agentmem}"
LOG_FILE="$LOG_DIR/verify-backup.log"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ============================================================================
# 函数
# ============================================================================

log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

print_success() {
    echo -e "${GREEN}✓${NC} $*"
    log "INFO" "✓ $*"
}

print_error() {
    echo -e "${RED}✗${NC} $*"
    log "ERROR" "✗ $*"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $*"
    log "WARN" "⚠ $*"
}

verify_file_exists() {
    local file="$1"
    local description="$2"

    if [ -f "$file" ]; then
        print_success "$description exists: $file"
        return 0
    else
        print_error "$description not found: $file"
        return 1
    fi
}

verify_file_size() {
    local file="$1"
    local min_size="${2:-1024}"  # 默认最小 1KB
    local description="$3"

    if [ ! -f "$file" ]; then
        print_error "$description not found"
        return 1
    fi

    local file_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
    local size_human=$(du -h "$file" | cut -f1)

    if [ "$file_size" -ge "$min_size" ]; then
        print_success "$description size OK: $size_human"
        return 0
    else
        print_error "$description too small: $size_human (minimum: $min_size bytes)"
        return 1
    fi
}

verify_gzip_integrity() {
    local file="$1"
    local description="$2"

    if [ ! -f "$file" ]; then
        print_error "$description not found"
        return 1
    fi

    if gzip -t "$file" 2>/dev/null; then
        print_success "$description gzip integrity OK"
        return 0
    else
        print_error "$description gzip integrity check failed"
        return 1
    fi
}

verify_tar_integrity() {
    local file="$1"
    local description="$2"

    if [ ! -f "$file" ]; then
        print_error "$description not found"
        return 1
    fi

    if tar -tzf "$file" > /dev/null 2>&1; then
        local file_count=$(tar -tzf "$file" 2>/dev/null | wc -l | tr -d ' ')
        print_success "$description tar integrity OK ($file_count files)"
        return 0
    else
        print_error "$description tar integrity check failed"
        return 1
    fi
}

verify_pg_dump() {
    local file="$1"
    local description="$2"

    if [ ! -f "$file" ]; then
        print_error "$description not found"
        return 1
    fi

    # 对于 .gz 文件，先解压再检查
    if [[ "$file" == *.gz ]]; then
        if gunzip -c "$file" 2>/dev/null | head -c 100 | grep -q "PGDMP"; then
            print_success "$description is valid PostgreSQL dump"
            return 0
        else
            print_error "$description is not a valid PostgreSQL dump"
            return 1
        fi
    else
        if head -c 100 "$file" 2>/dev/null | grep -q "PGDMP"; then
            print_success "$description is valid PostgreSQL dump"
            return 0
        else
            print_error "$description is not a valid PostgreSQL dump"
            return 1
        fi
    fi
}

calculate_checksum() {
    local file="$1"
    
    if [ ! -f "$file" ]; then
        echo "N/A"
        return 1
    fi
    
    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        echo "N/A"
    fi
}

verify_backup_set() {
    local backup_id="$1"
    local backup_dir="${2:-/backups/agentmem}"

    echo ""
    echo "========================================="
    echo "Verifying Backup Set: $backup_id"
    echo "========================================="
    echo ""

    local all_ok=true
    local db_file="$backup_dir/db_${backup_id}.dump.gz"
    local redis_file="$backup_dir/redis_${backup_id}.rdb"
    local qdrant_file="$backup_dir/qdrant_${backup_id}.tar.gz"
    local config_file="$backup_dir/config_${backup_id}.tar.gz"
    local manifest_file="$backup_dir/manifest_${backup_id}.txt"

    # 1. 验证数据库备份
    echo "1. Database Backup"
    echo "-------------------"
    if verify_file_exists "$db_file" "Database backup"; then
        verify_file_size "$db_file" 10240 "Database backup" || all_ok=false
        verify_gzip_integrity "$db_file" "Database backup" || all_ok=false
        verify_pg_dump "$db_file" "Database backup" || all_ok=false
        
        local checksum=$(calculate_checksum "$db_file")
        echo "  SHA256: $checksum"
    else
        all_ok=false
    fi
    echo ""

    # 2. 验证 Redis 备份
    echo "2. Redis Backup"
    echo "---------------"
    if [ -f "$redis_file" ]; then
        verify_file_size "$redis_file" 100 "Redis backup" || all_ok=false
        
        # 检查 Redis RDB 文件头
        if head -c 5 "$redis_file" 2>/dev/null | grep -q "REDIS"; then
            print_success "Redis backup format OK"
        else
            print_warning "Redis backup format check inconclusive"
        fi
        
        local checksum=$(calculate_checksum "$redis_file")
        echo "  SHA256: $checksum"
    else
        print_warning "Redis backup not found (optional)"
    fi
    echo ""

    # 3. 验证 Qdrant 备份
    echo "3. Qdrant Backup"
    echo "----------------"
    if [ -f "$qdrant_file" ]; then
        verify_file_size "$qdrant_file" 1024 "Qdrant backup" || all_ok=false
        verify_tar_integrity "$qdrant_file" "Qdrant backup" || all_ok=false
        
        local checksum=$(calculate_checksum "$qdrant_file")
        echo "  SHA256: $checksum"
    else
        print_warning "Qdrant backup not found (optional)"
    fi
    echo ""

    # 4. 验证配置备份
    echo "4. Configuration Backup"
    echo "-----------------------"
    if [ -f "$config_file" ]; then
        verify_file_size "$config_file" 100 "Config backup" || all_ok=false
        verify_tar_integrity "$config_file" "Config backup" || all_ok=false
        
        local checksum=$(calculate_checksum "$config_file")
        echo "  SHA256: $checksum"
    else
        print_warning "Config backup not found (optional)"
    fi
    echo ""

    # 5. 验证清单文件
    echo "5. Manifest File"
    echo "----------------"
    if [ -f "$manifest_file" ]; then
        print_success "Manifest file exists"
        echo "  Content preview:"
        head -n 10 "$manifest_file" | sed 's/^/    /'
    else
        print_warning "Manifest file not found"
    fi
    echo ""

    # 总结
    echo "========================================="
    if [ "$all_ok" = true ]; then
        print_success "Backup verification PASSED"
        echo ""
        echo "This backup set is valid and can be used for restore."
        return 0
    else
        print_error "Backup verification FAILED"
        echo ""
        echo "This backup set has issues and may not be fully restorable."
        return 1
    fi
}

show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] <backup_id_or_file>

Verify AgentMem backup integrity.

OPTIONS:
    --dir <path>        Backup directory (default: /backups/agentmem)
    --help              Show this help message

ARGUMENTS:
    backup_id_or_file   Backup ID (e.g., 20250103_120000) or specific backup file

EXAMPLES:
    # Verify a complete backup set by ID
    $0 20250103_120000

    # Verify a specific backup file
    $0 /backups/agentmem/db_20250103_120000.dump.gz

    # Verify with custom backup directory
    $0 --dir /custom/backup/path 20250103_120000

EOF
}

# ============================================================================
# 主流程
# ============================================================================

main() {
    local backup_dir="/backups/agentmem"
    local target=""

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dir)
                backup_dir="$2"
                shift 2
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                target="$1"
                shift
                ;;
        esac
    done

    if [ -z "$target" ]; then
        echo "Error: Backup ID or file path required"
        show_usage
        exit 1
    fi

    # 创建日志目录
    mkdir -p "$LOG_DIR"

    log "INFO" "========================================="
    log "INFO" "AgentMem Backup Verification Started"
    log "INFO" "Date: $(date)"
    log "INFO" "========================================="

    # 判断是文件还是备份 ID
    if [ -f "$target" ]; then
        # 验证单个文件
        echo "Verifying single file: $target"
        echo ""
        
        local filename=$(basename "$target")
        if [[ "$filename" == *.dump.gz ]] || [[ "$filename" == *.dump ]]; then
            verify_file_exists "$target" "Backup file" && \
            verify_file_size "$target" 10240 "Backup file" && \
            verify_gzip_integrity "$target" "Backup file" && \
            verify_pg_dump "$target" "Backup file"
        elif [[ "$filename" == *.tar.gz ]]; then
            verify_file_exists "$target" "Backup file" && \
            verify_file_size "$target" 1024 "Backup file" && \
            verify_tar_integrity "$target" "Backup file"
        elif [[ "$filename" == *.rdb ]]; then
            verify_file_exists "$target" "Backup file" && \
            verify_file_size "$target" 100 "Backup file"
        else
            print_error "Unknown backup file type: $filename"
            exit 1
        fi
    else
        # 验证完整备份集
        verify_backup_set "$target" "$backup_dir"
    fi

    local exit_code=$?

    log "INFO" "========================================="
    log "INFO" "AgentMem Backup Verification Completed"
    log "INFO" "========================================="

    exit $exit_code
}

# 运行主流程
main "$@"

