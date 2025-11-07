//! 记忆生命周期管理器 - 数据库持久化版本
//!
//! 提供记忆生命周期的完整管理，包括：
//! - 状态追踪和持久化
//! - 事件记录和查询
//! - 自动过期和归档
//! - 生命周期策略执行

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 记忆状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryState {
    Created,
    Active,
    Archived,
    Deprecated,
    Deleted,
}

impl MemoryState {
    pub fn as_str(&self) -> &str {
        match self {
            MemoryState::Created => "created",
            MemoryState::Active => "active",
            MemoryState::Archived => "archived",
            MemoryState::Deprecated => "deprecated",
            MemoryState::Deleted => "deleted",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "created" => Ok(MemoryState::Created),
            "active" => Ok(MemoryState::Active),
            "archived" => Ok(MemoryState::Archived),
            "deprecated" => Ok(MemoryState::Deprecated),
            "deleted" => Ok(MemoryState::Deleted),
            _ => Err(AgentMemError::validation_error(format!(
                "Invalid memory state: {}",
                s
            ))),
        }
    }
}

/// 生命周期事件类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEventType {
    Created,
    Accessed,
    Updated,
    Archived,
    Restored,
    Deprecated,
    Deleted,
    ImportanceChanged,
    ExpirationSet,
}

impl LifecycleEventType {
    pub fn as_str(&self) -> &str {
        match self {
            LifecycleEventType::Created => "created",
            LifecycleEventType::Accessed => "accessed",
            LifecycleEventType::Updated => "updated",
            LifecycleEventType::Archived => "archived",
            LifecycleEventType::Restored => "restored",
            LifecycleEventType::Deprecated => "deprecated",
            LifecycleEventType::Deleted => "deleted",
            LifecycleEventType::ImportanceChanged => "importance_changed",
            LifecycleEventType::ExpirationSet => "expiration_set",
        }
    }
}

/// 生命周期事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub id: i32,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub memory_id: String,
    pub event_type: String,
    pub old_state: Option<String>,
    pub new_state: Option<String>,
    pub metadata: serde_json::Value,
    pub event_timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// 记忆状态记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStateRecord {
    pub memory_id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub current_state: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: i64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub auto_archive_enabled: bool,
    pub auto_delete_enabled: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 生命周期管理器配置
#[derive(Debug, Clone)]
pub struct LifecycleManagerConfig {
    /// 自动归档年龄（秒）
    pub auto_archive_age: Option<i64>,
    /// 自动删除年龄（秒）
    pub auto_delete_age: Option<i64>,
    /// 归档重要性阈值
    pub archive_importance_threshold: f32,
    /// 删除重要性阈值
    pub delete_importance_threshold: f32,
    /// 工作记忆 TTL（秒）
    pub working_memory_ttl: i64,
}

impl Default for LifecycleManagerConfig {
    fn default() -> Self {
        Self {
            auto_archive_age: Some(30 * 24 * 3600), // 30 天
            auto_delete_age: Some(365 * 24 * 3600), // 1 年
            archive_importance_threshold: 0.3,
            delete_importance_threshold: 0.1,
            working_memory_ttl: 24 * 3600, // 1 天
        }
    }
}

/// 生命周期管理器
pub struct LifecycleManager {
    pool: Arc<PgPool>,
    config: LifecycleManagerConfig,
}

impl LifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new(pool: Arc<PgPool>, config: LifecycleManagerConfig) -> Self {
        Self { pool, config }
    }

    /// 使用默认配置创建
    pub fn with_default_config(pool: Arc<PgPool>) -> Self {
        Self::new(pool, LifecycleManagerConfig::default())
    }

    /// 记录生命周期事件
    pub async fn record_event(
        &self,
        organization_id: &str,
        user_id: &str,
        agent_id: &str,
        memory_id: &str,
        event_type: LifecycleEventType,
        old_state: Option<MemoryState>,
        new_state: Option<MemoryState>,
        metadata: serde_json::Value,
    ) -> Result<i32> {
        let event_type_str = event_type.as_str();
        let old_state_str = old_state.as_ref().map(|s| s.as_str());
        let new_state_str = new_state.as_ref().map(|s| s.as_str());

        let result = sqlx::query!(
            r#"
            INSERT INTO lifecycle_events (
                organization_id, user_id, agent_id, memory_id,
                event_type, old_state, new_state, metadata, event_timestamp
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
            organization_id,
            user_id,
            agent_id,
            memory_id,
            event_type_str,
            old_state_str,
            new_state_str,
            metadata,
            Utc::now(),
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        debug!(
            "Recorded lifecycle event: {} for memory {}",
            event_type_str, memory_id
        );

        Ok(result.id)
    }

    /// 更新记忆状态
    pub async fn update_state(
        &self,
        organization_id: &str,
        user_id: &str,
        agent_id: &str,
        memory_id: &str,
        new_state: MemoryState,
    ) -> Result<()> {
        let new_state_str = new_state.as_str().to_string();

        // 获取旧状态
        let old_state = self.get_state(memory_id, user_id).await?;

        // 更新状态
        sqlx::query!(
            r#"
            INSERT INTO memory_states (
                memory_id, organization_id, user_id, agent_id, current_state, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (memory_id) DO UPDATE SET
                current_state = $5,
                updated_at = $6
            "#,
            memory_id,
            organization_id,
            user_id,
            agent_id,
            new_state_str.as_str(),
            Utc::now(),
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        // 记录事件
        let event_type = match &new_state {
            MemoryState::Created => LifecycleEventType::Created,
            MemoryState::Active => LifecycleEventType::Accessed,
            MemoryState::Archived => LifecycleEventType::Archived,
            MemoryState::Deprecated => LifecycleEventType::Deprecated,
            MemoryState::Deleted => LifecycleEventType::Deleted,
        };

        self.record_event(
            organization_id,
            user_id,
            agent_id,
            memory_id,
            event_type,
            old_state
                .map(|s| MemoryState::from_str(&s.current_state).ok())
                .flatten(),
            Some(new_state),
            serde_json::json!({}),
        )
        .await?;

        info!("Updated memory {} state to {}", memory_id, new_state_str);

        Ok(())
    }

    /// 获取记忆状态
    pub async fn get_state(
        &self,
        memory_id: &str,
        user_id: &str,
    ) -> Result<Option<MemoryStateRecord>> {
        let result = sqlx::query_as!(
            MemoryStateRecord,
            r#"
            SELECT 
                memory_id, organization_id, user_id, agent_id,
                current_state, expires_at, access_count, last_accessed,
                auto_archive_enabled, auto_delete_enabled,
                metadata, created_at, updated_at
            FROM memory_states
            WHERE memory_id = $1 AND user_id = $2
            "#,
            memory_id,
            user_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(result)
    }

    /// 记录访问
    pub async fn record_access(&self, memory_id: &str, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE memory_states
            SET access_count = access_count + 1,
                last_accessed = $1,
                updated_at = $1
            WHERE memory_id = $2 AND user_id = $3
            "#,
            Utc::now(),
            memory_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(())
    }

    /// 设置过期时间
    pub async fn set_expiration(
        &self,
        organization_id: &str,
        user_id: &str,
        agent_id: &str,
        memory_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE memory_states
            SET expires_at = $1, updated_at = $2
            WHERE memory_id = $3 AND user_id = $4
            "#,
            expires_at,
            Utc::now(),
            memory_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        // 记录事件
        self.record_event(
            organization_id,
            user_id,
            agent_id,
            memory_id,
            LifecycleEventType::ExpirationSet,
            None,
            None,
            serde_json::json!({ "expires_at": expires_at }),
        )
        .await?;

        info!("Set expiration for memory {} to {}", memory_id, expires_at);

        Ok(())
    }

    /// 获取过期的记忆
    pub async fn get_expired_memories(&self, user_id: &str) -> Result<Vec<String>> {
        let now = Utc::now();

        let results = sqlx::query!(
            r#"
            SELECT memory_id
            FROM memory_states
            WHERE user_id = $1
            AND expires_at IS NOT NULL
            AND expires_at <= $2
            AND current_state != 'deleted'
            "#,
            user_id,
            now,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(results.into_iter().map(|r| r.memory_id).collect())
    }

    /// 获取需要归档的记忆
    pub async fn get_archivable_memories(
        &self,
        user_id: &str,
        importance_threshold: f32,
    ) -> Result<Vec<String>> {
        let archive_age = self.config.auto_archive_age.unwrap_or(30 * 24 * 3600);
        let cutoff_time = Utc::now() - chrono::Duration::seconds(archive_age);

        let results = sqlx::query!(
            r#"
            SELECT ms.memory_id
            FROM memory_states ms
            WHERE ms.user_id = $1
            AND ms.current_state IN ('created', 'active')
            AND ms.auto_archive_enabled = true
            AND ms.created_at <= $2
            "#,
            user_id,
            cutoff_time,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(results.into_iter().map(|r| r.memory_id).collect())
    }

    /// 获取需要删除的记忆
    pub async fn get_deletable_memories(
        &self,
        user_id: &str,
        importance_threshold: f32,
    ) -> Result<Vec<String>> {
        let delete_age = self.config.auto_delete_age.unwrap_or(365 * 24 * 3600);
        let cutoff_time = Utc::now() - chrono::Duration::seconds(delete_age);

        let results = sqlx::query!(
            r#"
            SELECT ms.memory_id
            FROM memory_states ms
            WHERE ms.user_id = $1
            AND ms.current_state IN ('archived', 'deprecated')
            AND ms.auto_delete_enabled = true
            AND ms.created_at <= $2
            "#,
            user_id,
            cutoff_time,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(results.into_iter().map(|r| r.memory_id).collect())
    }

    /// 获取记忆的事件历史
    pub async fn get_event_history(
        &self,
        memory_id: &str,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<LifecycleEvent>> {
        let results = sqlx::query_as!(
            LifecycleEvent,
            r#"
            SELECT
                id, organization_id, user_id, agent_id, memory_id,
                event_type, old_state, new_state, metadata,
                event_timestamp, created_at
            FROM lifecycle_events
            WHERE memory_id = $1 AND user_id = $2
            ORDER BY event_timestamp DESC
            LIMIT $3
            "#,
            memory_id,
            user_id,
            limit,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(results)
    }

    /// 获取生命周期统计
    pub async fn get_lifecycle_stats(&self, user_id: &str) -> Result<serde_json::Value> {
        let state_counts = sqlx::query!(
            r#"
            SELECT current_state, COUNT(*) as count
            FROM memory_states
            WHERE user_id = $1
            GROUP BY current_state
            "#,
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        let event_counts = sqlx::query!(
            r#"
            SELECT event_type, COUNT(*) as count
            FROM lifecycle_events
            WHERE user_id = $1
            AND event_timestamp >= NOW() - INTERVAL '7 days'
            GROUP BY event_type
            "#,
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(serde_json::json!({
            "state_counts": state_counts.into_iter().map(|r| {
                serde_json::json!({
                    "state": r.current_state,
                    "count": r.count.unwrap_or(0)
                })
            }).collect::<Vec<_>>(),
            "event_counts": event_counts.into_iter().map(|r| {
                serde_json::json!({
                    "event_type": r.event_type,
                    "count": r.count.unwrap_or(0)
                })
            }).collect::<Vec<_>>()
        }))
    }

    /// 清理旧事件
    pub async fn cleanup_old_events(&self, max_age_days: i64) -> Result<u64> {
        let cutoff_time = Utc::now() - chrono::Duration::days(max_age_days);

        let result = sqlx::query!(
            r#"
            DELETE FROM lifecycle_events
            WHERE event_timestamp < $1
            "#,
            cutoff_time,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        info!("Cleaned up {} old lifecycle events", result.rows_affected());

        Ok(result.rows_affected())
    }
}
