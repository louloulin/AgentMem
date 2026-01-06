//! 操作历史记录模块
//!
//! 参考 mem0 的 SQLiteManager 实现，提供完整的操作审计功能

use agent_mem_traits::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, warn};

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 记录 ID
    pub id: String,
    /// 记忆 ID
    pub memory_id: String,
    /// 旧内容
    pub old_memory: Option<String>,
    /// 新内容
    pub new_memory: Option<String>,
    /// 操作类型: ADD, UPDATE, DELETE
    pub event: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: Option<DateTime<Utc>>,
    /// 是否已删除
    pub is_deleted: bool,
    /// Actor ID
    pub actor_id: Option<String>,
    /// 角色
    pub role: Option<String>,
}

/// 历史记录管理器
///
/// 使用 SQLite 存储所有记忆操作的历史记录，支持：
/// - 操作审计（ADD/UPDATE/DELETE）
/// - 变更追溯
/// - 错误回滚
/// - 调试支持
pub struct HistoryManager {
    pool: Arc<SqlitePool>,
}

impl HistoryManager {
    /// 创建历史管理器
    ///
    /// # 参数
    ///
    /// * `db_path` - SQLite 数据库路径
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use agent_mem::history::HistoryManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = HistoryManager::new("./data/history.db").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(db_path: &str) -> Result<Self> {
        info!("创建历史记录管理器: {}", db_path);

        // 使用 SqliteConnectOptions 并设置 create_if_missing(true)
        // 这样 SQLx 会自动创建数据库文件（如果不存在）
        let options = SqliteConnectOptions::from_str(db_path)
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(format!(
                    "解析数据库路径失败: {e}"
                ))
            })?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(format!("连接数据库失败: {e}"))
            })?;

        let manager = Self {
            pool: Arc::new(pool),
        };

        manager.create_table().await?;
        info!("✅ 历史记录管理器创建成功");

        Ok(manager)
    }

    /// 创建历史表
    async fn create_table(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                memory_id TEXT NOT NULL,
                old_memory TEXT,
                new_memory TEXT,
                event TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                actor_id TEXT,
                role TEXT
            )
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            agent_mem_traits::AgentMemError::storage_error(format!("创建历史表失败: {e}"))
        })?;

        // 创建索引以提高查询性能
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_history_memory_id 
            ON history(memory_id)
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            agent_mem_traits::AgentMemError::storage_error(format!("创建索引失败: {e}"))
        })?;

        info!("✅ 历史记录表已创建");
        Ok(())
    }

    /// 添加历史记录
    ///
    /// # 参数
    ///
    /// * `entry` - 历史记录条目
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use agent_mem::history::{HistoryManager, HistoryEntry};
    /// # use chrono::Utc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new("./history.db").await?;
    ///
    /// let entry = HistoryEntry {
    ///     id: uuid::Uuid::new_v4().to_string(),
    ///     memory_id: "mem_123".to_string(),
    ///     old_memory: None,
    ///     new_memory: Some("New memory content".to_string()),
    ///     event: "ADD".to_string(),
    ///     created_at: Utc::now(),
    ///     updated_at: None,
    ///     is_deleted: false,
    ///     actor_id: Some("user_456".to_string()),
    ///     role: Some("user".to_string()),
    /// };
    ///
    /// manager.add_history(entry).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_history(&self, entry: HistoryEntry) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO history 
            (id, memory_id, old_memory, new_memory, event, created_at, updated_at, is_deleted, actor_id, role)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.memory_id)
        .bind(&entry.old_memory)
        .bind(&entry.new_memory)
        .bind(&entry.event)
        .bind(entry.created_at.to_rfc3339())
        .bind(entry.updated_at.map(|dt| dt.to_rfc3339()))
        .bind(entry.is_deleted as i32)
        .bind(&entry.actor_id)
        .bind(&entry.role)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            agent_mem_traits::AgentMemError::storage_error(format!(
                "添加历史记录失败: {e}"
            ))
        })?;

        Ok(())
    }

    /// 获取记忆的历史记录
    ///
    /// 返回指定记忆的所有变更历史，按时间倒序排列
    ///
    /// # 参数
    ///
    /// * `memory_id` - 记忆 ID
    ///
    /// # 返回
    ///
    /// 历史记录列表，最新的在前
    pub async fn get_history(&self, memory_id: &str) -> Result<Vec<HistoryEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, memory_id, old_memory, new_memory, event, 
                   created_at, updated_at, is_deleted, actor_id, role
            FROM history 
            WHERE memory_id = ? 
            ORDER BY created_at DESC
            "#,
        )
        .bind(memory_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            agent_mem_traits::AgentMemError::storage_error(format!("获取历史记录失败: {e}"))
        })?;

        let mut entries = Vec::new();
        for row in rows {
            let entry = HistoryEntry {
                id: row.get("id"),
                memory_id: row.get("memory_id"),
                old_memory: row.get("old_memory"),
                new_memory: row.get("new_memory"),
                event: row.get("event"),
                created_at: row
                    .get::<String, _>("created_at")
                    .parse()
                    .unwrap_or(Utc::now()),
                updated_at: row
                    .get::<Option<String>, _>("updated_at")
                    .and_then(|s| s.parse().ok()),
                is_deleted: row.get::<i32, _>("is_deleted") != 0,
                actor_id: row.get("actor_id"),
                role: row.get("role"),
            };
            entries.push(entry);
        }

        Ok(entries)
    }

    /// 获取所有历史记录
    ///
    /// 返回数据库中的所有历史记录
    ///
    /// # 参数
    ///
    /// * `limit` - 限制返回数量
    pub async fn get_all_history(&self, limit: Option<usize>) -> Result<Vec<HistoryEntry>> {
        let query_str = if let Some(limit) = limit {
            format!(
                "SELECT * FROM history ORDER BY created_at DESC LIMIT {limit}"
            )
        } else {
            "SELECT * FROM history ORDER BY created_at DESC".to_string()
        };

        let rows = sqlx::query(&query_str)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(format!(
                    "获取所有历史记录失败: {e}"
                ))
            })?;

        let mut entries = Vec::new();
        for row in rows {
            let entry = HistoryEntry {
                id: row.get("id"),
                memory_id: row.get("memory_id"),
                old_memory: row.get("old_memory"),
                new_memory: row.get("new_memory"),
                event: row.get("event"),
                created_at: row
                    .get::<String, _>("created_at")
                    .parse()
                    .unwrap_or(Utc::now()),
                updated_at: row
                    .get::<Option<String>, _>("updated_at")
                    .and_then(|s| s.parse().ok()),
                is_deleted: row.get::<i32, _>("is_deleted") != 0,
                actor_id: row.get("actor_id"),
                role: row.get("role"),
            };
            entries.push(entry);
        }

        Ok(entries)
    }

    /// 重置所有历史记录
    ///
    /// ⚠️ 危险操作：删除所有历史记录
    pub async fn reset(&self) -> Result<()> {
        warn!("⚠️ 重置所有历史记录（危险操作）");

        sqlx::query("DELETE FROM history")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(format!("重置历史记录失败: {e}"))
            })?;

        info!("✅ 所有历史记录已清空");
        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<HistoryStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_count,
                SUM(CASE WHEN event = 'ADD' THEN 1 ELSE 0 END) as add_count,
                SUM(CASE WHEN event = 'UPDATE' THEN 1 ELSE 0 END) as update_count,
                SUM(CASE WHEN event = 'DELETE' THEN 1 ELSE 0 END) as delete_count
            FROM history
            "#,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            agent_mem_traits::AgentMemError::storage_error(format!("获取统计信息失败: {e}"))
        })?;

        Ok(HistoryStats {
            total_count: row.get::<i64, _>("total_count") as usize,
            add_count: row.get::<i64, _>("add_count") as usize,
            update_count: row.get::<i64, _>("update_count") as usize,
            delete_count: row.get::<i64, _>("delete_count") as usize,
        })
    }
}

/// 历史统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    /// 总记录数
    pub total_count: usize,
    /// ADD 操作数
    pub add_count: usize,
    /// UPDATE 操作数
    pub update_count: usize,
    /// DELETE 操作数
    pub delete_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_history_manager_creation() {
        let manager = HistoryManager::new(":memory:").await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_add_and_get_history() {
        let manager = HistoryManager::new(":memory:").await?;

        let entry = HistoryEntry {
            id: Uuid::new_v4().to_string(),
            memory_id: "mem_test_123".to_string(),
            old_memory: None,
            new_memory: Some("Test memory content".to_string()),
            event: "ADD".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            is_deleted: false,
            actor_id: Some("user_456".to_string()),
            role: Some("user".to_string()),
        };

        // 添加历史记录
        let result = manager.add_history(entry.clone()).await;
        assert!(result.is_ok());

        // 获取历史记录
        let history = manager.get_history("mem_test_123").await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].memory_id, "mem_test_123");
        assert_eq!(history[0].event, "ADD");
    }

    #[tokio::test]
    async fn test_multiple_history_entries() {
        let manager = HistoryManager::new(":memory:").await?;
        let memory_id = "mem_multi_test";

        // 添加多个历史记录
        for (i, event) in ["ADD", "UPDATE", "UPDATE", "DELETE"].iter().enumerate() {
            let entry = HistoryEntry {
                id: Uuid::new_v4().to_string(),
                memory_id: memory_id.to_string(),
                old_memory: if i > 0 {
                    Some(format!("Old content {}", i))
                } else {
                    None
                },
                new_memory: Some(format!("New content {}", i + 1)),
                event: event.to_string(),
                created_at: Utc::now(),
                updated_at: None,
                is_deleted: *event == "DELETE",
                actor_id: None,
                role: Some("user".to_string()),
            };

            manager.add_history(entry).await?;
        }

        // 获取历史记录
        let history = manager.get_history(memory_id).await?;
        assert_eq!(history.len(), 4);

        // 验证顺序（最新的在前）
        assert_eq!(history[0].event, "DELETE");
        assert_eq!(history[3].event, "ADD");
    }

    #[tokio::test]
    async fn test_history_stats() {
        let manager = HistoryManager::new(":memory:").await?;

        // 添加不同类型的历史记录
        for event in ["ADD", "ADD", "UPDATE", "DELETE"].iter() {
            let entry = HistoryEntry {
                id: Uuid::new_v4().to_string(),
                memory_id: format!("mem_{}", Uuid::new_v4()),
                old_memory: None,
                new_memory: Some("content".to_string()),
                event: event.to_string(),
                created_at: Utc::now(),
                updated_at: None,
                is_deleted: false,
                actor_id: None,
                role: None,
            };

            manager.add_history(entry).await?;
        }

        // 获取统计
        let stats = manager.get_stats().await?;
        assert_eq!(stats.total_count, 4);
        assert_eq!(stats.add_count, 2);
        assert_eq!(stats.update_count, 1);
        assert_eq!(stats.delete_count, 1);
    }

    #[tokio::test]
    async fn test_reset() {
        let manager = HistoryManager::new(":memory:").await?;

        // 添加一些记录
        let entry = HistoryEntry {
            id: Uuid::new_v4().to_string(),
            memory_id: "test".to_string(),
            old_memory: None,
            new_memory: Some("content".to_string()),
            event: "ADD".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            is_deleted: false,
            actor_id: None,
            role: None,
        };

        manager.add_history(entry).await?;

        // 重置
        let result = manager.reset().await;
        assert!(result.is_ok());

        // 验证已清空
        let all = manager.get_all_history(None).await?;
        assert_eq!(all.len(), 0);
    }
}
