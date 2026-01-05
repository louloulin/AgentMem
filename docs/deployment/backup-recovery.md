# AgentMem 备份与恢复指南

本指南详细说明如何备份和恢复 AgentMem 系统，包括数据库、缓存、向量数据和配置文件。

**版本**: 1.0  
**更新日期**: 2025-10-03  
**目标读者**: 运维工程师、系统管理员

---

## 📋 目录

1. [概述](#1-概述)
2. [备份策略](#2-备份策略)
3. [备份操作](#3-备份操作)
4. [验证备份](#4-验证备份)
5. [恢复操作](#5-恢复操作)
6. [自动化备份](#6-自动化备份)
7. [故障排除](#7-故障排除)
8. [最佳实践](#8-最佳实践)

---

## 1. 概述

### 1.1 备份内容

AgentMem 备份包含以下组件：

| 组件 | 内容 | 重要性 | 大小估算 |
|------|------|--------|----------|
| **PostgreSQL** | 记忆数据、Agent 信息、用户数据 | 🔴 关键 | 100MB - 10GB |
| **Redis** | 缓存数据、会话信息 | 🟡 重要 | 10MB - 1GB |
| **Qdrant** | 向量嵌入、索引 | 🔴 关键 | 500MB - 50GB |
| **配置文件** | 环境变量、配置文件、证书 | 🟢 可选 | < 10MB |

### 1.2 备份脚本

AgentMem 提供三个备份脚本：

- **`backup.sh`**: 执行完整备份
- **`restore.sh`**: 从备份恢复
- **`verify-backup.sh`**: 验证备份完整性

### 1.3 系统要求

- **磁盘空间**: 至少是数据大小的 2 倍
- **权限**: root 或 sudo 权限
- **工具**: `pg_dump`, `pg_restore`, `redis-cli`, `tar`, `gzip`

---

## 2. 备份策略

### 2.1 备份类型

#### 完整备份（推荐）
- **频率**: 每天
- **内容**: 所有组件
- **保留**: 30 天

#### 增量备份（高级）
- **频率**: 每小时
- **内容**: 仅变更数据
- **保留**: 7 天

### 2.2 保留策略

```
每日备份:  保留 7 天   (7 个备份)
每周备份:  保留 4 周   (4 个备份)
每月备份:  保留 12 个月 (12 个备份)
```

**总存储需求**: 约 23 个备份 × 平均备份大小

### 2.3 备份时间窗口

建议在业务低峰期执行备份：

- **生产环境**: 凌晨 2:00 - 4:00
- **开发环境**: 任意时间
- **测试环境**: 下班后

---

## 3. 备份操作

### 3.1 手动备份

#### 基础用法

```bash
# 执行完整备份
cd /opt/agentmem
sudo ./scripts/backup.sh

# 查看备份日志
tail -f /var/log/agentmem/backup.log
```

#### 自定义配置

```bash
# 设置备份目录
export BACKUP_DIR=/custom/backup/path

# 设置保留天数
export BACKUP_RETENTION_DAYS=60

# 设置数据库连接
export DB_HOST=db.example.com
export DB_PORT=5432
export DB_NAME=agentmem
export DB_USER=agentmem
export DB_PASSWORD=your_password

# 执行备份
./scripts/backup.sh
```

#### 仅备份特定组件

修改 `backup.sh` 脚本，注释掉不需要的部分：

```bash
# 仅备份数据库
# backup_redis "$redis_backup_file"      # 注释掉
# backup_qdrant "$qdrant_backup_file"    # 注释掉
# backup_config "$config_backup_file"    # 注释掉
```

### 3.2 备份输出

成功的备份会生成以下文件：

```
/backups/agentmem/
├── db_20250103_120000.dump.gz          # PostgreSQL 备份
├── redis_20250103_120000.rdb           # Redis 备份
├── qdrant_20250103_120000.tar.gz       # Qdrant 备份
├── config_20250103_120000.tar.gz       # 配置备份
└── manifest_20250103_120000.txt        # 备份清单
```

### 3.3 备份清单示例

```
AgentMem Backup Manifest
Date: 2025-01-03 12:00:00
Backup ID: 20250103_120000
================================

Backup Files:
  - db_20250103_120000.dump.gz
    Size: 245M
    SHA256: a1b2c3d4e5f6...

  - redis_20250103_120000.rdb
    Size: 12M
    SHA256: f6e5d4c3b2a1...

  - qdrant_20250103_120000.tar.gz
    Size: 1.2G
    SHA256: 1a2b3c4d5e6f...

  - config_20250103_120000.tar.gz
    Size: 2.3M
    SHA256: 6f5e4d3c2b1a...

================================
Total Files: 4
```

### 3.4 备份通知

#### Email 通知

```bash
export NOTIFY_EMAIL="ops@example.com"
./scripts/backup.sh
```

#### Slack 通知

```bash
export NOTIFY_SLACK_WEBHOOK="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
./scripts/backup.sh
```

---

## 4. 验证备份

### 4.1 验证完整备份集

```bash
# 验证特定备份 ID
./scripts/verify-backup.sh 20250103_120000

# 预期输出
=========================================
Verifying Backup Set: 20250103_120000
=========================================

1. Database Backup
-------------------
✓ Database backup exists: /backups/agentmem/db_20250103_120000.dump.gz
✓ Database backup size OK: 245M
✓ Database backup gzip integrity OK
✓ Database backup is valid PostgreSQL dump
  SHA256: a1b2c3d4e5f6...

2. Redis Backup
---------------
✓ Redis backup size OK: 12M
✓ Redis backup format OK
  SHA256: f6e5d4c3b2a1...

3. Qdrant Backup
----------------
✓ Qdrant backup size OK: 1.2G
✓ Qdrant backup tar integrity OK (1234 files)
  SHA256: 1a2b3c4d5e6f...

4. Configuration Backup
-----------------------
✓ Config backup size OK: 2.3M
✓ Config backup tar integrity OK (45 files)
  SHA256: 6f5e4d3c2b1a...

5. Manifest File
----------------
✓ Manifest file exists

=========================================
✓ Backup verification PASSED
=========================================
```

### 4.2 验证单个文件

```bash
# 验证数据库备份
./scripts/verify-backup.sh /backups/agentmem/db_20250103_120000.dump.gz

# 验证配置备份
./scripts/verify-backup.sh /backups/agentmem/config_20250103_120000.tar.gz
```

### 4.3 定期验证

建议每周验证一次最新备份：

```bash
# 添加到 crontab
0 3 * * 0 /opt/agentmem/scripts/verify-backup.sh $(ls -t /backups/agentmem/manifest_*.txt | head -1 | sed 's/.*manifest_\(.*\)\.txt/\1/')
```

---

## 5. 恢复操作

### 5.1 完整恢复

#### 使用备份 ID（推荐）

```bash
# 恢复所有组件
./scripts/restore.sh --all 20250103_120000

# 系统会提示确认
⚠️  This will overwrite existing data. Continue? (yes/no): yes

# 恢复完成后重启服务
docker-compose restart
```

#### 指定备份文件

```bash
./scripts/restore.sh \
  --db /backups/agentmem/db_20250103_120000.dump.gz \
  --redis /backups/agentmem/redis_20250103_120000.rdb \
  --qdrant /backups/agentmem/qdrant_20250103_120000.tar.gz \
  --config /backups/agentmem/config_20250103_120000.tar.gz
```

### 5.2 部分恢复

#### 仅恢复数据库

```bash
./scripts/restore.sh --db /backups/agentmem/db_20250103_120000.dump.gz
```

#### 仅恢复配置

```bash
./scripts/restore.sh \
  --db /backups/agentmem/db_20250103_120000.dump.gz \
  --config /backups/agentmem/config_20250103_120000.tar.gz
```

### 5.3 恢复到不同环境

```bash
# 设置目标环境变量
export DB_HOST=new-db-server.example.com
export DB_PORT=5432
export DB_NAME=agentmem_restored
export DB_USER=agentmem
export DB_PASSWORD=new_password

# 执行恢复
./scripts/restore.sh --db /backups/agentmem/db_20250103_120000.dump.gz
```

### 5.4 恢复验证

恢复完成后，验证系统功能：

```bash
# 1. 检查服务状态
docker-compose ps

# 2. 检查健康状态
curl http://localhost:8080/health

# 3. 检查数据库连接
docker exec -it agentmem-postgres psql -U agentmem -d agentmem -c "SELECT COUNT(*) FROM memories;"

# 4. 检查 Redis
docker exec -it agentmem-redis redis-cli ping

# 5. 测试 API
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{"content": "Test after restore", "memory_type": "episodic"}'
```

---

## 6. 自动化备份

### 6.1 使用 Cron

#### 每日备份（凌晨 2 点）

```bash
# 编辑 crontab
crontab -e

# 添加以下行
0 2 * * * /opt/agentmem/scripts/backup.sh >> /var/log/agentmem/backup-cron.log 2>&1
```

#### 每周完整备份 + 每日增量备份

```bash
# 每天凌晨 2 点增量备份
0 2 * * * /opt/agentmem/scripts/backup.sh >> /var/log/agentmem/backup-cron.log 2>&1

# 每周日凌晨 3 点完整备份
0 3 * * 0 /opt/agentmem/scripts/backup.sh >> /var/log/agentmem/backup-weekly.log 2>&1
```

### 6.2 使用 Systemd Timer

#### 创建服务文件

**`/etc/systemd/system/agentmem-backup.service`**:
```ini
[Unit]
Description=AgentMem Backup Service
After=network.target

[Service]
Type=oneshot
User=root
WorkingDirectory=/opt/agentmem
ExecStart=/opt/agentmem/scripts/backup.sh
StandardOutput=append:/var/log/agentmem/backup.log
StandardError=append:/var/log/agentmem/backup.log
```

#### 创建定时器文件

**`/etc/systemd/system/agentmem-backup.timer`**:
```ini
[Unit]
Description=AgentMem Backup Timer
Requires=agentmem-backup.service

[Timer]
OnCalendar=daily
OnCalendar=02:00
Persistent=true

[Install]
WantedBy=timers.target
```

#### 启用定时器

```bash
sudo systemctl daemon-reload
sudo systemctl enable agentmem-backup.timer
sudo systemctl start agentmem-backup.timer

# 查看状态
sudo systemctl status agentmem-backup.timer
sudo systemctl list-timers agentmem-backup.timer
```

### 6.3 远程备份

#### 同步到远程服务器

```bash
# 在 backup.sh 末尾添加
rsync -avz --delete \
  /backups/agentmem/ \
  backup-server:/remote/backups/agentmem/
```

#### 上传到云存储

```bash
# AWS S3
aws s3 sync /backups/agentmem/ s3://my-bucket/agentmem-backups/

# Google Cloud Storage
gsutil -m rsync -r /backups/agentmem/ gs://my-bucket/agentmem-backups/

# Azure Blob Storage
az storage blob upload-batch \
  --destination agentmem-backups \
  --source /backups/agentmem/
```

---

## 7. 故障排除

### 7.1 备份失败

#### 问题: 磁盘空间不足

```bash
# 检查磁盘空间
df -h /backups

# 清理旧备份
find /backups/agentmem -mtime +30 -delete

# 或增加保留天数
export BACKUP_RETENTION_DAYS=15
```

#### 问题: 数据库连接失败

```bash
# 测试连接
PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c '\q'

# 检查密码
echo $DB_PASSWORD

# 检查网络
telnet $DB_HOST $DB_PORT
```

#### 问题: 权限不足

```bash
# 检查目录权限
ls -ld /backups/agentmem

# 修复权限
sudo chown -R $(whoami):$(whoami) /backups/agentmem
sudo chmod -R 755 /backups/agentmem
```

### 7.2 恢复失败

#### 问题: 备份文件损坏

```bash
# 验证备份
./scripts/verify-backup.sh 20250103_120000

# 使用前一天的备份
./scripts/restore.sh --all 20250102_120000
```

#### 问题: 数据库已存在

```bash
# 删除现有数据库（谨慎！）
PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "DROP DATABASE IF EXISTS agentmem;"
PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE agentmem;"

# 重新恢复
./scripts/restore.sh --db /backups/agentmem/db_20250103_120000.dump.gz
```

---

## 8. 最佳实践

### 8.1 备份

1. ✅ **定期备份**: 每天至少一次
2. ✅ **异地备份**: 同步到远程服务器或云存储
3. ✅ **验证备份**: 每周验证一次
4. ✅ **监控备份**: 设置告警通知
5. ✅ **文档记录**: 记录备份和恢复过程

### 8.2 恢复

1. ✅ **测试恢复**: 定期演练恢复流程
2. ✅ **备份现有数据**: 恢复前先备份当前数据
3. ✅ **验证恢复**: 恢复后验证数据完整性
4. ✅ **记录过程**: 记录恢复步骤和问题

### 8.3 安全

1. ✅ **加密备份**: 使用 GPG 加密敏感备份
2. ✅ **访问控制**: 限制备份文件访问权限
3. ✅ **审计日志**: 记录所有备份和恢复操作

---

## 附录

### A. 环境变量参考

详见 `scripts/backup.sh` 和 `scripts/restore.sh` 文件头部。

### B. 故障排除决策树

```
备份失败？
├─ 磁盘空间不足 → 清理旧备份或扩容
├─ 连接失败 → 检查网络和凭据
└─ 权限不足 → 修复文件权限

恢复失败？
├─ 备份损坏 → 使用其他备份
├─ 数据库冲突 → 删除现有数据库
└─ 权限不足 → 使用 sudo
```

### C. 相关文档

- [生产部署指南](./production-deployment-guide.md)
- [快速开始指南](./quickstart.md)
- [API 参考](./api-reference.md)

---

**文档版本**: 1.0  
**最后更新**: 2025-10-03  
**维护者**: AgentMem Team

