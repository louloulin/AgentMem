#!/bin/bash
# AgentMem 恢复脚本
# 版本: 1.0
# 作者: AgentMem Team
# 描述: 从备份恢复 PostgreSQL 数据库、Redis 数据、Qdrant 向量数据和配置文件

set -euo pipefail

# ============================================================================
# 配置
# ============================================================================

# 数据库配置
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-agentmem}"
DB_USER="${DB_USER:-agentmem}"
DB_PASSWORD="${DB_PASSWORD:-}"

# Redis 配置
REDIS_HOST="${REDIS_HOST:-localhost}"
REDIS_PORT="${REDIS_PORT:-6379}"
REDIS_PASSWORD="${REDIS_PASSWORD:-}"
REDIS_DATA_DIR="${REDIS_DATA_DIR:-/var/lib/redis}"

# Qdrant 配置
QDRANT_DATA_DIR="${QDRANT_DATA_DIR:-/var/lib/qdrant}"

# 配置文件路径
CONFIG_DIR="${CONFIG_DIR:-/opt/agentmem/config}"

# 日志配置
LOG_DIR="${LOG_DIR:-/var/log/agentmem}"
LOG_FILE="$LOG_DIR/restore.log"

# ============================================================================
# 函数
# ============================================================================

log() {
    local level="${1:-INFO}"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

confirm_action() {
    local prompt="$1"
    local response
    
    read -p "⚠️  $prompt (yes/no): " response
    if [ "$response" != "yes" ]; then
        log "INFO" "Operation cancelled by user"
        exit 0
    fi
}

restore_database() {
    local backup_file="$1"

    log "INFO" "Restoring PostgreSQL database from $backup_file..."

    if [ ! -f "$backup_file" ]; then
        log "ERROR" "Backup file not found: $backup_file"
        return 1
    fi

    # 检查数据库连接
    if ! PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c '\q' 2>/dev/null; then
        log "ERROR" "Cannot connect to PostgreSQL server"
        return 1
    fi

    # 解压并恢复
    if [[ "$backup_file" == *.gz ]]; then
        log "INFO" "Decompressing and restoring database..."
        gunzip -c "$backup_file" | PGPASSWORD="$DB_PASSWORD" pg_restore \
            -h "$DB_HOST" \
            -p "$DB_PORT" \
            -U "$DB_USER" \
            -d "$DB_NAME" \
            --clean \
            --if-exists \
            --no-owner \
            --no-acl 2>&1 | tee -a "$LOG_FILE"
    else
        log "INFO" "Restoring database..."
        PGPASSWORD="$DB_PASSWORD" pg_restore \
            -h "$DB_HOST" \
            -p "$DB_PORT" \
            -U "$DB_USER" \
            -d "$DB_NAME" \
            --clean \
            --if-exists \
            --no-owner \
            --no-acl \
            "$backup_file" 2>&1 | tee -a "$LOG_FILE"
    fi

    if [ "${PIPESTATUS[0]}" -eq 0 ] || [ "${PIPESTATUS[1]}" -eq 0 ]; then
        log "INFO" "Database restore completed successfully"
        return 0
    else
        log "ERROR" "Database restore failed"
        return 1
    fi
}

restore_redis() {
    local backup_file="$1"

    log "INFO" "Restoring Redis from $backup_file..."

    if [ ! -f "$backup_file" ]; then
        log "ERROR" "Backup file not found: $backup_file"
        return 1
    fi

    # 停止 Redis（如果使用 systemd）
    if systemctl is-active --quiet redis-server 2>/dev/null; then
        log "INFO" "Stopping Redis service..."
        sudo systemctl stop redis-server
    elif systemctl is-active --quiet redis 2>/dev/null; then
        log "INFO" "Stopping Redis service..."
        sudo systemctl stop redis
    fi

    # 复制备份文件
    log "INFO" "Copying Redis backup to data directory..."
    sudo cp "$backup_file" "$REDIS_DATA_DIR/dump.rdb"
    sudo chown redis:redis "$REDIS_DATA_DIR/dump.rdb" 2>/dev/null || true

    # 启动 Redis
    if command -v systemctl &> /dev/null; then
        log "INFO" "Starting Redis service..."
        if systemctl list-unit-files | grep -q redis-server; then
            sudo systemctl start redis-server
        elif systemctl list-unit-files | grep -q redis; then
            sudo systemctl start redis
        fi
    fi

    log "INFO" "Redis restore completed"
    return 0
}

restore_qdrant() {
    local backup_file="$1"

    log "INFO" "Restoring Qdrant from $backup_file..."

    if [ ! -f "$backup_file" ]; then
        log "ERROR" "Backup file not found: $backup_file"
        return 1
    fi

    # 停止 Qdrant（如果使用 Docker）
    if docker ps | grep -q qdrant; then
        log "INFO" "Stopping Qdrant container..."
        docker stop qdrant 2>/dev/null || true
    fi

    # 备份现有数据
    if [ -d "$QDRANT_DATA_DIR" ]; then
        local backup_existing="$QDRANT_DATA_DIR.backup.$(date +%Y%m%d_%H%M%S)"
        log "INFO" "Backing up existing Qdrant data to $backup_existing..."
        sudo mv "$QDRANT_DATA_DIR" "$backup_existing"
    fi

    # 解压备份
    log "INFO" "Extracting Qdrant backup..."
    sudo mkdir -p "$(dirname "$QDRANT_DATA_DIR")"
    sudo tar -xzf "$backup_file" -C "$(dirname "$QDRANT_DATA_DIR")" 2>&1 | tee -a "$LOG_FILE"

    if [ "${PIPESTATUS[0]}" -eq 0 ]; then
        log "INFO" "Qdrant restore completed successfully"
        
        # 重启 Qdrant
        if docker ps -a | grep -q qdrant; then
            log "INFO" "Starting Qdrant container..."
            docker start qdrant 2>/dev/null || true
        fi
        
        return 0
    else
        log "ERROR" "Qdrant restore failed"
        return 1
    fi
}

restore_config() {
    local backup_file="$1"
    local target_dir="${2:-$CONFIG_DIR}"

    log "INFO" "Restoring configuration from $backup_file to $target_dir..."

    if [ ! -f "$backup_file" ]; then
        log "ERROR" "Backup file not found: $backup_file"
        return 1
    fi

    # 备份现有配置
    if [ -d "$target_dir" ]; then
        local backup_existing="$target_dir.backup.$(date +%Y%m%d_%H%M%S)"
        log "INFO" "Backing up existing config to $backup_existing..."
        sudo mv "$target_dir" "$backup_existing"
    fi

    # 解压配置
    log "INFO" "Extracting configuration backup..."
    sudo mkdir -p "$(dirname "$target_dir")"
    sudo tar -xzf "$backup_file" -C "$(dirname "$target_dir")" 2>&1 | tee -a "$LOG_FILE"

    if [ "${PIPESTATUS[0]}" -eq 0 ]; then
        log "INFO" "Configuration restore completed successfully"
        return 0
    else
        log "ERROR" "Configuration restore failed"
        return 1
    fi
}

verify_restore() {
    log "INFO" "Verifying restore..."

    local all_ok=true

    # 验证数据库连接
    if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c '\q' 2>/dev/null; then
        log "INFO" "✓ Database connection OK"
    else
        log "ERROR" "✗ Database connection failed"
        all_ok=false
    fi

    # 验证数据库表
    local table_count=$(PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>/dev/null | tr -d ' ')
    if [ -n "$table_count" ] && [ "$table_count" -gt 0 ]; then
        log "INFO" "✓ Database has $table_count tables"
    else
        log "WARN" "⚠ Database has no tables or count failed"
    fi

    # 验证 Redis 连接
    if redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" ${REDIS_PASSWORD:+-a "$REDIS_PASSWORD"} ping 2>/dev/null | grep -q PONG; then
        log "INFO" "✓ Redis connection OK"
    else
        log "WARN" "⚠ Redis connection failed (may not be critical)"
    fi

    if [ "$all_ok" = true ]; then
        log "INFO" "✓ Restore verification passed"
        return 0
    else
        log "WARN" "⚠ Restore verification completed with warnings"
        return 0
    fi
}

show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Restore AgentMem from backup files.

OPTIONS:
    --db <file>         Database backup file (required)
    --redis <file>      Redis backup file (optional)
    --qdrant <file>     Qdrant backup file (optional)
    --config <file>     Configuration backup file (optional)
    --all <backup_id>   Restore all components from backup ID (e.g., 20250103_120000)
    --help              Show this help message

EXAMPLES:
    # Restore only database
    $0 --db /backups/agentmem/db_20250103_120000.dump.gz

    # Restore database and config
    $0 --db /backups/agentmem/db_20250103_120000.dump.gz \\
       --config /backups/agentmem/config_20250103_120000.tar.gz

    # Restore all components from a backup ID
    $0 --all 20250103_120000

ENVIRONMENT VARIABLES:
    DB_HOST             PostgreSQL host (default: localhost)
    DB_PORT             PostgreSQL port (default: 5432)
    DB_NAME             Database name (default: agentmem)
    DB_USER             Database user (default: agentmem)
    DB_PASSWORD         Database password
    REDIS_HOST          Redis host (default: localhost)
    REDIS_PORT          Redis port (default: 6379)

EOF
}

# ============================================================================
# 主流程
# ============================================================================

main() {
    local db_backup=""
    local redis_backup=""
    local qdrant_backup=""
    local config_backup=""
    local backup_id=""
    local restore_success=true

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --db)
                db_backup="$2"
                shift 2
                ;;
            --redis)
                redis_backup="$2"
                shift 2
                ;;
            --qdrant)
                qdrant_backup="$2"
                shift 2
                ;;
            --config)
                config_backup="$2"
                shift 2
                ;;
            --all)
                backup_id="$2"
                shift 2
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # 如果指定了 backup_id，自动查找备份文件
    if [ -n "$backup_id" ]; then
        local backup_dir="/backups/agentmem"
        db_backup="$backup_dir/db_${backup_id}.dump.gz"
        redis_backup="$backup_dir/redis_${backup_id}.rdb"
        qdrant_backup="$backup_dir/qdrant_${backup_id}.tar.gz"
        config_backup="$backup_dir/config_${backup_id}.tar.gz"
        
        log "INFO" "Using backup ID: $backup_id"
        log "INFO" "Backup directory: $backup_dir"
    fi

    # 检查是否至少指定了数据库备份
    if [ -z "$db_backup" ]; then
        echo "Error: Database backup file is required"
        show_usage
        exit 1
    fi

    # 创建日志目录
    mkdir -p "$LOG_DIR"

    log "INFO" "========================================="
    log "INFO" "AgentMem Restore Started"
    log "INFO" "Date: $(date)"
    log "INFO" "========================================="

    # 显示恢复计划
    log "INFO" "Restore plan:"
    [ -n "$db_backup" ] && log "INFO" "  - Database: $db_backup"
    [ -n "$redis_backup" ] && [ -f "$redis_backup" ] && log "INFO" "  - Redis: $redis_backup"
    [ -n "$qdrant_backup" ] && [ -f "$qdrant_backup" ] && log "INFO" "  - Qdrant: $qdrant_backup"
    [ -n "$config_backup" ] && [ -f "$config_backup" ] && log "INFO" "  - Config: $config_backup"

    # 确认操作
    confirm_action "This will overwrite existing data. Continue?"

    # 1. 恢复数据库
    if [ -n "$db_backup" ]; then
        if ! restore_database "$db_backup"; then
            restore_success=false
        fi
    fi

    # 2. 恢复 Redis
    if [ -n "$redis_backup" ] && [ -f "$redis_backup" ]; then
        if ! restore_redis "$redis_backup"; then
            log "WARN" "Redis restore failed, continuing..."
        fi
    fi

    # 3. 恢复 Qdrant
    if [ -n "$qdrant_backup" ] && [ -f "$qdrant_backup" ]; then
        if ! restore_qdrant "$qdrant_backup"; then
            log "WARN" "Qdrant restore failed, continuing..."
        fi
    fi

    # 4. 恢复配置
    if [ -n "$config_backup" ] && [ -f "$config_backup" ]; then
        if ! restore_config "$config_backup"; then
            log "WARN" "Config restore failed, continuing..."
        fi
    fi

    # 5. 验证恢复
    verify_restore

    # 完成
    if [ "$restore_success" = true ]; then
        log "INFO" "========================================="
        log "INFO" "AgentMem Restore Completed Successfully"
        log "INFO" "========================================="
        log "INFO" ""
        log "INFO" "Next steps:"
        log "INFO" "  1. Review the restore log: $LOG_FILE"
        log "INFO" "  2. Restart AgentMem services:"
        log "INFO" "     docker-compose restart"
        log "INFO" "  3. Verify application functionality"
        log "INFO" ""
        exit 0
    else
        log "ERROR" "========================================="
        log "ERROR" "AgentMem Restore Completed with Errors"
        log "ERROR" "========================================="
        log "ERROR" "Please check the log file: $LOG_FILE"
        exit 1
    fi
}

# 运行主流程
main "$@"

