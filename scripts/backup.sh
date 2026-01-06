#!/bin/bash
# AgentMem 生产级备份脚本
# 版本: 1.0
# 作者: AgentMem Team
# 描述: 自动备份 PostgreSQL 数据库、Redis 数据、Qdrant 向量数据和配置文件

set -euo pipefail

# ============================================================================
# 配置
# ============================================================================

# 备份目录
BACKUP_DIR="${BACKUP_DIR:-/backups/agentmem}"
BACKUP_RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"

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

# Qdrant 配置
QDRANT_HOST="${QDRANT_HOST:-localhost}"
QDRANT_PORT="${QDRANT_PORT:-6333}"
QDRANT_DATA_DIR="${QDRANT_DATA_DIR:-/var/lib/qdrant}"

# 配置文件路径
CONFIG_DIR="${CONFIG_DIR:-/opt/agentmem/config}"

# 日志配置
LOG_DIR="${LOG_DIR:-/var/log/agentmem}"
LOG_FILE="$LOG_DIR/backup.log"

# 通知配置
NOTIFY_EMAIL="${NOTIFY_EMAIL:-}"
NOTIFY_SLACK_WEBHOOK="${NOTIFY_SLACK_WEBHOOK:-}"

# 压缩级别 (1-9, 9 最高压缩率)
COMPRESSION_LEVEL="${COMPRESSION_LEVEL:-6}"

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

notify() {
    local message="$1"

    # Email 通知
    if [ -n "$NOTIFY_EMAIL" ]; then
        echo "$message" | mail -s "AgentMem Backup Notification" "$NOTIFY_EMAIL" 2>/dev/null || true
    fi

    # Slack 通知
    if [ -n "$NOTIFY_SLACK_WEBHOOK" ]; then
        curl -X POST "$NOTIFY_SLACK_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{\"text\": \"$message\"}" \
            --silent --show-error || true
    fi
}

backup_database() {
    local backup_file="$1"

    log "INFO" "Starting PostgreSQL database backup..."

    if [ -z "$DB_PASSWORD" ]; then
        log "WARN" "DB_PASSWORD not set, attempting passwordless connection"
    fi

    PGPASSWORD="$DB_PASSWORD" pg_dump \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        -F c \
        -f "$backup_file" 2>&1 | tee -a "$LOG_FILE"

    if [ "${PIPESTATUS[0]}" -eq 0 ]; then
        local size=$(du -h "$backup_file" | cut -f1)
        log "INFO" "Database backup completed: $backup_file ($size)"
        return 0
    else
        log "ERROR" "Database backup failed"
        return 1
    fi
}

backup_redis() {
    local backup_file="$1"

    log "INFO" "Starting Redis backup..."

    # 触发 Redis BGSAVE
    if [ -n "$REDIS_PASSWORD" ]; then
        redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" -a "$REDIS_PASSWORD" BGSAVE 2>&1 | tee -a "$LOG_FILE"
    else
        redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" BGSAVE 2>&1 | tee -a "$LOG_FILE"
    fi

    # 等待 BGSAVE 完成
    sleep 2

    # 复制 dump.rdb 文件
    if [ -f "/var/lib/redis/dump.rdb" ]; then
        cp /var/lib/redis/dump.rdb "$backup_file"
        local size=$(du -h "$backup_file" | cut -f1)
        log "INFO" "Redis backup completed: $backup_file ($size)"
        return 0
    else
        log "WARN" "Redis dump.rdb not found, skipping Redis backup"
        return 0
    fi
}

backup_qdrant() {
    local backup_file="$1"

    log "INFO" "Starting Qdrant vector database backup..."

    if [ -d "$QDRANT_DATA_DIR" ]; then
        tar -czf "$backup_file" -C "$(dirname "$QDRANT_DATA_DIR")" "$(basename "$QDRANT_DATA_DIR")" 2>&1 | tee -a "$LOG_FILE"
        
        if [ "${PIPESTATUS[0]}" -eq 0 ]; then
            local size=$(du -h "$backup_file" | cut -f1)
            log "INFO" "Qdrant backup completed: $backup_file ($size)"
            return 0
        else
            log "ERROR" "Qdrant backup failed"
            return 1
        fi
    else
        log "WARN" "Qdrant data directory not found: $QDRANT_DATA_DIR"
        return 0
    fi
}

backup_config() {
    local backup_file="$1"

    log "INFO" "Starting configuration backup..."

    if [ -d "$CONFIG_DIR" ]; then
        tar -czf "$backup_file" -C "$(dirname "$CONFIG_DIR")" "$(basename "$CONFIG_DIR")" 2>&1 | tee -a "$LOG_FILE"

        if [ "${PIPESTATUS[0]}" -eq 0 ]; then
            local size=$(du -h "$backup_file" | cut -f1)
            log "INFO" "Configuration backup completed: $backup_file ($size)"
            return 0
        else
            log "ERROR" "Configuration backup failed"
            return 1
        fi
    else
        log "WARN" "Config directory not found: $CONFIG_DIR"
        # 创建空的配置备份
        touch "$backup_file"
        return 0
    fi
}

verify_backup() {
    local backup_file="$1"

    log "INFO" "Verifying backup: $backup_file"

    # 检查文件存在
    if [ ! -f "$backup_file" ]; then
        log "ERROR" "Backup file not found: $backup_file"
        return 1
    fi

    # 检查文件大小
    local file_size=$(stat -f%z "$backup_file" 2>/dev/null || stat -c%s "$backup_file" 2>/dev/null || echo "0")
    if [ "$file_size" -lt 100 ]; then
        log "ERROR" "Backup file too small: $file_size bytes"
        return 1
    fi

    # 检查文件完整性
    if [[ "$backup_file" == *.gz ]]; then
        if gzip -t "$backup_file" 2>&1 | tee -a "$LOG_FILE"; then
            log "INFO" "Backup verification passed: $backup_file"
            return 0
        else
            log "ERROR" "Backup verification failed: $backup_file"
            return 1
        fi
    elif [[ "$backup_file" == *.tar.gz ]]; then
        if tar -tzf "$backup_file" > /dev/null 2>&1; then
            log "INFO" "Backup verification passed: $backup_file"
            return 0
        else
            log "ERROR" "Backup verification failed: $backup_file"
            return 1
        fi
    else
        # 对于其他文件类型，只检查大小
        log "INFO" "Backup verification passed (size check only): $backup_file"
        return 0
    fi
}

cleanup_old_backups() {
    log "INFO" "Cleaning up old backups (retention: $BACKUP_RETENTION_DAYS days)..."

    local deleted_count=0
    
    # 删除旧的备份文件
    while IFS= read -r file; do
        rm -f "$file"
        deleted_count=$((deleted_count + 1))
        log "INFO" "Deleted old backup: $file"
    done < <(find "$BACKUP_DIR" -type f \( -name "*.gz" -o -name "*.dump" -o -name "*.rdb" -o -name "*.txt" \) -mtime +"$BACKUP_RETENTION_DAYS")

    log "INFO" "Cleaned up $deleted_count old backup files"
}

calculate_checksum() {
    local file="$1"
    
    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        echo "N/A"
    fi
}

# ============================================================================
# 主流程
# ============================================================================

main() {
    local date_stamp=$(date +%Y%m%d_%H%M%S)
    local backup_success=true
    local backup_files=()

    log "INFO" "========================================="
    log "INFO" "AgentMem Backup Started"
    log "INFO" "Date: $(date)"
    log "INFO" "========================================="

    # 创建备份目录
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$LOG_DIR"

    # 1. 备份 PostgreSQL 数据库
    local db_backup_file="$BACKUP_DIR/db_$date_stamp.dump"
    if backup_database "$db_backup_file"; then
        # 压缩数据库备份
        log "INFO" "Compressing database backup..."
        gzip -"$COMPRESSION_LEVEL" "$db_backup_file"
        db_backup_file="$db_backup_file.gz"
        
        # 验证数据库备份
        if verify_backup "$db_backup_file"; then
            backup_files+=("$db_backup_file")
        else
            backup_success=false
        fi
    else
        backup_success=false
    fi

    # 2. 备份 Redis
    local redis_backup_file="$BACKUP_DIR/redis_$date_stamp.rdb"
    if backup_redis "$redis_backup_file"; then
        if [ -f "$redis_backup_file" ]; then
            backup_files+=("$redis_backup_file")
        fi
    fi

    # 3. 备份 Qdrant
    local qdrant_backup_file="$BACKUP_DIR/qdrant_$date_stamp.tar.gz"
    if backup_qdrant "$qdrant_backup_file"; then
        if [ -f "$qdrant_backup_file" ] && verify_backup "$qdrant_backup_file"; then
            backup_files+=("$qdrant_backup_file")
        fi
    fi

    # 4. 备份配置
    local config_backup_file="$BACKUP_DIR/config_$date_stamp.tar.gz"
    if backup_config "$config_backup_file"; then
        if [ -f "$config_backup_file" ] && verify_backup "$config_backup_file"; then
            backup_files+=("$config_backup_file")
        fi
    fi

    # 5. 清理旧备份
    cleanup_old_backups

    # 6. 生成备份清单
    local manifest_file="$BACKUP_DIR/manifest_$date_stamp.txt"
    {
        echo "AgentMem Backup Manifest"
        echo "Date: $(date)"
        echo "Backup ID: $date_stamp"
        echo "================================"
        echo ""
        echo "Backup Files:"
        for file in "${backup_files[@]}"; do
            if [ -f "$file" ]; then
                local size=$(du -h "$file" | cut -f1)
                local checksum=$(calculate_checksum "$file")
                echo "  - $(basename "$file")"
                echo "    Size: $size"
                echo "    SHA256: $checksum"
                echo ""
            fi
        done
        echo "================================"
        echo "Total Files: ${#backup_files[@]}"
        echo ""
        echo "Backup Directory Contents:"
        ls -lh "$BACKUP_DIR"/*_"$date_stamp"* 2>/dev/null || echo "  (none)"
    } > "$manifest_file"

    # 7. 发送通知
    if [ "$backup_success" = true ]; then
        log "INFO" "========================================="
        log "INFO" "AgentMem Backup Completed Successfully"
        log "INFO" "Backup ID: $date_stamp"
        log "INFO" "Files backed up: ${#backup_files[@]}"
        log "INFO" "========================================="
        notify "✅ AgentMem backup completed successfully at $(date)
Backup ID: $date_stamp
Files: ${#backup_files[@]}
Location: $BACKUP_DIR"
        exit 0
    else
        log "ERROR" "========================================="
        log "ERROR" "AgentMem Backup Completed with Errors"
        log "ERROR" "========================================="
        notify "❌ AgentMem backup completed with errors at $(date)
Please check logs: $LOG_FILE"
        exit 1
    fi
}

# 运行主流程
main "$@"

