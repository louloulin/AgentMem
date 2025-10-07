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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use serde_json::json;

    // Helper function to create test event
    fn create_test_event(id: &str, event_type: &str, importance: f32) -> EpisodicEvent {
        let now = Utc::now();
        EpisodicEvent {
            id: id.to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            occurred_at: now,
            event_type: event_type.to_string(),
            actor: Some("test-actor".to_string()),
            summary: format!("Test event: {}", id),
            details: Some("Test event details".to_string()),
            importance_score: importance,
            metadata: json!({"test": true}),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_episodic_event_creation() {
        let event = create_test_event("event-1", "conversation", 0.8);

        assert_eq!(event.id, "event-1");
        assert_eq!(event.event_type, "conversation");
        assert_eq!(event.importance_score, 0.8);
        assert_eq!(event.organization_id, "test-org");
        assert_eq!(event.user_id, "test-user");
        assert_eq!(event.agent_id, "test-agent");
    }

    #[test]
    fn test_episodic_event_serialization() {
        let event = create_test_event("event-2", "action", 0.6);

        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("event-2"));
        assert!(json.contains("action"));

        // Test deserialization
        let deserialized: EpisodicEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.event_type, event.event_type);
        assert_eq!(deserialized.importance_score, event.importance_score);
    }

    #[test]
    fn test_episodic_query_default() {
        let query = EpisodicQuery {
            start_time: None,
            end_time: None,
            event_type: None,
            min_importance: None,
            limit: None,
        };

        assert!(query.start_time.is_none());
        assert!(query.end_time.is_none());
        assert!(query.event_type.is_none());
        assert!(query.min_importance.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_episodic_query_with_filters() {
        let now = Utc::now();
        let query = EpisodicQuery {
            start_time: Some(now - Duration::hours(24)),
            end_time: Some(now),
            event_type: Some("conversation".to_string()),
            min_importance: Some(0.5),
            limit: Some(10),
        };

        assert!(query.start_time.is_some());
        assert!(query.end_time.is_some());
        assert_eq!(query.event_type.unwrap(), "conversation");
        assert_eq!(query.min_importance.unwrap(), 0.5);
        assert_eq!(query.limit.unwrap(), 10);
    }

    #[test]
    fn test_importance_score_range() {
        let event_low = create_test_event("event-low", "observation", 0.0);
        let event_high = create_test_event("event-high", "decision", 1.0);

        assert_eq!(event_low.importance_score, 0.0);
        assert_eq!(event_high.importance_score, 1.0);
        assert!(event_low.importance_score >= 0.0 && event_low.importance_score <= 1.0);
        assert!(event_high.importance_score >= 0.0 && event_high.importance_score <= 1.0);
    }

    #[test]
    fn test_event_metadata() {
        let event = create_test_event("event-meta", "conversation", 0.7);

        assert!(event.metadata.is_object());
        assert_eq!(event.metadata["test"], json!(true));
    }

    #[test]
    fn test_event_optional_fields() {
        let mut event = create_test_event("event-optional", "action", 0.5);

        // Test with actor
        assert!(event.actor.is_some());
        assert_eq!(event.actor.unwrap(), "test-actor");

        // Test without actor
        event.actor = None;
        assert!(event.actor.is_none());

        // Test with details
        event.details = Some("Detailed information".to_string());
        assert!(event.details.is_some());

        // Test without details
        event.details = None;
        assert!(event.details.is_none());
    }

    #[test]
    fn test_event_timestamps() {
        let event = create_test_event("event-time", "conversation", 0.8);

        // Timestamps should be set
        assert!(event.created_at <= Utc::now());
        assert!(event.updated_at <= Utc::now());
        assert!(event.occurred_at <= Utc::now());

        // Created and updated should be close
        let diff = event.updated_at - event.created_at;
        assert!(diff.num_seconds().abs() < 1);
    }

    #[test]
    fn test_event_type_variations() {
        let types = vec!["conversation", "action", "observation", "decision", "error"];

        for (i, event_type) in types.iter().enumerate() {
            let event = create_test_event(&format!("event-{}", i), event_type, 0.5);
            assert_eq!(event.event_type, *event_type);
        }
    }

    #[test]
    fn test_importance_score_boundaries() {
        // Test minimum score
        let event_min = create_test_event("event-min", "test", 0.0);
        assert_eq!(event_min.importance_score, 0.0);

        // Test maximum score
        let event_max = create_test_event("event-max", "test", 1.0);
        assert_eq!(event_max.importance_score, 1.0);

        // Test mid-range score
        let event_mid = create_test_event("event-mid", "test", 0.5);
        assert_eq!(event_mid.importance_score, 0.5);
    }

    #[test]
    fn test_event_with_empty_strings() {
        let now = Utc::now();
        let event = EpisodicEvent {
            id: "".to_string(),
            organization_id: "".to_string(),
            user_id: "".to_string(),
            agent_id: "".to_string(),
            occurred_at: now,
            event_type: "".to_string(),
            actor: Some("".to_string()),
            summary: "".to_string(),
            details: Some("".to_string()),
            importance_score: 0.0,
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        // Should handle empty strings without panicking
        assert_eq!(event.id, "");
        assert_eq!(event.summary, "");
    }

    #[test]
    fn test_event_with_long_strings() {
        let long_string = "a".repeat(10000);
        let now = Utc::now();

        let event = EpisodicEvent {
            id: "event-long".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            occurred_at: now,
            event_type: "test".to_string(),
            actor: Some("actor".to_string()),
            summary: long_string.clone(),
            details: Some(long_string.clone()),
            importance_score: 0.5,
            metadata: json!({"long_field": long_string}),
            created_at: now,
            updated_at: now,
        };

        // Should handle long strings
        assert_eq!(event.summary.len(), 10000);
        assert_eq!(event.details.as_ref().unwrap().len(), 10000);
    }

    #[test]
    fn test_query_with_multiple_filters() {
        let query = EpisodicQuery {
            user_id: Some("user-123".to_string()),
            agent_id: Some("agent-456".to_string()),
            event_type: Some("conversation".to_string()),
            start_time: Some(Utc::now() - Duration::days(7)),
            end_time: Some(Utc::now()),
            min_importance: Some(0.5),
            limit: Some(50),
            offset: Some(10),
        };

        assert!(query.user_id.is_some());
        assert!(query.agent_id.is_some());
        assert!(query.event_type.is_some());
        assert!(query.start_time.is_some());
        assert!(query.end_time.is_some());
        assert!(query.min_importance.is_some());
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(10));
    }

    #[test]
    fn test_event_metadata_complex() {
        let complex_metadata = json!({
            "tags": ["important", "urgent", "follow-up"],
            "location": {
                "city": "San Francisco",
                "coordinates": [37.7749, -122.4194]
            },
            "participants": ["user1", "user2", "user3"],
            "metrics": {
                "duration": 3600,
                "interactions": 42
            }
        });

        let now = Utc::now();
        let event = EpisodicEvent {
            id: "event-complex".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            occurred_at: now,
            event_type: "meeting".to_string(),
            actor: Some("organizer".to_string()),
            summary: "Team meeting".to_string(),
            details: Some("Quarterly planning session".to_string()),
            importance_score: 0.9,
            metadata: complex_metadata.clone(),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(event.metadata["tags"][0], "important");
        assert_eq!(event.metadata["location"]["city"], "San Francisco");
        assert_eq!(event.metadata["participants"].as_array().unwrap().len(), 3);
        assert_eq!(event.metadata["metrics"]["duration"], 3600);
    }

    #[test]
    fn test_query_time_range_validation() {
        let now = Utc::now();
        let start = now - Duration::days(7);
        let end = now;

        let query = EpisodicQuery {
            user_id: Some("user-123".to_string()),
            agent_id: None,
            event_type: None,
            start_time: Some(start),
            end_time: Some(end),
            min_importance: None,
            limit: None,
            offset: None,
        };

        // 验证时间范围
        assert!(query.start_time.unwrap() < query.end_time.unwrap());
        assert!(query.start_time.unwrap() <= now);
    }

    #[test]
    fn test_event_actor_variations() {
        let now = Utc::now();

        // 有 actor
        let event_with_actor = EpisodicEvent {
            id: "event-1".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            occurred_at: now,
            event_type: "action".to_string(),
            actor: Some("John Doe".to_string()),
            summary: "Test event".to_string(),
            details: None,
            importance_score: 0.5,
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        // 无 actor
        let event_without_actor = EpisodicEvent {
            id: "event-2".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            occurred_at: now,
            event_type: "observation".to_string(),
            actor: None,
            summary: "Test event".to_string(),
            details: None,
            importance_score: 0.5,
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        assert!(event_with_actor.actor.is_some());
        assert!(event_without_actor.actor.is_none());
    }

    #[test]
    fn test_query_pagination() {
        // 第一页
        let query_page1 = EpisodicQuery {
            user_id: Some("user-123".to_string()),
            agent_id: None,
            event_type: None,
            start_time: None,
            end_time: None,
            min_importance: None,
            limit: Some(10),
            offset: Some(0),
        };

        // 第二页
        let query_page2 = EpisodicQuery {
            user_id: Some("user-123".to_string()),
            agent_id: None,
            event_type: None,
            start_time: None,
            end_time: None,
            min_importance: None,
            limit: Some(10),
            offset: Some(10),
        };

        assert_eq!(query_page1.limit, Some(10));
        assert_eq!(query_page1.offset, Some(0));
        assert_eq!(query_page2.offset, Some(10));
    }

    #[test]
    fn test_event_importance_categories() {
        // 低重要性
        let low_importance = create_test_event("event-low", "observation", 0.2);
        assert!(low_importance.importance_score < 0.3);

        // 中等重要性
        let medium_importance = create_test_event("event-medium", "conversation", 0.5);
        assert!(medium_importance.importance_score >= 0.3 && medium_importance.importance_score < 0.7);

        // 高重要性
        let high_importance = create_test_event("event-high", "decision", 0.9);
        assert!(high_importance.importance_score >= 0.7);
    }
}

