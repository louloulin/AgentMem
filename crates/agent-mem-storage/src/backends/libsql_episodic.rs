//! LibSQL implementation of EpisodicMemoryStore

use agent_mem_traits::{AgentMemError, EpisodicEvent, EpisodicMemoryStore, EpisodicQuery, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// LibSQL-based episodic memory store
pub struct LibSqlEpisodicStore {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlEpisodicStore {
    /// Create a new LibSQL episodic store
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl EpisodicMemoryStore for LibSqlEpisodicStore {
    async fn create_event(&self, event: EpisodicEvent) -> Result<EpisodicEvent> {
        info!("Creating episodic event: {}", event.id);

        let conn = self.conn.lock().await;
        
        conn.execute(
            r#"
            INSERT INTO episodic_events (
                id, organization_id, user_id, agent_id, occurred_at,
                event_type, actor, summary, details, importance_score,
                metadata, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            libsql::params![
                event.id.clone(),
                event.organization_id.clone(),
                event.user_id.clone(),
                event.agent_id.clone(),
                event.occurred_at.to_rfc3339(),
                event.event_type.clone(),
                event.actor.clone(),
                event.summary.clone(),
                event.details.clone(),
                event.importance_score,
                event.metadata.to_string(),
                event.created_at.to_rfc3339(),
                event.updated_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create episodic event: {}", e)))?;

        Ok(event)
    }

    async fn get_event(&self, event_id: &str, user_id: &str) -> Result<Option<EpisodicEvent>> {
        debug!("Getting episodic event: {}", event_id);

        let conn = self.conn.lock().await;
        
        let mut stmt = conn
            .prepare("SELECT * FROM episodic_events WHERE id = ? AND user_id = ?")
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![event_id, user_id])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to query event: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::storage_error(&format!("Failed to fetch row: {}", e)))? {
            Ok(Some(row_to_event(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn query_events(&self, user_id: &str, query: EpisodicQuery) -> Result<Vec<EpisodicEvent>> {
        info!("Querying episodic events for user: {}", user_id);

        let mut sql = String::from("SELECT * FROM episodic_events WHERE user_id = ?");
        let mut params = vec![user_id.to_string()];

        // Build dynamic query
        if let Some(start_time) = query.start_time {
            sql.push_str(" AND occurred_at >= ?");
            params.push(start_time.to_rfc3339());
        }

        if let Some(end_time) = query.end_time {
            sql.push_str(" AND occurred_at <= ?");
            params.push(end_time.to_rfc3339());
        }

        if let Some(event_type) = query.event_type {
            sql.push_str(" AND event_type = ?");
            params.push(event_type);
        }

        if let Some(min_importance) = query.min_importance {
            sql.push_str(" AND importance_score >= ?");
            params.push(min_importance.to_string());
        }

        sql.push_str(" ORDER BY occurred_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ?");
            params.push(limit.to_string());
        }

        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(&sql)
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare query: {}", e)))?;

        // Build params tuple for libsql
        let mut rows = if params.len() == 1 {
            stmt.query(libsql::params![params[0].clone()]).await
        } else if params.len() == 2 {
            stmt.query(libsql::params![params[0].clone(), params[1].clone()]).await
        } else if params.len() == 3 {
            stmt.query(libsql::params![params[0].clone(), params[1].clone(), params[2].clone()]).await
        } else if params.len() == 4 {
            stmt.query(libsql::params![params[0].clone(), params[1].clone(), params[2].clone(), params[3].clone()]).await
        } else if params.len() == 5 {
            stmt.query(libsql::params![params[0].clone(), params[1].clone(), params[2].clone(), params[3].clone(), params[4].clone()]).await
        } else {
            stmt.query(libsql::params![params[0].clone()]).await
        }.map_err(|e| AgentMemError::storage_error(&format!("Failed to query events: {}", e)))?;

        let mut events = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentMemError::storage_error(&format!("Failed to fetch row: {}", e)))? {
            events.push(row_to_event(&row)?);
        }

        Ok(events)
    }

    async fn update_event(&self, event: EpisodicEvent) -> Result<bool> {
        debug!("Updating episodic event: {}", event.id);

        let conn = self.conn.lock().await;
        
        let result = conn.execute(
            r#"
            UPDATE episodic_events
            SET event_type = ?, actor = ?, summary = ?, details = ?,
                importance_score = ?, metadata = ?, updated_at = ?
            WHERE id = ? AND user_id = ?
            "#,
            libsql::params![
                event.event_type,
                event.actor,
                event.summary,
                event.details,
                event.importance_score,
                event.metadata.to_string(),
                Utc::now().to_rfc3339(),
                event.id,
                event.user_id,
            ],
        )
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update event: {}", e)))?;

        Ok(result > 0)
    }

    async fn delete_event(&self, event_id: &str, user_id: &str) -> Result<bool> {
        debug!("Deleting episodic event: {}", event_id);

        let conn = self.conn.lock().await;
        
        let result = conn.execute(
            "DELETE FROM episodic_events WHERE id = ? AND user_id = ?",
            libsql::params![event_id, user_id],
        )
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to delete event: {}", e)))?;

        Ok(result > 0)
    }

    async fn update_importance(&self, event_id: &str, user_id: &str, importance_score: f32) -> Result<bool> {
        debug!("Updating importance for event: {}", event_id);

        let conn = self.conn.lock().await;
        
        let result = conn.execute(
            "UPDATE episodic_events SET importance_score = ?, updated_at = ? WHERE id = ? AND user_id = ?",
            libsql::params![importance_score, Utc::now().to_rfc3339(), event_id, user_id],
        )
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update importance: {}", e)))?;

        Ok(result > 0)
    }

    async fn count_events_in_range(
        &self,
        user_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<i64> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn
            .prepare("SELECT COUNT(*) as count FROM episodic_events WHERE user_id = ? AND occurred_at >= ? AND occurred_at <= ?")
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![user_id, start_time.to_rfc3339(), end_time.to_rfc3339()])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to count events: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentMemError::storage_error(&format!("Failed to fetch row: {}", e)))? {
            let count: i64 = row.get(0).map_err(|e| AgentMemError::storage_error(&format!("Failed to get count: {}", e)))?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    async fn get_recent_events(&self, user_id: &str, limit: i64) -> Result<Vec<EpisodicEvent>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn
            .prepare("SELECT * FROM episodic_events WHERE user_id = ? ORDER BY occurred_at DESC LIMIT ?")
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![user_id, limit])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to query events: {}", e)))?;

        let mut events = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentMemError::storage_error(&format!("Failed to fetch row: {}", e)))? {
            events.push(row_to_event(&row)?);
        }

        Ok(events)
    }
}

/// Convert LibSQL row to EpisodicEvent
fn row_to_event(row: &libsql::Row) -> Result<EpisodicEvent> {
    Ok(EpisodicEvent {
        id: row.get(0).map_err(|e| AgentMemError::storage_error(&format!("Failed to get id: {}", e)))?,
        organization_id: row.get(1).map_err(|e| AgentMemError::storage_error(&format!("Failed to get organization_id: {}", e)))?,
        user_id: row.get(2).map_err(|e| AgentMemError::storage_error(&format!("Failed to get user_id: {}", e)))?,
        agent_id: row.get(3).map_err(|e| AgentMemError::storage_error(&format!("Failed to get agent_id: {}", e)))?,
        occurred_at: {
            let s: String = row.get(4).map_err(|e| AgentMemError::storage_error(&format!("Failed to get occurred_at: {}", e)))?;
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to parse occurred_at: {}", e)))?
                .with_timezone(&Utc)
        },
        event_type: row.get(5).map_err(|e| AgentMemError::storage_error(&format!("Failed to get event_type: {}", e)))?,
        actor: row.get(6).map_err(|e| AgentMemError::storage_error(&format!("Failed to get actor: {}", e)))?,
        summary: row.get(7).map_err(|e| AgentMemError::storage_error(&format!("Failed to get summary: {}", e)))?,
        details: row.get(8).map_err(|e| AgentMemError::storage_error(&format!("Failed to get details: {}", e)))?,
        importance_score: {
            let score: f64 = row.get(9).map_err(|e| AgentMemError::storage_error(&format!("Failed to get importance_score: {}", e)))?;
            score as f32
        },
        metadata: {
            let s: String = row.get(10).map_err(|e| AgentMemError::storage_error(&format!("Failed to get metadata: {}", e)))?;
            serde_json::from_str(&s).map_err(|e| AgentMemError::storage_error(&format!("Failed to parse metadata: {}", e)))?
        },
        created_at: {
            let s: String = row.get(11).map_err(|e| AgentMemError::storage_error(&format!("Failed to get created_at: {}", e)))?;
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to parse created_at: {}", e)))?
                .with_timezone(&Utc)
        },
        updated_at: {
            let s: String = row.get(12).map_err(|e| AgentMemError::storage_error(&format!("Failed to get updated_at: {}", e)))?;
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to parse updated_at: {}", e)))?
                .with_timezone(&Utc)
        },
    })
}

