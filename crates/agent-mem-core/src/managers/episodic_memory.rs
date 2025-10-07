//! Episodic Memory Manager
//!
//! 管理情景记忆（Episodic Memory）- 基于时间的事件和经历
//! 参考 MIRIX 的 EpisodicMemoryManager 实现

use agent_mem_traits::{AgentMemError, MemoryType, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// 情景记忆事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEvent {
    /// 事件 ID
    pub id: String,
    /// 组织 ID
    pub organization_id: String,
    /// 用户 ID
    pub user_id: String,
    /// Agent ID
    pub agent_id: String,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 事件类型（如：conversation, action, observation）
    pub event_type: String,
    /// 参与者
    pub actor: Option<String>,
    /// 事件摘要
    pub summary: String,
    /// 事件详情
    pub details: Option<String>,
    /// 重要性评分（0.0-1.0）
    pub importance_score: f32,
    /// 元数据
    pub metadata: serde_json::Value,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 情景记忆查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicQuery {
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 事件类型过滤
    pub event_type: Option<String>,
    /// 最小重要性评分
    pub min_importance: Option<f32>,
    /// 限制返回数量
    pub limit: Option<i64>,
}

/// 情景记忆管理器
pub struct EpisodicMemoryManager {
    pool: Arc<PgPool>,
}

impl EpisodicMemoryManager {
    /// 创建新的情景记忆管理器
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 创建情景记忆事件
    pub async fn create_event(&self, event: EpisodicEvent) -> Result<EpisodicEvent> {
        info!("Creating episodic event: {}", event.id);

        let result = sqlx::query_as!(
            EpisodicEventRow,
            r#"
            INSERT INTO episodic_events (
                id, organization_id, user_id, agent_id, occurred_at,
                event_type, actor, summary, details, importance_score,
                metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
            event.id,
            event.organization_id,
            event.user_id,
            event.agent_id,
            event.occurred_at,
            event.event_type,
            event.actor,
            event.summary,
            event.details,
            event.importance_score,
            event.metadata,
            event.created_at,
            event.updated_at,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create episodic event: {}", e)))?;

        Ok(result.into())
    }

    /// 获取情景记忆事件
    pub async fn get_event(&self, event_id: &str, user_id: &str) -> Result<Option<EpisodicEvent>> {
        debug!("Getting episodic event: {}", event_id);

        let result = sqlx::query_as!(
            EpisodicEventRow,
            r#"
            SELECT * FROM episodic_events
            WHERE id = $1 AND user_id = $2
            "#,
            event_id,
            user_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get episodic event: {}", e)))?;

        Ok(result.map(Into::into))
    }

    /// 查询情景记忆事件
    pub async fn query_events(&self, user_id: &str, query: EpisodicQuery) -> Result<Vec<EpisodicEvent>> {
        info!("Querying episodic events for user: {}", user_id);

        let mut sql = String::from("SELECT * FROM episodic_events WHERE user_id = $1");
        let mut param_count = 1;

        // 构建动态查询
        if query.start_time.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND occurred_at >= ${}", param_count));
        }

        if query.end_time.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND occurred_at <= ${}", param_count));
        }

        if query.event_type.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND event_type = ${}", param_count));
        }

        if query.min_importance.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND importance_score >= ${}", param_count));
        }

        sql.push_str(" ORDER BY occurred_at DESC");

        if let Some(limit) = query.limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
        }

        // 使用 query_as 执行查询
        let mut query_builder = sqlx::query_as::<_, EpisodicEventRow>(&sql);
        query_builder = query_builder.bind(user_id);

        if let Some(start_time) = query.start_time {
            query_builder = query_builder.bind(start_time);
        }

        if let Some(end_time) = query.end_time {
            query_builder = query_builder.bind(end_time);
        }

        if let Some(event_type) = query.event_type {
            query_builder = query_builder.bind(event_type);
        }

        if let Some(min_importance) = query.min_importance {
            query_builder = query_builder.bind(min_importance);
        }

        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }

        let results = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to query episodic events: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// 删除情景记忆事件
    pub async fn delete_event(&self, event_id: &str, user_id: &str) -> Result<bool> {
        info!("Deleting episodic event: {}", event_id);

        let result = sqlx::query!(
            r#"
            DELETE FROM episodic_events
            WHERE id = $1 AND user_id = $2
            "#,
            event_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to delete episodic event: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 更新事件重要性评分
    pub async fn update_importance(&self, event_id: &str, user_id: &str, importance_score: f32) -> Result<bool> {
        debug!("Updating importance for event: {}", event_id);

        let result = sqlx::query!(
            r#"
            UPDATE episodic_events
            SET importance_score = $1, updated_at = $2
            WHERE id = $3 AND user_id = $4
            "#,
            importance_score,
            Utc::now(),
            event_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update importance: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取时间范围内的事件数量
    pub async fn count_events_in_range(
        &self,
        user_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM episodic_events
            WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at <= $3
            "#,
            user_id,
            start_time,
            end_time,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to count events: {}", e)))?;

        Ok(result.count.unwrap_or(0))
    }
}

/// 数据库行结构
#[derive(Debug, sqlx::FromRow)]
struct EpisodicEventRow {
    id: String,
    organization_id: String,
    user_id: String,
    agent_id: String,
    occurred_at: DateTime<Utc>,
    event_type: String,
    actor: Option<String>,
    summary: String,
    details: Option<String>,
    importance_score: f32,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<EpisodicEventRow> for EpisodicEvent {
    fn from(row: EpisodicEventRow) -> Self {
        Self {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            occurred_at: row.occurred_at,
            event_type: row.event_type,
            actor: row.actor,
            summary: row.summary,
            details: row.details,
            importance_score: row.importance_score,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

